use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use quick_xml::Reader;
use std::collections::HashSet;

pub fn local_name(raw: &[u8]) -> String {
    let s = std::str::from_utf8(raw).unwrap_or("");
    s.rsplit(':').next().unwrap_or(s).to_string()
}

pub fn qname_prefix(raw: &[u8]) -> String {
    let s = std::str::from_utf8(raw).unwrap_or("");
    match s.rsplit_once(':') {
        Some((prefix, _)) => format!("{prefix}:"),
        None => String::new(),
    }
}

pub fn attr_local(e: &BytesStart, want: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if local_name(attr.key.as_ref()) == want {
            return Some(String::from_utf8_lossy(&attr.value).into_owned());
        }
    }
    None
}

pub enum ScanEvent<'a> {
    Start(&'a str, &'a BytesStart<'a>, usize),
    End(&'a str),
}

pub fn for_each_element(
    xml: &str,
    mut on_event: impl FnMut(ScanEvent<'_>),
) -> Result<(), quick_xml::Error> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut line = 1usize;
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = local_name(e.name().as_ref());
                on_event(ScanEvent::Start(&name, &e, line));
                stack.push(name);
            }
            Ok(Event::Empty(e)) => {
                let name = local_name(e.name().as_ref());
                on_event(ScanEvent::Start(&name, &e, line));
                on_event(ScanEvent::End(&name));
            }
            Ok(Event::End(e)) => {
                let name = local_name(e.name().as_ref());
                stack.pop();
                on_event(ScanEvent::End(&name));
            }
            Ok(Event::Text(t)) => {
                let bytes: &[u8] = t.as_ref();
                line += bytes.iter().filter(|&&b| b == b'\n').count();
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    }
    let _ = stack;
    Ok(())
}

pub fn root_element(xml: &str) -> Result<(String, Option<String>), quick_xml::Error> {
    let mut reader = NsReader::from_str(xml);
    reader.config_mut().trim_text(true);
    loop {
        match reader.read_resolved_event() {
            Ok((ns, Event::Start(e))) | Ok((ns, Event::Empty(e))) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                return Ok((name, namespace_uri(ns)));
            }
            Ok((_, Event::Eof)) => return Ok((String::new(), None)),
            Ok((_, _)) => continue,
            Err(e) => return Err(e),
        }
    }
}

fn namespace_uri(ns: ResolveResult<'_>) -> Option<String> {
    match ns {
        ResolveResult::Bound(ns) => Some(String::from_utf8_lossy(ns.0).into_owned()),
        ResolveResult::Unbound | ResolveResult::Unknown(_) => None,
    }
}

pub fn count_effective_child(
    xml: &str,
    parent: &str,
    child: &str,
) -> Result<usize, quick_xml::Error> {
    let mut count = 0usize;
    let mut stack: Vec<String> = Vec::new();
    let mut mce_depth: Option<usize> = None;
    let mut mce_has_child = false;

    for_each_element(xml, |ev| match ev {
        ScanEvent::Start(name, _, _) => {
            if stack.last().is_some_and(|current| current == parent) {
                if name == child {
                    count += 1;
                } else if name == "AlternateContent" {
                    mce_depth = Some(stack.len() + 1);
                    mce_has_child = false;
                }
            } else if mce_depth.is_some() && name == child {
                mce_has_child = true;
            }
            stack.push(name.to_string());
        }
        ScanEvent::End(name) => {
            if let Some(depth) = mce_depth {
                if stack.len() == depth && stack.last().is_some_and(|current| current == name) {
                    if mce_has_child {
                        count += 1;
                    }
                    mce_depth = None;
                    mce_has_child = false;
                }
            }
            if stack.last().is_some_and(|current| current == name) {
                stack.pop();
            }
        }
    })?;

    Ok(count)
}

pub fn has_duplicate_attr(xml: &str, element: &str, attr: &str) -> Option<(String, usize)> {
    let mut seen = HashSet::new();
    let mut dup = None;
    let _ = for_each_element(xml, |ev| {
        let ScanEvent::Start(name, e, line) = ev else {
            return;
        };
        if dup.is_some() || name != element {
            return;
        }
        if let Some(id) = attr_local(e, attr) {
            if id.is_empty() {
                return;
            }
            if !seen.insert(id.clone()) {
                dup = Some((id, line));
            }
        }
    });
    dup
}

#[derive(Default)]
pub struct ElementStack {
    stack: Vec<String>,
}

impl ElementStack {
    pub fn on_start(&mut self, name: &str) {
        self.stack.push(name.to_string());
    }

    pub fn on_end(&mut self, name: &str) {
        if self.stack.last().is_some_and(|n| n == name) {
            self.stack.pop();
        }
    }

    pub fn current(&self) -> Option<&str> {
        self.stack.last().map(String::as_str)
    }

    pub fn under(&self, ancestor: &str) -> bool {
        self.stack.iter().any(|n| n == ancestor)
    }
}
