#[cfg(test)]
mod tool_tests {
    use crate::core::sandbox::Sandbox;
    use crate::tools::ooxml::style_lint::lint_docx;
    use crate::tools::ooxml::validate;
    use crate::tools::{ToolContext, ToolError, ToolRegistry};
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;
    use zip::ZipArchive;

    fn assert_valid_ooxml(path: &std::path::Path) {
        validate::roundtrip_check(path)
            .unwrap_or_else(|e| panic!("invalid OOXML {}: {e}", path.display()));
    }

    fn setup(dir: &tempfile::TempDir) -> Sandbox {
        Sandbox::new(dir.path()).unwrap()
    }

    fn exec_tool(
        registry: &ToolRegistry,
        ctx: &ToolContext,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(registry.execute(ctx, name, args))
    }

    fn create_docx_via_skill_run(
        ctx: &ToolContext,
        registry: &ToolRegistry,
        path: &str,
        title: &str,
        body: &str,
    ) {
        let code = format!(
            r#"
async function main() {{
  const {{ Document, Packer, Paragraph, TextRun, HeadingLevel }} = docx;
  const doc = new Document({{
    styles: {{
      default: {{ document: {{ run: {{
        font: {{ ascii: "Calibri", eastAsia: "微软雅黑", hAnsi: "Calibri" }},
        size: 24,
      }} }} }},
    }},
    sections: [{{
      children: [
        new Paragraph({{ heading: HeadingLevel.HEADING_1, children: [new TextRun("{title}")] }}),
        new Paragraph({{ children: [new TextRun("{body}")] }}),
      ],
    }}],
  }});
  const b64 = await Packer.toBase64String(doc);
  doc_write("{path}", b64);
  return {{ ok: true }};
}}
"#
        );
        exec_tool(
            &registry,
            ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
    }

    fn make_multi_page_pdf(ctx: &ToolContext, registry: &ToolRegistry, path: &str, pages: u32) {
        let code = format!(
            r#"
async function main() {{
  const doc = await PDFLib.PDFDocument.create();
  const font = await doc.embedFont(PDFLib.StandardFonts.Helvetica);
  for (let i = 0; i < {pages}; i++) {{
    const page = doc.addPage([300, 200]);
    page.drawText("Page " + (i + 1), {{ x: 50, y: 100, size: 20, font }});
  }}
  const bytes = await doc.save();
  let bin = "";
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  doc_write("{path}", btoa(bin));
  return {{ pages: {pages} }};
}}
"#
        );
        exec_tool(
            &registry,
            ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
    }

    fn pdf_page_count(path: &std::path::Path) -> u32 {
        lopdf::Document::load(path).unwrap().get_pages().len() as u32
    }

    fn pdf_page_rotate(path: &std::path::Path, page: u32) -> i32 {
        let doc = lopdf::Document::load(path).unwrap();
        let page_id = *doc.get_pages().get(&page).unwrap();
        let dict = doc.get_object(page_id).unwrap().as_dict().unwrap();
        dict.get(b"Rotate").and_then(|o| o.as_i64()).unwrap_or(0) as i32
    }

    #[test]
    fn registry_exposes_expected_tools() {
        let registry = ToolRegistry::default_tools();
        let names: Vec<_> = registry
            .definitions(true)
            .into_iter()
            .map(|d| d.name)
            .collect();
        for expected in [
            "fs_list",
            "fs_read",
            "fs_write",
            "fs_patch",
            "fs_search",
            "office_read_to_markdown",
            "office_convert",
            "excel_read",
            "excel_write",
            "skill_read",
            "skill_run",
            "ooxml_unpack",
            "ooxml_pack",
            "docx_comment",
            "docx_accept_changes",
            "docx_extract_table",
            "excel_describe",
            "excel_normalize",
            "data_query",
            "xlsx_recalc",
            "pdf_merge",
            "pdf_split",
            "pdf_rotate",
            "pdf_delete_pages",
            "web_search",
            "web_extract",
        ] {
            assert!(
                names.contains(&expected.to_string()),
                "missing tool {expected}"
            );
        }
    }

    #[test]
    fn skill_run_schema_declares_code_or_path_one_of() {
        let registry = ToolRegistry::default_tools();
        let skill_run = registry
            .definitions(true)
            .into_iter()
            .find(|tool| tool.name == "skill_run")
            .expect("skill_run tool definition");
        let one_of = skill_run.parameters["oneOf"].as_array().unwrap();
        assert_eq!(one_of.len(), 2);
        assert!(one_of
            .iter()
            .any(|schema| schema["required"] == json!(["code"])));
        assert!(one_of
            .iter()
            .any(|schema| schema["required"] == json!(["path"])));
    }

    #[test]
    fn web_tools_hidden_without_include_flag() {
        let registry = ToolRegistry::default_tools();
        let names: Vec<_> = registry
            .definitions(false)
            .into_iter()
            .map(|d| d.name)
            .collect();
        assert!(!names.contains(&"web_search".to_string()));
        assert!(!names.contains(&"web_extract".to_string()));
    }

    #[test]
    fn unknown_tool_is_rejected() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err = exec_tool(&registry, &ctx, "missing_tool", json!({})).unwrap_err();
        assert!(matches!(err, ToolError::Unknown(_)));
    }

    #[test]
    fn fs_tools_read_write_and_search() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("alpha.txt"), "hello").unwrap();
        fs::create_dir_all(dir.path().join("nested")).unwrap();
        fs::write(dir.path().join("nested/beta.txt"), "world").unwrap();

        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();

