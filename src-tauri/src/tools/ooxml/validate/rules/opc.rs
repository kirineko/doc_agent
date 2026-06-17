use super::scan::{attr_local, for_each_element, root_element, ScanEvent};
use crate::tools::ooxml::validate::error::RuleViolation;
use std::collections::HashSet;
use std::path::Path;

pub const NS_CONTENT_TYPES: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
pub const NS_RELATIONSHIPS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

#[derive(Debug, Default)]
pub struct ContentTypesIndex {
    pub override_parts: HashSet<String>,
    pub default_extensions: HashSet<String>,
}

impl ContentTypesIndex {
    pub fn is_part_registered(&self, rel_part: &str) -> bool {
        let key = normalize_part(rel_part);
        if self.override_parts.contains(&key) {
            return true;
        }
        if let Some(ext) = part_content_extension(&key) {
            return self.default_extensions.contains(&ext);
        }
        false
    }
}

fn part_content_extension(part: &str) -> Option<String> {
    if part.ends_with(".rels") {
        return Some("rels".into());
    }
    Path::new(part)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

pub fn normalize_part(path: &str) -> String {
    path.trim_start_matches('/')
        .replace('\\', "/")
        .to_lowercase()
}

pub fn parse_content_types_index(xml: &str) -> Result<ContentTypesIndex, quick_xml::Error> {
    let mut index = ContentTypesIndex::default();
    for_each_element(xml, |ev| {
        let ScanEvent::Start(name, e, _) = ev else {
            return;
        };
        match name {
            "Default" => {
                if let Some(ext) = attr_local(e, "Extension") {
                    index.default_extensions.insert(ext.to_lowercase());
                }
            }
            "Override" => {
                if let Some(part) = attr_local(e, "PartName") {
                    index.override_parts.insert(normalize_part(&part));
                }
            }
            _ => {}
        }
    })?;
    Ok(index)
}

pub fn validate_content_types(xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    let root = root_element(xml);
    match root {
        Ok((name, ns)) => {
            if name != "Types" {
                v.push(RuleViolation::new(
                    "opc.ct.01",
                    "ecma/fouth-edition/opc-contentTypes.xsd#CT_Types",
                    format!("root element must be Types, got {name}"),
                    None,
                ));
            }
            if ns.as_deref() != Some(NS_CONTENT_TYPES) {
                v.push(RuleViolation::new(
                    "opc.ct.01",
                    "ecma/fouth-edition/opc-contentTypes.xsd#CT_Types",
                    "root namespace must be OPC content-types",
                    None,
                ));
            }
        }
        Err(e) => {
            v.push(RuleViolation::new(
                "opc.ct.01",
                "ecma/fouth-edition/opc-contentTypes.xsd#CT_Types",
                format!("parse error: {e}"),
                None,
            ));
            return v;
        }
    }

    let _ = for_each_element(xml, |ev| {
        let ScanEvent::Start(name, e, line) = ev else {
            return;
        };
        if name == "Default" {
            for attr in ["Extension", "ContentType"] {
                if attr_local(e, attr).is_none() {
                    v.push(RuleViolation::new(
                        "opc.ct.03",
                        "ecma/fouth-edition/opc-contentTypes.xsd#CT_Default",
                        format!("Default missing @{attr}"),
                        Some(line),
                    ));
                }
            }
        } else if name == "Override" {
            for attr in ["PartName", "ContentType"] {
                if attr_local(e, attr).is_none() {
                    v.push(RuleViolation::new(
                        "opc.ct.04",
                        "ecma/fouth-edition/opc-contentTypes.xsd#CT_Override",
                        format!("Override missing @{attr}"),
                        Some(line),
                    ));
                }
            }
            if let Some(part) = attr_local(e, "PartName") {
                if !part.starts_with('/') {
                    v.push(RuleViolation::new(
                        "opc.ct.04",
                        "ecma/fouth-edition/opc-contentTypes.xsd#CT_Override",
                        format!("PartName must start with '/', got {part}"),
                        Some(line),
                    ));
                }
            }
        } else if name != "Types" {
            v.push(RuleViolation::new(
                "opc.ct.02",
                "ecma/fouth-edition/opc-contentTypes.xsd#CT_Types",
                format!("unexpected child element {name}"),
                Some(line),
            ));
        }
    });
    v
}

pub fn validate_content_types_parts(base: &Path, index: &ContentTypesIndex) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    for entry in walkdir::WalkDir::new(base)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(base)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        if rel.eq_ignore_ascii_case("[Content_Types].xml") {
            continue;
        }
        if !(rel.ends_with(".xml") || rel.ends_with(".rels")) {
            continue;
        }
        if !index.is_part_registered(&rel) {
            v.push(
                RuleViolation::new(
                    "opc.ct.05",
                    "ecma/fouth-edition/opc-contentTypes.xsd#CT_Types",
                    format!("part /{rel} is not registered in [Content_Types].xml"),
                    None,
                )
                .with_part(rel),
            );
        }
    }
    v
}

