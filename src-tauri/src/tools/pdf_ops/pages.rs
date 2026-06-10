use crate::tools::{ensure_parent_dir, ToolError};
use lopdf::{Document, Object};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub fn split_by_ranges(src: &Path, ranges: &str, out: &Path) -> Result<u32, ToolError> {
    let mut doc = load_doc(src)?;
    let total = page_count(&doc)?;
    let keep = parse_ranges(ranges, total)?;
    let delete = complement_pages(&doc, &keep)?;
    doc.delete_pages(&delete);
    ensure_parent_dir(out)?;
    save_doc(&mut doc, out)?;
    Ok(keep.len() as u32)
}

pub fn split_burst(src: &Path, out_dir: &Path) -> Result<Vec<String>, ToolError> {
    let probe = load_doc(src)?;
    let total = page_count(&probe)?;
    fs::create_dir_all(out_dir).map_err(|e| ToolError::Execution(e.to_string()))?;

    let mut files = Vec::with_capacity(total as usize);
    for page in 1..=total {
        let mut doc = load_doc(src)?;
        let delete: Vec<u32> = (1..=total).filter(|p| *p != page).collect();
        doc.delete_pages(&delete);
        let out = out_dir.join(format!("page_{page}.pdf"));
        save_doc(&mut doc, &out)?;
        files.push(out.display().to_string());
    }
    Ok(files)
}

pub fn rotate_pages(
    src: &Path,
    out: &Path,
    rotation: i32,
    pages: Option<&[u32]>,
    relative: bool,
) -> Result<u32, ToolError> {
    validate_rotation(rotation)?;
    let mut doc = load_doc(src)?;
    let total = page_count(&doc)?;
    let targets: Vec<u32> = match pages {
        Some(list) => {
            validate_page_list(list, total)?;
            list.to_vec()
        }
        None => (1..=total).collect(),
    };

    let rotated = targets.len() as u32;
    for page in &targets {
        let page_id = *doc
            .get_pages()
            .get(page)
            .ok_or_else(|| ToolError::Execution(format!("page {page} not found")))?;
        let dict = doc
            .get_object_mut(page_id)
            .map_err(|e| ToolError::Execution(e.to_string()))?
            .as_dict_mut()
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let angle = if relative {
            let current = dict.get(b"Rotate").and_then(|o| o.as_i64()).unwrap_or(0) as i32;
            normalize_rotation(current + rotation)
        } else {
            normalize_rotation(rotation)
        };
        dict.set("Rotate", Object::Integer(angle as i64));
    }

    ensure_parent_dir(out)?;
    save_doc(&mut doc, out)?;
    Ok(rotated)
}

pub fn delete_pages(src: &Path, out: &Path, pages: &[u32]) -> Result<u32, ToolError> {
    let mut doc = load_doc(src)?;
    let total = page_count(&doc)?;
    if pages.is_empty() {
        return Err(ToolError::InvalidArgs("pages 不能为空".into()));
    }
    validate_page_list(pages, total)?;
    let unique: Vec<u32> = pages
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if unique.len() >= total as usize {
        return Err(ToolError::InvalidArgs("不能删除所有页".into()));
    }
    doc.delete_pages(&unique);
    ensure_parent_dir(out)?;
    save_doc(&mut doc, out)?;
    page_count(&doc)
}

pub fn parse_ranges(spec: &str, total: u32) -> Result<Vec<u32>, ToolError> {
    let mut pages = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start, end)) = part.split_once('-') {
            let start: u32 = parse_page_num(start.trim(), "ranges")?;
            let end: u32 = parse_page_num(end.trim(), "ranges")?;
            if start > end {
                return Err(ToolError::InvalidArgs(format!("无效范围 {start}-{end}")));
            }
            pages.extend(start..=end);
        } else {
            pages.push(parse_page_num(part, "ranges")?);
        }
    }
    if pages.is_empty() {
        return Err(ToolError::InvalidArgs("ranges 不能为空".into()));
    }
    pages.sort_unstable();
    pages.dedup();
    validate_page_list(&pages, total)?;
    Ok(pages)
}

pub fn parse_page_array(values: &[Value], total: u32) -> Result<Vec<u32>, ToolError> {
    if values.is_empty() {
        return Err(ToolError::InvalidArgs("pages 不能为空".into()));
    }
    let mut pages = Vec::with_capacity(values.len());
    for value in values {
        let page = value
            .as_u64()
            .ok_or_else(|| ToolError::InvalidArgs("pages 须为整数数组".into()))?
            as u32;
        pages.push(page);
    }
    pages.sort_unstable();
    pages.dedup();
    validate_page_list(&pages, total)?;
    Ok(pages)
}

fn load_doc(path: &Path) -> Result<Document, ToolError> {
    Document::load(path).map_err(|e| ToolError::Execution(format!("load {}: {e}", path.display())))
}

fn save_doc(doc: &mut Document, path: &Path) -> Result<(), ToolError> {
    doc.save(path)
        .map(|_| ())
        .map_err(|e| ToolError::Execution(format!("save {}: {e}", path.display())))
}

fn page_count(doc: &Document) -> Result<u32, ToolError> {
    let count = doc.get_pages().len();
    if count == 0 {
        return Err(ToolError::Execution("PDF 无页面".into()));
    }
    Ok(count as u32)
}

fn complement_pages(doc: &Document, keep: &[u32]) -> Result<Vec<u32>, ToolError> {
    let keep_set: BTreeSet<u32> = keep.iter().copied().collect();
    Ok(doc
        .get_pages()
        .keys()
        .copied()
        .filter(|p| !keep_set.contains(p))
        .collect())
}

fn parse_page_num(raw: &str, field: &str) -> Result<u32, ToolError> {
    raw.parse::<u32>()
        .map_err(|_| ToolError::InvalidArgs(format!("{field} 含无效页码: {raw}")))
}

fn validate_page_list(pages: &[u32], total: u32) -> Result<(), ToolError> {
    for &page in pages {
        if page == 0 || page > total {
            return Err(ToolError::InvalidArgs(format!(
                "页码 {page} 越界（共 {total} 页）"
            )));
        }
    }
    Ok(())
}

fn validate_rotation(rotation: i32) -> Result<(), ToolError> {
    if rotation % 90 != 0 {
        return Err(ToolError::InvalidArgs("旋转角度必须为 90 的倍数".into()));
    }
    Ok(())
}

fn normalize_rotation(angle: i32) -> i32 {
    let mut value = angle % 360;
    if value < 0 {
        value += 360;
    }
    value
}