        let listed = exec_tool(&registry, &ctx, "fs_list", json!({ "path": "." })).unwrap();
        assert!(listed["entries"].as_array().unwrap().len() >= 2);

        let read = exec_tool(&registry, &ctx, "fs_read", json!({ "path": "alpha.txt" })).unwrap();
        assert_eq!(read["content"], "hello");

        exec_tool(
            &registry,
            &ctx,
            "fs_write",
            json!({ "path": "out.txt", "content": "saved" }),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("out.txt")).unwrap(),
            "saved"
        );

        let found = exec_tool(&registry, &ctx, "fs_search", json!({ "query": "beta" })).unwrap();
        assert!(!found["matches"].as_array().unwrap().is_empty());
    }

    #[test]
    fn fs_patch_applies_unique_replacements() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("script.js"),
            "const x = 'old';\nconst y = 'old';",
        )
        .unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();

        let out = exec_tool(
            &registry,
            &ctx,
            "fs_patch",
            json!({
                "path": "script.js",
                "edits": [{ "old": "const x = 'old';", "new": "const x = 'new';" }]
            }),
        )
        .unwrap();
        assert_eq!(out["applied"], 1);
        let content = fs::read_to_string(dir.path().join("script.js")).unwrap();
        assert!(content.contains("const x = 'new';"));
        assert!(content.contains("const y = 'old';"));
    }

    #[test]
    fn fs_patch_replace_all_counts_every_match() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("script.js"), "foo foo foo").unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();

        let out = exec_tool(
            &registry,
            &ctx,
            "fs_patch",
            json!({
                "path": "script.js",
                "edits": [{ "old": "foo", "new": "bar", "replace_all": true }]
            }),
        )
        .unwrap();
        assert_eq!(out["applied"], 3);
        assert_eq!(
            fs::read_to_string(dir.path().join("script.js")).unwrap(),
            "bar bar bar"
        );
    }

    #[test]
    fn fs_patch_is_atomic_when_any_edit_misses() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("script.js"), "alpha beta beta").unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();

        // 第 1 条命中、第 2 条 not found、第 3 条 ambiguous → 全部不应用
        let err = exec_tool(
            &registry,
            &ctx,
            "fs_patch",
            json!({
                "path": "script.js",
                "edits": [
                    { "old": "alpha", "new": "ALPHA" },
                    { "old": "gamma", "new": "GAMMA" },
                    { "old": "beta", "new": "BETA" }
                ]
            }),
        )
        .unwrap_err();
        let value = err.to_json_value();
        assert_eq!(value["error"], "fs_patch not applied");
        let missed = value["missed"].as_array().unwrap();
        assert_eq!(missed.len(), 2);
        assert_eq!(missed[0]["reason"], "not found");
        assert_eq!(missed[1]["reason"], "multiple matches");
        assert_eq!(
            fs::read_to_string(dir.path().join("script.js")).unwrap(),
            "alpha beta beta",
            "file must be untouched when any edit misses"
        );
    }

    #[test]
    fn fs_patch_rejects_empty_or_identical_edits() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("script.js"), "abc").unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();

        let empty = exec_tool(
            &registry,
            &ctx,
            "fs_patch",
            json!({ "path": "script.js", "edits": [{ "old": "", "new": "x" }] }),
        )
        .unwrap_err();
        assert!(empty.to_string().contains("must not be empty"));

        let identical = exec_tool(
            &registry,
            &ctx,
            "fs_patch",
            json!({ "path": "script.js", "edits": [{ "old": "abc", "new": "abc" }] }),
        )
        .unwrap_err();
        assert!(identical.to_string().contains("identical"));
    }

    #[test]
    fn skill_run_docx_and_excel_emit_valid_ooxml() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();

        create_docx_via_skill_run(&ctx, &registry, "report.docx", "报告", "正文");
        assert_valid_ooxml(&dir.path().join("report.docx"));

        exec_tool(
            &registry,
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
    fn skill_read_docx_editing_md() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_read",
            json!({ "skill": "docx", "doc": "editing.md" }),
        )
        .unwrap();
        assert_eq!(out["skill"], "docx");
        assert_eq!(out["doc"], "editing.md");
        let content = out["content"].as_str().unwrap();
        assert!(content.contains("ooxml_unpack"));
        assert!(content.contains("fs.readFileSync"));
    }

    #[test]
    fn skill_run_styled_chinese_docx_has_no_style_warnings() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        create_docx_via_skill_run(
            &ctx,
            &registry,
            "intro.docx",
            "广州软件学院 人工智能专业介绍",
            "本专业培养具备人工智能理论基础与应用能力的人才。",
        );
        let path = dir.path().join("intro.docx");
        assert_valid_ooxml(&path);
        let warnings = lint_docx(&path).unwrap();
        assert!(
            warnings.is_empty(),
            "styled doc should not warn: {warnings:?}"
        );
    }

    #[test]
    fn skill_run_unstyled_docx_returns_style_warnings() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"
