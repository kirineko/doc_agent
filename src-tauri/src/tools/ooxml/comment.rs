use crate::tools::ToolError;
use chrono::Utc;
use std::fs;
use std::path::Path;

pub fn add_comment(
    dir: &Path,
    id: u32,
    text: &str,
    author: &str,
    parent: Option<u32>,
) -> Result<(), ToolError> {
    let comments_path = dir.join("word/comments.xml");
    ensure_comments_file(&comments_path, author)?;
    let date = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let initials: String = author
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
    let para_id = format!("{:08X}", id.wrapping_mul(0x9E37_79B9));
    let entry = format!(
        r#"<w:comment w:id="{id}" w:author="{author}" w:date="{date}" w:initials="{initials}">
  <w:p w14:paraId="{para_id}" w14:textId="77777777">
    <w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:annotationRef/></w:r>
    <w:r><w:rPr><w:color w:val="000000"/><w:sz w:val="20"/><w:szCs w:val="20"/></w:rPr>
      <w:t>{text}</w:t></w:r>
  </w:p>
</w:comment>"#
    );
    let mut xml =
        fs::read_to_string(&comments_path).map_err(|e| ToolError::Execution(e.to_string()))?;
    if let Some(pos) = xml.rfind("</w:comments>") {
        xml.insert_str(pos, &entry);
    }
    fs::write(comments_path, xml).map_err(|e| ToolError::Execution(e.to_string()))?;
    if parent.is_some() {
        // Reply linkage is recorded in commentsExtended in full port; MVP stores comment body only.
    }
    Ok(())
}

fn ensure_comments_file(path: &Path, _author: &str) -> Result<(), ToolError> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::Execution(e.to_string()))?;
    }
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
</w:comments>"#;
    fs::write(path, xml).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(())
}
