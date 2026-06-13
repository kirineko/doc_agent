# 更新说明

本文件记录各版本的用户可见变更。安装包见 [GitHub Releases](https://github.com/kirineko/doc_agent/releases)；国内用户亦可从阿里云 OSS 下载（见 README）。

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