async function main() {
  const { Document, Packer, Paragraph, TextRun } = docx;
  const body = "这是一段很长的正文内容用于触发排版检查。".repeat(20);
  const doc = new Document({ sections: [{ children: [
    new Paragraph({ children: [new TextRun(body)] }),
  ] }] });
  const b64 = await Packer.toBase64String(doc);
  doc_write("bad.docx", b64);
  return { ok: true };
}
"#;
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert!(
            out.get("style_warnings").is_some(),
            "expected style_warnings in {:?}",
            out
        );
        let warnings = out["style_warnings"]["bad.docx"].as_array().unwrap();
        assert!(!warnings.is_empty());
        assert!(out.get("style_hint").is_some());
        assert_eq!(out["script_path"], ".skill-run/script.js");
        assert_eq!(out["script_retain_reason"], "style_warnings");
        assert!(dir.path().join(".skill-run/script.js").exists());
    }

    #[test]
    fn skill_run_docx_retains_script_for_post_check() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        create_docx_via_skill_run(&ctx, &registry, "report.docx", "标题", "正文");
        assert!(dir.path().join(".skill-run/script.js").exists());
    }

    #[test]
    fn skill_run_path_rerun_after_docx_fix_keeps_script_within_turn() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        create_docx_via_skill_run(&ctx, &registry, "report.docx", "标题", "正文");
        assert!(dir.path().join(".skill-run/script.js").exists());

        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "path": ".skill-run/script.js", "timeout_secs": 60 }),
        )
        .unwrap();
        assert_eq!(out["result"]["ok"], true);
        assert!(
            dir.path().join(".skill-run/script.js").exists(),
            "path rerun should keep script for further in-turn fixes"
        );

        // Turn 结束兜底：无失败现场 → 清理
        crate::tools::skill_run_tmp::cleanup_on_turn_end(&ctx);
        assert!(!dir.path().join(".skill-run").exists());
    }

    #[test]
    fn skill_run_success_clears_stale_error_json() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let bad = r#"async function main() {
  p("简称"广软"），");
}"#;
        exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": bad, "timeout_secs": 30 }),
        )
        .unwrap_err();
        assert!(dir.path().join(".skill-run/error.json").exists());

        // 修复后重跑成功（写出 docx → 保留脚本），error.json 必须被清除
        create_docx_via_skill_run(&ctx, &registry, "report.docx", "标题", "正文");
        assert!(dir.path().join(".skill-run/script.js").exists());
        assert!(
            !dir.path().join(".skill-run/error.json").exists(),
            "successful run must clear stale error.json"
        );
    }

    #[test]
    fn cleanup_on_turn_end_keeps_dir_when_failure_pending() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let bad = r#"async function main() {
  p("简称"广软"），");
}"#;
        exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": bad, "timeout_secs": 30 }),
        )
        .unwrap_err();

        // 失败现场（error.json 存在）→ turn 结束不清理，留给下一 turn 修复
        crate::tools::skill_run_tmp::cleanup_on_turn_end(&ctx);
        assert!(dir.path().join(".skill-run/script.js").exists());
        assert!(dir.path().join(".skill-run/error.json").exists());
    }

    #[test]
    fn skill_run_fs_read_binary_returns_bytes() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        std::fs::write(dir.path().join("blob.bin"), [0u8, 159, 146, 150]).unwrap();
        let code = r#"
const bytes = fs.readFileSync('blob.bin');
return { isBytes: bytes instanceof Uint8Array, len: bytes.length, first: bytes[0], last: bytes[3] };
"#;
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 30 }),
        )
        .unwrap();
        assert_eq!(out["result"]["isBytes"], true);
        assert_eq!(out["result"]["len"].as_f64(), Some(4.0));
        assert_eq!(out["result"]["first"].as_f64(), Some(0.0));
        assert_eq!(out["result"]["last"].as_f64(), Some(150.0));
    }

    #[test]
    fn skill_run_fs_edits_unpacked_docx_xml() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        create_docx_via_skill_run(&ctx, &registry, "tpl.docx", "第N讲", "占位");
        exec_tool(
            &registry,
            &ctx,
            "ooxml_unpack",
            json!({ "path": "tpl.docx", "out_dir": "unpacked" }),
        )
        .unwrap();
        let code = r#"
const fs = require('fs');
const xmlPath = 'unpacked/word/document.xml';
let xml = fs.readFileSync(xmlPath, 'utf-8');
xml = xml.replace('第N讲', '第2讲 AI辅助应用开发工具');
fs.writeFileSync(xmlPath, xml, 'utf-8');
return { ok: true };
"#;
        exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 30 }),
        )
        .unwrap();
        exec_tool(
            &registry,
            &ctx,
            "ooxml_pack",
            json!({
                "dir": "unpacked",
                "out_path": "filled.docx",
                "original": "tpl.docx"
            }),
        )
        .unwrap();
        assert_valid_ooxml(&dir.path().join("filled.docx"));
    }

    #[test]
    fn excel_read_after_write() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();

        exec_tool(
            &registry,
            &ctx,
            "excel_write",
            json!({
                "path": "sheet.xlsx",
                "cells": [{ "cell": "A1", "value": "名称" }]
            }),
        )
        .unwrap();
        let read = exec_tool(
            &registry,
            &ctx,
            "excel_read",
            json!({ "path": "sheet.xlsx" }),
        )
        .unwrap();
        let rows = read["rows"].as_array().unwrap();
        assert_eq!(rows[0][0], "名称");
    }

    #[test]
    fn skill_read_returns_docx_guide_without_python_commands() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let out = exec_tool(&registry, &ctx, "skill_read", json!({ "skill": "docx" })).unwrap();
        let content = out["content"].as_str().unwrap();
        assert!(content.contains("doc-agent 系统约束"));
        assert!(content.contains("office_read_to_markdown"));
        assert!(!content.contains("python "));
    }

    #[test]
    fn skill_docs_contain_no_external_commands() {
        for skill in crate::core::skills::SKILLS {
            for doc in skill.docs {
                for forbidden in [
                    "python ",
                    "npm install",
                    "pip install",
                    "pdftoppm",
                    "soffice",
                    "qpdf",
                    "pdftk",
                    "pandoc ",
                    "openpyxl",
                    "pdfplumber",
                ] {
                    assert!(
                        !doc.content.contains(forbidden),
                        "{}/{} 残留外部命令: {forbidden}",
                        skill.name,
                        doc.name
                    );
                }
            }
        }
    }

    #[test]
    fn skill_run_exceljs_loads_and_modifies_existing_xlsx() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        exec_tool(
            &registry,
            &ctx,
            "excel_write",
            json!({ "path": "existing.xlsx", "cells": [{ "cell": "A1", "value": "旧值" }] }),
        )
        .unwrap();
        let code = r#"