pub fn validate_relationships(xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    match root_element(xml) {
        Ok((name, ns)) => {
            if name != "Relationships" {
                v.push(RuleViolation::new(
                    "opc.rels.01",
                    "ecma/fouth-edition/opc-relationships.xsd#CT_Relationships",
                    format!("root must be Relationships, got {name}"),
                    None,
                ));
            }
            if ns.as_deref() != Some(NS_RELATIONSHIPS) {
                v.push(RuleViolation::new(
                    "opc.rels.01",
                    "ecma/fouth-edition/opc-relationships.xsd#CT_Relationships",
                    "root namespace must be OPC relationships",
                    None,
                ));
            }
        }
        Err(e) => {
            v.push(RuleViolation::new(
                "opc.rels.01",
                "ecma/fouth-edition/opc-relationships.xsd#CT_Relationships",
                format!("parse error: {e}"),
                None,
            ));
            return v;
        }
    }

    if let Some((id, line)) = super::scan::has_duplicate_attr(xml, "Relationship", "Id") {
        v.push(RuleViolation::new(
            "opc.rels.03",
            "ecma/fouth-edition/opc-relationships.xsd#CT_Relationship",
            format!("duplicate Relationship Id '{id}'"),
            Some(line),
        ));
    }

    let _ = for_each_element(xml, |ev| {
        let ScanEvent::Start(name, e, line) = ev else {
            return;
        };
        if name != "Relationship" {
            if name != "Relationships" {
                v.push(RuleViolation::new(
                    "opc.rels.01",
                    "ecma/fouth-edition/opc-relationships.xsd#CT_Relationships",
                    format!("unexpected element {name}"),
                    Some(line),
                ));
            }
            return;
        }
        for attr in ["Id", "Type", "Target"] {
            if attr_local(e, attr).is_none() {
                v.push(RuleViolation::new(
                    "opc.rels.02",
                    "ecma/fouth-edition/opc-relationships.xsd#CT_Relationship",
                    format!("Relationship missing @{attr}"),
                    Some(line),
                ));
            }
        }
        if let Some(mode) = attr_local(e, "TargetMode") {
            if mode != "Internal" && mode != "External" {
                v.push(RuleViolation::new(
                    "opc.rels.04",
                    "ecma/fouth-edition/opc-relationships.xsd#ST_TargetMode",
                    format!("invalid TargetMode '{mode}'"),
                    Some(line),
                ));
            }
        }
    });
    v
}

pub fn resolve_rel_target(rels_path: &str, target: &str) -> String {
    let target = normalize_target_uri(target);
    if target.starts_with('/') {
        return target.trim_start_matches('/').replace('\\', "/");
    }
    let rel = rels_path.replace('\\', "/");
    let base = if let Some(idx) = rel.find("/_rels/") {
        &rel[..idx]
    } else {
        return target.replace('\\', "/");
    };
    let mut parts: Vec<&str> = base.split('/').filter(|s| !s.is_empty()).collect();
    for seg in target.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            parts.pop();
        } else {
            parts.push(seg);
        }
    }
    parts.join("/")
}

fn normalize_target_uri(target: &str) -> String {
    let path = target
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(target);
    let path = path.split_once('?').map(|(path, _)| path).unwrap_or(path);
    percent_decode(path).replace('\\', "/")
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn internal_part_exists(base: &Path, rel: &str) -> bool {
    let direct = base.join(rel);
    if direct.is_file() {
        return true;
    }
    let rel_lower = rel.replace('\\', "/").to_lowercase();
    for entry in walkdir::WalkDir::new(base)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let entry_rel = path
            .strip_prefix(base)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
            .to_lowercase();
        if entry_rel == rel_lower {
            return true;
        }
    }
    false
}

pub fn validate_relationship_targets(base: &Path, rels_rel: &str, xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    let _ = for_each_element(xml, |ev| {
        let ScanEvent::Start(name, e, line) = ev else {
            return;
        };
        if name != "Relationship" {
            return;
        }
        let mode = attr_local(e, "TargetMode").unwrap_or_else(|| "Internal".into());
        if mode != "Internal" {
            return;
        }
        let Some(target) = attr_local(e, "Target") else {
            return;
        };
        let resolved = resolve_rel_target(rels_rel, &target);
        if !internal_part_exists(base, &resolved) {
            v.push(RuleViolation::new(
                "pkg.rels.01",
                "OPC",
                format!("Internal Target '{target}' resolves to missing part /{resolved}"),
                Some(line),
            ));
        }
    });
    v
}

pub fn is_entry_rels(rels_rel: &str) -> bool {
    matches!(
        rels_rel.replace('\\', "/").to_lowercase().as_str(),
        "word/_rels/document.xml.rels"
            | "ppt/_rels/presentation.xml.rels"
            | "xl/_rels/workbook.xml.rels"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_relative_target() {
        assert_eq!(
            resolve_rel_target("word/_rels/document.xml.rels", "fontTable.xml"),
            "word/fontTable.xml"
        );
    }

    #[test]
    fn resolve_target_strips_uri_suffix_and_decodes_path() {
        assert_eq!(
            resolve_rel_target(
                "ppt/_rels/presentation.xml.rels",
                "slides/slide%201.xml#frag"
            ),
            "ppt/slides/slide 1.xml"
        );
        assert_eq!(
            resolve_rel_target(
                "xl/_rels/workbook.xml.rels",
                "/xl/worksheets/sheet1.xml?rev=1"
            ),
            "xl/worksheets/sheet1.xml"
        );
    }

    #[test]
    fn internal_part_exists_is_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("word")).unwrap();
        std::fs::write(dir.path().join("word/fontTable.xml"), "<x/>").unwrap();
        assert!(internal_part_exists(dir.path(), "word/fonttable.xml"));
    }

    #[test]
    fn package_rels_matches_default_extension() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
</Types>"#;
        let index = parse_content_types_index(xml).unwrap();
        assert!(index.is_part_registered("_rels/.rels"));
        assert!(index.is_part_registered("word/_rels/document.xml.rels"));
    }
}
