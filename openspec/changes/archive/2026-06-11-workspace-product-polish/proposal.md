## Why

用户在真实项目（含旧版 Office 97–03 文档）中使用 Doc Agent 时，暴露出四类体验缺口：旧格式无法被数据分析工具直接处理、Windows 安装路径含空格、品牌仍用 Tauri 默认图标、右侧栏缺少项目文件浏览与系统打开能力。本次变更在不大改架构的前提下补齐这些发布与日常使用短板。

## What Changes

- 新增 Agent 工具 `office_convert`：将 `.doc/.xls/.ppt` 转为现代 OOXML；输出文件名 MUST 带 `-converted` 后缀（如 `报告.xls` → `报告-converted.xlsx`），与用户手动另存为的文件区分
- 扩展 `data_query`：支持以 `.xls` 为数据源（经读取或转换路径加载）
- 安装目录：`productName` 改为 `DocAgent`（无空格）；窗口标题保持 `Doc Agent`
- 定制 Logo：文档 + AI 弧线概念，青蓝主色；替换 Tauri 默认图标集并在顶栏展示
- 右侧栏下半区：项目文件浏览（仅当前项目根下子目录，单层懒加载导航）；支持用系统默认应用打开文件
- 更新相关 Skill 文档：旧格式可读、可经 `office_convert` 转换，但不可 OOXML 解包编辑

## Capabilities

### New Capabilities

- `legacy-office-convert`：Agent 旧版 Office 转现代格式的工具契约与命名规则
- `project-file-browser`：右侧项目文件浏览与系统打开

### Modified Capabilities

- `workspace-ui`：右侧栏由「仅工具链」扩展为「工具链 + 文件浏览」；顶栏增加 Logo；安装产物目录名
- `office-tools`：新增转换工具需求；读取工具与旧格式说明对齐
- `data-analysis`：`data_query` 支持 `.xls` 数据源

## Impact

- **Rust**：`tools/office.rs`（新 convert）、`tools/data/query.rs`、`ipc/mod.rs`（`list_project_dir`、`open_project_file`）、`core/sandbox` 路径校验复用
- **前端**：`ToolChainPanel` 或新 `RightPanel` 组合布局、`ProjectFileExplorer` 组件、顶栏 Logo
- **配置**：`src-tauri/tauri.conf.json`（`productName`）、`src-tauri/icons/*`
- **文档**：`assets/skills/docx|xlsx|pptx/SKILL.md` 旧格式说明
- **依赖**：无新增 crate（复用已有 `office_oxide`、`tauri-plugin-opener`）
