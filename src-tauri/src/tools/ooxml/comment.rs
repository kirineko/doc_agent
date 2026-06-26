use crate::tools::ooxml::validate::rules::scan::{
    attr_local, for_each_element, local_name, qname_prefix, ScanEvent,
};
use crate::tools::ToolError;
use chrono::Utc;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// WordprocessingML main namespace URI.
const WML_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
/// WordprocessingML 2010 (w14) namespace URI — carries `paraId`/`textId`.
const W14_NS: &str = "http://schemas.microsoft.com/office/word/2010/wordml";

/// In-memory template for a fresh `word/comments.xml` (paired, empty root).
const EMPTY_COMMENTS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"></w:comments>"#;

/// WordprocessingML 2012 (w15) namespace URI — commentsExtended / people parts.
const W15_NS: &str = "http://schemas.microsoft.com/office/word/2012/wordml";

/// In-memory template for a fresh `word/commentsExtended.xml`.
const EMPTY_COMMENTS_EX_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"></w15:commentsEx>"#;

/// In-memory template for a fresh `word/people.xml`.
const EMPTY_PEOPLE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w15:people xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"></w15:people>"#;

/// In-memory template for a fresh `word/_rels/document.xml.rels`.
const EMPTY_DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#;

/// Add a comment to an unpacked docx directory.
///
/// This performs the full wiring that makes a comment visible in Word:
/// 1. writes `<w:comment>` into `word/comments.xml` (handling self-closing shells),
/// 2. inserts `commentRangeStart/End` + `commentReference` anchors around the
///    target paragraph in `word/document.xml`,
/// 3. records reply linkage in `word/commentsExtended.xml` when `parent` is set,
/// 4. lazily builds `word/people.xml` and registers content-types/relationships.
pub fn add_comment(
    dir: &Path,
    id: u32,
    text: &str,
    author: &str,
    parent: Option<u32>,
    paragraph_index: usize,
    text_hint: Option<&str>,
) -> Result<(), ToolError> {
    let comments_path = dir.join("word/comments.xml");
    let document_path = dir.join("word/document.xml");
    if !document_path.exists() {
        return Err(ToolError::Execution(
            "word/document.xml not found; dir must be an unpacked docx out_dir".into(),
        ));
    }

    // Read existing comments or use an in-memory template. A failed call writes
    // nothing, so retry semantics stay correct.
    let comments_xml = if comments_path.exists() {
        fs::read_to_string(&comments_path).map_err(|e| ToolError::Execution(e.to_string()))?
    } else {
        EMPTY_COMMENTS_XML.to_string()
    };
    let document_xml =
        fs::read_to_string(&document_path).map_err(|e| ToolError::Execution(e.to_string()))?;

    // Reject duplicate id before deriving anything.
    let existing_ids = collect_comment_ids(&comments_xml);
    if existing_ids.contains(&id) {
        return Err(ToolError::Execution(format!(
            "comment id {id} already exists in comments.xml"
        )));
    }
    if comment_id_referenced_in_unpack(dir, &document_xml, id)? {
        return Err(ToolError::Execution(format!(
            "comment id {id} already has anchors in the document package"
        )));
    }

    let date = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let initials: String = author
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
    let para_id = format!("{:08X}", id.wrapping_mul(0x9E37_79B9));

    // --- Compute phase: derive every file's new content as a String. No file
    // is touched until all derivations (including the fallible ones) succeed,
    // so any failure leaves the unpacked directory byte-for-byte unchanged. ---

    // 1) New <w:comment> entry in comments.xml (robust against self-closing shell).
    let comment_entry = format_comment_entry(id, &date, &initials, author, text, &para_id);
    let new_comments_xml = append_into_root(&comments_xml, &comment_entry)?;

    // 2) Anchors in document.xml at the target paragraph (may fail on bad index/hint).
    let new_document_xml = insert_paragraph_anchors(&document_xml, id, paragraph_index, text_hint)?;

    // 3) Reply linkage in commentsExtended.xml (may fail when parent is missing).
    let extended_path = dir.join("word/commentsExtended.xml");
    let extended_update: Option<String> = if let Some(parent_id) = parent {
        if !existing_ids.contains(&parent_id) {
            return Err(ToolError::Execution(format!(
                "parent comment id {parent_id} not found in comments.xml"
            )));
        }
        // Thread under the parent's ACTUAL paragraph paraId. Deriving it from
        // the id only holds for comments this tool created; a pre-existing Word
        // comment has an arbitrary paraId, and a derived value would point
        // paraIdParent at a non-existent paragraph, un-threading the reply.
        let parent_para_id = find_comment_para_id(&comments_xml, parent_id).ok_or_else(|| {
            ToolError::Execution(format!(
                "parent comment id {parent_id} has no w14:paraId in comments.xml; cannot thread reply"
            ))
        })?;
        let entry = format!(
            r#"<w15:commentEx xmlns:w15="{W15_NS}" w15:paraId="{para_id}" w15:paraIdParent="{parent_para_id}" w15:done="0"/>"#
        );
        let (current, _existed) = read_or_template(&extended_path, EMPTY_COMMENTS_EX_XML)?;
        Some(append_into_root(&current, &entry)?)
    } else {
        None
    };

    // 4) people.xml records the author (deduplicated). Derived, not yet written.
    let people_path = dir.join("word/people.xml");
    let esc_author = xml_escape(author);
    let person_entry = format!(
        r#"<w15:person xmlns:w15="{W15_NS}" w15:author="{esc_author}" w15:initials="{}"/>"#,
        xml_escape(&initials)
    );
    let (people_current, people_existed) = read_or_template(&people_path, EMPTY_PEOPLE_XML)?;
    let author_present =
        people_existed && people_current.contains(&format!(r#"w15:author="{esc_author}""#));
    let people_update: Option<String> = if author_present {
        None
    } else {
        Some(append_into_root(&people_current, &person_entry)?)
    };

    // 5) OPC registration (content-types + document rels) — derived in compute
    // so commit can roll back if any write fails mid-flight.
    let ct_path = dir.join("[Content_Types].xml");
    let rels_path = dir.join("word/_rels/document.xml.rels");
    let ct_current = if ct_path.exists() {
        Some(fs::read_to_string(&ct_path).map_err(|e| ToolError::Execution(e.to_string()))?)
    } else {
        None
    };
    let rels_existed = rels_path.exists();
    let rels_current = if rels_existed {
        fs::read_to_string(&rels_path).map_err(|e| ToolError::Execution(e.to_string()))?
    } else {
        EMPTY_DOCUMENT_RELS.to_string()
    };
    let mut ct = ct_current.clone().unwrap_or_default();
    let mut rels = rels_current.clone();
    let mut parts_to_register = vec!["comments.xml"];
    if extended_update.is_some() {
        parts_to_register.push("commentsExtended.xml");
    }
    parts_to_register.push("people.xml");
    for part in parts_to_register {
        (ct, rels) = apply_part_registration(&ct, &rels, part)?;
    }

    // --- Commit phase: write every derived part atomically (rollback on failure). ---
    let mut writes = vec![
        PendingWrite {
            path: comments_path.clone(),
            prior: snapshot_part(&comments_path)?,
            content: new_comments_xml,
        },
        PendingWrite {
            path: document_path.clone(),
            prior: snapshot_part(&document_path)?,
            content: new_document_xml,
        },
    ];
    if let Some(xml) = extended_update {
        writes.push(PendingWrite {
            path: extended_path.clone(),
            prior: snapshot_part(&extended_path)?,
            content: xml,
        });
    }
    if let Some(xml) = people_update {
        writes.push(PendingWrite {
            path: people_path.clone(),
            prior: snapshot_part(&people_path)?,
            content: xml,
        });
    }
    if let Some(ref ct_before) = ct_current {
        if ct != *ct_before {
            writes.push(PendingWrite {
                path: ct_path.clone(),
                prior: Some(ct_before.clone()),
                content: ct,
            });
        }
    }
    if rels != rels_current {
        writes.push(PendingWrite {
            path: rels_path.clone(),
            prior: if rels_existed {
                Some(rels_current)
            } else {
                None
            },
            content: rels,
        });
    }
    commit_writes(&writes)?;

    Ok(())
}

struct PendingWrite {
    path: std::path::PathBuf,
    /// `None` when the path did not exist before this call (rollback deletes it).
    prior: Option<String>,
    content: String,
}

fn snapshot_part(path: &Path) -> Result<Option<String>, ToolError> {
    if path.exists() {
        Ok(Some(
            fs::read_to_string(path).map_err(|e| ToolError::Execution(e.to_string()))?,
        ))
    } else {
        Ok(None)
    }
}

fn commit_writes(writes: &[PendingWrite]) -> Result<(), ToolError> {
    for (done, w) in writes.iter().enumerate() {
        if let Err(e) = write_part(&w.path, &w.content) {
            rollback_writes(&writes[..done]);
            return Err(e);
        }
    }
    Ok(())
}

fn rollback_writes(writes: &[PendingWrite]) {
    for w in writes.iter().rev() {
        match &w.prior {
            Some(content) => {
                let _ = write_part(&w.path, content);
            }
            None => {
                let _ = fs::remove_file(&w.path);
            }
        }
    }
}

/// Read an existing part, or fall back to `template` when the file is absent.
/// The returned bool is true when the file already existed on disk.
fn read_or_template(path: &Path, template: &str) -> Result<(String, bool), ToolError> {
    if path.exists() {
        let xml = fs::read_to_string(path).map_err(|e| ToolError::Execution(e.to_string()))?;
        Ok((xml, true))
    } else {
        Ok((template.to_string(), false))
    }
}

/// Write a part to disk, creating its parent directory if necessary.
fn write_part(path: &Path, contents: &str) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::Execution(e.to_string()))?;
    }
    fs::write(path, contents).map_err(|e| ToolError::Execution(e.to_string()))
}

