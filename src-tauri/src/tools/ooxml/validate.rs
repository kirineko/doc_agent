use crate::tools::ToolError;
use quick_xml::events::Event;
use quick_xml::Reader;
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
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.map_err(|e| ToolError::Execution(e.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("xml") {
            validate_xml_file(dir, path)?;
        }
    }
    Ok(())
}

fn validate_xml_file(base: &Path, path: &Path) -> Result<(), ToolError> {
    let text = std::fs::read_to_string(path).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut reader = Reader::from_str(&text);
    reader.config_mut().trim_text(true);
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => {
                return Err(ToolError::Execution(format!(
                    "{}: XML parse error: {e}",
                    rel(base, path)
                )));
            }
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

fn rel(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
