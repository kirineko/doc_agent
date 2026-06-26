pub(crate) mod opc;
pub(crate) mod pml;
pub(crate) mod scan;
pub(crate) mod sml;
pub(crate) mod wml;

use crate::tools::ooxml::validate::error::RuleViolation;
use std::path::Path;

pub use opc::ContentTypesIndex;

pub fn validate_part_structure(base: &Path, rel_part: &str, xml: &str) -> Vec<RuleViolation> {
    let rel = rel_part.replace('\\', "/").to_lowercase();
    if rel.ends_with(".rels") {
        let mut v = opc::validate_relationships(xml);
        if opc::is_entry_rels(&rel) {
            v.extend(opc::validate_relationship_targets(base, &rel, xml));
        }
        return v;
    }
    if rel == "[content_types].xml" {
        return opc::validate_content_types(xml);
    }
    match rel.as_str() {
        "word/document.xml" => {
            let mut v = wml::validate_wml_document(xml);
            v.extend(wml::validate_comment_consistency(base, xml));
            v
        }
        "word/glossarydocument.xml" => wml::validate_wml_glossary(xml),
        "ppt/presentation.xml" => pml::validate_presentation(xml),
        "xl/workbook.xml" => sml::validate_workbook(xml),
        _ if rel.starts_with("ppt/slides/") && rel.ends_with(".xml") => pml::validate_slide(xml),
        _ if rel.starts_with("xl/worksheets/") && rel.ends_with(".xml") => {
            sml::validate_worksheet(xml)
        }
        _ => Vec::new(),
    }
}

pub fn parse_content_types(xml: &str) -> Result<ContentTypesIndex, quick_xml::Error> {
    opc::parse_content_types_index(xml)
}

pub fn validate_package_parts(base: &Path, index: &ContentTypesIndex) -> Vec<RuleViolation> {
    opc::validate_content_types_parts(base, index)
}
