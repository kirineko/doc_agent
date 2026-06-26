## Context

`docx_comment`（`src-tauri/src/tools/ooxml/comment.rs` + `mod.rs`）是 agent 给已解包 docx 加批注的工具。当前它只构造一条 `<w:comment>` 并尝试字符串插入 `word/comments.xml`，既不碰 `document.xml` 的锚点，也不装配 people/commentsExtended。实测打包产物里 comments.xml 仍是空壳、正文无锚点，Word 无法渲染批注——工具事实上未实现。

OOXML 里一条可见批注需要多个部件协同：

```
  document.xml                comments.xml              commentsExtended.xml
  ┌────────────────┐          ┌──────────────────┐      ┌──────────────────┐
  │commentRangeStart│  id=X   │<w:comment w:id=X>│      │<w15:commentEx     │
  │  ...正文范围... │◀────────│  <w:p>批注正文</w:p>│◀────│  paraIdParent=.. │ (回复链)
  │commentRangeEnd │          │</w:comment>      │      │/>                │
  │<w:r><commentRef│          └──────────────────┘      └──────────────────┘
  │  erence w:id=X/></w:r>│
  └────────────────┘
        │ rId                                  ┌── people.xml (作者元数据)
        ▼                                      │
  document.xml.rels ──comments──▶comments.xml ◀┘
  [Content_Types].xml: Override 登记 comments / commentsExtended / people
```

部件 ③④⑤（rels / Content_Types / people）在 docx-js 生成的基础文档里 comments 部分已预置（见 diag dump：`document.xml.rels` 有 comments 关系、`[Content_Types].xml` 有 comments Override），故 MVP 可聚焦部件 ①② 与回复链；people.xml 随需补建。

## Goals / Non-Goals

**Goals:**
- 让 `docx_comment` 产出的批注在 Word 中真实可见（含回复归属正确）。
- 堵住验证器对"批注断链"的盲区，使此类缺陷在打包阶段即暴露。
- 用端到端"可见性"断言替换当前仅查文件存在的测试。

**Non-Goals:**
- 不重写整个 OOXML 编辑栈、不引入 python-docx 等重依赖；定位与插入基于现有 `quick-xml`。
- 不支持任意复杂的跨段落/表格内嵌范围锚点（MVP 锚定到段落级）。
- 不实现批注的删除/解析/接受，仅覆盖"写入 + 校验"。
- 不改动 docx-js skill 侧的 `add_comment` Python API（那是独立的 Python 脚本栈，本次只修 Rust 工具）。

## Decisions

### 决策 1：写入采用 XML 解析而非字符串拼接（修主因）

**取舍**：`rfind("</w:comments>")` 的字符串方案对自闭合空壳（`<w:comments/>`）和命名空间前缀变化都极其脆弱，这正是当前 bug 根因。

**决定**：`add_comment` 用 `quick-xml`（验证器已依赖）读取并定位根 `<w:comments>`：
- 若根为自闭合 → 改写为 `<w:comments …>…</w:comments>` 成对结构；
- 在根元素内部末尾追加构造好的 `<w:comment>` 片段；
- 写回前做 wellformed 校验，确保不破坏既有内容。
不引入 DOM 全量解析（OOXML 文件可能很大），用流式/事件定位即可。

### 决策 2：定位策略 —— 段落级锚点，参数用"段落定位器"（最高风险点）

工具需要告诉后端"批注挂在哪段正文"。可选方案：

| 方案 | 优点 | 缺点 |
|------|------|------|
| A. XPath | 精确 | quick-xml 不支持 XPath，需引依赖；XML 命名空间让 XPath 对 agent 极难写对 |
| B. 文本子串匹配 | agent 易表达 | 正文可能重复；多义；文本被 run 切碎后子串未必连续 |
| C. 段落序号（0-based index） | 简单无歧义 | agent 需先数段落，易数错；插入其他段落后漂移 |
| **D. 段落定位器：`paragraph_index`（首选）+ 可选 `text_hint`（辅助消歧）** | 主参数无歧义；hint 帮 agent 自检 | 略增参数 |

**决定**：采用 D。
- 主参数 `paragraph_index: integer`（0-based，对 `document.xml` 的顶层 `<w:p>` 计数，跳过 `<w:tbl>` 等非段落——MVP 不支持表内批注）。
- 可选 `text_hint: string`：若提供，后端校验该 index 处段落的纯文本包含此子串，不匹配则报错（防止 agent 数错段落）。语义是"断言式校验"而非"搜索式定位"。
- 锚点在该段落的**整个段落范围**：`commentRangeStart` 置于段落首个 run 之前、`commentRangeEnd`+`commentReference` 置于段落末 run 之后。
- 越界 index、命中非 `<w:p>`、`text_hint` 不匹配 → 均报错（满足 spec 的"找不到定位目标时报错"场景）。

