# doc-agent

本地 Office 文档 Agent（Tauri 2 + Rust + React）。

## Document Skills 运行时（内置）

内置 docx / pdf / pptx / xlsx 四类 skill 知识库，Agent 可通过 `skill_read` 渐进披露全文，通过 `skill_run` 执行 JavaScript（内置 exceljs / docx / pptxgenjs / pdf-lib bundle，按需加载）。

新增工具：`ooxml_unpack` / `ooxml_pack`、`docx_comment`、`docx_accept_changes`、`docx_extract_table`、`data_query`（polars-sql）、`xlsx_recalc`（IronCalc）、`pdf_merge` / `pdf_split` / `pdf_rotate` / `pdf_delete_pages`。

构建前需打包 JS 库：`npm run bundle:js`

体积增量约 +50–70MB（polars、boa_engine、IronCalc 等）。

## 开发

```bash
npm run bundle:js
npm run tauri dev
cd src-tauri && cargo test
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
