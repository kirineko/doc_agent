use pdfium_render::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::tools::pdf_cache::{self, page_image_rel, RenderManifest};
use crate::tools::pdf_text_quality::format_extracted_text;

static PDFIUM: OnceLock<Result<Mutex<Pdfium>, String>> = OnceLock::new();

pub struct RenderPagesResult {
    pub cache_key: String,
    pub cache_hit: bool,
    pub manifest: RenderManifest,
}

pub fn page_count(path: &Path) -> Result<u32, String> {
    let guard = pdfium_instance()?
        .lock()
        .map_err(|_| "PDFium lock poisoned".to_string())?;
    let document = guard
        .load_pdf_from_file(path, None)
        .map_err(|e| format!("PDF 打开失败: {e}"))?;
    Ok(document.pages().len() as u32)
}

pub fn render_pages_cached(
    sandbox_root: &Path,
    rel_path: &str,
    abs_path: &Path,
    dpi: u32,
    pages_spec: Option<&str>,
) -> Result<RenderPagesResult, String> {
    let total = page_count(abs_path)?;
    let (page_list, pages_spec_norm) = pdf_cache::parse_pages_spec(pages_spec, total)?;
    let fingerprint = pdf_cache::fingerprint_from_path(rel_path, abs_path, dpi, &pages_spec_norm)?;
    let key = pdf_cache::cache_key(&fingerprint);

    if let Some(manifest) = pdf_cache::try_cache_hit(sandbox_root, &fingerprint, &page_list) {
        return Ok(RenderPagesResult {
            cache_key: key,
            cache_hit: true,
            manifest,
        });
    }

    pdf_cache::with_render_lock(&key, || {
        if let Some(manifest) = pdf_cache::try_cache_hit(sandbox_root, &fingerprint, &page_list) {
            return Ok(RenderPagesResult {
                cache_key: key.clone(),
                cache_hit: true,
                manifest,
            });
        }

        let cache_abs = sandbox_root.join(pdf_cache::cache_dir_rel(&key));
        pdf_cache::clear_cache_dir(&cache_abs)?;
        fs::create_dir_all(&cache_abs).map_err(|e| e.to_string())?;

        let guard = pdfium_instance()?
            .lock()
            .map_err(|_| "PDFium lock poisoned".to_string())?;
        let document = guard
            .load_pdf_from_file(abs_path, None)
            .map_err(|e| format!("PDF 打开失败: {e}"))?;

        let mut entries = Vec::with_capacity(page_list.len());
        for page_no in &page_list {
            let index = (*page_no - 1) as i32;
            let page = document
                .pages()
                .get(index)
                .map_err(|e| format!("PDF 第 {page_no} 页读取失败: {e}"))?;
            let width_pts = page.width().value;
            let target_width = ((width_pts / 72.0) * dpi as f32).round().max(1.0) as i32;
            let config = PdfRenderConfig::new().set_target_width(target_width);
            let bitmap = page
                .render_with_config(&config)
                .map_err(|e| format!("PDF 第 {page_no} 页渲染失败: {e}"))?;
            let image = bitmap
                .as_image()
                .map_err(|e| format!("PDF 第 {page_no} 页转图像失败: {e}"))?;
            let rel_image = page_image_rel(&key, *page_no);
            let out_abs = sandbox_root.join(&rel_image);
            if let Some(parent) = out_abs.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            image
                .save(&out_abs)
                .map_err(|e| format!("保存 {rel_image} 失败: {e}"))?;
            entries.push(pdf_cache::PageEntry {
                index: *page_no,
                path: rel_image,
            });
        }

        let manifest = RenderManifest {
            version: 1,
            source_path: fingerprint.rel_path.clone(),
            source_size: fingerprint.size,
            source_mtime_secs: fingerprint.mtime_secs,
            dpi: fingerprint.dpi,
            pages_spec: fingerprint.pages_spec.clone(),
            page_count: entries.len() as u32,
            pages: entries,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        pdf_cache::write_manifest(&cache_abs, &manifest)?;

        Ok(RenderPagesResult {
            cache_key: key.clone(),
            cache_hit: false,
            manifest,
        })
    })
}

pub fn configure_resource_dir(path: PathBuf) {
    std::env::set_var("DOC_AGENT_PDFIUM_DIR", path);
}

pub struct PageText {
    pub index: u32,
    pub text: String,
}

pub fn extract_text_pages(path: &Path, pages_spec: Option<&str>) -> Result<Vec<PageText>, String> {
    let guard = pdfium_instance()?
        .lock()
        .map_err(|_| "PDFium lock poisoned".to_string())?;

    let document = guard
        .load_pdf_from_file(path, None)
        .map_err(|e| format!("PDF 打开失败: {e}"))?;

    let total = document.pages().len() as u32;
    let (page_list, _) = pdf_cache::parse_pages_spec(pages_spec, total)?;

    let mut pages = Vec::with_capacity(page_list.len());
    for page_no in page_list {
        let index = (page_no - 1) as i32;
        let page = document
            .pages()
            .get(index)
            .map_err(|e| format!("PDF 第 {page_no} 页读取失败: {e}"))?;
        let page_text = page
            .text()
            .map_err(|e| format!("PDF 第 {page_no} 页文本提取失败: {e}"))?;
        pages.push(PageText {
            index: page_no,
            text: page_text.all(),
        });
    }
    Ok(pages)
}

/// 按页原文规范后拼接，供 `extract_text` 与 `pdf_read` 共用。
pub fn join_extracted_pages(pages: &[PageText]) -> String {
    format_extracted_text(
        &pages
            .iter()
            .map(|p| (p.index, p.text.as_str()))
            .collect::<Vec<_>>(),
    )
}

pub fn extract_text(path: &Path) -> Result<String, String> {
    let pages = extract_text_pages(path, None)?;
    Ok(join_extracted_pages(&pages))
}

fn pdfium_instance() -> Result<&'static Mutex<Pdfium>, String> {
    match PDFIUM.get_or_init(|| init_pdfium().map(Mutex::new)) {
        Ok(mutex) => Ok(mutex),
        Err(err) => Err(err.clone()),
    }
}

fn init_pdfium() -> Result<Pdfium, String> {
    for dir in library_search_paths() {
        let library_path = Pdfium::pdfium_platform_library_name_at_path(&dir);
        if let Ok(bindings) = Pdfium::bind_to_library(&library_path) {
            return Ok(Pdfium::new(bindings));
        }
    }
    Err("无法加载 PDFium 库，请重新构建应用以下载 PDF 引擎".into())
}

fn library_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(dir) = std::env::var("DOC_AGENT_PDFIUM_DIR") {
        paths.push(PathBuf::from(dir));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            paths.push(parent.to_path_buf());
            if let Some(contents) = parent.parent().and_then(|p| p.parent()) {
                if contents.file_name().is_some_and(|name| name == "Contents") {
                    paths.push(contents.join("Resources").join("pdfium"));
                    paths.push(contents.join("Resources"));
                }
            }
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        paths.push(PathBuf::from(manifest_dir).join("pdfium"));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_search_paths_is_not_empty() {
        assert!(!library_search_paths().is_empty());
    }
}
