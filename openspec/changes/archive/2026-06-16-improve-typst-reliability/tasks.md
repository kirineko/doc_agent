## 1. 诊断基础设施（diagnostics.rs）

- [x] 1.1 新建 `src-tauri/src/tools/typst_export/diagnostics.rs`，定义 `DiagnosticInfo`（error_type/file/line/column/message/snippet/hints/fix_guidance）与 `WarningInfo` 结构及其 `serde` 序列化
- [x] 1.2 实现 `build_source_map(sandbox_root, entry)`：用 entry 文本 + `bundled::static_sources()` 构造 `HashMap<FileId, Source>`（FileId 与编译时虚拟路径一致）
- [x] 1.3 实现 `resolve_location(span, &source_map) -> Option<(file, line, column, snippet)>`，用 `Source::range`/`byte_to_line`/`byte_to_column` 还原并截取出错行加位置指示符
- [x] 1.4 实现 `classify(message) -> (error_type, fix_guidance)` 启发式分类表（unknown-variable / unexpected-argument / unknown-font / file-not-found / syntax / type-error / other）
- [x] 1.5 实现 `to_diagnostics(diags, &source_map)` 与 `to_warnings(warnings, &source_map)`，未命中安全降级为 message-only（标记 unlocated，绝不 panic）
- [x] 1.6 单元测试：用含已知错误（拼写错误的函数名）的源串断言行列与 snippet 正确；含 detached span 的降级路径断言不 panic

## 2. 编译路径接入（compile.rs / mod.rs）

- [x] 2.1 `CompileOutput` 增加 `warnings: Vec<WarningInfo>`；`compile_to_temp_pdf` 收集 warnings 不再 `eprintln!`
- [x] 2.2 编译失败改为返回携带结构化 `Vec<DiagnosticInfo>` 的错误（替换 `format_typst_error`/`{d:?}`）；PDF 导出失败同样结构化
- [x] 2.3 `mod.rs` 失败时用 `ToolError::Structured(json!({ error, diagnostics, warnings }))` 回传；成功时若有 warnings 在 `{ path, pages }` 增加 `warnings`
- [x] 2.4 确认 `registry.rs` `ToolError::Structured` 透传符合预期（必要时微调 `to_json_value`）
- [x] 2.5 更新 `typst_to_pdf` 工具描述：失败时优先 `fs_patch` 局部修复、禁止整篇重写

## 3. 手册与 exports 一致性校验

- [x] 3.1 为 `typst-guide.md` 可独立编译的代码块前置 `<!-- doc-agent:compile -->` 标记，逐一标注现有自包含块
- [x] 3.2 测试：抽取并编译被标记的手册代码块，断言无 error；未标记块跳过
- [x] 3.3 测试：正则提取 `common/*.typ` 顶层 `#let` 公开符号，与手册 §0.2 导出表比对，偏离即失败
- [x] 3.4 依据 3.2/3.3 结果修订手册错误/缺漏（含 §0.2 表、示例语法），并补充 `tokens.typ` 到内置模块表

## 4. 设计系统与模板重构

- [x] 4.1 新建 `common/tokens.typ`：按 design 决策 6 写入字号阶/间距阶/行距/线宽/页边距 token 与字体角色别名；按决策 7 写入 ≥5 套 `palettes`、`default-theme` 与 `make-theme(...)` 合并函数（锁定轴不可覆盖、自由轴有界）；在 `bundled.rs` 挂载其虚拟路径与静态源
- [x] 4.2 重构 `fonts.typ`：保留平台字体栈，新增 `font-body/font-heading/font-emphasis/font-math/font-mono` 语义别名；`apply-zh-body`/`apply-en-body` 增加 `theme:` 入参，按 theme 生成 heading/table/link/math 的 show 规则（accent 仅染非正文）
- [x] 4.3 重构 `page.typ`、`exam.typ`、`lecture.typ`：页边距/线宽/间距/字号改取 token；exam 锁定 charcoal 主题、忽略彩色 accent
- [x] 4.4 重构 8 个场景模板：移除硬编码样式魔数与字体名字符串统一引用 token；各场景预设彼此不同的默认 palette（report→academic-blue、paper→slate、lecture→forest、exam→charcoal 等）
- [x] 4.5 在 `typst-guide.md` 补「主题与配色」一节：说明 `make-theme` 自由轴/锁定轴与用法示例（供 Agent 定制）
- [x] 4.6 测试：扩展零警告编译至全部 8 模板 + 4 common 模块；用例覆盖「自定义 accent」「exam 锁墨色」两条主题路径，断言 `warnings.is_empty()` 且无弃用 API

## 5. 收尾验证

- [x] 5.1 `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [x] 5.2 `cd .. && npm run typecheck && npm test`（确认前端对结构化工具错误的展示兜底正常）
- [x] 5.3 实测：构造一个故意报错的 `.typ` 调 `typst_to_pdf`，确认返回含行列/片段/fix_guidance，且 Agent 能据此局部 `fs_patch`
