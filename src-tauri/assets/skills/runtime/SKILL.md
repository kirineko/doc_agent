---
name: runtime
description: skill_run 嵌入式 JavaScript 运行时能力矩阵（API、polyfill、限制）。编写或修复 skill_run 脚本前必读。
---

# skill_run 运行时

> 引擎：**boa_engine**（纯 Rust 嵌入式 JS，**非** Node/V8）。语言：**JavaScript only**（无 TypeScript）。无网络、无 npm、无 shell。

## 入口约定

```javascript
async function main() {
  // ...
  return { ok: true };  // 必须 JSON 可序列化
}
// ❌ 不要在末尾写 main() — 运行时自动调用
// ❌ 不要用 ES module import — 见下方「模块」
```

**自动 normalize**（执行前）：剥离 `require('fs'|'exceljs'|…)` 冗余行；常见 `import … from '…'` 改写成全局/require 等价；无 `main` 时自动包裹 `async function main() { … }`；移除末尾 `main()` 调用。

## 模块与库

| 方式 | 说明 |
|------|------|
| 全局 | `ExcelJS`、`PptxGenJS`、`docx`、`PDFLib`（bundle 按需加载后可用） |
| `require(id)` | 白名单：`fs`、`path`、`exceljs`、`pptxgenjs`、`docx`、`pdf-lib` |
| `import` | **不支持** ESM；单行 `import` 由 normalize 改写，无法识别则报错 |

Bundle 按代码关键字注入（含 `pptxgenjs` / `PptxGenJS` / `docx` / `exceljs` / `pdf-lib`）。**仅路径字符串 `.pptx` 不会加载 pptxgenjs**。

## 文件 API

| API | 说明 |
|-----|------|
| `doc_read(path)` | 读文件 → base64 字符串 |
| `doc_write(path, b64)` | 写文件（base64） |
| `doc_write_bytes(path, Uint8Array)` | 写二进制 |
| `doc_exists(path)` | 沙箱内路径是否存在（bool） |
| `doc_list(path?)` | 列目录直接子项 → `[{ name, is_dir }, …]`（默认 `"."`） |
| `fs.readFileSync(path, 'utf-8' \| 'base64')` | 文本或 base64；无 encoding → Buffer 字节 |
| `fs.writeFileSync(path, data, 'utf-8')` | 文本或字节写入 |
| `fs.existsSync(path)` | 同 `doc_exists` |
| `fs.readdirSync(path)` | 同 `doc_list` 的 name 数组（无 `is_dir` 时用 `doc_list`） |
| `path.join` / `dirname` / `basename` | 最小 path shim |

## 库保存（已 shim）

- xlsx：`await wb.xlsx.writeFile('out.xlsx')`
- pptx：`await pptx.writeFile({ fileName: 'deck.pptx' })`
- docx：`await docx.Packer.toBase64String(doc)` → `doc_write('out.docx', b64)`

## Polyfill（有限）

`setTimeout`/`setImmediate`：**无真实延迟**（微任务）；`console.*` → `doc_log`；`Buffer`、`TextEncoder`/`TextDecoder`、`btoa`/`atob`、`crypto.getRandomValues`。

## 限制

- 无 `fetch`、无任意 npm 包、无 `child_process`
- 单次默认超时 30s（可传 `timeout_secs`）
- OOXML 模板编辑：先 `ooxml_unpack`（省略 `out_dir`），用返回的 `out_dir` 拼 XML 路径（自动目录在 `.cache/ooxml/` 下，段名为短 hash）；列 slide 用 `doc_list('<out_dir>/ppt/slides')`

## 故障修复

1. 失败脚本路径见工具返回的 `script_path` 或错误 JSON，错误含行列号
2. 用 `fs_patch` 局部修复（勿 `fs_write` 整文件重写）
3. `skill_run {"path":"<script_path>"}` 重跑
4. `script.js` 在同 session 内跨 turn 保留；`error.json` 仅在失败时存在，修复成功后删除；用户 cancel turn 时删除 scratch 目录

## 最小示例

**docx 生成：**

```javascript
async function main() {
  const { Document, Packer, Paragraph, TextRun } = docx;
  const doc = new Document({ sections: [{ children: [new Paragraph({ children: [new TextRun("Hi")] })] }] });
  const b64 = await Packer.toBase64String(doc);
  doc_write("out.docx", b64);
  return { ok: true };
}
```

**OOXML fs 编辑：**

```javascript
async function main() {
  const xmlPath = "<out_dir>/word/document.xml";
  let xml = fs.readFileSync(xmlPath, "utf-8");
  xml = xml.replace("OLD", "NEW");
  fs.writeFileSync(xmlPath, xml, "utf-8");
  return { ok: true };
}
```