fn format_comment_entry(
    id: u32,
    date: &str,
    initials: &str,
    author: &str,
    text: &str,
    para_id: &str,
) -> String {
    let esc_text = xml_escape(text);
    let esc_author = xml_escape(author);
    let esc_initials = xml_escape(initials);
    // Declare w/w14 on the inserted element itself: the existing comments.xml
    // root may omit `xmlns:w14` (or bind the WML namespace to a different/default
    // prefix), and this fragment uses both `w:` and `w14:`. Self-declaring keeps
    // the spliced subtree valid regardless of the host root, and redeclaring an
    // identical prefix→URI binding is well-formed XML.
    format!(
        r#"<w:comment xmlns:w="{WML_NS}" xmlns:w14="{W14_NS}" w:id="{id}" w:author="{esc_author}" w:date="{date}" w:initials="{esc_initials}">
  <w:p w14:paraId="{para_id}" w14:textId="77777777">
    <w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:annotationRef/></w:r>
    <w:r><w:rPr><w:color w:val="000000"/><w:sz w:val="20"/><w:szCs w:val="20"/></w:rPr>
      <w:t>{esc_text}</w:t></w:r>
  </w:p>
</w:comment>"#
    )
}

/// Append `entry` as the first child inside the single root element of `xml`.
///
/// Works for any comment-related part — `comments.xml`, `commentsExtended.xml`,
/// `people.xml` — regardless of the root's namespace prefix. Uses quick-xml to
/// locate the root element byte range so we can splice reliably whether the
/// root is a self-closing shell (`<root .../>`) or a paired container
/// (`<root ...></root>`). String `rfind` is deliberately avoided — it silently
/// no-ops on self-closing shells, which was the original comments.xml bug and
/// would equally break a self-closing `<w15:people/>` / `<w15:commentsEx/>`.
fn append_into_root(xml: &str, entry: &str) -> Result<String, ToolError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut root_start: Option<usize> = None;
    let mut root_end_byte: Option<usize> = None;
    let mut root_was_empty = false;
    let mut root_qname: Option<String> = None;
    let mut depth: i64 = 0;
    let mut root_opened = false;
    loop {
        let event_start: usize = reader.buffer_position().try_into().unwrap_or(0);
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let now: usize = reader.buffer_position().try_into().unwrap_or(0);
                if !root_opened {
                    root_start = Some(now);
                    root_qname = Some(String::from_utf8_lossy(e.name().as_ref()).into_owned());
                    root_opened = true;
                }
                depth += 1;
            }
            Ok(Event::Empty(e)) => {
                let now: usize = reader.buffer_position().try_into().unwrap_or(0);
                if !root_opened {
                    // Self-closing root: this is the only element.
                    root_start = Some(now);
                    root_qname = Some(String::from_utf8_lossy(e.name().as_ref()).into_owned());
                    root_was_empty = true;
                    root_end_byte = Some(now);
                    break;
                }
                // Empty child inside the root: depth unchanged, ignore.
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 && root_opened {
                    // Matching close of the root element.
                    root_end_byte = Some(event_start);
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(ToolError::Execution(format!("parse xml root: {e}"))),
        }
    }

    let name_end_byte =
        root_start.ok_or_else(|| ToolError::Execution("xml has no root element".into()))?;
    let root_name =
        root_qname.ok_or_else(|| ToolError::Execution("xml has no root element".into()))?;

    // Reconstruct with a guaranteed paired root.
    // Strategy: rewrite the root so it is always `<root ...>ENTRY</root>`.
    // 1. Take everything up to and including the root open tag (for Empty, that
    //    is the self-closing tag which we convert to a paired open).
    let (open_tag_str, after_open_byte) = if root_was_empty {
        // Self-closing: the element ends with `/>`. Strip the trailing `/>` and
        // append `>` so it becomes a paired open tag.
        let self_closing = &xml[..name_end_byte];
        let trimmed = self_closing.trim_end();
        let open_tag = if let Some(without_gt) = trimmed.strip_suffix('>') {
            // remove the '/' right before '>'
            let without_slash = without_gt
                .trim_end()
                .strip_suffix('/')
                .unwrap_or(without_gt);
            format!("{without_slash}>")
        } else {
            format!("{trimmed}>")
        };
        (open_tag, name_end_byte)
    } else {
        (xml[..name_end_byte].to_string(), name_end_byte)
    };

    let tail = if root_was_empty {
        String::new()
    } else {
        // Everything after the root open tag up to the root close tag.
        let end_byte = root_end_byte.unwrap_or(xml.len());
        xml[after_open_byte..end_byte].to_string()
    };

    let result = format!("{open_tag_str}{entry}{tail}</{root_name}>");
    Ok(result)
}

