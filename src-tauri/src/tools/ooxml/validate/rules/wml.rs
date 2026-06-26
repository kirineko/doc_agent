use super::scan::{attr_local, for_each_element, root_element, ElementStack, ScanEvent};
use crate::tools::ooxml::validate::error::RuleViolation;
use std::collections::{HashMap, HashSet};
use std::path::Path;

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

/// Validate that comments.xml and the WML story parts agree on comment ids.
///
/// Every top-level `<w:comment w:id="X">` in comments.xml must have a matching
/// `<w:commentReference w:id="X"/>` in *some* WML story (document body, headers,
/// footers, footnotes or endnotes), and every reference must have a comment.
/// This catches "comment written but not anchored" (the original bug) and
/// "anchor left dangling".
///
/// References are collected across all story parts, not just `document.xml`: a
/// comment legitimately anchored in a header/footer/footnote has its reference
/// there, so checking only the main body would falsely reject it.
///
/// Reply comments are exempt from the "missing reference" direction: Word links
/// a reply to its parent through `commentsExtended.xml` (`paraIdParent`) and
/// does not place a `commentReference` for the reply, so requiring one would
/// falsely reject valid threaded documents.
pub fn validate_comment_consistency(base: &Path, document_xml: &str) -> Vec<RuleViolation> {
    let comments_xml = std::fs::read_to_string(base.join("word/comments.xml")).ok();
    // A missing comments.xml is treated as an empty comment set: any
    // commentReference in a story part then has no matching comment and must be
    // reported as a dangling reference (rather than silently skipped).
    let comment_ids: HashSet<String> = comments_xml
        .as_deref()
        .map(|xml| collect_ids(xml, "comment"))
        .unwrap_or_default();
    let reference_ids: HashSet<String> = collect_reference_ids(base, document_xml);
    let reply_ids: HashSet<String> = comments_xml
        .as_deref()
        .map(|xml| collect_reply_comment_ids(base, xml))
        .unwrap_or_default();

    // Reply comments legitimately lack a document.xml reference.
    let only_in_comments: Vec<&String> = comment_ids
        .difference(&reference_ids)
        .filter(|id| !reply_ids.contains(*id))
        .collect();
    let only_in_refs: Vec<&String> = reference_ids.difference(&comment_ids).collect();

    let mut v = Vec::new();
    if !only_in_comments.is_empty() {
        let ids: Vec<&str> = only_in_comments.iter().map(|s| s.as_str()).collect();
        v.push(RuleViolation::new(
            "wml.comment.consistency",
            "ISO-IEC29500-4_2016/wml.xsd#CT_Comment_reference",
            format!(
                "comments present without matching commentReference in document.xml: [{}]",
                ids.join(", ")
            ),
            None,
        ));
    }
    if !only_in_refs.is_empty() {
        let ids: Vec<&str> = only_in_refs.iter().map(|s| s.as_str()).collect();
        v.push(RuleViolation::new(
            "wml.comment.consistency",
            "ISO-IEC29500-4_2016/wml.xsd#CT_Comment_reference",
            format!(
                "commentReference present without matching comment in comments.xml: [{}]",
                ids.join(", ")
            ),
            None,
        ));
    }
    v
}

fn collect_ids(xml: &str, element_local: &str) -> HashSet<String> {
    let mut ids = HashSet::new();
    let _ = for_each_element(xml, |ev| {
        let ScanEvent::Start(name, e, _) = ev else {
            return;
        };
        if name == element_local {
            if let Some(id) = attr_local(e, "id") {
                ids.insert(id);
            }
        }
    });
    ids
}

/// Relationship types whose targets are WML story parts that may carry
/// `commentReference` anchors (headers, footers, footnotes, endnotes).
const STORY_REL_TYPES: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes",
];

