# 设计：improve-docx-aesthetics

## Context

生成 Word 的现状有两条路径：

```
路线 A（现状主路径，质量差）              路线 B（已具备，少被使用）
word_create(markdown)                  skill_read("docx") → skill_run + docx-js
  → office_oxide 转换                    → 完整样式/字体/表格控制
  → 无样式、无中文字体、无标题层级           → 但 skill 内容缺中文排版指导
```

关键事实：

- `word_create` 注册于 `registry.rs:68`，实现在 `tools/word.rs`，依赖 `office_oxide::create_from_markdown` 与 `docx-rs`
- `skill_run`（`tools/skill.rs` + `tools/runtime/`）内嵌 docx-js 9.x bundle，能力完整
- `assets/skills/docx/SKILL.md` 移植自 Anthropic 原版，保留了 Arial / US Letter 等美式默认，无中文排版内容
- 主要 Provider 为 DeepSeek（无视觉能力），「生成 → 渲染 → 看图 → 迭代」回路不可行
- `tools/ooxml/validate.rs` 已有 quick_xml + zip 的 OOXML 解析基础，lint 可复用

## Goals / Non-Goals

**Goals:**

- 生成 Word 收敛到唯一的高质量路径（skill_run + docx-js）
- 中文文档默认产出专业排版（正确字体、标题层级、缩进行距）
- 风格可变：质量有下限，风格不固化
- 无视觉模型也有质量反馈回路（确定性 lint）

**Non-Goals:**

- 不做应用内 docx 预览面板（后续单独立项）
- 不做视觉模型截图回灌
- 不做结构化中间层（文档规格 JSON → Rust 渲染）
- 不重构 excel_write / pptx 工具面（仅 skill 文档补中文字体指引）

## Decisions

### D1：删除 `word_create`，而非降级 description

只要工具存在，模型就有概率走捷径；description 约束是软的。生成职责完全移交 skill_run。

**触点清单**（实现时逐项处理）：

| 文件 | 操作 |
|---|---|
| `src-tauri/src/tools/word.rs` | 删除文件 |
| `src-tauri/src/tools/mod.rs` | 移除 `mod word` |
| `src-tauri/src/tools/registry.rs:68` | 移除注册 |
| `src-tauri/src/tools/changed_paths.rs:7` | match 分支移除 `"word_create"` |
| `src-tauri/src/tools/tests.rs`（5 处） | word_create 用例改为 skill_run + docx-js 等价用例（保留「生成合法 docx」断言） |
| `src-tauri/Cargo.toml` | 移除 `docx-rs`（仅 word.rs 使用）；`office_oxide` 保留（office.rs 读取/转换仍用） |
| `src/lib/toolLabels.ts` / `toolLabels.test.ts` | 移除 `word_create` 条目 |
| `assets/skills/docx/SKILL.md:10,36`、`editing.md:115` | 清理引用，改指 skill_run |

历史会话兼容：前端对未知工具名已有兜底展示（toolLabels 缺省回退原始名），旧记录不受影响。

### D2：docx SKILL.md 采用「硬规则 + 风格菜单」双层结构

不做 `docTheme("report-cn")` 黑盒函数——固定主题牺牲灵活性，且黑盒让模型失去微调能力。改为 skill 文档内可复制配置片段，模型每次自行组装、按内容调整。参照 pptx skill 的 Design Ideas + Color Palettes 已验证模式。

**第一层：中文排版硬规则**（新增章节，违反即 lint 告警）：

````markdown
## 中文文档排版（CRITICAL）

中文内容必须遵守以下规则，否则字体回退、版式坍塌：

1. **必须配置 eastAsia 字体**——`font: "Arial"` 这类纯西文设置会让中文回退到默认衬线字体：

```javascript
// ✅ 默认字体：西文 + 中文分别指定
styles: {
  default: { document: { run: {
    font: { ascii: "Calibri", eastAsia: "微软雅黑", hAnsi: "Calibri" },
    size: 24,  // 12pt（小四），half-points
  } } },
}
```

2. **必须用 Heading 样式分层**——禁止整篇连续大段；每 3~6 段内容应有一个标题；
   标题编号用中文习惯（一、/（一）/ 1. / （1））写入标题文本或 numbering 配置。

3. **中文文档用 A4**（11906 × 16838 DXA），不是 US Letter；
   页边距常用上下 2.54cm / 左右 3.18cm（1440 / 1800 DXA）。

4. **正文段落设置**——首行缩进两字符 + 适度行距：

