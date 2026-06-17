use crate::tools::ToolError;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

pub fn validate_xml_file(base: &Path, path: &Path) -> Result<(), ToolError> {
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

fn rel(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
