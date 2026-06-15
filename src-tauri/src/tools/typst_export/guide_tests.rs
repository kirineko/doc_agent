use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::compile::{compile_to_temp_pdf, CompileInput, CompileOutput};

pub fn extract_compile_marked_blocks(guide: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut lines = guide.lines().peekable();
    let mut line_no = 0usize;
    while let Some(line) = lines.next() {
        line_no += 1;
        if line.trim() != "<!-- doc-agent:compile -->" {
            continue;
        }
        if lines.next_if(|l| l.starts_with("```typst")).is_none() {
            continue;
        }
        line_no += 1;
        let mut body = String::new();
        for code_line in lines.by_ref() {
            line_no += 1;
            if code_line.starts_with("```") {
                break;
            }
            body.push_str(code_line);
            body.push('\n');
        }
        blocks.push((line_no, body));
    }
    blocks
}

pub fn parse_guide_export_table(guide: &str) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();
    let mut in_section = false;
    for line in guide.lines() {
        if line.starts_with("### 0.2") {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("### ") {
            break;
        }
        if !in_section || !line.starts_with('|') || line.contains("路径") {
            continue;
        }
        let cols: Vec<_> = line.split('|').map(str::trim).collect();
        if cols.len() < 4 {
            continue;
        }
        let path = cols[1].trim_matches('`');
        if !path.ends_with(".typ") {
            continue;
        }
        let exports = cols[2]
            .split(['、', ','])
            .map(|s| s.trim().trim_matches('`'))
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect::<HashSet<_>>();
        map.insert(path.to_string(), exports);
    }
    map
}

pub fn extract_public_lets(source: &str) -> HashSet<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("#let ")
                .and_then(|rest| rest.split(['(', ' ', '=']).next())
                .map(str::to_string)
        })
        .collect()
}

pub fn compile_snippet(dir: &Path, name: &str, source: &str) -> Result<CompileOutput, String> {
    let file = dir.join(name);
    std::fs::write(&file, source).map_err(|e| e.to_string())?;
    compile_to_temp_pdf(CompileInput {
        sandbox_root: dir.to_path_buf(),
        entry: file,
        out_path: dir.join(format!("{name}.pdf")),
    })
    .map_err(|failure| {
        failure
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

#[cfg(test)]
mod tests {
    use super::{
        compile_snippet, compile_to_temp_pdf, extract_compile_marked_blocks, extract_public_lets,
        parse_guide_export_table, CompileInput,
    };
    use crate::tools::typst_export::bundled;
    use std::path::Path;

    fn require_font_dirs() {
        let ready = std::env::var("CARGO_MANIFEST_DIR")
            .map(|d| Path::new(&d).join("fonts").is_dir())
            .unwrap_or(false);
        assert!(
            ready,
            "Noto fonts dir missing; run `cargo build` to download bundled fonts"
        );
    }

    #[test]
    fn typst_guide_marked_blocks_compile() {
        require_font_dirs();

        let guide = bundled::typst_guide_source();
        let blocks = extract_compile_marked_blocks(guide);
        assert!(
            !blocks.is_empty(),
            "expected at least one <!-- doc-agent:compile --> typst block"
        );

        let dir = tempfile::tempdir().unwrap();
        for (idx, (_, source)) in blocks.into_iter().enumerate() {
            let name = format!("guide-block-{idx}.typ");
            compile_snippet(dir.path(), &name, &source)
                .unwrap_or_else(|e| panic!("guide block {idx} failed:\n{e}\n---\n{source}"));
        }
    }

    #[test]
    fn guide_exports_match_common_modules() {
        let guide = bundled::typst_guide_source();
        let table = parse_guide_export_table(guide);
        assert!(table.contains_key("common/tokens.typ"));

        for (rel, expected) in table {
            let source = bundled::find_source(&rel).unwrap_or_else(|| panic!("missing {rel}"));
            let actual = extract_public_lets(source);
            for export in expected {
                if export.ends_with('*') {
                    let prefix = export.trim_end_matches('*');
                    assert!(
                        actual.iter().any(|name| name.starts_with(prefix)),
                        "{rel}: no export matching {export}"
                    );
                } else {
                    assert!(
                        actual.contains(export.as_str()),
                        "{rel}: missing export {export}; have {actual:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn unknown_variable_produces_structured_tool_error() {
        require_font_dirs();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.typ"), "#fillblank()").unwrap();
        let failure = compile_to_temp_pdf(CompileInput {
            sandbox_root: dir.path().to_path_buf(),
            entry: dir.path().join("bad.typ"),
            out_path: dir.path().join("bad.pdf"),
        })
        .expect_err("should fail");
        let diag = &failure.diagnostics[0];
        assert_eq!(diag.error_type, "unknown-variable");
        assert!(diag.file.is_some());
        assert!(diag.line.is_some());
        assert!(diag.snippet.is_some());
        assert!(diag.fix_guidance.contains("fs_patch"));
    }

    #[test]
    fn theme_free_axes_compile_and_exam_locks_colored_accent() {
        require_font_dirs();

        let dir = tempfile::tempdir().unwrap();
        compile_snippet(
            dir.path(),
            "theme-free-axes.typ",
            r##"#import "/doc-agent/typst/common/fonts.typ": apply-zh-body, apply-zh-title
#import "/doc-agent/typst/common/page.typ": page-a4
#import "/doc-agent/typst/common/tokens.typ": make-theme, exam-theme

#let theme = make-theme(
  accent: rgb("#0b6e6e"),
  heading-style: "accent-number",
  cover: "banner",
)
#show: apply-zh-body.with(theme: theme)
#page-a4()
#apply-zh-title([主题测试], subtitle: [banner cover], theme: theme)
= 一级标题
#table(
  columns: (auto, auto),
  table.header([列 A], [列 B]),
  [1], [2],
)

#let locked = exam-theme(accent: rgb("#ff0000"))
#assert.eq(locked.accent, rgb("#1a1a1a"))
"##,
        )
        .expect("theme axes should compile");
    }
}
