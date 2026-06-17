use std::path::{Path, PathBuf};

use typst::diag::{SourceDiagnostic, Warned};
use typst_as_lib::typst_kit_options::TypstKitFontOptions;
use typst_as_lib::{TypstAsLibError, TypstEngine, TypstTemplateCollection};
use typst_pdf::PdfOptions;
use uuid::Uuid;

use super::bundled;
use super::diagnostics::{
    build_source_map, to_diagnostics, to_warnings, CompileFailure, WarningInfo,
};

pub struct CompileInput {
    pub sandbox_root: PathBuf,
    pub entry: PathBuf,
    /// Used only to choose a temp file directory; final copy happens in the async handler.
    pub out_path: PathBuf,
}

#[derive(Debug)]
pub struct CompileOutput {
    pub temp_path: PathBuf,
    pub pages: u32,
    pub warnings: Vec<WarningInfo>,
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
pub fn compile_to_temp_pdf(input: CompileInput) -> Result<CompileOutput, CompileFailure> {
    let main_vpath = typst_vpath(&input.sandbox_root, &input.entry)
        .map_err(|message| compile_failure_message(message, &input))?;
    let source_map = build_source_map(&input.sandbox_root, &input.entry)
        .map_err(|message| compile_failure_message(message, &input))?;
    let engine = build_engine(&input.sandbox_root);
    let Warned { output, warnings } = engine.compile(main_vpath.as_str());
    let warning_infos = to_warnings(&warnings, &source_map);

    let doc = match output {
        Ok(doc) => doc,
        Err(err) => {
            return Err(map_compile_error(err, &source_map, warning_infos));
        }
    };

    let pdf = match typst_pdf::pdf(&doc, &PdfOptions::default()) {
        Ok(pdf) => pdf,
        Err(errors) => {
            let diags: Vec<SourceDiagnostic> = errors
                .into_iter()
                .map(|e| SourceDiagnostic {
                    severity: typst::diag::Severity::Error,
                    span: e.span,
                    message: e.message,
                    trace: Default::default(),
                    hints: Default::default(),
                })
                .collect();
            return Err(CompileFailure {
                diagnostics: to_diagnostics(&diags, &source_map),
                warnings: warning_infos,
            });
        }
    };

    let temp_path = temp_pdf_path(&input.out_path);
    if let Some(parent) = temp_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| compile_failure_message(format!("无法创建输出目录: {e}"), &input))?;
    }
    std::fs::write(&temp_path, pdf)
        .map_err(|e| compile_failure_message(format!("无法写入 PDF: {e}"), &input))?;
    let pages = pdf_page_count(&temp_path).map_err(|e| compile_failure_message(e, &input))?;
    if pages == 0 {
        let _ = std::fs::remove_file(&temp_path);
        return Err(compile_failure_message("导出的 PDF 无页面", &input));
    }
    Ok(CompileOutput {
        temp_path,
        pages,
        warnings: warning_infos,
    })
}

/// Copy temp PDF to final destination via a staging file.
pub fn finalize_pdf(temp_path: &Path, out_path: &Path) -> Result<(), String> {
    let staging = staging_path(out_path);
    std::fs::copy(temp_path, &staging).map_err(|e| format!("无法写入暂存 PDF: {e}"))?;
    let _ = std::fs::remove_file(temp_path);

    if std::fs::rename(&staging, out_path).is_ok() {
        return Ok(());
    }

    std::fs::copy(&staging, out_path).map_err(|e| format!("无法写入最终 PDF: {e}"))?;
    let _ = std::fs::remove_file(&staging);
    Ok(())
}

pub fn remove_temp_pdf(temp_path: &Path) {
    let _ = std::fs::remove_file(temp_path);
}

fn compile_failure_message(message: impl Into<String>, input: &CompileInput) -> CompileFailure {
    let _ = input;
    CompileFailure {
        diagnostics: vec![super::diagnostics::DiagnosticInfo {
            error_type: "other".into(),
            message: message.into(),
            hints: vec![],
            fix_guidance: "检查输入路径与沙箱权限后重试。".into(),
            file: None,
            line: None,
            column: None,
            snippet: None,
        }],
        warnings: vec![],
    }
}

fn map_compile_error(
    err: TypstAsLibError,
    source_map: &std::collections::HashMap<typst::syntax::FileId, typst::syntax::Source>,
    warnings: Vec<WarningInfo>,
) -> CompileFailure {
    match err {
        TypstAsLibError::TypstSource(diags) => CompileFailure {
            diagnostics: to_diagnostics(diags.as_slice(), source_map),
            warnings,
        },
        other => CompileFailure {
            diagnostics: vec![super::diagnostics::DiagnosticInfo {
                error_type: "other".into(),
                message: other.to_string(),
                hints: vec![],
                fix_guidance: "根据 message 用 fs_patch 局部修改；禁止整篇重写 .typ。".into(),
                file: None,
                line: None,
                column: None,
                snippet: None,
            }],
            warnings,
        },
    }
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
    let font_dirs = font_search_paths();
    let font_options = if font_dirs.is_empty() {
        TypstKitFontOptions::default()
    } else {
        TypstKitFontOptions::default().include_dirs(font_dirs)
    };
    TypstEngine::builder()
        .search_fonts_with(font_options)
        .with_static_source_file_resolver(sources)
        .with_file_system_resolver(sandbox_root)
        .build()
}