async function main() {
  const buf = fs.readFileSync("existing.xlsx");
  const wb = new ExcelJS.Workbook();
  await wb.xlsx.load(buf.buffer);
  const ws = wb.getWorksheet(1);
  ws.getCell("A1").value = "新值";
  await wb.xlsx.writeFile("modified.xlsx");
  return { ok: true };
}
"#;
        exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert_valid_ooxml(&dir.path().join("modified.xlsx"));
        let read = exec_tool(
            &registry,
            &ctx,
            "excel_read",
            json!({ "path": "modified.xlsx" }),
        )
        .unwrap();
        assert!(read.to_string().contains("新值"));
    }

    #[test]
    fn skill_read_resolves_doc_filename_as_skill() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_read",
            json!({ "skill": "pptxgenjs.md" }),
        )
        .unwrap();
        assert_eq!(out["skill"], "pptx");
        assert_eq!(out["doc"], "pptxgenjs.md");
        let content = out["content"].as_str().unwrap();
        assert!(content.contains("PptxGenJS"));
    }

    #[test]
    fn skill_run_pptx_without_main_wrapper() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"
const PptxGenJS = require('pptxgenjs');
const pptx = new PptxGenJS();
pptx.addSlide().addText("封面", { x: 1, y: 1, fontSize: 24 });
await pptx.writeFile({ fileName: "deck.pptx" });
return { ok: true };
"#;
        exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert_valid_ooxml(&dir.path().join("deck.pptx"));
    }

    #[test]
    fn skill_read_unknown_lists_available() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err =
            exec_tool(&registry, &ctx, "skill_read", json!({ "skill": "unknown" })).unwrap_err();
        assert!(err.to_string().contains("docx"));
    }

    #[test]
    fn skill_run_executes_simple_script() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({
                "code": "function main() { return { ok: true, n: 1 + 2 }; }"
            }),
        )
        .unwrap();
        assert_eq!(out["result"]["ok"], true);
        assert_eq!(out["result"]["n"], 3);
        assert!(
            !dir.path().join(".skill-run").exists(),
            "successful skill_run should clean .skill-run"
        );
    }

    #[test]
    fn skill_run_rejects_code_and_path_together() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": "function main() { return 1; }", "path": "a.js" }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("either code or path"));
    }

    #[test]
    fn skill_run_rejects_missing_source() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err = exec_tool(&registry, &ctx, "skill_run", json!({})).unwrap_err();
        assert!(err.to_string().contains("code or path required"));
    }

    #[test]
    fn skill_run_failure_preserves_temp_script_and_error_json() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"async function main() {
  p("简称"广软"），");
}"#;
        let err = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 30 }),
        )
        .unwrap_err();
        let value = err.to_json_value();
        assert_eq!(value["error"], "JavaScript parse error");
        assert_eq!(value["script_path"], ".skill-run/script.js");
        assert!(value.get("quote_diagnostics").is_some());
        assert!(dir.path().join(".skill-run/script.js").exists());
        assert!(dir.path().join(".skill-run/error.json").exists());
        let saved = fs::read_to_string(dir.path().join(".skill-run/script.js")).unwrap();
        assert!(saved.contains("广软"));
    }

    #[test]
    fn skill_run_path_rerun_after_repair_cleans_temp_dir() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let bad = r#"async function main() {
  p("简称"广软"），");
}"#;
        exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": bad, "timeout_secs": 30 }),
        )
        .unwrap_err();
        let fixed = r#"async function main() {
  return { ok: true, text: '简称"广软"）' };
}"#;
        fs::write(dir.path().join(".skill-run/script.js"), fixed).unwrap();
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "path": ".skill-run/script.js", "timeout_secs": 30 }),
        )
        .unwrap();
        assert_eq!(out["result"]["ok"], true);
        assert!(!dir.path().join(".skill-run").exists());
    }

    #[test]
    fn skill_run_path_rejects_escape_outside_project() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "path": "../outside.js" }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("sandbox") || err.to_string().contains("escapes"));
    }

    #[test]
    fn skill_run_result_carries_written_paths() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let out = exec_tool(&registry,
                &ctx,
                "skill_run",
                json!({
                    "code": "function main() { doc_write('out/a.txt', btoa('hi')); return { ok: true }; }"
                }),
            )
            .unwrap();
        assert_eq!(out["written_paths"], json!(["out/a.txt"]));

        // 无写入时不携带 written_paths 字段
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": "function main() { return { ok: true }; }" }),
        )
        .unwrap();
        assert!(out.get("written_paths").is_none());
    }

    #[test]
    fn skill_run_supports_async_main() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let out = exec_tool(&registry,
                &ctx,
                "skill_run",
                json!({
                    "code": "async function main() { const v = await Promise.resolve(41); return { n: v + 1 }; }"
                }),
            )
            .unwrap();
        assert_eq!(out["result"]["n"], 42);
    }

    #[test]
    fn skill_run_exceljs_writes_xlsx() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"
