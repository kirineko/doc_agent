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
        registry
            .execute(
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
        let names: Vec<_> = registry.definitions().into_iter().map(|d| d.name).collect();
        for expected in [
            "fs_list",
            "fs_read",
            "fs_write",
            "fs_search",
            "office_read_to_markdown",
            "word_create",
            "excel_read",
            "excel_write",
            "skill_read",
            "skill_run",
            "ooxml_unpack",
            "ooxml_pack",
            "docx_comment",
            "docx_accept_changes",
            "docx_extract_table",
            "data_query",
            "xlsx_recalc",
            "pdf_merge",
            "pdf_split",
            "pdf_rotate",
            "pdf_delete_pages",
        ] {
            assert!(
                names.contains(&expected.to_string()),
                "missing tool {expected}"
            );
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
        assert_eq!(
            fs::read_to_string(dir.path().join("out.txt")).unwrap(),
            "saved"
        );

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
    fn skill_read_docx_editing_md() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let out = registry
            .execute(
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
    fn skill_run_fs_read_binary_returns_bytes() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        std::fs::write(dir.path().join("blob.bin"), [0u8, 159, 146, 150]).unwrap();
        let code = r#"
const bytes = fs.readFileSync('blob.bin');
return { isBytes: bytes instanceof Uint8Array, len: bytes.length, first: bytes[0], last: bytes[3] };
"#;
        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        registry
            .execute(
                &ctx,
                "word_create",
                json!({ "path": "tpl.docx", "title": "第N讲", "body": "占位" }),
            )
            .unwrap();
        registry
            .execute(
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
        registry
            .execute(
                &ctx,
                "skill_run",
                json!({ "code": code, "timeout_secs": 30 }),
            )
            .unwrap();
        registry
            .execute(
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
    fn skill_read_returns_docx_guide_without_python_commands() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let out = registry
            .execute(&ctx, "skill_read", json!({ "skill": "docx" }))
            .unwrap();
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        registry
            .execute(
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
        registry
            .execute(
                &ctx,
                "skill_run",
                json!({ "code": code, "timeout_secs": 60 }),
            )
            .unwrap();
        assert_valid_ooxml(&dir.path().join("modified.xlsx"));
        let read = registry
            .execute(&ctx, "excel_read", json!({ "path": "modified.xlsx" }))
            .unwrap();
        assert!(read.to_string().contains("新值"));
    }

    #[test]
    fn skill_read_resolves_doc_filename_as_skill() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let out = registry
            .execute(&ctx, "skill_read", json!({ "skill": "pptxgenjs.md" }))
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let code = r#"
const PptxGenJS = require('pptxgenjs');
const pptx = new PptxGenJS();
pptx.addSlide().addText("封面", { x: 1, y: 1, fontSize: 24 });
await pptx.writeFile({ fileName: "deck.pptx" });
return { ok: true };
"#;
        registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let err = registry
            .execute(&ctx, "skill_read", json!({ "skill": "unknown" }))
            .unwrap_err();
        assert!(err.to_string().contains("docx"));
    }

    #[test]
    fn skill_run_executes_simple_script() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let out = registry
            .execute(
                &ctx,
                "skill_run",
                json!({
                    "code": "function main() { return { ok: true, n: 1 + 2 }; }"
                }),
            )
            .unwrap();
        assert_eq!(out["result"]["ok"], true);
        assert_eq!(out["result"]["n"], 3);
    }

    #[test]
    fn skill_run_supports_async_main() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
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
        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
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
        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
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
        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
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
        registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let err = registry
            .execute(
                &ctx,
                "skill_run",
                json!({
                    "code": "function main() { require('exceljs'); return require('unknown-pkg'); }"
                }),
            )
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Cannot find module"), "msg: {msg}");
    }

    #[test]
    fn ooxml_unpack_pack_roundtrip() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        registry
            .execute(
                &ctx,
                "word_create",
                json!({ "path": "src.docx", "title": "标题", "body": "正文内容" }),
            )
            .unwrap();
        let unpacked = registry
            .execute(
                &ctx,
                "ooxml_unpack",
                json!({ "path": "src.docx", "out_dir": "unpacked" }),
            )
            .unwrap();
        assert!(unpacked["parts"].as_u64().unwrap() > 0);
        assert!(dir.path().join("unpacked/word/document.xml").exists());

        let packed = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        registry
            .execute(
                &ctx,
                "excel_write",
                json!({
                    "path": "sheet.xlsx",
                    "cells": [{ "cell": "A1", "value": "100" }]
                }),
            )
            .unwrap();
        let out = registry
            .execute(&ctx, "xlsx_recalc", json!({ "path": "sheet.xlsx" }))
            .unwrap();
        assert!(out["errors"].is_array());
        assert!(out["warnings"].is_array());
    }

    #[test]
    fn smoke_ppt_creation_chain() {
        let dir = tempdir().unwrap();
        let sandbox = setup(&dir);
        let ctx = ToolContext { sandbox: &sandbox };
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
        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        registry
            .execute(
                &ctx,
                "word_create",
                json!({ "path": "contract.docx", "title": "合同", "body": "甲方应于30日内付款。" }),
            )
            .unwrap();
        registry
            .execute(
                &ctx,
                "ooxml_unpack",
                json!({ "path": "contract.docx", "out_dir": "contract_unpacked" }),
            )
            .unwrap();
        registry
            .execute(
                &ctx,
                "docx_comment",
                json!({ "dir": "contract_unpacked", "id": 1, "text": "建议明确付款方式", "author": "审阅人" }),
            )
            .unwrap();
        registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
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
        registry
            .execute(
                &ctx,
                "skill_run",
                json!({ "code": code, "timeout_secs": 60 }),
            )
            .unwrap();
        assert!(dir.path().join("report.pdf").exists());

        registry
            .execute(
                &ctx,
                "fs_write",
                json!({
                    "path": "table.csv",
                    "content": "\"item\",\"amount\"\n\"apple\",\"10\"\n\"pear\",\"25\"\n"
                }),
            )
            .unwrap();

        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "a.pdf", 2);
        make_multi_page_pdf(&ctx, &registry, "b.pdf", 3);

        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let err = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "src.pdf", 4);

        let out = registry
            .execute(
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

        let burst = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "src.pdf", 2);
        let err = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "src.pdf", 3);

        registry
            .execute(
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

        let err = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        make_multi_page_pdf(&ctx, &registry, "src.pdf", 5);

        let out = registry
            .execute(
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

        let err = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let out = registry
            .execute(
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
        let ctx = ToolContext { sandbox: &sandbox };
        let registry = ToolRegistry::default_tools();
        let err = registry
            .execute(
                &ctx,
                "office_read_to_markdown",
                json!({ "path": "data.xyz" }),
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
