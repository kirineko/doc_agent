---
name: pdf
description: Use this skill whenever the user wants to do anything with PDF files. This includes reading or extracting text/tables from PDFs, combining or merging multiple PDFs into one, splitting PDFs apart, rotating pages, deleting pages, creating new PDFs, and filling PDF forms. If the user mentions a .pdf file or asks to produce one, use this skill.
license: Proprietary. LICENSE.txt has complete terms
---

# PDF Processing Guide

> 本系统无 shell/Python 环境，外部 PDF 命令行与 Python 库一律不可用。所有操作通过内置工具完成。

## Quick Reference

| Task | Tool |
|------|------|
| **读取 PDF（推荐）** | `pdf_read {"path": "doc.pdf"}` — **只传 path，不要传 mode**；vision 模型自动走图片理解 |
| 读取正文/纯文本 | `office_read_to_markdown {"path": "doc.pdf"}`（vision 模型纯文本用这个，勿 pdf_read+mode=text） |
| 仅渲染页图为 PNG | `pdf_render_pages {"path": "doc.pdf"}` → `.cache/pdf/`（可缓存命中） |
| 读取 1–4 张图片 | `image_read {"paths": ["a.png","b.png"]}`（vision 模型） |
| 合并 | `pdf_merge {"inputs": ["a.pdf", "b.pdf"], "out_path": "merged.pdf"}` |
| 拆分（按范围/逐页） | `pdf_split`（详见 reference.md） |
| 旋转页面 | `pdf_rotate`（90/180/270，详见 reference.md） |
| 删除页面 | `pdf_delete_pages {"path": "in.pdf", "pages": [2, 4], "out_path": "out.pdf"}` |
| 创建新 PDF | `skill_run` + `PDFLib`（见下方模板） |
| 表格数据整理 | `office_read_to_markdown` 提取文本 → 手工整理为 CSV → `data_query` |
| 填表单 | 见 forms.md（当前无自动填表工具） |

页码一律 **1-based**。页面工具详细参数与典型流程：`skill_read {"skill": "pdf", "doc": "reference.md"}`。

## 创建新 PDF（skill_run + pdf-lib）

```javascript
async function main() {
  const { PDFDocument, StandardFonts, rgb } = PDFLib;  // 全局 PDFLib，require('pdf-lib') 也可
  const pdfDoc = await PDFDocument.create();
  const page = pdfDoc.addPage([595, 842]);             // A4 点数
  const font = await pdfDoc.embedFont(StandardFonts.Helvetica);

  page.drawText("Hello PDF", { x: 50, y: 780, size: 24, font, color: rgb(0, 0, 0) });
  page.drawLine({ start: { x: 50, y: 770 }, end: { x: 545, y: 770 }, thickness: 1 });

  const b64 = await pdfDoc.saveAsBase64();
  doc_write("out.pdf", b64);
  return { ok: true };
}
// 不要在末尾调用 main()，运行时会自动调用
```

### ⚠️ 中文限制（重要）

pdf-lib 的 `StandardFonts`（Helvetica 等）**不支持中文**，`drawText` 中文会直接报编码错误，且本系统未内置中文字体文件。

**中文交付物降级方案**（按优先级）：
1. 改交付 **docx**（`skill_run` + docx 库，中文完整支持），如用户接受
2. 用户提供 `.ttf` 中文字体文件 → `fs.readFileSync(p)` 读字节 → `pdfDoc.embedFont(bytes)`（需 fontkit，可能不可用，先小样验证）
3. 仅含少量英文/数字的 PDF 才用 pdf-lib 直接生成

### 修改已有 PDF（pdf-lib load）

```javascript
async function main() {
  const { PDFDocument, degrees } = PDFLib;
  const bytes = fs.readFileSync("in.pdf");             // 不带 encoding = 字节
  const pdfDoc = await PDFDocument.load(bytes);
  // 加水印文字 / 改 metadata / 复制页面到新文档等
  pdfDoc.setTitle("New Title");
  const b64 = await pdfDoc.saveAsBase64();
  doc_write("out.pdf", b64);
  return { pages: pdfDoc.getPageCount() };
}
```

简单页面操作（合并/拆分/旋转/删除）**优先用原生工具**（`pdf_merge` 等），比 pdf-lib 更快更稳。

## 不支持的操作

| 操作 | 说明 |
|---|---|
| OCR 扫描件 | 无独立 OCR；扫描件用 **vision 模型** `pdf_read`（`mode=auto` 或 `mode=vision`），纯 `mode=text` 会报错 |
| 加密/解密 | 加密 PDF 不支持操作；提示用户先在外部工具解密 |
| 提取内嵌图片 | 无工具 |
| 自动填 AcroForm 表单 | 见 forms.md 的降级方案 |
| 结构化表格提取 | 无自动工具；用 `office_read_to_markdown` 文本 + 人工整理 |

## 典型流程

1. **含公式 / 扫描件 / 一般阅读**：`pdf_read` 只传 `path`（vision 会话）
2. **纯文本快速试探**：`office_read_to_markdown`（vision 会话不要用 pdf_read 的 mode=text）
3. `pdf_split` / `pdf_delete_pages` / `pdf_rotate` 调整结构
4. `pdf_merge` 合并交付物
5. 需要新建/重绘 → `skill_run` + PDFLib（注意中文限制）

---

## doc-agent 系统约束

- **页码 1-based**；越界返回明确错误。
- **合并**不保证保留书签/表单/注释，仅保证页面内容与顺序。
- **扫描 PDF**：无 OCR 能力。