fn collect_comment_ids(xml: &str) -> HashSet<u32> {
    let mut ids = HashSet::new();
    let _ = for_each_element(xml, |ev| {
        let ScanEvent::Start(name, e, _) = ev else {
            return;
        };
        if name == "comment" {
            if let Some(id_str) = attr_local(e, "id") {
                if let Ok(n) = id_str.parse::<u32>() {
                    ids.insert(n);
                }
            }
        }
    });
    ids
}

fn comment_id_referenced_in_xml(xml: &str, id: u32) -> bool {
    let target = id.to_string();
    let mut found = false;
    let _ = for_each_element(xml, |ev| {
        let ScanEvent::Start(name, e, _) = ev else {
            return;
        };
        if matches!(
            name,
            "commentReference" | "commentRangeStart" | "commentRangeEnd"
        ) && attr_local(e, "id").as_deref() == Some(target.as_str())
        {
            found = true;
        }
    });
    found
}

const STORY_REL_TYPES: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes",
];

fn comment_id_referenced_in_unpack(
    dir: &Path,
    document_xml: &str,
    id: u32,
) -> Result<bool, ToolError> {
    if comment_id_referenced_in_xml(document_xml, id) {
        return Ok(true);
    }
    let referenced = referenced_story_targets(dir, document_xml);
    if referenced.is_empty() {
        return Ok(false);
    }
    let word_dir = dir.join("word");
    for name in referenced {
        let path = word_dir.join(&name);
        if !path.is_file() {
            continue;
        }
        let xml = fs::read_to_string(&path).map_err(|e| ToolError::Execution(e.to_string()))?;
        if comment_id_referenced_in_xml(&xml, id) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn referenced_story_targets(base: &Path, document_xml: &str) -> HashSet<String> {
    let rels_path = base.join("word/_rels/document.xml.rels");
    let Ok(rels) = fs::read_to_string(&rels_path) else {
        return HashSet::new();
    };
    let used_hf_rids = header_footer_rids_in_document(document_xml);
    let mut targets = HashSet::new();
    let _ = for_each_element(&rels, |ev| {
        let ScanEvent::Start(name, e, _) = ev else {
            return;
        };
        if name != "Relationship" {
            return;
        }
        let Some(ty) = attr_local(e, "Type") else {
            return;
        };
        if !STORY_REL_TYPES.contains(&ty.as_str()) {
            return;
        }
        if ty.ends_with("/header") || ty.ends_with("/footer") {
            let Some(rid) = attr_local(e, "Id") else {
                return;
            };
            if !used_hf_rids.contains(&rid) {
                return;
            }
        }
        if let Some(target) = attr_local(e, "Target") {
            let file = target
                .rsplit('/')
                .next()
                .unwrap_or(target.as_str())
                .to_ascii_lowercase();
            targets.insert(file);
        }
    });
    targets
}

fn header_footer_rids_in_document(document_xml: &str) -> HashSet<String> {
    let mut ids = HashSet::new();
    let _ = for_each_element(document_xml, |ev| {
        let ScanEvent::Start(name, e, _) = ev else {
            return;
        };
        if name == "headerReference" || name == "footerReference" {
            if let Some(id) = attr_local(e, "id") {
                ids.insert(id);
            }
        }
    });
    ids
}

/// The `w14:paraId` of the paragraph belonging to comment `comment_id`, read
/// from `comments.xml`. Returns the first non-empty paraId inside the matching
/// `<w:comment>`, or None when the comment is absent or carries no paraId.
fn find_comment_para_id(xml: &str, comment_id: u32) -> Option<String> {
    let target = comment_id.to_string();
    let mut in_target = false;
    let mut found: Option<String> = None;
    let _ = for_each_element(xml, |ev| match ev {
        ScanEvent::Start(name, e, _) => {
            if name == "comment" {
                in_target = attr_local(e, "id").as_deref() == Some(target.as_str());
            } else if name == "p" && in_target && found.is_none() {
                if let Some(pid) = attr_local(e, "paraId").filter(|s| !s.is_empty()) {
                    found = Some(pid);
                }
            }
        }
        ScanEvent::End(name) => {
            if name == "comment" {
                in_target = false;
            }
        }
    });
    found
}

/// Insert `commentRangeStart`/`commentRangeEnd`/`commentReference` anchors
/// around the N-th top-level `<w:p>` in document.xml.
fn insert_paragraph_anchors(
    xml: &str,
    id: u32,
    paragraph_index: usize,
    text_hint: Option<&str>,
) -> Result<String, ToolError> {
    // Find byte ranges of top-level <w:p> elements directly under <w:body>.
    let paragraphs = find_top_level_paragraphs(xml)?;

    let para = paragraphs.get(paragraph_index).ok_or_else(|| {
        ToolError::Execution(format!(
            "paragraph_index {paragraph_index} out of range (document has {} top-level paragraphs)",
            paragraphs.len()
        ))
    })?;

    if let Some(hint) = text_hint {
        let para_text = extract_text(&xml[para.start..para.end]);
        if !para_text.contains(hint) {
            return Err(ToolError::Execution(format!(
                "text_hint {:?} not found in paragraph {paragraph_index} (text: {:?})",
                hint, para_text
            )));
        }
    }

    // Match the target paragraph's own namespace prefix (`w:`, a custom `w2:`,
    // or none for a default-namespace document) so the inserted anchors bind to
    // the same WordprocessingML namespace and the close-tag search succeeds.
    // Hard-coding `w:` would error or emit unbound markup for those documents.
    //
    // Attributes are handled separately: default XML namespaces do NOT apply to
    // attributes, so a default-namespace document still needs `w:id` / `w:val`
    // even when elements are unprefixed. When `pfx` is empty we also bind `w:`
    // on the paragraph open tag so those prefixed attributes are in scope for all
    // anchor siblings inserted inside the paragraph.
    let pfx = element_prefix(&xml[para.start..]);
    let use_w_attr_prefix = pfx.is_empty();
    let apfx = if use_w_attr_prefix {
        "w:"
    } else {
        pfx.as_str()
    };
    let start_anchor = format!(r#"<{pfx}commentRangeStart {apfx}id="{id}"/>"#);
    let end_anchor = format!(
        r#"<{pfx}commentRangeEnd {apfx}id="{id}"/><{pfx}r><{pfx}rPr><{pfx}rStyle {apfx}val="CommentReference"/></{pfx}rPr><{pfx}commentReference {apfx}id="{id}"/></{pfx}r>"#
    );

    // Self-closing <w:p/> (a blank paragraph): para.start == para.end spans the
    // whole self-closing tag. Expand it into a paired paragraph with the anchors.
    let is_self_closing = xml[para.start..para.end].trim_end().ends_with("/>");
    if is_self_closing {
        let p_tag = &xml[para.start..para.end]; // e.g. `<w:p .../>` or `<w:p pPr.../>`
                                                // Strip trailing `/>`, then trailing `>` to get `<w:p ...`.
        let open_inner = p_tag.trim_end();
        let mut open_tag = open_inner
            .strip_suffix("/>")
            .or_else(|| open_inner.strip_suffix('>'))
            .unwrap_or(open_inner)
            .to_string();
        if use_w_attr_prefix {
            open_tag = inject_w_namespace_decl(&open_tag);
        }
        // Extract any pPr that was serialized inside a self-closing form is not
        // standard; pPr normally lives in a paired paragraph, so here we just
        // re-open as a paired tag with the anchors inside.
        let expanded = format!("{open_tag}>{start_anchor}{end_anchor}</{pfx}p>");
        let mut out = String::with_capacity(xml.len() + expanded.len());
        out.push_str(&xml[..para.start]);
        out.push_str(&expanded);
        out.push_str(&xml[para.end..]);
        return Ok(out);
    }

    // Paired paragraph: find the end of the <w:p ...> open tag.
    let tag_start = xml[para.start..]
        .find('<')
        .map(|i| para.start + i)
        .unwrap_or(para.start);
    let p_open_end = find_open_tag_end(&xml[para.start..para.end])
        .ok_or_else(|| ToolError::Execution("malformed <w:p> open tag".into()))?;
    let open_abs = para.start + p_open_end;

    // Locate the matching close tag (`</w:p>`, `</w2:p>` or `</p>`); para.end is
    // the byte just past it.
    let close_tag = format!("</{pfx}p>");
    let close_tag_start = xml[..para.end]
        .rfind(&close_tag)
        .ok_or_else(|| ToolError::Execution("malformed <w:p>: missing close tag".into()))?;

    // WordprocessingML requires <w:pPr> to be the FIRST child of <w:p>. If a
    // pPr is present, the commentRangeStart must go AFTER it, not before.
    let start_insert_abs = end_of_ppr(&xml[open_abs..close_tag_start]).unwrap_or(0) + open_abs;

    let open_tag = if use_w_attr_prefix {
        inject_w_namespace_decl(&xml[tag_start..open_abs])
    } else {
        xml[tag_start..open_abs].to_string()
    };

    // Splice: rewrite open tag (possibly with xmlns:w), then anchors.
    let mut out = String::with_capacity(xml.len() + start_anchor.len() + end_anchor.len() + 32);
    out.push_str(&xml[..tag_start]);
    out.push_str(&open_tag);
    out.push_str(&xml[open_abs..start_insert_abs]);
    out.push_str(&start_anchor);
    out.push_str(&xml[start_insert_abs..close_tag_start]);
    out.push_str(&end_anchor);
    out.push_str(&xml[close_tag_start..]);
    Ok(out)
}

/// Add `xmlns:w` to a paragraph open tag when the document uses the WML default
/// namespace (unprefixed elements) and has not already declared `xmlns:w`.
fn inject_w_namespace_decl(open_tag: &str) -> String {
    if open_tag.contains("xmlns:w=\"") {
        return open_tag.to_string();
    }
    let trimmed = open_tag.trim_end();
    if let Some(stem) = trimmed.strip_suffix("/>") {
        format!("{stem} xmlns:w=\"{WML_NS}\"/>")
    } else if let Some(stem) = trimmed.strip_suffix('>') {
        format!("{stem} xmlns:w=\"{WML_NS}\">")
    } else {
        open_tag.to_string()
    }
}

/// The element-name prefix (including the trailing `:`) of the first start tag
/// in `slice`. Returns "" when the element name is unprefixed (default-namespace
/// document). `slice` is a paragraph range that may include leading whitespace
/// (paragraph offsets can start at the indentation before `<`), so we scan to
/// the first `<` rather than assuming the slice begins with it.
fn element_prefix(slice: &str) -> String {
    let Some(lt) = slice.find('<') else {
        return String::new();
    };
    let after = &slice[lt + 1..];
    let end = after
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(after.len());
    qname_prefix(&after.as_bytes()[..end])
}

/// If the paragraph content (after the `<w:p ...>` open tag) begins with a
/// `<w:pPr>...</w:pPr>` (or self-closing `<w:pPr/>`), return the byte offset
/// just past that pPr element within `content`. Otherwise return None, so the
/// caller inserts the anchor at the very start of the paragraph body.
///
/// Uses depth counting rather than a `</w:pPr>` substring search: a `pPr` may
/// nest another `pPr` inside a `<w:pPrChange>` revision marker, and a naive
/// `find` would stop at that inner close tag and splice anchors *inside* the
/// paragraph properties (invalid WordprocessingML).
fn end_of_ppr(content: &str) -> Option<usize> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut depth: i64 = 0;
    let mut started = false;
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if !started {
                    started = true;
                    // The leading element must be pPr, otherwise there is no
                    // paragraph-properties block to step over.
                    if local_name(e.name().as_ref()) != "pPr" {
                        return None;
                    }
                }
                depth += 1;
            }
            Ok(Event::Empty(e)) => {
                if !started {
                    // A self-closing leading element: only pPr counts.
                    return if local_name(e.name().as_ref()) == "pPr" {
                        Some(reader.buffer_position().try_into().unwrap_or(0))
                    } else {
                        None
                    };
                }
                // Empty child inside pPr: depth unchanged.
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    // started is true and the first element was pPr, so this is
                    // the matching close of the leading pPr (depth counting
                    // skips any nested pPr inside pPrChange).
                    return Some(reader.buffer_position().try_into().unwrap_or(0));
                }
            }
            Ok(Event::Eof) => return None,
            Ok(_) => {}
            Err(_) => return None,
        }
    }
}

