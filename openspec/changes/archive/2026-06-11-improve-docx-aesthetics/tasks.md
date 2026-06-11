# Tasks: improve-docx-aesthetics

## 1. 删除 word_create 路径

- [x] 1.1 删除 `src-tauri/src/tools/word.rs`，从 `tools/mod.rs` 移除 `mod word`，从 `registry.rs` 移除注册
- [x] 1.2 `changed_paths.rs` match 分支移除 `"word_create"`
- [x] 1.3 改写 `tools/tests.rs` 中 5 处 word_create 用例：改为 `skill_run` + docx-js 等价用例，保留「产物为合法 OOXML（zip 含 [Content_Types].xml）」断言
- [x] 1.4 `Cargo.toml` 移除 `docx-rs` 依赖（确认 `office_oxide` 仍被 office.rs 使用后保留），`cargo build` 通过
- [x] 1.5 前端 `src/lib/toolLabels.ts` 移除 `word_create` 条目并更新 `toolLabels.test.ts`，确认未知工具名兜底展示正常

## 2. docx skill 中文排版重构

- [x] 2.1 `assets/skills/docx/SKILL.md` 新增「中文文档排版（CRITICAL）」章节：eastAsia 字体片段、Heading 分层强制、A4 页面、首行缩进与行距片段、numbering 强制（内容以 design.md D2 为准）
- [x] 2.2 新增「风格菜单」章节：公文 / 商务报告 / 学术 / 现代简洁四套完整 `styles` 配置片段 + 适用场景 + 「按内容调整、避免千篇一律」指示
- [x] 2.3 移除/改写 Arial 默认字体与 US Letter 默认页面表述（标注「西文文档适用」），清理 SKILL.md 第 10、36 行与 `editing.md` 第 115 行的 word_create 引用
- [x] 2.4 `pptx/SKILL.md` 补充中文 `fontFace` 指引；`xlsx/SKILL.md` 补充中文字体与列宽估算指引

## 3. docx 样式 lint

- [x] 3.1 新建 `src-tauri/src/tools/ooxml/style_lint.rs`：`lint_docx(path) -> Result<Vec<String>>`，quick_xml 单次遍历提取段落（文本 / heading / numPr），实现 W1–W5 五条规则（阈值常量化）
- [x] 3.2 `tools/skill.rs::run_handler` 挂接：对 `written_paths` 中的 `.docx` 逐个 lint，告警并入响应 `style_warnings` + `style_hint`；lint 异常静默跳过
- [x] 3.3 lint 单元测试：构造触发 W1–W5 的最小 docx（可用 zip 直接打包测试 XML）+ 一个合格文档不告警 + 一个损坏文件不报错
- [x] 3.4 集成测试：`skill_run` 生成无样式 docx 时响应包含 `style_warnings`

## 4. 引导强化与收尾

- [x] 4.1 `agent/loop_runner.rs` system prompt 增加「生成 Office 交付物前 MUST 先 skill_read」强制指示
- [x] 4.2 `tools/skill.rs` 中 `skill_run` description 头部增加同等强制提示
- [x] 4.3 更新 `.cursor/rules/rust-module-testing.mdc` 中 word_create 示例为 skill_run 等价示例
- [x] 4.4 本地自检全绿：`cargo fmt --check && cargo clippy -- -D warnings && cargo test`，`npm run typecheck && npm test && npm run build && npm run bundle:js`
- [x] 4.5 端到端人工验证：用 DeepSeek 实际生成一份中文专业介绍 Word（复现原始场景），确认有标题分层、中文字体正确、无 style_warnings（已加自动化冒烟测试 `skill_run_styled_chinese_docx_has_no_style_warnings`，用户已在真实会话中确认）