```javascript
new Paragraph({
  indent: { firstLine: 480 },              // 12pt 正文两字符 ≈ 480 twips（字号变则等比调整）
  spacing: { line: 360, lineRule: "auto" }, // 1.5 倍行距（240 = 单倍）
  children: [new TextRun("正文内容……")],
})
```

5. **列表必须用 numbering config**——沿用本文档 Lists 章节规则，禁止手打 `•` `·` `1.`。
````

**第二层：风格菜单**（新增章节；四套各给完整 `styles` 片段 + 适用场景；明确「按内容选择并调整，不要每次同一风格」）：

| 风格 | 标题字体 | 正文字体 | 特征 |
|---|---|---|---|
| 公文 | 黑体（标题）/ 小标宋（文头） | 仿宋_GB2312 三号(32) | 首行缩进、无彩色、居中大标题 |
| 商务报告 | 微软雅黑 加粗 + 主题色 | 微软雅黑 小四(24) | 无缩进、段后距、彩色标题/分隔线、封面页 |
| 学术 | 黑体 / Times New Roman | 宋体 + Times New Roman 五号(21) | 两端对齐、摘要/关键词版式 |
| 现代简洁 | 思源黑体/微软雅黑 细体大字号 | 微软雅黑 小四(24) | 大留白、浅灰分隔线、强调色块 |

风格片段示例（商务报告，其余三套同构）：

```javascript
const ACCENT = "1F4E79"; // 按文档主题换色，不要永远用这个蓝
const styles = {
  default: { document: { run: {
    font: { ascii: "Calibri", eastAsia: "微软雅黑", hAnsi: "Calibri" }, size: 24,
  } } },
  paragraphStyles: [
    { id: "Heading1", name: "Heading 1", basedOn: "Normal", next: "Normal", quickFormat: true,
      run: { size: 32, bold: true, color: ACCENT,
             font: { ascii: "Calibri", eastAsia: "微软雅黑" } },
      paragraph: { spacing: { before: 360, after: 180 }, outlineLevel: 0,
                   border: { bottom: { style: BorderStyle.SINGLE, size: 6, color: ACCENT, space: 4 } } } },
    { id: "Heading2", name: "Heading 2", basedOn: "Normal", next: "Normal", quickFormat: true,
      run: { size: 28, bold: true, font: { ascii: "Calibri", eastAsia: "微软雅黑" } },
      paragraph: { spacing: { before: 240, after: 120 }, outlineLevel: 1 } },
  ],
};
```

同时删除原文「Use Arial as the default font」「use US Letter」表述（保留 US Letter 数据表供西文文档参考）。

### D3：docx 样式 lint —— 确定性反馈回路

**位置**：新模块 `src-tauri/src/tools/ooxml/style_lint.rs`（~200 行 + 测试），复用 zip + quick_xml。

**挂接点**：`tools/skill.rs::run_handler` —— `execute_script` 返回的 `written_paths` 中以 `.docx` 结尾者逐个 lint，结果并入工具响应：

```rust
// tools/skill.rs::run_handler（execute_script 之后）
let mut style_warnings = serde_json::Map::new();
for p in &written_paths {
    if p.to_lowercase().ends_with(".docx") {
        if let Ok(resolved) = ctx.sandbox.resolve(p) {
            if let Ok(warnings) = crate::tools::ooxml::style_lint::lint_docx(&resolved) {
                if !warnings.is_empty() {
                    style_warnings.insert(p.clone(), json!(warnings));
                }
            }
        }
    }
}
if !style_warnings.is_empty() {
    response["style_warnings"] = Value::Object(style_warnings);
    response["style_hint"] = json!("检测到排版问题，请修正后重新生成（参考 docx skill 的中文排版章节）");
}
```

lint 失败（IO/解析错误）只跳过不报错——lint 是增强不是门禁，绝不能让合法产物因 lint bug 而失败。

**检查规则**（首版 5 条，输出中文 warning + 修复指引）：

| 规则 | 判定（document.xml / styles.xml 文本层） | warning 示例 |
|---|---|---|
| W1 缺标题 | 正文 > 600 字且无 `<w:pStyle w:val="Heading` | 全文超过 600 字但没有任何标题样式，请用 HeadingLevel 分层 |
| W2 缺中文字体 | 正文含 CJK 字符，但 styles.xml 与 document.xml 均无 `w:eastAsia=` 字体声明 | 文档含中文但未配置 eastAsia 字体，中文将回退为默认衬线字体 |
| W3 超长段落 | 单个 `<w:p>` 内文本 > 500 字 | 第 N 段超过 500 字，建议拆分段落或转为列表 |
| W4 手打 bullet | 段落文本以 `•` `·` `●` 或 `数字.` 开头且该段无 `<w:numPr>` | 检测到手工输入的项目符号，请改用 numbering 配置 |
| W5 表格缺宽度 | `<w:tbl>` 内无 `<w:tblW` 或无 `<w:gridCol` | 表格未设置 columnWidths/width，跨平台渲染会变形 |

