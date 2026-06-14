# 更新说明

本文件记录各版本的用户可见变更。安装包见 [GitHub Releases](https://github.com/kirineko/doc_agent/releases)；国内用户亦可从阿里云 OSS 下载（见 README）。

**版本号策略（自首个 CalVer 发版起）**：正式版本采用 **SemVer 兼容日历版本 `YYYY.M.D`**（年.月.日，各段无前导零，如 `2026.6.14`）。`1.0.0` / `1.0.1` 为历史 SemVer，保留不改动。

---

## [Unreleased]

---

## [2026.6.14] — 2026-06-14

首个 CalVer 正式版本（自本版起 tag 与安装包版本号为 `YYYY.M.D`）。相较 `1.0.1`，本版本新增会话上下文自动压缩，并改进侧栏会话排序体验。

### 上下文管理与压缩

- **自动上下文压缩**：接近模型上限时自动摘要较早历史并归档，保留最近轮次；对齐 kimi/deepy 双触发（比例 85% + 预留空间）与 tool-call 配对保护
- **Token 用量采集**：流式响应解析 `usage`（`stream_options.include_usage`），结合 pending 字符估算驱动压缩判定；各模型暴露上下文上限（DeepSeek 1M、Kimi 256K、Mock 100K 便于测试）
- **持久化与重建**：压缩摘要以 `user` 角色写入 DB，旧消息标记 `archived`；后续 turn 仅从「摘要 + 未归档消息」重建上下文
- **循环内触发**：工具循环每一步开头检查压缩（非仅 turn 开始），覆盖单轮内大工具输出累加场景
- **工具步数上限**：单 turn 最大工具循环步数由 32 提升至 64
- **压缩失败兜底**：摘要 LLM 失败时截断最旧非保留消息，避免 turn 因超限彻底卡死

### 界面

- **上下文占用指示**：会话标题栏展示图标 + 占用比例（≥70% 琥珀、≥85% 红色）；压缩完成后一次性轻提示
- **压缩后同步**：收到 `context_compacted` 后立即刷新消息列表，已归档内容不再显示；压缩摘要不在聊天气泡中展示
- **侧栏会话拖动排序**：支持 drag handle 拖动重排；**懒激活**——未拖动前仍按 `updated_at` 自动排序，首次拖动后按项目写入 `localStorage` 持久化手动序

### 版本与发布

- **CalVer 版本号**：应用版本由 `1.0.1` 切换为 `2026.6.14`（`YYYY.M.D`，无前导零）
- **历史版本保留**：`1.0.0` / `1.0.1` 的 tag 与 OSS 路径不变；自本版起 Release tag 与 `latest.json` 使用 CalVer

---

## [1.0.1] — 2026-06-14

### 自动更新验证

- **发版验证**：用于验证 1.0.0 → 1.0.1 应用内自动更新闭环

### 界面

- **设置抽屉**：「检查更新」移至顶栏设置抽屉，展示当前/最新版本；打开抽屉时查询 OSS `latest.json`
- **版本查询修复**：通过 Tauri 后端拉取 manifest，避免 dev/生产环境 CORS 导致拿不到最新版本

---

## [1.0.0] — 2026-06-14

### 自动更新与分发

- **应用内自动更新**：启动时检查新版本，侧栏提供「检查更新」；确认后下载安装并重启（自 1.0.0 起生效，此前版本需手动安装本版基线包）
- **国内下载加速**：Release 产物同步上传阿里云 OSS（广州），`latest.json` 供 updater 使用；GitHub Releases 保留为备用渠道

---

## [0.2.0] — 2026-06-12

相较 0.1.0，本版本聚焦工作区体验、文档生成质量、数据分析与需求澄清，共 9 项功能交付。

### 工作区与界面

- **右侧文件浏览区**：在项目目录内浏览、打开文件；支持 Finder 风格返回行与 `⌂` 面包屑导航（不再显示难懂的 `..`）
- **明暗主题切换**：顶栏一键切换深色 / 浅色（Notion 风格浅色模式），偏好写入 `localStorage` 重启后保留
- **会话与侧栏优化**：懒创建会话、发送提示、侧栏控件重组；会话标题由模型自动生成，失败时二次重试
- **文件变更同步**：Agent 写入 / 修改文件后，文件浏览区与 `@` 文件选择器即时刷新（基于 `changed_paths`，无需轮询）
- **品牌与安装**：自定义 Logo / 图标；Windows 安装路径调整为 `DocAgent`（窗口标题仍为 Doc Agent）

### 文档与 Agent 能力

- **需求澄清（clarify）**：新增 `clarify` skill 与 `clarify_ask` 工具；模糊需求时 Agent 可暂停当前轮次、以结构化单选 / 多选 / 文本收集用户输入，确认「创作简报」后继续生成或编辑
- **HTML 报告**：新增 `html-report` skill；可在项目目录生成静态 HTML 报告（表格、样式、打印 CSS）；可选 `html_to_pdf` 导出 PDF（macOS / Windows）
- **Word 生成质量**：移除低质量捷径 `word_create`；统一走 `skill_read(docx)` + `skill_run` + docx-js；补充中文排版与样式指引；新增 docx 样式 lint
- **skill_run 容错**：失败脚本保留在 `.skill-run/`；新增 `fs_patch` 局部修复；错误定位到行列号；本轮结束自动清理临时脚本
- **旧版 Office**：`office_convert` 将 `.doc` / `.xls` / `.ppt` 转为现代格式；`.xls` 可直接 `data_query` 分析，无需先另存

### Excel 与数据分析

- **不规则表格预处理**：`excel_describe` 侦察合并单元格、表头行与结构警告；`excel_normalize` 清洗为规整 CSV
- **data_query 增强**：列名归一化与错误提示优化；不规则 Excel 建议先 describe → normalize 再查询
- **工具链中文标签**：前端工具调用展示补全中文名称

### 联网与 PDF

- **联网搜索（可选）**：配置 Tavily API Key 后可使用 `web_search` / `web_extract` 检索与提取网页内容

### 修复

- **流式消息展示**：多轮工具调用时，每一步 assistant 回复独立展示，不再混在同一流式框中

### 破坏性变更

- **`word_create` 已移除**：由 Markdown 一步生成 Word 的路径已删除；请使用 `skill_read` + `skill_run` 生成 `.docx`。历史会话中的调用记录仍可查看，新会话不再提供该工具。

---

## [0.1.0] — 2026-06-10

首个公开发布版本。

- 以项目文件夹为边界的多会话本地 AI 助手（Tauri 2 · Rust · React）
- Word / Excel / PPT / PDF 阅读、生成与 OOXML 编辑工具链
- 内置 docx / pdf / pptx / xlsx Document Skills 与 `skill_run` 运行时
- DeepSeek V4 Flash / Pro、Kimi K2.6；思考模式与流式 Markdown
- `@` 引用文件、智能推荐问（需 DeepSeek Key）
- Windows（x86_64）与 macOS（Apple Silicon）安装包
