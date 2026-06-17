use super::scan::{for_each_element, root_element, ElementStack, ScanEvent};
use crate::tools::ooxml::validate::error::RuleViolation;

const WML_RANGE_MARKUP: &[&str] = &[
    "bookmarkStart",
    "bookmarkEnd",
    "commentRangeStart",
    "commentRangeEnd",
    "moveFromRangeStart",
    "moveFromRangeEnd",
    "moveToRangeStart",
    "moveToRangeEnd",
    "customXmlInsRangeStart",
    "customXmlInsRangeEnd",
    "customXmlDelRangeStart",
    "customXmlDelRangeEnd",
    "customXmlMoveFromRangeStart",
    "customXmlMoveFromRangeEnd",
    "customXmlMoveToRangeStart",
    "customXmlMoveToRangeEnd",
];

const WML_RUN_LEVEL: &[&str] = &[
    "proofErr",
    "permStart",
    "permEnd",
    "ins",
    "del",
    "moveFrom",
    "moveTo",
    "oMath",
    "oMathPara",
];

fn is_wml_run_level(name: &str) -> bool {
    WML_RANGE_MARKUP.contains(&name) || WML_RUN_LEVEL.contains(&name)
}

fn is_wml_block(name: &str) -> bool {
    matches!(name, "p" | "tbl" | "sdt" | "customXml" | "altChunk")
        || is_wml_run_level(name)
        || is_mce_wrapper(name)
}

fn is_mce_wrapper(name: &str) -> bool {
    matches!(name, "AlternateContent" | "Choice" | "Fallback")
}

pub fn validate_wml_document(xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    match root_element(xml) {
        Ok((name, _)) if name == "document" => {}
        Ok((name, _)) => {
            v.push(RuleViolation::new(
                "wml.doc.01",
                "ISO-IEC29500-4_2016/wml.xsd#CT_Document",
                format!("root must be document, got {name}"),
                None,
            ));
            return v;
        }
        Err(e) => {
            v.push(RuleViolation::new(
                "wml.doc.01",
                "ISO-IEC29500-4_2016/wml.xsd#CT_Document",
                format!("parse error: {e}"),
                None,
            ));
            return v;
        }
    }

    let mut stack = ElementStack::default();
    let mut direct_under_body: Vec<(String, usize)> = Vec::new();
    let _ = for_each_element(xml, |ev| match ev {
        ScanEvent::Start(name, _, line) => {
            if stack.current() == Some("body") {
                direct_under_body.push((name.to_string(), line));
                if matches!(name, "r" | "t") {
                    v.push(RuleViolation::new(
                        "wml.body.01",
                        "ISO-IEC29500-4_2016/wml.xsd#EG_ContentBlockContent",
                        format!("body must not contain run-level element '{name}'"),
                        Some(line),
                    ));
                } else if !is_wml_block(name) && name != "sectPr" {
                    v.push(RuleViolation::new(
                        "wml.body.01",
                        "ISO-IEC29500-4_2016/wml.xsd#EG_ContentBlockContent",
                        format!("unexpected element '{name}' under body"),
                        Some(line),
                    ));
                }
            }
            stack.on_start(name);
        }
        ScanEvent::End(name) => stack.on_end(name),
    });

    if let Some(pos) = direct_under_body.iter().position(|(n, _)| n == "sectPr") {
        if direct_under_body[pos + 1..]
            .iter()
            .any(|(n, _)| n != "sectPr")
        {
            v.push(RuleViolation::new(
                "wml.body.02",
                "ISO-IEC29500-4_2016/wml.xsd#CT_Body",
                "sectPr must be the last child of body",
                direct_under_body
                    .iter()
                    .find(|(n, _)| n == "sectPr")
                    .map(|(_, l)| *l),
            ));
        }
    }
    if direct_under_body
        .iter()
        .filter(|(n, _)| n == "sectPr")
        .count()
        > 1
    {
        v.push(RuleViolation::new(
            "wml.body.02",
            "ISO-IEC29500-4_2016/wml.xsd#CT_Body",
            "body must contain at most one sectPr",
            None,
        ));
    }

    validate_wml_tables(xml, &mut v);
    v
}

