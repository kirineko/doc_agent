use super::ToolError;
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

const MIN_BODY_FOR_HEADING_CHECK: usize = 600;
const OVERLONG_PARA_CHARS: usize = 500;

#[derive(Debug)]
struct ParagraphInfo {
    text: String,
    char_len: usize,
    has_heading: bool,
    has_numpr: bool,
    index: usize,
}

pub fn lint_docx(path: &Path) -> Result<Vec<String>, ToolError> {
    let file = File::open(path).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut zip = ZipArchive::new(file).map_err(|e| ToolError::Execution(e.to_string()))?;
    let document = read_zip_entry(&mut zip, "word/document.xml")?;
    let paragraphs = extract_paragraphs(&document);
    let mut warnings = Vec::new();
    check_missing_headings(&paragraphs, &mut warnings);
    check_east_asia_font(&mut zip, &document, &paragraphs, &mut warnings);
    check_paragraph_issues(&paragraphs, &mut warnings);
    check_table_widths(&document, &mut warnings);
    Ok(warnings)
}

fn read_zip_entry(zip: &mut ZipArchive<File>, name: &str) -> Result<String, ToolError> {
    let mut entry = zip
        .by_name(name)
        .map_err(|e| ToolError::Execution(format!("{name}: {e}")))?;
    let mut buf = String::new();
    std::io::Read::read_to_string(&mut entry, &mut buf)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(buf)
}

fn extract_paragraphs(document_xml: &str) -> Vec<ParagraphInfo> {
    split_paragraph_chunks(document_xml)
        .into_iter()
        .enumerate()
        .map(|(idx, chunk)| {
            let text = extract_text_from_para(&chunk);
            let char_len = text.chars().count();
            ParagraphInfo {
                text,
                char_len,
                has_heading: chunk.contains("w:pStyle")
                    && (chunk.contains("Heading") || chunk.contains("heading")),
                has_numpr: chunk.contains("w:numPr"),
                index: idx + 1,
            }
        })
        .collect()
}

fn split_paragraph_chunks(xml: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end = after
            .find("</w:p>")
            .map(|i| i + "</w:p>".len())
            .unwrap_or(after.len());
        chunks.push(after[..end].to_string());
        rest = &after[end..];
    }
    chunks
}

fn extract_text_from_para(chunk: &str) -> String {
    let mut text = String::new();
    let mut rest = chunk;
    while let Some(start) = rest.find("<w:t") {
        let after = &rest[start..];
        let content_start = after.find('>').map(|i| i + 1).unwrap_or(after.len());
        let content_end = after[content_start..]
            .find("</w:t>")
            .map(|i| content_start + i)
            .unwrap_or(after.len());
        text.push_str(&after[content_start..content_end]);
        rest = &after[content_end..];
    }
    text
}

fn body_paragraphs(paragraphs: &[ParagraphInfo]) -> impl Iterator<Item = &ParagraphInfo> {
    paragraphs.iter().filter(|p| !p.has_heading)
}

fn total_body_chars(paragraphs: &[ParagraphInfo]) -> usize {
    body_paragraphs(paragraphs).map(|p| p.char_len).sum()
}

fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c,
            '\u{4E00}'..='\u{9FFF}'
                | '\u{3400}'..='\u{4DBF}'
                | '\u{F900}'..='\u{FAFF}'
        )
    })
}

fn check_missing_headings(paragraphs: &[ParagraphInfo], warnings: &mut Vec<String>) {
    let body_chars = total_body_chars(paragraphs);
    if body_chars > MIN_BODY_FOR_HEADING_CHECK && !paragraphs.iter().any(|p| p.has_heading) {
        warnings.push(format!(
            "全文超过 {MIN_BODY_FOR_HEADING_CHECK} 字但没有任何标题样式，请用 HeadingLevel 分层"
        ));
    }
}

fn check_east_asia_font(
    zip: &mut ZipArchive<File>,
    document: &str,
    paragraphs: &[ParagraphInfo],
    warnings: &mut Vec<String>,
) {
    if !body_paragraphs(paragraphs).any(|p| contains_cjk(&p.text)) {
        return;
    }
    if document.contains("eastAsia") {
        return;
    }
    let styles = read_zip_entry(zip, "word/styles.xml").unwrap_or_default();
    if styles.contains("eastAsia") {
        return;
    }
    warnings.push(
        "文档含中文但未配置 eastAsia 字体，中文将回退为默认衬线字体；请在 styles.default 中设置 font.eastAsia（如微软雅黑）"
            .into(),
    );
}