struct ParaRange {
    start: usize, // byte offset of '<' of <w:p ...>
    end: usize,   // byte offset just past '>' of </w:p>
}

fn find_top_level_paragraphs(xml: &str) -> Result<Vec<ParaRange>, ToolError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    // Track the byte offset where each top-level <w:p> starts.
    let mut paras: Vec<ParaRange> = Vec::new();
    let mut p_start: Option<(usize, usize)> = None; // (open_byte, depth_at_open)
    loop {
        let ev_start: usize = reader.buffer_position().try_into().unwrap_or(0);
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = local_name(e.name().as_ref());
                let direct_under_body = stack.last().is_some_and(|p| p == "body") && name == "p";
                stack.push(name.clone());
                if direct_under_body && p_start.is_none() {
                    // Record the position just before this event's bytes.
                    // reader.buffer_position() points AFTER the tag; the '<' is at ev_start.
                    p_start = Some((ev_start, stack.len()));
                }
            }
            Ok(Event::End(e)) => {
                let name = local_name(e.name().as_ref());
                if let Some((start_byte, depth)) = p_start {
                    if stack.len() == depth && name == "p" {
                        let now: usize = reader.buffer_position().try_into().unwrap_or(0);
                        paras.push(ParaRange {
                            start: start_byte,
                            end: now,
                        });
                        p_start = None;
                    }
                }
                if stack.last().is_some_and(|n| n == &name) {
                    stack.pop();
                }
            }
            Ok(Event::Empty(e)) => {
                // A self-closing element under <w:body> (e.g. a blank <w:p/>)
                // counts as a top-level paragraph per the tool contract.
                let name = local_name(e.name().as_ref());
                if stack.last().is_some_and(|p| p == "body") && name == "p" {
                    let now: usize = reader.buffer_position().try_into().unwrap_or(0);
                    paras.push(ParaRange {
                        start: ev_start,
                        end: now,
                    });
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(ToolError::Execution(format!("parse document.xml: {e}"))),
        }
    }
    Ok(paras)
}