理由：方案 A/B 的实现与 agent 易用性成本都高且不可靠；C 单独用太脆；D 给 agent 一个确定锚点（index）加一道安全网（hint），符合本仓库工具"宁可显式报错也不静默错写"的一贯风格。

### 决策 3：回复链用 commentsExtended.xml，paraId 作为连接键

父批注 `<w:comment>` 的 `w14:paraId` 作为 `paraIdParent` 写入子批注的 `<w15:commentEx>`。`commentsExtended.xml` 随需创建（与 people.xml 同样的"不存在则建并登记 Override"流程）。现有代码用 `id.wrapping_mul(0x9E37_79B9)` 生成 paraId（`comment.rs:20`）——保留该算法，确保父子可对应。

### 决策 4：验证规则归入 wml 规则族，双向一致性

新增规则 `wml.comment.consistency`（规则族见 `validate/rules/wml.rs`）：收集 `comments.xml` 的 `w:comment/@w:id` 集合与 `document.xml` 的 `commentReference/@w:id` 集合，做对称差。任一非空 → 违规，`message` 列出失配 id。规则在 `validate_dir` 遍历到 `word/document.xml` 时触发（此时 comments.xml 已可读）。

### 决策 5：people.xml 与额外 Override 随需建立

docx-js 基础文档已有 comments 的 rels 与 Content_Types Override，但**没有 people.xml 及其 Override**，也**没有 commentsExtended**。`add_comment` 首次执行时：
- 建 `word/people.xml`（含作者 `<w15:person>`）；
- 追加 people / commentsExtended 的 Content_Types Override；
- 追加 commentsExtended 的 rels 关系（people.xml 的关系走 comments.xml.rels 或 document.xml.rels，按 OOXML 惯例）。
为避免重复登记，每次写入前先检查是否已存在对应条目。

## Risks / Trade-offs

- **定位用 index 仍可能被 agent 数错**：`text_hint` 是缓解而非根治。若实践中频繁出错，后续可考虑让 `ooxml_unpack` 在返回里带上每段的 `(index, 纯文本预览)`，帮 agent 对齐——列为后续优化，不进 MVP。
- **段落级锚点粒度偏粗**：用户要批注"句中某词"时，本方案只能批注整段。OOXML 支持更细粒度的 run 级范围，但定位 run 对 agent 极不友好；MVP 接受整段粒度，文档里讲清。
- **新增验证规则可能让历史"无效批注"文档打包失败**：这正是期望行为（暴露既有问题），但对正在用旧产物的用户是 breaking change。由于旧产物本就不可见批注，可接受。
- **people.xml 关系归属的 OOXML 细节**：不同 Office 版本对 people.xml 的关系挂载点（document.xml.rels vs comments.xml.rels）略有差异；MVP 选其一并经打包验证，若 Word 报警再调整。
- **quick-xml 写入 vs 字符串**：用事件流定位+字符串片段插入根部内部，比全量 DOM 轻量，但要小心 XML 实体转义（批注文本里的 `<` `&` 等），构造 `<w:comment>` 片段时必须转义文本节点。

## MVP 范围限制（实现时确认）

下列不在本次实现范围，`comment_tool` 的 description 已写明：
- **仅段落级锚点**：`paragraph_index` 只对 `word/document.xml` 中 `<w:body>` 直接子级的 `<w:p>` 计数；不支持表格内（`<w:tc>` 下）的段落，也不支持 run 级的句中词范围。定位表格内段落会因不在顶层而越界报错。
- **定位为整段范围**：`commentRangeStart/End` 包住整段，而非段内片段。

## 实现笔记（落地后补充）

- **根定位必须用深度计数**：初版用"首个 `Event::End` 即根闭合"的假设，在已含 `<w:comment>`（内部有 `<w:rStyle/>` 等自闭合/嵌套）的容器上会误把第一个 `</w:rStyle>` 当根闭合，导致新条目插到旧条目内部、破坏 XML。已改为 `Start +1 / End -1` 的深度计数，`depth==0 && root_opened` 才是根闭合。同时 `Event::Empty`（自闭合子元素）不改变 depth。
- **自闭合根展开**：对 `<w:comments .../>`，剥掉结尾 `/>` 后补 `>`，再拼 `entry + </root>`，保证产物一定是成对根。
- **一致性校验需要读 comments.xml**：`validate_part_structure` 原签名 `(base, rel_part, xml)` 已带 `base`，直接在 `word/document.xml` 分支里追加 `wml::validate_comment_consistency(base, xml)`，函数内自行读 `base/word/comments.xml`；文件不存在则跳过（空集不报错）。
- **people/commentsExtended 的关系**：统一挂到 `word/_rels/document.xml.rels`，通过 `register_part` 自动分配 `rId` 并去重。
