#[cfg(test)]
mod tool_tests {
    use crate::core::sandbox::Sandbox;
    use crate::tools::{ToolContext, ToolError, ToolRegistry};
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;
    use zip::ZipArchive;

    fn assert_valid_ooxml(path: &std::path::Path) {
        let file = fs::File::open(path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let mut found = false;
        for i in 0..archive.len() {
            let name = archive.by_index(i).unwrap().name().to_string();
            if name.eq_ignore_ascii_case("[Content_Types].xml") {
                found = true;
            }
        }
        assert!(found, "missing [Content_Types].xml in {}", path.display());
    }

    fn setup(dir: &tempfile::TempDir) -> Sandbox {
        Sandbox::new(dir.path()).unwrap()
    }

    #[test]
    fn registry_exposes_expected_tools() {
        let registry = ToolRegistry::default_tools();
        let names: Vec<_> = registry.definitions().into_iter().map(|d| d.name).collect();
        for expected in [
            "fs_list",
            "fs_read",
            "fs_write",
            "fs_search",
            "office_read_to_markdown",
            "word_create",
            "word_edit",
            "excel_read",
            "excel_write",
            "skill_run",
        ] {
            assert!(names.contains(&expected.to_string()), "missing tool {expected}");
        }
    }

    #[test]
    fn unknown_tool_is_rejected() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let err = registry
            .execute(&ctx, "missing_tool", json!({}))
            .unwrap_err();
        assert!(matches!(err, ToolError::Unknown(_)));
    }

    #[test]
    fn fs_tools_read_write_and_search() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("alpha.txt"), "hello").unwrap();
        fs::create_dir_all(dir.path().join("nested")).unwrap();
        fs::write(dir.path().join("nested/beta.txt"), "world").unwrap();

        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();

        let listed = registry
            .execute(&ctx, "fs_list", json!({ "path": "." }))
            .unwrap();
        assert!(listed["entries"].as_array().unwrap().len() >= 2);

        let read = registry
            .execute(&ctx, "fs_read", json!({ "path": "alpha.txt" }))
            .unwrap();
        assert_eq!(read["content"], "hello");

        registry
            .execute(
                &ctx,
                "fs_write",
                json!({ "path": "out.txt", "content": "saved" }),
            )
            .unwrap();
        assert_eq!(fs::read_to_string(dir.path().join("out.txt")).unwrap(), "saved");

        let found = registry
            .execute(&ctx, "fs_search", json!({ "query": "beta" }))
            .unwrap();
        assert!(!found["matches"].as_array().unwrap().is_empty());
    }

    #[test]
    fn word_and_excel_tools_emit_valid_ooxml() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();

        registry
            .execute(
                &ctx,
                "word_create",
                json!({ "path": "report.docx", "title": "报告", "body": "正文" }),
            )
            .unwrap();
        assert_valid_ooxml(&dir.path().join("report.docx"));

        registry
            .execute(
                &ctx,
                "excel_write",
                json!({
                    "path": "budget.xlsx",
                    "cells": [
                        { "cell": "A1", "value": "项目" },
                        { "cell": "B1", "value": 1000 }
                    ]
                }),
            )
            .unwrap();
        assert_valid_ooxml(&dir.path().join("budget.xlsx"));
    }

    #[test]
    fn word_edit_replaces_text() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();

        registry
            .execute(
                &ctx,
                "word_create",
                json!({ "path": "doc.docx", "title": "季度报告", "body": "正文" }),
            )
            .unwrap();
        let edited = registry
            .execute(
                &ctx,
                "word_edit",
                json!({
                    "path": "doc.docx",
                    "find": "季度报告",
                    "replace": "年度报告",
                    "output_path": "edited.docx"
                }),
            )
            .unwrap();
        assert_eq!(edited["replacements"], 1);
        assert_valid_ooxml(&dir.path().join("edited.docx"));
    }

    #[test]
    fn excel_read_after_write() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();

        registry
            .execute(
                &ctx,
                "excel_write",
                json!({
                    "path": "sheet.xlsx",
                    "cells": [{ "cell": "A1", "value": "名称" }]
                }),
            )
            .unwrap();
        let read = registry
            .execute(&ctx, "excel_read", json!({ "path": "sheet.xlsx" }))
            .unwrap();
        let rows = read["rows"].as_array().unwrap();
        assert_eq!(rows[0][0], "名称");
    }

    #[test]
    fn skill_run_is_reserved_not_implemented() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let err = registry
            .execute(&ctx, "skill_run", json!({ "skill": "pptx" }))
            .unwrap_err();
        assert!(matches!(err, ToolError::NotImplemented));
    }

    #[test]
    fn office_read_rejects_unsupported_extension() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("notes.txt"), "hello").unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let err = registry
            .execute(
                &ctx,
                "office_read_to_markdown",
                json!({ "path": "notes.txt" }),
            )
            .unwrap_err();
        assert!(err.to_string().contains("unsupported"));
    }

    #[test]
    fn office_read_rejects_invalid_pdf() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("broken.pdf"), b"not-a-pdf").unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let err = registry
            .execute(
                &ctx,
                "office_read_to_markdown",
                json!({ "path": "broken.pdf" }),
            )
            .unwrap_err();
        assert!(err.to_string().contains("PDF"));
    }
}
