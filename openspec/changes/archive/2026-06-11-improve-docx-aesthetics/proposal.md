# 提案：提升 Word 文档生成美观性（improve-docx-aesthetics）

## Why

当前 Agent 生成的 Word 文档美观性无法保证：`word_create` 工具提供了「markdown → office_oxide 转换」的低质量捷径，模型倾向于走这条一步到位的路径，产出无标题样式、无中文字体配置、整篇糊成大段的文档（用户实测截图确认）。而项目已内置的高质量路径（`skill_read` docx 指南 + `skill_run` + docx-js）因为不是强制的、且 skill 内容缺中文排版指导，实际很少被正确使用。文档美观性是本产品的核心价值，必须修复。

## What Changes

- **BREAKING** 删除 `word_create` 工具（含 `tools/word.rs`、registry 注册、changed_paths、前端 toolLabels、相关测试），移除 `docx-rs` 依赖；生成 Word 文档收敛到唯一路径：`skill_read("docx")` → `skill_run` + docx-js
- 重构 `assets/skills/docx/SKILL.md`：
  - 新增「中文排版硬规则」章节：eastAsia 字体配置、标题分层强制、列表 numbering 强制
  - 新增「风格菜单」章节：公文 / 商务报告 / 学术 / 现代简洁四套可复制的 docx-js 样式配置片段，模型按文档类型自选并鼓励变化（参照 pptx skill 的 Design Ideas + Color Palettes 模式）
  - 去掉 Arial 默认字体等美式建议，清理 `word_create` 残留引用（含 editing.md）
- 新增 **docx 样式 lint**：`skill_run` 写出 `.docx` 后自动做 XML 层确定性检查（无 Heading、无 eastAsia 字体、超长段落、手打 bullet、表格缺宽度等），warnings 随工具结果回灌模型，模型据此自行修正（DeepSeek 无视觉能力下的反馈回路替代品）
- 强化引导：system prompt 与 `skill_run` 工具 description 明确「生成 .docx/.pptx/.xlsx 交付物前 MUST 先 skill_read」
- 顺带小改：pptx / xlsx SKILL.md 补充中文字体指引（微软雅黑等）

## Capabilities

### New Capabilities

- `docx-style-lint`: docx 产物的确定性样式检查——在 `.docx` 写出后对 OOXML 做规则检查并向 Agent 返回 warnings，形成无视觉模型可用的质量反馈回路

### Modified Capabilities

- `office-tools`: 移除「Word 文档生成」需求（`word_create` 由 Markdown 生成 Word 的工具路径整体删除，生成职责移交 script-runtime + document-skills）
- `document-skills`: docx skill 内容新增「中文排版硬规则 + 风格菜单」要求；skill 索引注入需求升级为「交付物生成前强制 skill_read」的指引措辞；pptx / xlsx skill 增加中文字体指引

## Impact

- **Rust**：删除 `src-tauri/src/tools/word.rs`；修改 `registry.rs`、`changed_paths.rs`、`tools/tests.rs`、`agent/loop_runner.rs`（system prompt）、`tools/skill.rs`（description + lint 挂接）；新增 `tools/ooxml/style_lint.rs`
- **依赖**：移除 `docx-rs`（仅 word.rs 使用）；`office_oxide` 保留（office.rs 读取/转换仍用）
- **资产**：重写 `assets/skills/docx/SKILL.md` 相关章节，小改 `docx/editing.md`、`pptx/SKILL.md`、`xlsx/SKILL.md`
- **前端**：`src/lib/toolLabels.ts`（移除 word_create 条目）及其测试
- **兼容性**：历史会话中已存在的 `word_create` 工具调用记录仅作展示，不受影响；新会话模型不再看到该工具