pub fn configure_font_dir(path: PathBuf) {
    std::env::set_var("DOC_AGENT_FONTS_DIR", path);
}

fn font_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(dir) = std::env::var("DOC_AGENT_FONTS_DIR") {
        paths.push(PathBuf::from(dir));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            paths.push(parent.join("fonts"));
            if let Some(contents) = parent.parent().and_then(|p| p.parent()) {
                if contents.file_name().is_some_and(|name| name == "Contents") {
                    paths.push(contents.join("Resources").join("fonts"));
                }
            }
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        paths.push(PathBuf::from(manifest_dir).join("fonts"));
    }

    paths.into_iter().filter(|path| path.is_dir()).collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn assert_no_font_warnings(warnings: &[WarningInfo]) {
        assert!(
            warnings.is_empty(),
            "unexpected font warnings:\n{}",
            warnings
                .iter()
                .map(|w| w.message.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    fn compile_template(dir: &Path, rel: &str, source: &str) -> CompileOutput {
        fs::write(dir.join(rel), source).expect("write template");
        let entry = dir.join(rel);
        compile_to_temp_pdf(CompileInput {
            sandbox_root: dir.to_path_buf(),
            entry,
            out_path: dir.join("out.pdf"),
        })
        .expect("compile template")
    }

    #[test]
    fn bundled_exam_zh_compiles_without_font_warnings() {
        let font_dirs = font_search_paths();
        assert!(
            !font_dirs.is_empty(),
            "Noto fonts dir missing; run `cargo build` to download bundled fonts"
        );

        let dir = tempfile::tempdir().expect("tempdir");
        let exam = bundled::find_source("exam/exam-zh").expect("exam-zh template");
        let output = compile_template(dir.path(), "exam.typ", exam);
        assert_no_font_warnings(&output.warnings);
        assert!(output.pages > 0);
    }

    #[test]
    fn all_scene_templates_compile_without_warnings() {
        let font_dirs = font_search_paths();
        assert!(!font_dirs.is_empty(), "Noto fonts dir missing");

        let dir = tempfile::tempdir().expect("tempdir");
        for id in bundled::scene_template_ids() {
            let source = bundled::find_source(id).unwrap_or_else(|| panic!("missing {id}"));
            let file = format!("{}.typ", id.replace('/', "_"));
            let output = compile_template(dir.path(), &file, source);
            assert_no_font_warnings(&output.warnings);
            assert!(output.pages > 0, "{id} produced no pages");
        }
    }

    #[test]
    fn custom_accent_theme_compiles_without_warnings() {
        let font_dirs = font_search_paths();
        assert!(!font_dirs.is_empty(), "Noto fonts dir missing");

        let dir = tempfile::tempdir().expect("tempdir");
        let source = r##"#import "/doc-agent/typst/common/fonts.typ": apply-zh-body
#import "/doc-agent/typst/common/page.typ": page-a4, footer-page-no
#import "/doc-agent/typst/common/tokens.typ": make-theme

#let theme = make-theme(accent: rgb("#0b6e6e"))
#show: apply-zh-body.with(theme: theme)
#page-a4()
#footer-page-no()

= 标题
正文 $x^2$。
"##;
        let output = compile_template(dir.path(), "custom.typ", source);
        assert_no_font_warnings(&output.warnings);
    }

    #[test]
    fn code_block_with_cjk_compiles_without_font_warnings() {
        let font_dirs = font_search_paths();
        assert!(!font_dirs.is_empty(), "Noto fonts dir missing");

        let dir = tempfile::tempdir().expect("tempdir");
        let source = r##"#import "/doc-agent/typst/common/fonts.typ": apply-zh-body
#import "/doc-agent/typst/common/page.typ": page-a4, footer-page-no

#show: apply-zh-body
#page-a4()
#footer-page-no()

= 代码示例

```python
def greet(name: str) -> str:
    # 中文注释与 English mixed
    return f"你好, {name}"
```
"##;
        let output = compile_template(dir.path(), "code.typ", source);
        assert_no_font_warnings(&output.warnings);
        assert!(output.pages > 0);
    }

    #[test]
    fn compile_failure_returns_structured_diagnostics() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("bad.typ"), "#fillblank()").unwrap();
        let err = compile_to_temp_pdf(CompileInput {
            sandbox_root: dir.path().to_path_buf(),
            entry: dir.path().join("bad.typ"),
            out_path: dir.path().join("bad.pdf"),
        })
        .expect_err("should fail");
        assert!(!err.diagnostics.is_empty());
        assert_eq!(err.diagnostics[0].error_type, "unknown-variable");
        assert!(err.diagnostics[0].snippet.is_some());
    }
}