/// `commentReference` ids across every WML story part a comment can be anchored
/// in: the main document (passed in as `document_xml`) plus headers, footers,
/// footnotes and endnotes **that are referenced from `document.xml.rels`**. Stale
/// unreferenced story files on disk are ignored — Word does not load them.
fn collect_reference_ids(base: &Path, document_xml: &str) -> HashSet<String> {
    let mut ids = collect_ids(document_xml, "commentReference");
    let referenced = collect_referenced_story_targets(base, document_xml);
    if referenced.is_empty() {
        return ids;
    }
    let Ok(entries) = std::fs::read_dir(base.join("word")) else {
        return ids;
    };
    for entry in entries.flatten() {
        let lower = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !referenced.contains(&lower) {
            continue;
        }
        if let Ok(xml) = std::fs::read_to_string(entry.path()) {
            ids.extend(collect_ids(&xml, "commentReference"));
        }
    }
    ids
}

/// Story-part filenames (lowercase, e.g. `header1.xml`) reachable from
/// `word/_rels/document.xml.rels` and actually used by `document.xml`.
///
/// Header/footer rels must be referenced by a matching `headerReference` /
/// `footerReference` (`r:id`); a stale rel alone does not make the story load.
/// Footnotes/endnotes are package-wide — a rel entry is sufficient.
fn collect_referenced_story_targets(base: &Path, document_xml: &str) -> HashSet<String> {
    let rels_path = base.join("word/_rels/document.xml.rels");
    let Ok(rels) = std::fs::read_to_string(&rels_path) else {
        return HashSet::new();
    };
    let used_hf_rids = collect_used_header_footer_rids(document_xml);
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
        let is_header = ty.ends_with("/header");
        let is_footer = ty.ends_with("/footer");
        if is_header || is_footer {
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

/// `r:id` values on `headerReference` / `footerReference` in document.xml.
fn collect_used_header_footer_rids(document_xml: &str) -> HashSet<String> {
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

/// Comment ids (`w:id`) that are threaded replies: their paragraph `paraId`
/// appears in `commentsExtended.xml` with a non-empty `paraIdParent`. These are
/// linked to a parent comment and carry no `commentReference` of their own.
fn collect_reply_comment_ids(base: &Path, comments_xml: &str) -> HashSet<String> {
    let Ok(ce_xml) = std::fs::read_to_string(base.join("word/commentsExtended.xml")) else {
        return HashSet::new();
    };

    let mut para_ids = HashSet::new();
    let mut para_to_comment: HashMap<String, String> = HashMap::new();
    let mut current_comment_id: Option<String> = None;
    let _ = for_each_element(comments_xml, |ev| match ev {
        ScanEvent::Start(name, e, _) => {
            if name == "comment" {
                current_comment_id = attr_local(e, "id");
            } else if name == "p" {
                if let Some(para_id) = attr_local(e, "paraId").filter(|s| !s.is_empty()) {
                    para_ids.insert(para_id.clone());
                    if let Some(ref cid) = current_comment_id {
                        para_to_comment.insert(para_id, cid.clone());
                    }
                }
            }
        }
        ScanEvent::End(name) => {
            if name == "comment" {
                current_comment_id = None;
            }
        }
    });

    let reply_para_ids = collect_verified_reply_para_ids(&ce_xml, &para_ids);
    reply_para_ids
        .into_iter()
        .filter_map(|para_id| para_to_comment.get(&para_id).cloned())
        .collect()
}

/// `paraId`s in commentsExtended.xml whose `paraIdParent` points at an existing
/// comment paragraph. Stale/corrupt metadata with a dangling parent must not
/// exempt a comment from requiring a `commentReference`.
fn collect_verified_reply_para_ids(
    ce_xml: &str,
    parent_para_ids: &HashSet<String>,
) -> HashSet<String> {
    let mut ids = HashSet::new();
    let _ = for_each_element(ce_xml, |ev| {
        let ScanEvent::Start(name, e, _) = ev else {
            return;
        };
        if name == "commentEx" {
            if let Some(parent) = attr_local(e, "paraIdParent").filter(|p| !p.is_empty()) {
                if parent_para_ids.contains(&parent) {
                    if let Some(para_id) = attr_local(e, "paraId") {
                        ids.insert(para_id);
                    }
                }
            }
        }
    });
    ids
}