**实现骨架**：

```rust
// src-tauri/src/tools/ooxml/style_lint.rs
pub fn lint_docx(path: &Path) -> Result<Vec<String>, ToolError> {
    let file = File::open(path).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut zip = ZipArchive::new(file).map_err(|e| ToolError::Execution(e.to_string()))?;
    let document = read_entry(&mut zip, "word/document.xml")?;       // 必须存在
    let styles = read_entry(&mut zip, "word/styles.xml").unwrap_or_default();

    let paragraphs = extract_paragraphs(&document); // Vec<Para { text, has_heading, has_numpr }>
    let mut warnings = Vec::new();
    check_missing_headings(&paragraphs, &mut warnings);   // W1
    check_east_asia_font(&document, &styles, &paragraphs, &mut warnings); // W2
    check_overlong_paragraphs(&paragraphs, &mut warnings); // W3
    check_manual_bullets(&paragraphs, &mut warnings);      // W4
    check_table_widths(&document, &mut warnings);          // W5
    Ok(warnings)
}
```

`extract_paragraphs` 用 quick_xml 流式遍历 `<w:p>`，收集每段纯文本、是否含 `pStyle=Heading*`、是否含 `numPr`——一次遍历喂所有规则，避免重复解析。

**阈值常量化**（`const OVERLONG_PARA_CHARS: usize = 500;` 等），首版拍定，后续按实际误报调。

### D4：引导强化——system prompt 与工具 description

`agent/loop_runner.rs` system prompt 增加一句硬性要求：

```text
生成 .docx/.pptx/.xlsx 交付物前，MUST 先 skill_read 对应 skill 获取规范；
直接凭记忆写 skill_run 代码属于错误行为。
```

`tools/skill.rs` 中 `skill_run` 的 description 头部加：

```text
Before generating any .docx/.pptx/.xlsx deliverable you MUST first call
skill_read for that format. ...
```

模型对工具 description 的服从度高于 system prompt，两处都加，双保险。

### D5：pptx / xlsx skill 中文补丁（最小改动）

- `pptx/SKILL.md`：Design Ideas 后补一段——中文演示文稿字体用 `微软雅黑`（pptxgenjs `fontFace: "微软雅黑"`），标题可用思源黑体；避免西文字体渲染中文
- `xlsx/SKILL.md`：补一句——中文表格 ExcelJS 字体设 `name: "微软雅黑"`，列宽按中文字符宽度估算（中文 ≈ 2 个西文字符宽）

## Risks / Trade-offs

- **[lint 误报骚扰模型]** 规则过严会让模型反复重生成、浪费 token → 首版仅 5 条高置信度规则；warning 措辞明确「建议修正」而非「必须」；阈值常量化便于调整
- **[删除 word_create 后简单需求变重]**「给我一个空白 docx」也要写 30 行 JS → 可接受：skill SKILL.md 已有最小可复制模板，成本是一次复制粘贴；质量一致性收益更大
- **[风格菜单被模型当成仅有的四种选择]** → 章节明确写「菜单是下限参考，鼓励按内容调整颜色与细节」，与 pptx palette 的措辞一致
- **[lint 基于文本层匹配可能漏检]**（如样式继承链上的 eastAsia 声明）→ W2 同时查 styles.xml 与 document.xml 两处，漏检方向是「少报」，不产生误伤
- **[skill 文档变长，token 成本上升]** docx SKILL.md 约 +150 行 → 可接受；渐进披露机制（skill_read 按需加载）已限制影响范围

## Migration Plan

一次性变更，无数据迁移。回滚 = revert 提交。历史会话中 `word_create` 调用记录仅展示用途，前端对未知工具名有兜底，无需迁移。

## Open Questions

- W1/W3 字数阈值（600 / 500）基于经验拍定，实装后用真实生成样本校准
- 公文风格是否需要严格对齐 GB/T 9704（字号/版式精确到毫米）？首版按「接近公文观感」处理，严格合规留待用户反馈
