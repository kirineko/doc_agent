use super::csv_row;
use crate::tools::ToolError;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::read::ZipArchive;

pub fn extract_tables(
    src: &Path,
    out_dir: &Path,
    table_index: Option<usize>,
) -> Result<Vec<String>, ToolError> {
    fs::create_dir_all(out_dir).map_err(|e| ToolError::Execution(e.to_string()))?;
    let xml = read_document_xml(src)?;
    let tables = parse_tables(&xml);
    let mut files = Vec::new();
    for (idx, table) in tables.into_iter().enumerate() {
        if let Some(want) = table_index {
            if idx != want {
                continue;
            }
        }
        let file_name = format!("table_{idx}.csv");
        let out = out_dir.join(&file_name);
        write_csv(&out, &table)?;
        files.push(out.display().to_string());
    }
    Ok(files)
}

fn read_document_xml(src: &Path) -> Result<String, ToolError> {
    let file = File::open(src).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut archive = ZipArchive::new(file).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut doc = archive
        .by_name("word/document.xml")
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut xml = String::new();
    doc.read_to_string(&mut xml)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(xml)
}

fn parse_tables(xml: &str) -> Vec<Vec<Vec<String>>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut tables = Vec::new();
    let mut current_table: Option<Vec<Vec<String>>> = None;
    let mut current_row: Option<Vec<String>> = None;
    let mut current_cell = String::new();
    let mut in_tbl = false;
    let mut in_tr = false;
    let mut in_tc = false;
    let mut in_t = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                match name.as_str() {
                    "w:tbl" => {
                        in_tbl = true;
                        current_table = Some(Vec::new());
                    }
                    "w:tr" if in_tbl => {
                        in_tr = true;
                        current_row = Some(Vec::new());
                    }
                    "w:tc" if in_tr => {
                        in_tc = true;
                        current_cell.clear();
                    }
                    "w:t" if in_tc => in_t = true,
                    _ => {}
                }
            }
            Ok(Event::Text(t)) if in_t => {
                current_cell.push_str(&t.unescape().unwrap_or_default());
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                match name.as_str() {
                    "w:t" => in_t = false,
                    "w:tc" if in_tc => {
                        in_tc = false;
                        if let Some(row) = current_row.as_mut() {
                            row.push(current_cell.clone());
                        }
                    }
                    "w:tr" if in_tr => {
                        in_tr = false;
                        if let (Some(table), Some(row)) =
                            (current_table.as_mut(), current_row.take())
                        {
                            table.push(row);
                        }
                    }
                    "w:tbl" if in_tbl => {
                        in_tbl = false;
                        if let Some(table) = current_table.take() {
                            tables.push(table);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    tables
}

fn write_csv(path: &Path, table: &[Vec<String>]) -> Result<(), ToolError> {
    let mut file = fs::File::create(path).map_err(|e| ToolError::Execution(e.to_string()))?;
    for row in table {
        let line = csv_row(row.iter().map(String::as_str));
        writeln!(file, "{line}").map_err(|e| ToolError::Execution(e.to_string()))?;
    }
    Ok(())
}
