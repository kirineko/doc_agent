use super::scan::{
    attr_local, count_effective_child, for_each_element, root_element, ElementStack, ScanEvent,
};
use crate::tools::ooxml::validate::error::RuleViolation;

const PML_PRES_CHILDREN: &[&str] = &[
    "sldMasterIdLst",
    "notesMasterIdLst",
    "handoutMasterIdLst",
    "sldIdLst",
    "sldSz",
    "notesSz",
    "smartTags",
    "embeddedFontLst",
    "custShowLst",
    "photoAlbum",
    "custDataLst",
    "kinsoku",
    "defaultTextStyle",
    "modifyVerifier",
    "extLst",
];

fn is_mce_wrapper(name: &str) -> bool {
    matches!(name, "AlternateContent" | "Choice" | "Fallback")
}

pub fn validate_presentation(xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    match root_element(xml) {
        Ok((name, _)) if name == "presentation" => {}
        Ok((name, _)) => {
            v.push(RuleViolation::new(
                "pml.pres.01",
                "ISO-IEC29500-4_2016/pml.xsd#CT_Presentation",
                format!("root must be presentation, got {name}"),
                None,
            ));
            return v;
        }
        Err(e) => {
            v.push(RuleViolation::new(
                "pml.pres.01",
                "ISO-IEC29500-4_2016/pml.xsd#CT_Presentation",
                format!("parse error: {e}"),
                None,
            ));
            return v;
        }
    }

    let notes_sz = count_effective_child(xml, "presentation", "notesSz").unwrap_or(0);
    if notes_sz != 1 {
        v.push(RuleViolation::new(
            "pml.pres.03",
            "ISO-IEC29500-4_2016/pml.xsd#CT_Presentation",
            format!("presentation must contain exactly one notesSz, found {notes_sz}"),
            None,
        ));
    }

    let mut stack = ElementStack::default();
    let _ = for_each_element(xml, |ev| match ev {
        ScanEvent::Start(name, e, line) => {
            if stack.current() == Some("presentation") {
                if !PML_PRES_CHILDREN.contains(&name) && !is_mce_wrapper(name) {
                    v.push(RuleViolation::new(
                        "pml.pres.02",
                        "ISO-IEC29500-4_2016/pml.xsd#CT_Presentation",
                        format!("unexpected child '{name}' under presentation"),
                        Some(line),
                    ));
                }
                if name == "sldSz" {
                    for attr in ["cx", "cy"] {
                        if attr_local(e, attr).is_none() {
                            v.push(RuleViolation::new(
                                "pml.pres.04",
                                "ISO-IEC29500-4_2016/pml.xsd#CT_SlideSize",
                                format!("sldSz missing @{attr}"),
                                Some(line),
                            ));
                        }
                    }
                }
            }
            stack.on_start(name);
        }
        ScanEvent::End(name) => stack.on_end(name),
    });
    v
}

pub fn validate_slide(xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    match root_element(xml) {
        Ok((name, _)) if name == "sld" => {}
        Ok((name, _)) => {
            v.push(RuleViolation::new(
                "pml.sld.01",
                "ISO-IEC29500-4_2016/pml.xsd#CT_Slide",
                format!("root must be sld, got {name}"),
                None,
            ));
            return v;
        }
        Err(e) => {
            v.push(RuleViolation::new(
                "pml.sld.01",
                "ISO-IEC29500-4_2016/pml.xsd#CT_Slide",
                format!("parse error: {e}"),
                None,
            ));
            return v;
        }
    }

    let mut stack = ElementStack::default();
    let mut first_child: Option<(String, usize)> = None;
    let _ = for_each_element(xml, |ev| match ev {
        ScanEvent::Start(name, _, line) => {
            let is_visible_first_child = stack.current() == Some("sld") && !is_mce_wrapper(name);
            let is_mce_wrapped_csld = name == "cSld" && stack.under("AlternateContent");
            if first_child.is_none() && (is_visible_first_child || is_mce_wrapped_csld) {
                first_child = Some((name.to_string(), line));
            }
            stack.on_start(name);
        }
        ScanEvent::End(name) => stack.on_end(name),
    });
    match first_child {
        Some((n, line)) if n != "cSld" => {
            v.push(RuleViolation::new(
                "pml.sld.02",
                "ISO-IEC29500-4_2016/pml.xsd#CT_Slide",
                format!("first child of sld must be cSld, got {n}"),
                Some(line),
            ));
        }
        None => {
            v.push(RuleViolation::new(
                "pml.sld.02",
                "ISO-IEC29500-4_2016/pml.xsd#CT_Slide",
                "sld must contain cSld",
                None,
            ));
        }
        _ => {}
    }
    v
}