pub fn validate_wml_glossary(xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    match root_element(xml) {
        Ok((name, _)) if name == "glossaryDocument" => {}
        Ok((name, _)) => {
            v.push(RuleViolation::new(
                "wml.doc.01",
                "ISO-IEC29500-4_2016/wml.xsd#CT_GlossaryDocument",
                format!("root must be glossaryDocument, got {name}"),
                None,
            ));
        }
        Err(e) => v.push(RuleViolation::new(
            "wml.doc.01",
            "ISO-IEC29500-4_2016/wml.xsd#CT_GlossaryDocument",
            format!("parse error: {e}"),
            None,
        )),
    }
    v
}

fn validate_wml_tables(xml: &str, v: &mut Vec<RuleViolation>) {
    struct TblState {
        saw_pr: bool,
        saw_grid: bool,
    }
    struct TcState {
        has_block: bool,
        line: usize,
    }

    let mut stack = ElementStack::default();
    let mut tbl_stack: Vec<TblState> = Vec::new();
    let mut tc_stack: Vec<TcState> = Vec::new();

    let _ = for_each_element(xml, |ev| match ev {
        ScanEvent::Start(name, _, line) => {
            if stack.current() == Some("tbl")
                && !is_wml_run_level(name)
                && !is_mce_wrapper(name)
                && !matches!(name, "tblPr" | "tblGrid" | "tr" | "sdt" | "customXml")
            {
                v.push(RuleViolation::new(
                    "wml.tbl.02",
                    "ISO-IEC29500-4_2016/wml.xsd#EG_ContentRowContent",
                    format!("tbl must not contain '{name}' directly"),
                    Some(line),
                ));
            }
            if name == "tr" && stack.under("tbl") {
                if let Some(state) = tbl_stack.last() {
                    if !state.saw_pr || !state.saw_grid {
                        v.push(RuleViolation::new(
                            "wml.tbl.01",
                            "ISO-IEC29500-4_2016/wml.xsd#CT_Tbl",
                            "tbl must contain tblPr and tblGrid before first tr",
                            Some(line),
                        ));
                    }
                }
            }
            if stack.current() == Some("tr")
                && !is_wml_run_level(name)
                && !is_mce_wrapper(name)
                && !matches!(name, "tc" | "sdt" | "customXml" | "trPr" | "tblPrEx")
            {
                v.push(RuleViolation::new(
                    "wml.tr.01",
                    "ISO-IEC29500-4_2016/wml.xsd#EG_ContentCellContent",
                    format!("tr must not contain '{name}' directly"),
                    Some(line),
                ));
            }
            if stack.current() == Some("tc") && name == "tr" {
                v.push(RuleViolation::new(
                    "wml.tc.02",
                    "ISO-IEC29500-4_2016/wml.xsd#CT_Tc",
                    "tc must not contain tr",
                    Some(line),
                ));
            }
            if stack.under("tc") && is_wml_block(name) {
                if let Some(state) = tc_stack.last_mut() {
                    state.has_block = true;
                }
            }

            if name == "tbl" {
                tbl_stack.push(TblState {
                    saw_pr: false,
                    saw_grid: false,
                });
            } else if name == "tblPr" {
                if stack.under("tbl") {
                    if let Some(state) = tbl_stack.last_mut() {
                        state.saw_pr = true;
                    }
                }
            } else if name == "tblGrid" {
                if stack.under("tbl") {
                    if let Some(state) = tbl_stack.last_mut() {
                        state.saw_grid = true;
                    }
                }
            } else if name == "tc" {
                tc_stack.push(TcState {
                    has_block: false,
                    line,
                });
            }
            stack.on_start(name);
        }
        ScanEvent::End(name) => {
            if name == "tc" {
                if let Some(state) = tc_stack.pop() {
                    if !state.has_block {
                        v.push(RuleViolation::new(
                            "wml.tc.01",
                            "ISO-IEC29500-4_2016/wml.xsd#CT_Tc",
                            "tc must contain at least one block-level element",
                            Some(state.line),
                        ));
                    }
                }
            }
            if name == "tbl" {
                tbl_stack.pop();
            }
            stack.on_end(name);
        }
    });
}
