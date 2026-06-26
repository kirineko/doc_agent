## Why

`docx_comment` 今天宣称成功，但生成的批注在 Word 里根本看不到。实测证据（见下）显示调用 `ooxml_pack` 后：

- `word/comments.xml` **仍然是空壳** `<w:comments .../>`（自闭合），`<w:comment>` 条目从未被写入；
- `word/document.xml` 里没有任何 `commentRangeStart` / `commentRangeEnd` / `commentReference` 锚点；
- 因此 Word 无法把批注关联到正文，完全不渲染批注。

根因有两层：
1. **写入失败（主因）**：`comment::add_comment` 用 `xml.rfind("</w:comments>")` 定位插入点（`src-tauri/src/tools/ooxml/comment.rs:32`）。但 docx-js（本项目的文档生成器）产出的 `comments.xml` 是**自闭合**的 `<w:comments .../>`，没有 `</w:comments>` 闭合标签，`rfind` 返回 `None`，`if let Some(pos)` 分支整体跳过 → 条目被构造后丢弃，文件原样写回。
2. **锚点缺失（设计缺陷）**：即便修好写入，工具 schema 只有 `dir/id/text/author/parent`，**没有任何定位参数**。无法在 `document.xml` 里插入 `commentRange*` 锚点，批注即使存在也无处附着。

工具工具同时被两个"放哨者"放行，使这个缺陷长期隐蔽：
- **验证器**：`is_part_registered` 因 `[Content_Types].xml` 含 `<Default Extension="xml"/>`，对任意 `.xml` 一律判为"已注册"（`validate/rules/opc.rs:16-25`），从不检查批注三件套（comments.xml 内容 / rels 关系 / document.xml 锚点）是否一致；
- **测试**：`smoke_redline_comment_chain` 只断言 `comments.xml` 出现在 zip 列表里（`tools/tests.rs:1709-1712`），与 bug 同源——"文件存在 ≠ 批注生效"。

现在修，是因为批注是合同/审阅场景的核心能力，当前等同完全不可用。

### 实测证据（diag dump 摘要）

对 `contract.docx`（正文 "甲方应于30日内付款。"）调用 `docx_comment id=1` 后打包：

```
comments.xml has <w:comment w:id="1"> : false   ← 条目没写进去
document.xml has commentRangeStart     : false   ← 没有锚点
document.xml has commentRangeEnd       : false
document.xml has commentReference      : false
[Content_Types] has comments Override  : true    ← 这两项是 docx-js 原本就生成的
document.xml.rels has comments rel     : true    ← 同上，与本次调用无关
people.xml present in zip              : false
```

`word/comments.xml` 实际内容（自闭合空壳）：
```xml
<w:comments xmlns:w="..." .../>
```

## What Changes

1. **修复 `add_comment` 写入**：放弃脆弱的 `rfind("</w:comments>")` 字符串插入；改为解析根元素 → 保证有显式 `<w:comments>...</w:comments>` 结构 → 在内部追加 `<w:comment>`。同时处理自闭合空壳（`<w:comments/>` → 展开为成对标签）和已有内容两种情况。
2. **新增锚点装配**：`docx_comment` 增加**定位参数**（段落定位），在 `document.xml` 目标段落内插入 `commentRangeStart` / `commentRangeEnd` / 带 `commentReference` 的 run，使批注真正附着到正文。
3. **补齐 commentsExtended / people.xml（回复链）**：当 `parent` 存在时，写入 `commentsExtended.xml` 的 `<w15:commentEx>` 父子关系，并在首次需要时建立 `word/people.xml` 与对应关系/内容类型注册，使回复（reply）在 Word 里正确归属。
4. **加固验证器**：新增"批注一致性"校验——`comments.xml` 里每个 `w:comment/@w:id` 必须在 `document.xml` 有对应的 `commentReference`；`document.xml` 里每个 `commentReference` 必须能在 `comments.xml` 找到 `w:comment`。不一致即报告违规，堵住"文件存在但没接通"的盲区。
5. **重写测试断言**：现有测试升级为验证"端到端可见性"——解包打包后的 docx，断言 comments.xml 含目标 `<w:comment>`、document.xml 含匹配 id 的 `commentRangeStart`+`commentReference`。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `ooxml-toolchain`：现有 "Word 批注注入" Requirement 的 Scenario 存在规范级缺陷——原文（`openspec/specs/ooxml-toolchain/spec.md:78-80`）写"Agent 调用 docx_comment...**并在 document.xml 加入对应 range 标记后回包**"，把锚点装配推给了调用方，与工具"自动维护 comments.xml 及关联部件"的描述自相矛盾，正是 bug 的规范层根源。本次将 Requirement 收紧为"工具自行装配全部部件（含正文锚点）"，并新增"批注三件套一致性校验"Requirement。

## Impact

- **代码**：
  - `src-tauri/src/tools/ooxml/comment.rs`（重写 `add_comment`，新增锚点/回复装配）
  - `src-tauri/src/tools/ooxml/mod.rs`（`comment_tool` schema 增加定位参数；`comment_handler` 传递定位信息）
  - `src-tauri/src/tools/ooxml/validate/rules/wml.rs`（新增批注一致性规则）
  - `src-tauri/src/tools/ooxml/validate/rules/mod.rs`（规则分发）
  - `src-tauri/src/tools/tests.rs`（`smoke_redline_comment_chain` 断言升级 + 新增端到端测试）
- **工具 API**：`docx_comment` 的入参 schema 变更（新增定位参数），向后不兼容——调用方需更新。考虑到当前工具产出的批注一律无效，无现实兼容性负担。
- **依赖**：预计无需新增 crate；定位/插入用现有 `quick-xml`（验证器已用）即可，避免引入重量级 OOXML 库。
- **风险**：定位参数的设计是本次最需斟酌点（见 design.md 的"定位策略"决策）。
