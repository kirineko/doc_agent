mod error;
mod rules;
mod wellformed;

#[cfg(test)]
mod tests;

use crate::tools::ToolError;
use error::violations_to_error;
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

pub fn validate_dir(dir: &Path, _original: Option<&Path>) -> Result<(), ToolError> {
    let content_types = dir.join("[Content_Types].xml");
    if !content_types.exists() {
        return Err(ToolError::Execution(
            "[Content_Types].xml: missing required part".into(),
        ));
    }

    let ct_xml =
        std::fs::read_to_string(&content_types).map_err(|e| ToolError::Execution(e.to_string()))?;
    wellformed::validate_xml_file(dir, &content_types)?;

    let mut ct_violations = rules::validate_part_structure(dir, "[Content_Types].xml", &ct_xml);
    let index =
        rules::parse_content_types(&ct_xml).map_err(|e| ToolError::Execution(e.to_string()))?;
    ct_violations.extend(rules::validate_package_parts(dir, &index));
    if !ct_violations.is_empty() {
        return Err(ToolError::Execution(violations_to_error(
            "[Content_Types].xml",
            ct_violations,
        )));
    }

    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.map_err(|e| ToolError::Execution(e.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("xml")
            && path.extension().and_then(|e| e.to_str()) != Some("rels")
        {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("[Content_Types].xml") {
            continue;
        }
        wellformed::validate_xml_file(dir, path)?;
        let rel = rel_part(dir, path);
        let text =
            std::fs::read_to_string(path).map_err(|e| ToolError::Execution(e.to_string()))?;
        let violations = rules::validate_part_structure(dir, &rel, &text);
        if !violations.is_empty() {
            return Err(ToolError::Execution(violations_to_error(&rel, violations)));
        }
    }
    Ok(())
}

pub fn roundtrip_check(path: &Path) -> Result<(), ToolError> {
    let file = File::open(path).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut archive = ZipArchive::new(file).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut found = false;
    for i in 0..archive.len() {
        let name = archive
            .by_index(i)
            .map_err(|e| ToolError::Execution(e.to_string()))?
            .name()
            .to_string();
        if name.eq_ignore_ascii_case("[Content_Types].xml") {
            found = true;
        }
    }
    if !found {
        return Err(ToolError::Execution(
            "roundtrip: missing [Content_Types].xml".into(),
        ));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    if ext == "xlsx" {
        let _: calamine::Xlsx<_> = calamine::open_workbook(path)
            .map_err(|e| ToolError::Execution(format!("roundtrip xlsx: {e}")))?;
    }
    Ok(())
}

fn rel_part(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