fn check_paragraph_issues(paragraphs: &[ParagraphInfo], warnings: &mut Vec<String>) {
    for para in paragraphs {
        if para.char_len > OVERLONG_PARA_CHARS {
            warnings.push(format!(
                "第 {} 段超过 {OVERLONG_PARA_CHARS} 字（当前 {} 字），建议拆分段落或转为列表",
                para.index, para.char_len
            ));
        }
        if para.has_numpr {
            continue;
        }
        let trimmed = para.text.trim_start();
        if trimmed.starts_with('•')
            || trimmed.starts_with('·')
            || trimmed.starts_with('●')
            || manual_numbered_prefix(trimmed)
        {
            warnings.push(format!(
                "第 {} 段检测到手工输入的项目符号，请改用 numbering 配置",
                para.index
            ));
        }
    }
}

fn manual_numbered_prefix(text: &str) -> bool {
    let mut digits = 0;
    for c in text.chars() {
        if c.is_ascii_digit() {
            digits += 1;
            continue;
        }
        if c == '.' && digits > 0 {
            return true;
        }
        break;
    }
    false
}

fn check_table_widths(document: &str, warnings: &mut Vec<String>) {
    let mut rest = document;
    let mut table_index = 0;
    while let Some(start) = rest.find("<w:tbl") {
        table_index += 1;
        let after = &rest[start..];
        let end = after
            .find("</w:tbl>")
            .map(|i| i + "</w:tbl>".len())
            .unwrap_or(after.len());
        let chunk = &after[..end];
        let has_tblw = chunk.contains("w:tblW");
        let has_grid = chunk.contains("w:gridCol");
        if !has_tblw || !has_grid {
            warnings.push(format!(
                "第 {table_index} 个表格未设置 columnWidths/width，跨平台渲染会变形"
            ));
        }
        rest = &after[end..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn write_test_docx(path: &Path, document_xml: &str, styles_xml: Option<&str>) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        zip.start_file("word/document.xml", opts).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        if let Some(styles) = styles_xml {
            zip.start_file("word/styles.xml", opts).unwrap();
            zip.write_all(styles.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn w1_missing_headings_warns() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("no-heading.docx");
        let body = "这是一段很长的正文内容，用于触发缺少标题分层的检查规则。".repeat(25);
        write_test_docx(
            &path,
            &format!(
                r#"<w:document><w:body><w:p><w:r><w:t>{body}</w:t></w:r></w:p></w:body></w:document>"#
            ),
            None,
        );
        let warnings = lint_docx(&path).unwrap();
        assert!(warnings.iter().any(|w| w.contains("标题样式")));
    }

    #[test]
    fn w2_missing_east_asia_font_warns() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("no-font.docx");
        write_test_docx(
            &path,
            r#"<w:document><w:body><w:p><w:r><w:t>中文正文</w:t></w:r></w:p></w:body></w:document>"#,
            None,
        );
        let warnings = lint_docx(&path).unwrap();
        assert!(warnings.iter().any(|w| w.contains("eastAsia")));
    }

    #[test]
    fn w3_overlong_paragraph_warns() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("long-para.docx");
        let body = "字".repeat(600);
        write_test_docx(
            &path,
            &format!(
                r#"<w:document><w:body><w:p><w:r><w:t>{body}</w:t></w:r></w:p></w:body></w:document>"#
            ),
            None,
        );
        let warnings = lint_docx(&path).unwrap();
        assert!(warnings.iter().any(|w| w.contains("超过")));
    }

    #[test]
    fn w4_manual_bullet_warns() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bullet.docx");
        write_test_docx(
            &path,
            r#"<w:document><w:body><w:p><w:r><w:t>• 手工列表项</w:t></w:r></w:p></w:body></w:document>"#,
            None,
        );
        let warnings = lint_docx(&path).unwrap();
        assert!(warnings.iter().any(|w| w.contains("项目符号")));
    }

    #[test]
    fn w5_table_missing_width_warns() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("table.docx");
        write_test_docx(
            &path,
            r#"<w:document><w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>单元格</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#,
            None,
        );
        let warnings = lint_docx(&path).unwrap();
        assert!(warnings.iter().any(|w| w.contains("表格")));
    }

    #[test]
    fn well_formed_docx_has_no_warnings() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("good.docx");
        let styles = r#"<w:styles><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:eastAsia="微软雅黑"/></w:rPr></w:rPrDefault></w:docDefaults></w:styles>"#;
        let document = r#"<w:document><w:body>
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>标题</w:t></w:r></w:p>
<w:p><w:r><w:t>简短正文。</w:t></w:r></w:p>
<w:tbl><w:tblPr><w:tblW w:w="5000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid>
<w:tr><w:tc><w:p><w:r><w:t>单元格</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
</w:body></w:document>"#;
        write_test_docx(&path, document, Some(styles));
        let warnings = lint_docx(&path).unwrap();
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    }

    #[test]
    fn corrupt_zip_returns_error_not_panic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.docx");
        std::fs::write(&path, b"not a zip").unwrap();
        assert!(lint_docx(&path).is_err());
    }
}