/// Find the byte offset just past the first `>` of the leading open tag.
fn find_open_tag_end(slice: &str) -> Option<usize> {
    let mut in_quotes = false;
    let mut prev = '\0';
    for (i, c) in slice.char_indices() {
        if c == '"' {
            in_quotes = !in_quotes;
        }
        if c == '>' && !in_quotes && prev != '/' {
            return Some(i + 1);
        }
        prev = c;
    }
    None
}

/// Concatenate all text inside the given XML fragment.
fn extract_text(fragment: &str) -> String {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    let mut reader = Reader::from_str(fragment);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut out = String::new();
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(t)) => {
                if let Ok(s) = t.unescape() {
                    out.push_str(&s);
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }
    out
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Register a word/ part in [Content_Types].xml and document.xml.rels if missing.
fn apply_part_registration(
    ct: &str,
    rels: &str,
    part_name: &str,
) -> Result<(String, String), ToolError> {
    let (content_type, rel_type) = match part_name {
        "comments.xml" => (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        ),
        "commentsExtended.xml" => (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml",
            "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
        ),
        "people.xml" => (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.people+xml",
            "http://schemas.microsoft.com/office/2011/relationships/people",
        ),
        _ => return Ok((ct.to_string(), rels.to_string())),
    };

    let mut new_ct = ct.to_string();
    if !new_ct.is_empty() {
        let part_marker = format!("PartName=\"/word/{part_name}\"");
        if !new_ct.contains(&part_marker) {
            let ctp = root_element_prefix(&new_ct)?;
            let override_entry = format!(
                r#"<{ctp}Override ContentType="{content_type}" PartName="/word/{part_name}"/>"#
            );
            new_ct = append_into_root(&new_ct, &override_entry)?;
        }
    }

    let mut new_rels = rels.to_string();
    let target_marker = format!(r#"Target="{part_name}""#);
    if !new_rels.contains(&target_marker) {
        let rid = next_rid(&new_rels);
        let relp = root_element_prefix(&new_rels)?;
        let rel_entry =
            format!(r#"<{relp}Relationship Id="{rid}" Type="{rel_type}" Target="{part_name}"/>"#);
        new_rels = append_into_root(&new_rels, &rel_entry)?;
    }
    Ok((new_ct, new_rels))
}

/// Prefix (including trailing `:`) on the first element in an XML document.
/// Returns "" for unprefixed roots, whose children inherit the default namespace
/// when one is declared on the root.
fn root_element_prefix(xml: &str) -> Result<String, ToolError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                return Ok(qname_prefix(e.name().as_ref()))
            }
            Ok(Event::Eof) => return Err(ToolError::Execution("xml has no root element".into())),
            Ok(_) => {}
            Err(e) => return Err(ToolError::Execution(format!("parse xml root: {e}"))),
        }
    }
}

