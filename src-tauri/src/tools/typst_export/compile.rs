use std::path::{Path, PathBuf};

use typst::diag::{SourceDiagnostic, Warned};
use typst_as_lib::typst_kit_options::TypstKitFontOptions;
use typst_as_lib::{TypstAsLibError, TypstEngine, TypstTemplateCollection};
use typst_pdf::PdfOptions;
use uuid::Uuid;

use super::bundled;

pub struct CompileInput {
    pub sandbox_root: PathBuf,
    pub entry: PathBuf,
    /// Used only to choose a temp file directory; final copy happens in the async handler.
    pub out_path: PathBuf,
}

pub struct CompileOutput {
    pub temp_path: PathBuf,
    pub pages: u32,
}

pub fn resolve_typst_entry(resolved: &Path) -> Result<PathBuf, String> {
    if resolved.is_file() {
        let is_typ = resolved
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("typ"));
        if !is_typ {
            return Err("path must be a .typ file or a directory containing main.typ".into());
        }
        return Ok(resolved.to_path_buf());
    }
    if resolved.is_dir() {
        let main = resolved.join("main.typ");
        if main.is_file() {
            return Ok(main);
        }
        return Err(format!("目录缺少 main.typ: {}", resolved.display()));
    }
    Err("path does not exist".into())
}

pub fn typst_vpath(sandbox_root: &Path, entry: &Path) -> Result<String, String> {
    let rel = entry
        .strip_prefix(sandbox_root)
        .map_err(|_| format!("入口文件不在沙箱根目录下: {}", entry.display()))?;
    Ok(to_forward_slashes(rel))
}

/// Compile Typst to a temp PDF. Does not write `out_path`.
pub fn compile_to_temp_pdf(input: CompileInput) -> Result<CompileOutput, String> {
    let main_vpath = typst_vpath(&input.sandbox_root, &input.entry)?;
    let engine = build_engine(&input.sandbox_root);
    let Warned { output, warnings } = engine.compile(main_vpath.as_str());
    if !warnings.is_empty() {
        eprintln!("typst warnings: {}", format_warnings(&warnings));
    }
    let doc = output.map_err(format_typst_error)?;
    let pdf = typst_pdf::pdf(&doc, &PdfOptions::default())
        .map_err(|errors| format!("typst PDF export failed: {errors:?}"))?;
    let temp_path = temp_pdf_path(&input.out_path);
    if let Some(parent) = temp_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("无法创建输出目录: {e}"))?;
    }
    std::fs::write(&temp_path, pdf).map_err(|e| format!("无法写入 PDF: {e}"))?;
    let pages = pdf_page_count(&temp_path)?;
    if pages == 0 {
        let _ = std::fs::remove_file(&temp_path);
        return Err("导出的 PDF 无页面".into());
    }
    Ok(CompileOutput { temp_path, pages })
}

/// Copy temp PDF to final destination via a staging file.
pub fn finalize_pdf(temp_path: &Path, out_path: &Path) -> Result<(), String> {
    let staging = staging_path(out_path);
    std::fs::copy(temp_path, &staging).map_err(|e| format!("无法写入暂存 PDF: {e}"))?;
    let _ = std::fs::remove_file(temp_path);

    if std::fs::rename(&staging, out_path).is_ok() {
        return Ok(());
    }

    // Windows: rename does not replace an existing file; copy leaves the old PDF intact on failure.
    std::fs::copy(&staging, out_path).map_err(|e| format!("无法写入最终 PDF: {e}"))?;
    let _ = std::fs::remove_file(&staging);
    Ok(())
}

pub fn remove_temp_pdf(temp_path: &Path) {
    let _ = std::fs::remove_file(temp_path);
}

fn staging_path(out_path: &Path) -> PathBuf {
    let stem = out_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("output.pdf");
    let name = format!(".{stem}.staging");
    out_path
        .parent()
        .map(|p| p.join(&name))
        .unwrap_or_else(|| PathBuf::from(name))
}

fn temp_pdf_path(out_path: &Path) -> PathBuf {
    let name = format!(".typst-export-{}.part", Uuid::new_v4().simple());
    out_path
        .parent()
        .map(|p| p.join(&name))
        .unwrap_or_else(|| PathBuf::from(name))
}

fn build_engine(sandbox_root: &Path) -> TypstEngine<TypstTemplateCollection> {
    let sources = bundled::static_sources();
    TypstEngine::builder()
        .search_fonts_with(TypstKitFontOptions::default())
        .with_static_source_file_resolver(sources)
        .with_file_system_resolver(sandbox_root)
        .build()
}

fn to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn pdf_page_count(path: &Path) -> Result<u32, String> {
    lopdf::Document::load(path)
        .map_err(|e| format!("无法读取 PDF: {e}"))?
        .get_pages()
        .len()
        .try_into()
        .map_err(|_| "页数溢出".to_string())
}

fn format_typst_error(err: TypstAsLibError) -> String {
    match err {
        TypstAsLibError::TypstSource(diags) => format_diagnostics(&diags),
        other => other.to_string(),
    }
}

fn format_diagnostics(diags: &[SourceDiagnostic]) -> String {
    diags
        .iter()
        .map(|d| format!("{d:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_warnings(warnings: &[SourceDiagnostic]) -> String {
    warnings
        .iter()
        .map(|d| format!("{d:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}
