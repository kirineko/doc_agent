use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static PDFIUM: OnceLock<Result<Mutex<Pdfium>, String>> = OnceLock::new();

pub fn configure_resource_dir(path: PathBuf) {
    std::env::set_var("DOC_AGENT_PDFIUM_DIR", path);
}

pub fn extract_text(path: &Path) -> Result<String, String> {
    let guard = pdfium_instance()?
        .lock()
        .map_err(|_| "PDFium lock poisoned".to_string())?;

    let document = guard
        .load_pdf_from_file(path, None)
        .map_err(|e| format!("PDF 打开失败: {e}"))?;

    let page_count = document.pages().len();
    let mut text = String::new();

    for index in 0..page_count {
        let page = document
            .pages()
            .get(index)
            .map_err(|e| format!("PDF 第 {} 页读取失败: {e}", index + 1))?;
        let page_text = page
            .text()
            .map_err(|e| format!("PDF 第 {} 页文本提取失败: {e}", index + 1))?;
        text.push_str(&page_text.all());
        if index + 1 < page_count {
            text.push('\n');
        }
    }

    Ok(text)
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