async function main() {
  const wb = new ExcelJS.Workbook();
  const ws = wb.addWorksheet("S1");
  ws.getCell("A1").value = "标题";
  const buf = await wb.xlsx.writeBuffer();
  const bytes = new Uint8Array(buf);
  let bin = "";
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  doc_write("out.xlsx", btoa(bin));
  return { bytes: bytes.length };
}
"#;
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert!(out["result"]["bytes"].as_u64().unwrap() > 1000);
        assert_valid_ooxml(&dir.path().join("out.xlsx"));
    }

    #[test]
    fn skill_run_exceljs_write_file_shim() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"
async function main() {
  const wb = new ExcelJS.Workbook();
  const ws = wb.addWorksheet("公共必修课");
  ws.columns = [{ header: "课程", key: "n", width: 20 }];
  ws.addRow({ n: "高等数学" });
  await wb.xlsx.writeFile("out.xlsx");
  return { ok: true };
}
"#;
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert_eq!(out["result"]["ok"], true);
        assert_valid_ooxml(&dir.path().join("out.xlsx"));
    }

    #[test]
    fn skill_run_require_exceljs_pattern() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"
const ExcelJS = require('exceljs');
async function main() {
  const wb = new ExcelJS.Workbook();
  const ws = wb.addWorksheet('test');
  ws.getCell('A1').value = 'hello';
  await wb.xlsx.writeFile('out.xlsx');
  return { ok: true };
}
main();
"#;
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert_eq!(out["result"]["ok"], true);
        assert_valid_ooxml(&dir.path().join("out.xlsx"));
    }

    #[test]
    fn skill_run_buffer_from_doc_write_pattern() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"
const ExcelJS = require('exceljs');
async function main() {
  const wb = new ExcelJS.Workbook();
  wb.addWorksheet('s').getCell('A1').value = 'x';
  const buf = await wb.xlsx.writeBuffer();
  doc_write('buf.xlsx', Buffer.from(buf).toString('base64'));
  return { ok: true };
}
"#;
        exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert_valid_ooxml(&dir.path().join("buf.xlsx"));
    }

    #[test]
    fn skill_run_unknown_require_returns_hint() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({
                "code": "function main() { require('exceljs'); return require('unknown-pkg'); }"
            }),
        )
        .unwrap_err();
        let msg = err.to_json_value()["detail"]
            .as_str()
            .unwrap_or(&err.to_string())
            .to_string();
        assert!(msg.contains("Cannot find module"), "msg: {msg}");
    }

    #[test]
    fn ooxml_unpack_pack_roundtrip() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        create_docx_via_skill_run(&ctx, &registry, "src.docx", "标题", "正文内容");
        let unpacked = exec_tool(
            &registry,
            &ctx,
            "ooxml_unpack",
            json!({ "path": "src.docx", "out_dir": "unpacked" }),
        )
        .unwrap();
        assert!(unpacked["parts"].as_u64().unwrap() > 0);
        assert!(dir.path().join("unpacked/word/document.xml").exists());

        let packed = exec_tool(
            &registry,
            &ctx,
            "ooxml_pack",
            json!({ "dir": "unpacked", "out_path": "repacked.docx", "original": "src.docx" }),
        )
        .unwrap();
        assert!(packed["path"].as_str().unwrap().ends_with("repacked.docx"));
        assert_valid_ooxml(&dir.path().join("repacked.docx"));
    }

    #[test]
    fn xlsx_recalc_returns_report_shape() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        exec_tool(
            &registry,
            &ctx,
            "excel_write",
            json!({
                "path": "sheet.xlsx",
                "cells": [{ "cell": "A1", "value": "100" }]
            }),
        )
        .unwrap();
        let out = exec_tool(
            &registry,
            &ctx,
            "xlsx_recalc",
            json!({ "path": "sheet.xlsx" }),
        )
        .unwrap();
        assert!(out["errors"].is_array());
        assert!(out["warnings"].is_array());
    }

    #[test]
    fn smoke_ppt_creation_chain() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"
async function main() {
  const Pptx = PptxGenJS.default ?? PptxGenJS;
  const pres = new Pptx();
  const slide = pres.addSlide();
  slide.addText("季度汇报", { x: 1, y: 1, fontSize: 28, bold: true });
  const buf = await pres.write({ outputType: "arraybuffer" });
  const bytes = new Uint8Array(buf);
  let bin = "";
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  doc_write("deck.pptx", btoa(bin));
  return { ok: true, bytes: bytes.length };
}
"#;
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert_eq!(out["result"]["ok"], true);
        assert_valid_ooxml(&dir.path().join("deck.pptx"));
    }

    #[test]
    fn smoke_redline_comment_chain() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        create_docx_via_skill_run(
            &ctx,
            &registry,
            "contract.docx",
            "合同",
            "甲方应于30日内付款。",
        );
        exec_tool(
            &registry,
            &ctx,
            "ooxml_unpack",
            json!({ "path": "contract.docx", "out_dir": "contract_unpacked" }),
        )
        .unwrap();
        exec_tool(&registry,
                &ctx,
                "docx_comment",
                json!({ "dir": "contract_unpacked", "id": 1, "text": "建议明确付款方式", "author": "审阅人" }),
            )
            .unwrap();
        exec_tool(&registry,
                &ctx,
                "ooxml_pack",
                json!({ "dir": "contract_unpacked", "out_path": "contract_reviewed.docx", "original": "contract.docx" }),
            )
            .unwrap();
        let reviewed = dir.path().join("contract_reviewed.docx");
        assert_valid_ooxml(&reviewed);
        let file = fs::File::open(&reviewed).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(
            names.iter().any(|n| n.contains("comments.xml")),
            "expected comments.xml in {names:?}"
        );
    }

    #[test]
    fn smoke_pdf_data_pipeline() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let code = r#"
