# Editing Word Documents (XML Workflow)

**本系统编辑已有 Word 文档只支持 XML 流程**，不使用 `word_edit` 或 Python/Node 脚本。

## 三步流程

### 1. 解包

```json
{ "path": "template.docx", "out_dir": "unpacked/" }
```

工具：`ooxml_unpack`

### 2. 编辑 XML

主文件：`unpacked/word/document.xml`（批注/页眉页脚等同目录其他 part）。

**推荐方式 A — `skill_run`（批量替换，可直接复制）**

```javascript
async function main() {
  const xmlPath = "unpacked/word/document.xml";
  let xml = fs.readFileSync(xmlPath, "utf-8");

  // [旧文本, 新文本] 列表；旧文本必须与 XML 中可见文本逐字一致
  const edits = [
    ["第N讲", "第2讲 AI辅助应用开发工具"],
    ["【内容分析：…】", "【内容分析：实际内容】"],
  ];

  const missed = [];
  for (const [oldText, newText] of edits) {
    if (xml.includes(oldText)) {
      xml = xml.split(oldText).join(newText); // 替换全部出现
    } else {
      missed.push(oldText); // 记录未命中，便于排查跨 run 切分
    }
  }

  fs.writeFileSync(xmlPath, xml, "utf-8");
  return { ok: missed.length === 0, replaced: edits.length - missed.length, missed };
}
```

- **必须检查返回的 `missed`**：非空说明占位符在 XML 中被拆分（见下方陷阱），用 `fs_read` 查看实际切分后重试
- `fs.readFileSync(path, 'utf-8')` 读文本；不带 encoding 返回字节（用于二进制）
- 全局已有 `fs`，`require('fs')` 也可
- 勿在末尾写 `main();`（运行时自动调用）

**推荐方式 B — 直接写文件**

1. `fs_read` 读取 `unpacked/word/document.xml`
2. 在 Agent 侧完成替换
3. `fs_write` 写回同路径

**排版实体**：新增含引号/撇号的中文时，在 XML 中用实体：

| 实体 | 字符 |
|------|------|
| `&#x2018;` | ‘ |
| `&#x2019;` | ’ |
| `&#x201C;` | “ |
| `&#x201D;` | ” |

**批注**（解包后）：

```json
{ "dir": "unpacked/", "id": 0, "text": "批注正文" }
```

工具：`docx_comment`

### 3. 打包

```json
{ "dir": "unpacked/", "out_path": "output.docx", "original": "template.docx" }
```

工具：`ooxml_pack`（含校验与自动修复）

## 读取内容（编辑前）

```json
{ "path": "template.docx" }
```

工具：`office_read_to_markdown` — 了解结构与占位符文本。

## 常见陷阱

- **占位符跨 run 匹配不到**（最常见）：Word 可能把 `第N讲` 拆成 `<w:t>第</w:t>...<w:t>N讲</w:t>` 两个 run。解包默认 `merge_runs: true` 已缓解多数情况；若 `missed` 非空：
  1. `fs_search` 或 `fs_read` 定位占位符在 `document.xml` 中的实际形态
  2. 按实际切分缩小替换单位（如分别替换 `N讲` 与前缀），或替换包含标签的整个片段
- **只改可见文本**：替换目标尽量限定在 `<w:t>` 内的文字，不要误改标签名/属性。
- **修订**：替换整个 `<w:r>…</w:r>` 为 `<w:del>…</w:ins>` 兄弟节点，保留原 `<w:rPr>`。
- **列表删除**：删光段落文字时，还需在 `<w:pPr><w:rPr>` 内加 `<w:del/>` 标记段落删除。
- 打包前勿手动删 `[Content_Types].xml`、`_rels` 等 OPC 结构文件。

## 完整流程示例（填教案模板）

```text
1. office_read_to_markdown {"path": "模板.docx"}      # 了解结构与占位符
2. ooxml_unpack {"path": "模板.docx", "out_dir": "unpacked/"}
3. skill_run（上方方式 A 脚本；检查返回的 missed）
4. ooxml_pack {"dir": "unpacked/", "out_path": "输出.docx", "original": "模板.docx"}
```

## 与「从零创建」的区别

| 场景 | 方式 |
|------|------|
| 编辑已有 docx / 填模板 | **本文** — `ooxml_unpack` → XML → `ooxml_pack` |
| 从零生成新文档 | `skill_read` 主文档 `SKILL.md` — `skill_run` + `docx` 库 |
| 简单空白文档 | `word_create` |
