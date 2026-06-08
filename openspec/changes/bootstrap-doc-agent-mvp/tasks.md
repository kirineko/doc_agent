# 实施任务：doc_agent MVP

## 1. 工程脚手架
- [x] 1.1 初始化 git 仓库与 `.gitignore`（Rust / Node 产物、`out/`、密钥）
- [x] 1.2 用 Tauri 2 创建工程：`src-tauri/`（Rust）+ `src/`（React + TS + Vite）
- [x] 1.3 前端引入 UI 基础：Tailwind / shadcn、`react-markdown` + `remark-gfm` + shiki + KaTeX
- [x] 1.4 添加 Rust 依赖：`reqwest`(stream)、`tokio`、`serde`、`rusqlite`/`sqlx`、`keyring`、`office_oxide`、`docx-rs`、`umya-spreadsheet`、`calamine`
- [x] 1.5 跑通空壳应用（Win + macOS 各一次冒烟）— macOS 已通过 `npm run tauri build`；Windows 待验证

## 2. 持久化与沙箱（core）
- [x] 2.1 设计并建表：`projects` / `sessions` / `messages` / `tool_calls` / `settings`
- [x] 2.2 `core::store`：项目 / 会话 / 消息 / 工具调用的增删查（含 `reasoning_content` 字段）
- [x] 2.3 `core::sandbox`：路径 `canonicalize` + 项目根前缀校验 + 软链拒绝；补充越界用例测试
- [x] 2.4 `core::secrets`：API Key 经 OS keychain 存取，禁明文落库 / 落日志

## 3. 模型 Provider 与 Agent Loop（agent）
- [x] 3.1 定义 `LlmProvider` trait 与统一请求 / 事件类型（含 `ThinkingConfig{enabled, effort}`）
- [x] 3.2 实现 SSE 流式解析：分离累积 `delta.reasoning_content` 与 `delta.content`，解析 `tool_calls`
- [x] 3.3 DeepSeek Provider：`extra_body.thinking` + `reasoning_effort: high/max`，base_url 配置
- [x] 3.4 Kimi Provider：`thinking.type` 开关（无强度），`thinking.keep` 处理
- [x] 3.5 Mock Provider：无密钥下产出思考 / 正文 / 工具调用事件
- [x] 3.6 `agent::loop`：多轮工具调用循环 + 最大轮次保护
- [x] 3.7 `reasoning_content` 回填规则（工具调用轮必带）+ 单元测试覆盖防 400
- [x] 3.8 四类事件（reasoning_token / content_token / tool_call / tool_result）经 Tauri event 推送

## 4. 原生工具（tools，沙箱内）
- [x] 4.1 统一 `Tool` 接口（name + JSON Schema + handler），对齐 rmcp 工具体系
- [x] 4.2 `fs`：list / read / write / search
- [x] 4.3 `office.read_to_markdown`：office_oxide 读取 6 格式 → Markdown（含 PPT 只读）
- [x] 4.4 `word.create`：docx-rs / `create_from_markdown` 生成 Word
- [x] 4.5 `word.edit`：office_oxide `EditableDocument` 保格式文本替换
- [x] 4.6 `excel.read`：calamine 读取工作表
- [x] 4.7 `excel.write`：umya-spreadsheet 写单元格并保存
- [x] 4.8 生成产物有效性校验（OOXML 结构）纳入工具测试
- [x] 4.9 预留 `skill.run` 工具位与执行接口（不实现执行器）

## 5. 前端三栏 UI（src）
- [x] 5.1 应用骨架与三栏布局
- [x] 5.2 左栏：项目选择（目录）、会话列表与切换、模型 / 思考开关 / 强度配置（按模型差异化显隐）
- [x] 5.3 中栏：消息流 Markdown 流式渲染；思考内容可折叠分区；代码高亮 / 表格 / 数学
- [x] 5.4 右栏：工具调用链卡片（名称 / 参数 / 状态 / 结果 / 耗时），随事件实时更新
- [x] 5.5 Tauri command / event 前后端打通；按 session + turn 路由事件
- [x] 5.6 API Key 配置界面（写入 keychain，无明文回显）

## 6. 集成与验证
- [ ] 6.1 端到端（Mock Provider）：建项目 → 新会话 → 读取/生成/编辑 Word、读写 Excel → 工具链与流式正常（需手动 `npm run tauri dev` 验证）
- [ ] 6.2 接真实模型（DeepSeek / Kimi）：thinking + 工具调用闭环不报 400（需 API Key）
- [ ] 6.3 会话隔离与重启恢复验证（持久化已实现，需手动验证）
- [x] 6.4 沙箱越界安全用例（`..` / 软链）全部拒绝
- [x] 6.5 Win + macOS 双平台冒烟；安装包体积核对 — macOS DMG 约 9.4MB / App 约 22MB；Windows 待验证