fn next_rid(rels: &str) -> String {
    let mut max = 0u32;
    let _ = for_each_element(rels, |ev| {
        let ScanEvent::Start(name, e, _) = ev else {
            return;
        };
        if name == "Relationship" {
            if let Some(id) = attr_local(e, "Id") {
                if let Some(n) = id.strip_prefix("rId").and_then(|s| s.parse::<u32>().ok()) {
                    if n > max {
                        max = n;
                    }
                }
            }
        }
    });
    format!("rId{}", max + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn self_closing_shell() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"/>"#
    }

    fn paired_container() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
</w:comments>"#
    }

    #[test]
    fn appends_into_self_closing_shell() {
        let entry = "<w:comment w:id=\"1\"/>";
        let out = append_into_root(self_closing_shell(), entry).unwrap();
        assert!(
            out.contains("<w:comment w:id=\"1\"/>"),
            "entry missing: {out}"
        );
        assert!(
            out.ends_with("</w:comments>"),
            "must end with close tag: {out}"
        );
        // The original self-closing `/>` must have been converted to a paired open.
        assert!(
            !out.contains("wordml\"/>"),
            "self-closing root should be converted: {out}"
        );
        // The root must now be paired and contain the entry directly after the open tag.
        assert!(out.contains("><w:comment w:id=\"1\"/></w:comments>"));
    }

    #[test]
    fn appends_into_paired_container() {
        let entry = "<w:comment w:id=\"2\"/>";
        let out = append_into_root(paired_container(), entry).unwrap();
        assert!(out.contains("<w:comment w:id=\"2\"/>"));
        assert!(out.ends_with("</w:comments>"));
    }

    #[test]
    fn append_into_root_handles_self_closing_aux_part() {
        // An existing people.xml/commentsExtended.xml may itself be a
        // self-closing shell; the generic append must expand it, not error.
        let people = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w15:people xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"/>"#;
        let out = append_into_root(people, r#"<w15:person w15:author="A"/>"#).unwrap();
        assert!(out.contains(r#"<w15:person w15:author="A"/>"#));
        assert!(out.trim_end().ends_with("</w15:people>"), "got: {out}");
        assert_well_formed(&out);

        let cex =
            r#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"/>"#;
        let out = append_into_root(cex, r#"<w15:commentEx w15:paraId="1"/>"#).unwrap();
        assert!(out.contains(r#"<w15:commentEx w15:paraId="1"/>"#));
        assert!(out.trim_end().ends_with("</w15:commentsEx>"), "got: {out}");
        assert_well_formed(&out);
    }

    #[test]
    fn duplicate_id_detected() {
        let xml = r#"<w:comments xmlns:w="w"><w:comment w:id="5"/></w:comments>"#;
        let ids = collect_comment_ids(xml);
        assert!(ids.contains(&5));
    }

    #[test]
    fn escaping_handles_special_chars() {
        assert_eq!(xml_escape("a<b>&c\"d"), "a&lt;b&gt;&amp;c&quot;d");
    }

    #[test]
    fn comment_entry_escapes_text() {
        let entry = format_comment_entry(1, "2026", "Z", "Claude", "<x>", "DEADBEEF");
        assert!(entry.contains("&lt;x&gt;"));
        assert!(!entry.contains("<x>"));
    }

    #[test]
    fn comment_entry_self_declares_namespaces() {
        // The fragment uses both w: and w14:, so it must declare both on its own
        // root element to stay valid inside any host comments.xml.
        let entry = format_comment_entry(1, "2026", "Z", "Claude", "hi", "DEADBEEF");
        assert!(
            entry.contains(&format!(r#"xmlns:w="{WML_NS}""#)),
            "missing xmlns:w: {entry}"
        );
        assert!(
            entry.contains(&format!(r#"xmlns:w14="{W14_NS}""#)),
            "missing xmlns:w14: {entry}"
        );
    }

    #[test]
    fn append_comment_into_w14less_root_keeps_w14_declared() {
        // Real Word documents can have a comments.xml that declares only xmlns:w.
        // Appending our comment (which uses w14:paraId) must not leave w14
        // undeclared: the fragment self-declares it.
        let w14less = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"></w:comments>"#;
        let entry = format_comment_entry(7, "2026", "Z", "Claude", "hi", "DEADBEEF");
        let out = append_into_root(w14less, &entry).unwrap();
        assert_well_formed(&out);
        assert!(
            out.contains(&format!(r#"xmlns:w14="{W14_NS}""#)),
            "w14 must be declared in the spliced subtree: {out}"
        );
    }

    #[test]
    fn finds_actual_parent_para_id() {
        // A pre-existing (Word-authored) comment whose paraId is NOT the value we
        // would derive from its id. Replies must thread under this actual paraId.
        let comments = r#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
<w:comment w:id="5"><w:p w14:paraId="0A1B2C3D"><w:r><w:t>parent</w:t></w:r></w:p></w:comment>
</w:comments>"#;
        let pid = find_comment_para_id(comments, 5).unwrap();
        assert_eq!(pid, "0A1B2C3D");
        // It must differ from the naive id-derived value (the old bug).
        let derived = format!("{:08X}", 5u32.wrapping_mul(0x9E37_79B9));
        assert_ne!(pid, derived, "test fixture should use a non-derived paraId");
        assert_eq!(find_comment_para_id(comments, 99), None);
    }

    #[test]
    fn anchors_use_document_paragraph_prefix() {
        // document.xml binds WML to a custom `w2:` prefix. Anchors must use w2:,
        // and the `</w2:p>` close must be found — hard-coded `w:` used to error.
        let xml = r#"<?xml version="1.0"?>
<w2:document xmlns:w2="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w2:body><w2:p><w2:r><w2:t>hi</w2:t></w2:r></w2:p></w2:body></w2:document>"#;
        let out = insert_paragraph_anchors(xml, 3, 0, None).unwrap();
        assert!(
            out.contains(r#"<w2:commentRangeStart w2:id="3"/>"#),
            "anchors must adopt the w2: prefix: {out}"
        );
        assert!(out.contains(r#"<w2:commentReference w2:id="3"/>"#), "{out}");
        assert!(
            !out.contains("<w:commentRangeStart"),
            "must not hard-code the w: prefix: {out}"
        );
        assert_well_formed(&out);
    }

    #[test]
    fn anchors_handle_default_namespace_paragraph() {
        // Default-namespace document (no prefix on elements). Elements inherit the
        // default WML namespace, but attributes must still be prefixed (`w:id`,
        // `w:val`) because default namespaces do not apply to attributes in XML.
        // The paragraph open tag gets xmlns:w so w:-prefixed attributes are in scope.
        let xml = r#"<?xml version="1.0"?>
<document xmlns="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><body><p><r><t>hi</t></r></p></body></document>"#;
        let out = insert_paragraph_anchors(xml, 1, 0, None).unwrap();
        assert!(
            out.contains(&format!(r#"xmlns:w="{WML_NS}""#)),
            "paragraph open must declare xmlns:w: {out}"
        );
        assert!(
            out.contains(r#"<commentRangeStart w:id="1"/>"#),
            "element unprefixed, attribute w:id: {out}"
        );
        assert!(
            out.contains(r#"<commentReference w:id="1"/>"#),
            "commentReference must use w:id: {out}"
        );
        assert!(
            out.contains(r#"<rStyle w:val="CommentReference"/>"#),
            "rStyle must use w:val: {out}"
        );
        assert!(
            !out.contains(r#"commentRangeStart id="1""#),
            "bare id is invalid: {out}"
        );
        assert!(!out.contains("<w:commentRangeStart"), "{out}");
        assert_well_formed(&out);
    }

    #[test]
    fn comment_ex_entry_self_declares_w15_namespace() {
        let entry = format!(
            r#"<w15:commentEx xmlns:w15="{W15_NS}" w15:paraId="AAA" w15:paraIdParent="BBB" w15:done="0"/>"#
        );
        let host = r#"<commentsEx xmlns="http://schemas.microsoft.com/office/word/2012/wordml"></commentsEx>"#;
        let out = append_into_root(host, &entry).unwrap();
        assert!(out.contains(&format!(r#"xmlns:w15="{W15_NS}""#)));
        assert_well_formed(&out);
    }

    #[test]
    fn registers_part_with_prefixed_content_types_root() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        write_minimal_docx(base, true);
        fs::write(
            base.join("[Content_Types].xml"),
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<ct:Types xmlns:ct="http://schemas.openxmlformats.org/package/2006/content-types"><ct:Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><ct:Default Extension="xml" ContentType="application/xml"/><ct:Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></ct:Types>"#,
        )
        .unwrap();

        add_comment(base, 1, "x", "A", None, 0, None).unwrap();

        let ct = fs::read_to_string(base.join("[Content_Types].xml")).unwrap();
        assert!(
            ct.contains(
                r#"<ct:Override ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml" PartName="/word/comments.xml"/>"#
            ),
            "prefixed Types root must get a namespaced Override: {ct}"
        );
    }

    #[test]
    fn registers_part_with_prefixed_relationships_root() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        write_minimal_docx(base, true);
        fs::write(
            base.join("word/_rels/document.xml.rels"),
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<rel:Relationships xmlns:rel="http://schemas.openxmlformats.org/package/2006/relationships"><rel:Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></rel:Relationships>"#,
        )
        .unwrap();

        add_comment(base, 1, "new", "A", None, 0, None).unwrap();

        let rels = fs::read_to_string(base.join("word/_rels/document.xml.rels")).unwrap();
        assert!(
            rels.contains(
                r#"<rel:Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/>"#
            ),
            "prefixed Relationships root must get a namespaced Relationship: {rels}"
        );
    }

    #[test]
    fn registers_part_when_rels_is_self_closing_shell() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        write_minimal_docx(base, true);
        fs::write(
            base.join("word/_rels/document.xml.rels"),
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#,
        )
        .unwrap();

        add_comment(base, 1, "new", "A", None, 0, None).unwrap();

        let rels = fs::read_to_string(base.join("word/_rels/document.xml.rels")).unwrap();
        assert!(
            rels.contains("relationships/comments"),
            "self-closing rels shell must get comments relationship: {rels}"
        );
        assert!(
            rels.contains("</Relationships>"),
            "self-closing shell must be expanded to paired root: {rels}"
        );
    }

    #[test]
    fn registers_comments_when_orphan_shell_lacks_rels() {
        // Orphan comments.xml shell (old bug) without content-type/rels registration.
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        write_minimal_docx(base, true);
        fs::write(
            base.join("word/comments.xml"),
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"></w:comments>"#,
        )
        .unwrap();

        add_comment(base, 1, "new", "A", None, 0, None).unwrap();

        let ct = fs::read_to_string(base.join("[Content_Types].xml")).unwrap();
        assert!(
            ct.contains(r#"PartName="/word/comments.xml""#),
            "orphan shell must still get content-type registration: {ct}"
        );
        let rels = fs::read_to_string(base.join("word/_rels/document.xml.rels")).unwrap();
        assert!(
            rels.contains("relationships/comments"),
            "orphan shell must still get rels registration: {rels}"
        );
    }

    #[test]
    fn element_prefix_extracts_prefix() {
        assert_eq!(element_prefix("<w:p>"), "w:");
        assert_eq!(element_prefix("<w2:p attr=\"x\">"), "w2:");
        assert_eq!(element_prefix("<p/>"), "");
        assert_eq!(element_prefix("<w:p/>"), "w:");
        // Paragraph offsets may include leading indentation before the '<'.
        assert_eq!(element_prefix("\n    <w:p/>"), "w:");
        assert_eq!(element_prefix("\n    <p>"), "");
    }

    fn doc_with_mixed_paragraphs() -> &'static str {
        // body has: <w:p> (real), <w:p/> (blank self-closing), <w:p> (real)
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>first</w:t></w:r></w:p>
    <w:p/>
    <w:p><w:r><w:t>third</w:t></w:r></w:p>
  </w:body>
</w:document>"#
    }

    #[test]
    fn blank_self_closing_paragraph_is_counted() {
        let xml = doc_with_mixed_paragraphs();
        let paras = find_top_level_paragraphs(xml).unwrap();
        assert_eq!(
            paras.len(),
            3,
            "all three top-level paragraphs (incl. blank <w:p/>) must be counted"
        );
        // index 2 must be "third", not skipped.
        let text = extract_text(&xml[paras[2].start..paras[2].end]);
        assert!(
            text.contains("third"),
            "index 2 should be 'third', got '{text}'"
        );
    }

    #[test]
    fn anchors_skip_self_closing_paragraph_can_target_blank() {
        // Targeting the blank <w:p/> at index 1 should expand it with anchors.
        let xml = doc_with_mixed_paragraphs();
        let out = insert_paragraph_anchors(xml, 7, 1, None).unwrap();
        assert!(
            out.contains(r#"<w:commentRangeStart w:id="7"/>"#),
            "start anchor missing: {out}"
        );
        assert!(
            out.contains(r#"<w:commentReference w:id="7"/>"#),
            "reference missing: {out}"
        );
        // Must be well-formed (no leftover self-closing <w:p/> for that paragraph).
        assert!(!out.contains("<w:p/>"));
    }

    fn doc_with_styled_paragraph() -> &'static str {
        // A paragraph with <w:pPr> (heading style) — pPr must stay first child.
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>heading text</w:t></w:r></w:p>
  </w:body>
</w:document>"#
    }

    #[test]
    fn comment_range_start_goes_after_ppr() {
        let xml = doc_with_styled_paragraph();
        let out = insert_paragraph_anchors(xml, 5, 0, Some("heading text")).unwrap();
        // pPr must come BEFORE commentRangeStart.
        let ppr_pos = out.find("<w:pPr>").unwrap();
        let start_pos = out.find(r#"<w:commentRangeStart w:id="5"/>"#).unwrap();
        assert!(
            ppr_pos < start_pos,
            "pPr must precede commentRangeStart; got pPr@{ppr_pos} start@{start_pos}"
        );
        // And pPr should still be the first child of <w:p>.
        let p_open_end = out.find("<w:p>").map(|i| i + "<w:p>".len()).unwrap();
        let after_open = &out[p_open_end..start_pos];
        assert!(
            after_open.contains("</w:pPr>"),
            "nothing but pPr should sit between <w:p> and commentRangeStart"
        );
    }

    #[test]
    fn comment_anchor_after_ppr_with_nested_pprchange() {
        // A paragraph whose pPr contains a <w:pPrChange> revision marker that
        // nests an inner <w:pPr>. The start anchor must land after the OUTER
        // pPr close, not the inner one — otherwise it is spliced inside the
        // property element, producing invalid WordprocessingML.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="Heading1"/><w:pPrChange w:id="9" w:author="a" w:date="d"><w:pPr><w:pStyle w:val="Normal"/></w:pPr></w:pPrChange></w:pPr><w:r><w:t>title</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
        let out = insert_paragraph_anchors(xml, 5, 0, Some("title")).unwrap();
        let start_pos = out.find(r#"<w:commentRangeStart w:id="5"/>"#).unwrap();
        let pprchange_close = out.find("</w:pPrChange>").unwrap();
        let run_pos = out.find("<w:r>").unwrap();
        assert!(
            pprchange_close < start_pos && start_pos < run_pos,
            "commentRangeStart must sit between </w:pPrChange>…</w:pPr> and <w:r>; got pPrChange_close@{pprchange_close} start@{start_pos} run@{run_pos}"
        );
        assert_well_formed(&out);
    }

    const MIN_CT: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    const MIN_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>hello</w:t></w:r></w:p></w:body></w:document>"#;

    fn write_minimal_docx(base: &Path, with_rels: bool) {
        fs::create_dir_all(base.join("word")).unwrap();
        fs::write(base.join("[Content_Types].xml"), MIN_CT).unwrap();
        fs::write(base.join("word/document.xml"), MIN_DOC).unwrap();
        if with_rels {
            fs::create_dir_all(base.join("word/_rels")).unwrap();
            fs::write(
                base.join("word/_rels/document.xml.rels"),
                EMPTY_DOCUMENT_RELS,
            )
            .unwrap();
        }
    }

    #[test]
    fn failed_call_does_not_create_comments_then_retry_registers() {
        // Source doc has NO comments part. A failed call must not leave an
        // empty comments.xml behind, otherwise the retry would see it as
        // pre-existing and skip content-type/relationship registration.
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        write_minimal_docx(base, true);

        // First call fails on an out-of-range paragraph index.
        assert!(add_comment(base, 1, "x", "A", None, 99, None).is_err());
        assert!(
            !base.join("word/comments.xml").exists(),
            "failed call must not create comments.xml"
        );

        // Retry on a valid paragraph succeeds and registers the new part.
        add_comment(base, 1, "x", "A", None, 0, None).unwrap();
        let ct = fs::read_to_string(base.join("[Content_Types].xml")).unwrap();
        assert!(
            ct.contains(r#"PartName="/word/comments.xml""#),
            "comments.xml must be registered in content types after retry: {ct}"
        );
        let rels = fs::read_to_string(base.join("word/_rels/document.xml.rels")).unwrap();
        assert!(
            rels.contains("relationships/comments"),
            "comments relationship must be declared after retry: {rels}"
        );
    }

    #[test]
    fn registers_comments_when_document_rels_missing() {
        // A minimal DOCX without word/_rels/document.xml.rels: the first comment
        // must create that rels file and declare the comments relationship.
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        write_minimal_docx(base, false);
        assert!(!base.join("word/_rels/document.xml.rels").exists());

        add_comment(base, 1, "x", "A", None, 0, None).unwrap();

        let rels_path = base.join("word/_rels/document.xml.rels");
        assert!(rels_path.exists(), "rels file must be created when missing");
        let rels = fs::read_to_string(&rels_path).unwrap();
        assert!(
            rels.contains("relationships/comments"),
            "comments relationship must be declared: {rels}"
        );
    }

    fn assert_well_formed(xml: &str) {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Err(e) => panic!("output is not well-formed XML: {e}\n{xml}"),
                _ => {}
            }
            buf.clear();
        }
    }
}