async function main() {
  const doc = await PDFLib.PDFDocument.create();
  const page = doc.addPage([400, 300]);
  const font = await doc.embedFont(PDFLib.StandardFonts.Helvetica);
  const rows = [
    ["item", "amount"],
    ["apple", "10"],
    ["pear", "25"],
  ];
  rows.forEach((row, r) => {
    page.drawText(row[0], { x: 50, y: 250 - r * 30, size: 12, font });
    page.drawText(row[1], { x: 200, y: 250 - r * 30, size: 12, font });
  });
  const bytes = await doc.save();
  let bin = "";
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  doc_write("report.pdf", btoa(bin));
  return { ok: true };
}
"#;
        exec_tool(
            &registry,
            &ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
        assert!(dir.path().join("report.pdf").exists());

        exec_tool(
            &registry,
            &ctx,
            "fs_write",
            json!({
                "path": "table.csv",
                "content": "\"item\",\"amount\"\n\"apple\",\"10\"\n\"pear\",\"25\"\n"
            }),
        )
        .unwrap();

        let out = exec_tool(
            &registry,
            &ctx,
            "data_query",
            json!({
                "sources": [{ "name": "t", "path": "table.csv" }],
                "sql": "SELECT SUM(CAST(amount AS INT)) AS total FROM t"
            }),
        )
        .unwrap();
        let text = out.to_string();
        assert!(text.contains("35"), "query out: {text}");
    }

    #[test]
    fn pdf_merge_combines_pages_in_order() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "a.pdf", 2);
        make_multi_page_pdf(&ctx, &registry, "b.pdf", 3);

        let out = exec_tool(
            &registry,
            &ctx,
            "pdf_merge",
            json!({ "inputs": ["a.pdf", "b.pdf"], "out_path": "merged.pdf" }),
        )
        .unwrap();
        assert_eq!(out["pages"], 5);
        assert_eq!(pdf_page_count(&dir.path().join("merged.pdf")), 5);

        let doc = lopdf::Document::load(dir.path().join("merged.pdf")).unwrap();
        let first = doc.extract_text(&[1]).unwrap_or_default();
        let third = doc.extract_text(&[3]).unwrap_or_default();
        assert!(first.contains("Page 1"), "page1 text: {first}");
        assert!(third.contains("Page 1"), "page3 text: {third}");
    }

    #[test]
    fn pdf_merge_rejects_empty_inputs() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err = exec_tool(
            &registry,
            &ctx,
            "pdf_merge",
            json!({ "inputs": [], "out_path": "out.pdf" }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("至少需要一个输入 PDF"));
    }

    #[test]
    fn pdf_split_range_and_burst() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "src.pdf", 4);

        let out = exec_tool(
            &registry,
            &ctx,
            "pdf_split",
            json!({
                "path": "src.pdf",
                "ranges": "1-2,4",
                "out_path": "subset.pdf"
            }),
        )
        .unwrap();
        assert_eq!(out["pages"], 3);
        assert_eq!(pdf_page_count(&dir.path().join("subset.pdf")), 3);

        let burst = exec_tool(
            &registry,
            &ctx,
            "pdf_split",
            json!({ "path": "src.pdf", "mode": "burst", "out_dir": "pages" }),
        )
        .unwrap();
        let files = burst["files"].as_array().unwrap();
        assert_eq!(files.len(), 4);
        for i in 1..=4 {
            assert_eq!(
                pdf_page_count(&dir.path().join(format!("pages/page_{i}.pdf"))),
                1
            );
        }
    }

    #[test]
    fn pdf_split_rejects_out_of_range() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "src.pdf", 2);
        let err = exec_tool(
            &registry,
            &ctx,
            "pdf_split",
            json!({
                "path": "src.pdf",
                "ranges": "1-5",
                "out_path": "subset.pdf"
            }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("越界"));
    }

    #[test]
    fn pdf_rotate_and_invalid_angle() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "src.pdf", 3);

        exec_tool(
            &registry,
            &ctx,
            "pdf_rotate",
            json!({
                "path": "src.pdf",
                "rotation": 90,
                "pages": [2],
                "out_path": "rotated.pdf"
            }),
        )
        .unwrap();
        assert_eq!(pdf_page_rotate(&dir.path().join("rotated.pdf"), 2), 90);
        assert_eq!(pdf_page_rotate(&dir.path().join("rotated.pdf"), 1), 0);

        let err = exec_tool(
            &registry,
            &ctx,
            "pdf_rotate",
            json!({
                "path": "src.pdf",
                "rotation": 45,
                "out_path": "bad.pdf"
            }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("90 的倍数"));
    }

    #[test]
    fn pdf_delete_pages_and_reject_delete_all() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "src.pdf", 5);

        let out = exec_tool(
            &registry,
            &ctx,
            "pdf_delete_pages",
            json!({
                "path": "src.pdf",
                "pages": [2, 4],
                "out_path": "trimmed.pdf"
            }),
        )
        .unwrap();
        assert_eq!(out["pages"], 3);
        assert_eq!(pdf_page_count(&dir.path().join("trimmed.pdf")), 3);

        let err = exec_tool(
            &registry,
            &ctx,
            "pdf_delete_pages",
            json!({
                "path": "src.pdf",
                "pages": [1, 2, 3, 4, 5],
                "out_path": "empty.pdf"
            }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("不能删除所有页"));
        assert!(!dir.path().join("empty.pdf").exists());
    }

    #[test]
    fn skill_read_pdf_reference_doc() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let out = exec_tool(
            &registry,
            &ctx,
            "skill_read",
            json!({ "skill": "pdf", "doc": "reference.md" }),
        )
        .unwrap();
        let content = out["content"].as_str().unwrap();
        assert!(content.contains("pdf_merge"));
        assert!(content.contains("1-based"));
        assert!(!content.contains("qpdf"));
    }

    #[test]
    fn office_read_rejects_unsupported_extension() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("data.xyz"), "hello").unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err = exec_tool(
            &registry,
            &ctx,
            "office_read_to_markdown",
            json!({ "path": "data.xyz" }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("unsupported"));
    }

    #[test]
    fn preprocess_normalize_headers_empty_names() {
        use crate::tools::data::preprocess::normalize_headers;
        let raw = vec!["A".into(), "".into(), "C".into()];
        let (names, renamed) = normalize_headers(&raw);
        assert_eq!(names, vec!["A", "column_2", "C"]);
        assert_eq!(renamed.len(), 1);
        assert_eq!(renamed[0], ("".to_string(), "column_2".to_string()));
    }

    #[test]
    fn preprocess_normalize_headers_duplicates() {
        use crate::tools::data::preprocess::normalize_headers;
        let raw = vec!["完成人".into(), "完成人".into(), "完成人".into()];
        let (names, _) = normalize_headers(&raw);
        assert_eq!(names, vec!["完成人", "完成人_2", "完成人_3"]);
    }

    #[test]
    fn preprocess_normalize_headers_trim_and_newlines() {
        use crate::tools::data::preprocess::normalize_headers;
        let raw = vec!["  完成\n人  ".into(), "A".into()];
        let (names, renamed) = normalize_headers(&raw);
        assert_eq!(names, vec!["完成人", "A"]);
        assert!(renamed
            .iter()
            .any(|(o, n)| o.contains('\n') && n == "完成人"));
    }

    #[test]
    fn preprocess_normalize_headers_suffix_collision() {
        use crate::tools::data::preprocess::normalize_headers;
        let raw = vec!["完成人_2".into(), "完成人".into(), "完成人".into()];
        let (names, _) = normalize_headers(&raw);
        assert_eq!(names, vec!["完成人_2", "完成人", "完成人_3"]);
    }

    #[test]
    fn preprocess_fill_merged_vertical() {
        use crate::tools::data::preprocess::{fill_merged, MergedRegion};
        let mut cells = vec![
            vec!["hdr".into(), "b".into()],
            vec!["anchor".into(), "x".into()],
            vec!["".into(), "y".into()],
            vec!["".into(), "z".into()],
        ];
        fill_merged(
            &mut cells,
            &[MergedRegion {
                start: (1, 0),
                end: (3, 0),
            }],
        );
        assert_eq!(cells[2][0], "anchor");
        assert_eq!(cells[3][0], "anchor");
    }

    #[test]
    fn preprocess_fill_merged_horizontal() {
        use crate::tools::data::preprocess::{fill_merged, MergedRegion};
        let mut cells = vec![vec!["hdr".into(), "".into(), "".into()]];
        fill_merged(
            &mut cells,
            &[MergedRegion {
                start: (0, 0),
                end: (0, 2),
            }],
        );
        assert_eq!(cells[0], vec!["hdr", "hdr", "hdr"]);
    }

    #[test]
    fn preprocess_fill_merged_skips_empty_anchor() {
        use crate::tools::data::preprocess::{fill_merged, MergedRegion};
        let mut cells = vec![vec!["".into(), "b".into()], vec!["".into(), "c".into()]];
        fill_merged(
            &mut cells,
            &[MergedRegion {
                start: (0, 0),
                end: (1, 0),
            }],
        );
        assert_eq!(cells[1][0], "");
    }

    #[test]
    fn preprocess_suggest_header_row_with_title_line() {
        use crate::tools::data::preprocess::suggest_header_row;
        let cells = vec![
            vec!["软件工程专业评估指标点".into(), "".into(), "".into()],
            vec!["指标点".into(), "材料提供人".into(), "完成人".into()],
            vec!["1.1".into(), "谭".into(), "张".into()],
        ];
        assert_eq!(suggest_header_row(&cells), 1);
    }

    #[test]
    fn preprocess_suggest_header_row_regular_table() {
        use crate::tools::data::preprocess::suggest_header_row;
        let cells = vec![
            vec!["A".into(), "B".into(), "C".into()],
            vec!["1".into(), "2".into(), "3".into()],
        ];
        assert_eq!(suggest_header_row(&cells), 0);
    }

    fn build_messy_xlsx(path: &std::path::Path) {
        let mut book = umya_spreadsheet::new_file();
        let sheet = book.sheet_mut(0).unwrap();
        sheet.cell_mut("A1").set_value("软件工程专业评估指标点");
        sheet.cell_mut("A2").set_value("指标点");
        sheet.cell_mut("B2").set_value("材料提供人");
        sheet.cell_mut("C2").set_value("完成人");
        sheet.cell_mut("D2").set_value("");
        sheet.cell_mut("E2").set_value("完成人");
        sheet.cell_mut("A3").set_value("1.1 毕业要求");
        sheet.add_merge_cells("A3:A5");
        sheet.cell_mut("B3").set_value("谭艳娴");
        sheet.cell_mut("C3").set_value("张三");
        sheet.cell_mut("D3").set_value("10");
        sheet.cell_mut("E3").set_value("20");
        sheet.cell_mut("B4").set_value("李四");
        sheet.cell_mut("C4").set_value("王五");
        sheet.cell_mut("D4").set_value("30");
        sheet.cell_mut("E4").set_value("40");
        sheet.cell_mut("B5").set_value("赵六");
        sheet.cell_mut("C5").set_value("孙七");
        sheet.cell_mut("D5").set_value("50");
        sheet.cell_mut("E5").set_value("60");
        umya_spreadsheet::writer::xlsx::write(&book, path).unwrap();
    }

    #[test]
    fn excel_normalize_messy_xlsx() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        build_messy_xlsx(&dir.path().join("messy.xlsx"));

        let out = exec_tool(
            &registry,
            &ctx,
            "excel_normalize",
            json!({
                "path": "messy.xlsx",
                "header_row": 1,
                "out_path": "normalized/messy.csv"
            }),
        )
        .unwrap();
        let columns = out["columns"].as_array().unwrap();
        let col_names: Vec<&str> = columns.iter().map(|c| c.as_str().unwrap()).collect();
        assert_eq!(col_names[0], "指标点");
        assert_eq!(col_names[3], "column_4");
        assert_eq!(col_names[4], "完成人_2");
        assert_eq!(out["rows"], 3);

        let csv = fs::read_to_string(dir.path().join("normalized/messy.csv")).unwrap();
        assert!(csv.contains("1.1 毕业要求"));
        assert!(csv.matches("1.1 毕业要求").count() >= 3);
    }

    #[test]
    fn excel_describe_messy_xlsx() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        build_messy_xlsx(&dir.path().join("messy.xlsx"));

        let out = exec_tool(
            &registry,
            &ctx,
            "excel_describe",
            json!({ "path": "messy.xlsx", "preview_rows": 10 }),
        )
        .unwrap();
        assert_eq!(out["suggested_header_row"], 1);
        let merged = out["merged_regions"].as_array().unwrap();
        assert!(
            merged
                .iter()
                .any(|m| m["range"].as_str().unwrap().contains("A3")),
            "merged: {merged:?}"
        );
        let warnings = out["warnings"].as_array().unwrap();
        let text = warnings
            .iter()
            .map(|w| w.as_str().unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(text.contains("空表头"));
        assert!(text.contains("重复"));
        assert!(text.contains("合并单元格"));
        assert!(text.contains("表头不在首行"));
    }

    #[test]
    fn data_query_messy_xlsx_no_dup_error() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        build_messy_xlsx(&dir.path().join("messy.xlsx"));

        let out = exec_tool(
            &registry,
            &ctx,
            "data_query",
            json!({
                "sources": [{ "name": "t", "path": "messy.xlsx" }],
                "sql": "SELECT * FROM t LIMIT 3"
            }),
        )
        .unwrap();
        assert!(out["data"].is_array());
    }

    #[test]
    fn data_query_error_contains_schema() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        exec_tool(
            &registry,
            &ctx,
            "fs_write",
            json!({
                "path": "t.csv",
                "content": "\"a\",\"b\"\n\"1\",\"2\"\n"
            }),
        )
        .unwrap();

        let err = exec_tool(
            &registry,
            &ctx,
            "data_query",
            json!({
                "sources": [{ "name": "t", "path": "t.csv" }],
                "sql": "SELECT missing_col FROM t"
            }),
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("可用表结构"));
        assert!(msg.contains("excel_describe"));
        assert!(msg.contains("a"));
    }

    #[test]
    fn normalize_csv_sum_query() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        build_messy_xlsx(&dir.path().join("messy.xlsx"));
        exec_tool(
            &registry,
            &ctx,
            "excel_normalize",
            json!({
                "path": "messy.xlsx",
                "header_row": 1,
                "out_path": "normalized/messy.csv"
            }),
        )
        .unwrap();

        let out = exec_tool(
            &registry,
            &ctx,
            "data_query",
            json!({
                "sources": [{ "name": "t", "path": "normalized/messy.csv" }],
                "sql": "SELECT SUM(CAST(column_4 AS INT)) AS total FROM t"
            }),
        )
        .unwrap();
        let text = out.to_string();
        assert!(text.contains("90"), "sum out: {text}");
    }

    #[test]
    fn office_read_rejects_invalid_pdf() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("broken.pdf"), b"not-a-pdf").unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext::new(&sandbox);
        let registry = ToolRegistry::default_tools();
        let err = exec_tool(
            &registry,
            &ctx,
            "office_read_to_markdown",
            json!({ "path": "broken.pdf" }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("PDF"));
    }
}
