use crate::tools::ToolError;
use std::fs;
use std::path::Path;

pub fn accept_changes(src: &Path, dst: &Path) -> Result<(), ToolError> {
    let tmp = dst.with_extension("accept.tmp");
    fs::copy(src, &tmp).map_err(|e| ToolError::Execution(e.to_string()))?;
    let dir = tempfile::tempdir().map_err(|e| ToolError::Execution(e.to_string()))?;
    super::unpack::unpack(&tmp, dir.path(), false)?;
    let doc = dir.path().join("word/document.xml");
    let mut xml = fs::read_to_string(&doc).map_err(|e| ToolError::Execution(e.to_string()))?;
    xml = strip_deletions(&xml);
    xml = unwrap_insertions(&xml);
    fs::write(&doc, xml).map_err(|e| ToolError::Execution(e.to_string()))?;
    super::pack::pack(dir.path(), dst, Some(src))?;
    fs::remove_file(tmp).ok();
    Ok(())
}

fn strip_deletions(xml: &str) -> String {
    remove_balanced_tags(xml, "<w:del", "</w:del>")
}

fn unwrap_insertions(xml: &str) -> String {
    let mut out = xml.to_string();
    while let Some(start) = out.find("<w:ins") {
        let tag_end = out[start..]
            .find('>')
            .map(|i| start + i + 1)
            .unwrap_or(start);
        let close = out[tag_end..]
            .find("</w:ins>")
            .map(|i| tag_end + i)
            .unwrap_or(out.len());
        let inner = out[tag_end..close].to_string();
        out.replace_range(start..close + "</w:ins>".len(), &inner);
    }
    out
}

fn remove_balanced_tags(xml: &str, open_prefix: &str, close_tag: &str) -> String {
    let mut out = xml.to_string();
    while let Some(start) = out.find(open_prefix) {
        let close = out[start..]
            .find(close_tag)
            .map(|i| start + i + close_tag.len())
            .unwrap_or(out.len());
        out.replace_range(start..close, "");
    }
    out
}
