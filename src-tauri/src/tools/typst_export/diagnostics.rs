use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use typst::diag::SourceDiagnostic;
use typst::syntax::{FileId, Source, Span, VirtualPath};

use super::bundled;
use super::compile::typst_vpath;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiagnosticInfo {
    pub error_type: String,
    pub message: String,
    pub hints: Vec<String>,
    pub fix_guidance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WarningInfo {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug)]
pub struct CompileFailure {
    pub diagnostics: Vec<DiagnosticInfo>,
    pub warnings: Vec<WarningInfo>,
}

impl CompileFailure {
    pub fn into_message(self) -> String {
        if self.diagnostics.is_empty() {
            return "typst 编译失败".into();
        }
        self.diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn build_source_map(
    sandbox_root: &Path,
    entry: &Path,
) -> Result<HashMap<FileId, Source>, String> {
    let mut map = HashMap::new();

    for (vpath, text) in bundled::static_sources() {
        insert_source(&mut map, vpath, text.to_string());
    }

    let entry_vpath = typst_vpath(sandbox_root, entry)?;
    let entry_text = std::fs::read_to_string(entry)
        .map_err(|e| format!("无法读取入口文件 {}: {e}", entry.display()))?;
    insert_source(&mut map, &entry_vpath, entry_text);

    walk_typ_files_best_effort(sandbox_root, sandbox_root, &mut map);
    Ok(map)
}

pub fn to_diagnostics(
    diags: &[SourceDiagnostic],
    source_map: &HashMap<FileId, Source>,
) -> Vec<DiagnosticInfo> {
    diags
        .iter()
        .map(|d| diagnostic_from(d, source_map))
        .collect()
}

pub fn to_warnings(
    warnings: &[SourceDiagnostic],
    source_map: &HashMap<FileId, Source>,
) -> Vec<WarningInfo> {
    warnings
        .iter()
        .map(|d| warning_from(d, source_map))
        .take(10)
        .collect()
}

fn diagnostic_from(d: &SourceDiagnostic, source_map: &HashMap<FileId, Source>) -> DiagnosticInfo {
    let message = d.message.to_string();
    let hints = d.hints.iter().map(|h| h.to_string()).collect();
    let (error_type, fix_guidance) = classify(&message);
    let location = resolve_location(d.span, source_map);

    match location {
        Some((file, line, column, snippet)) => DiagnosticInfo {
            error_type,
            message,
            hints,
            fix_guidance,
            file: Some(file),
            line: Some(line),
            column: Some(column),
            snippet: Some(snippet),
        },
        None => DiagnosticInfo {
            error_type: "unlocated".into(),
            message,
            hints,
            fix_guidance,
            file: None,
            line: None,
            column: None,
            snippet: None,
        },
    }
}

fn warning_from(d: &SourceDiagnostic, source_map: &HashMap<FileId, Source>) -> WarningInfo {
    let message = d.message.to_string();
    let location = resolve_location(d.span, source_map);
    match location {
        Some((file, line, _, _)) => WarningInfo {
            message,
            file: Some(file),
            line: Some(line),
        },
        None => WarningInfo {
            message,
            file: None,
            line: None,
        },
    }
}

pub fn resolve_location(
    span: Span,
    source_map: &HashMap<FileId, Source>,
) -> Option<(String, u32, u32, String)> {
    let file_id = span.id()?;
    let source = source_map.get(&file_id)?;
    let range = source.range(span)?;
    let line = source.byte_to_line(range.start)?;
    let column = source.byte_to_column(range.start)?;
    let file = file_display_name(file_id);
    let snippet = make_snippet(source, line, column, &range)?;
    Some((file, (line + 1) as u32, (column + 1) as u32, snippet))
}

fn classify(message: &str) -> (String, String) {
    let lower = message.to_lowercase();
    if lower.contains("unknown variable") || lower.contains("unknown function") {
        (
            "unknown-variable".into(),
            "检查函数/变量名拼写；内置函数多用连字符，如 fill-blank、calc-item。用 fs_patch 只改出错行。".into(),
        )
    } else if lower.contains("unexpected argument") || lower.contains("missing argument") {
        (
            "unexpected-argument".into(),
            "检查函数参数是否使用命名形式，如 fill-blank(width: 2.5cm) 而非 fill-blank(2.5cm)。用 fs_patch 局部修改。".into(),
        )
    } else if lower.contains("unknown font") || lower.contains("font family") {
        (
            "unknown-font".into(),
            "字体未找到；优先使用 apply-zh-body/apply-en-body 与内置 fonts.typ，勿手写系统字体名。"
                .into(),
        )
    } else if lower.contains("file not found") || lower.contains("failed to load") {
        (
            "file-not-found".into(),
            "检查 #import 路径；内置模块用 \"/doc-agent/typst/...\"，项目内用相对路径。".into(),
        )
    } else if lower.contains("expected") || lower.contains("unexpected") {
        (
            "syntax".into(),
            "Typst 语法错误；对照 typst-guide 与场景模板，用 fs_patch 修正出错行，勿整篇重写。"
                .into(),
        )
    } else if lower.contains("type") {
        (
            "type-error".into(),
            "类型不匹配；检查函数参数类型与内容块 [] 用法，用 fs_patch 局部修改。".into(),
        )
    } else {
        (
            "other".into(),
            "根据 message 与 snippet 定位出错行，用 fs_patch 做最小修改；禁止整篇重写 .typ。"
                .into(),
        )
    }
}

fn make_snippet(
    source: &Source,
    line: usize,
    column: usize,
    range: &std::ops::Range<usize>,
) -> Option<String> {
    let line_start = source.line_to_byte(line)?;
    let line_end = if line + 1 < source.len_lines() {
        source.line_to_byte(line + 1)?
    } else {
        source.len_bytes()
    };
    let line_text = source
        .get(line_start..line_end)?
        .trim_end_matches(['\r', '\n']);
    let highlight_len = source
        .get(range.clone())
        .map(|s| s.chars().count())
        .unwrap_or(1)
        .max(1);
    let caret = format!(
        "{}^{}",
        " ".repeat(column),
        "^".repeat(highlight_len.saturating_sub(1))
    );
    Some(format!("{line_text}\n{caret}"))
}

fn file_display_name(id: FileId) -> String {
    id.vpath()
        .as_rootless_path()
        .to_string_lossy()
        .replace('\\', "/")
}

fn insert_source(map: &mut HashMap<FileId, Source>, vpath: &str, text: String) {
    let id = FileId::new(None, VirtualPath::new(vpath));
    map.insert(id, Source::new(id, text));
}

fn walk_typ_files_best_effort(sandbox_root: &Path, dir: &Path, map: &mut HashMap<FileId, Source>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_typ_files_best_effort(sandbox_root, &path, map);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("typ") {
            continue;
        }
        let Ok(rel) = path.strip_prefix(sandbox_root) else {
            continue;
        };
        let vpath = rel.to_string_lossy().replace('\\', "/");
        if map.values().any(|s| file_display_name(s.id()) == vpath) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        insert_source(map, &vpath, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst::diag::Warned;
    use typst::layout::PagedDocument;
    use typst_as_lib::{TypstAsLibError, TypstEngine};

    fn compile_bad_source(dir: &Path, source: &str) -> Vec<SourceDiagnostic> {
        std::fs::write(dir.join("bad.typ"), source).unwrap();
        let engine = TypstEngine::builder()
            .with_file_system_resolver(dir)
            .build();
        let Warned {
            output,
            warnings: _,
        } = engine.compile::<_, PagedDocument>("bad.typ");
        match output {
            Err(TypstAsLibError::TypstSource(diags)) => diags.into_iter().collect(),
            other => panic!("expected compile error, got {other:?}"),
        }
    }

    #[test]
    fn resolve_location_for_unknown_variable() {
        let dir = tempfile::tempdir().unwrap();
        let source = r#"#fillblank()"#;
        let diags = compile_bad_source(dir.path(), source);
        assert!(!diags.is_empty());

        let entry = dir.path().join("bad.typ");
        let map = build_source_map(dir.path(), &entry).unwrap();
        let info = diagnostic_from(&diags[0], &map);
        assert_eq!(info.error_type, "unknown-variable");
        assert_eq!(info.file.as_deref(), Some("bad.typ"));
        assert_eq!(info.line, Some(1));
        assert!(info.column.unwrap() >= 1);
        assert!(info.snippet.as_ref().unwrap().contains("fillblank"));
        assert!(info.fix_guidance.contains("fs_patch"));
    }

    #[test]
    fn detached_span_degrades_safely() {
        let map: HashMap<FileId, Source> = HashMap::new();
        let diag = SourceDiagnostic {
            severity: typst::diag::Severity::Error,
            span: Span::detached(),
            message: "detached error".into(),
            trace: Default::default(),
            hints: Default::default(),
        };
        let info = diagnostic_from(&diag, &map);
        assert_eq!(info.error_type, "unlocated");
        assert!(info.file.is_none());
        assert!(info.line.is_none());
    }

    #[test]
    fn source_map_includes_sibling_typ_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::write(dir.path().join("chapter.typ"), "= Chapter").unwrap();
        std::fs::write(
            dir.path().join("docs/main.typ"),
            "#import \"../chapter.typ\"\n= Main",
        )
        .unwrap();

        let map = build_source_map(dir.path(), &dir.path().join("docs/main.typ"))
            .expect("source map should include sibling imports");
        assert!(map
            .values()
            .any(|source| file_display_name(source.id()) == "chapter.typ"));
    }

    #[test]
    fn source_map_ignores_unrelated_invalid_typ_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::create_dir_all(dir.path().join("archive")).unwrap();
        std::fs::write(dir.path().join("docs/main.typ"), "= Ok").unwrap();
        std::fs::write(dir.path().join("archive/bad.typ"), [0xff, 0xfe]).unwrap();

        let map = build_source_map(dir.path(), &dir.path().join("docs/main.typ"))
            .expect("source map should ignore unrelated invalid files");
        assert!(map
            .values()
            .any(|source| file_display_name(source.id()) == "docs/main.typ"));
    }
}
