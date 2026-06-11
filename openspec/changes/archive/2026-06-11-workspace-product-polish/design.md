## Context

Doc Agent 是 Tauri 2 桌面应用，项目目录经 Sandbox 约束，右侧栏当前仅 `ToolChainPanel`。依赖侧已具备 `office_oxide`（支持 DOC/XLS/PPT 读取与 `save_as` 转 OOXML）和 `tauri-plugin-opener`（未在前端使用）。用户决策：

1. 旧格式转换仅经 Agent 工具，输出文件名加 `-converted` 后缀
2. 安装目录 `DocAgent`，窗口标题保持 `Doc Agent`
3. Logo 采用「文档 + AI 弧线」概念（青蓝主色）
4. 文件浏览仅项目根下子目录单层导航，非完整树

## Goals / Non-Goals

**Goals:**

- 提供 `office_convert` 工具，将 `.doc/.xls/.ppt` 转为 `.docx/.xlsx/.pptx`，默认输出 `{stem}-converted.{ext}`
- `data_query` 可直接以 `.xls` 为数据源
- 右侧栏下半区展示可导航的文件列表，双击/操作打开系统默认应用
- 替换应用图标与顶栏 Logo
- `productName` 改为 `DocAgent`

**Non-Goals:**

- 文件管理器内的「一键转换」按钮（转换仅 Agent 工具）
- VS Code 式递归文件树、拖拽、重命名、删除
- 批量转换整个项目目录
- 旧格式 OOXML 解包编辑（仍拒绝）
- 转换保真度视觉 diff / LibreOffice 渲染校验

## Decisions

### D1：转换输出命名 `-converted` 后缀

- 规则：`{basename}-converted.{target_ext}`，扩展名映射 `doc→docx`、`xls→xlsx`、`ppt→pptx`
- 示例：`课程体系.xls` → `课程体系-converted.xlsx`（stem 为去掉原扩展名后的文件名）
- 若目标路径已存在：返回错误，不覆盖
- 可选参数 `out_path` 允许 Agent 显式指定路径，但仍 MUST 含 `-converted` 后缀（校验文件名为 `*-converted.{docx|xlsx|pptx}`），防止与用户手动另存为混淆

### D2：转换实现复用 `office_oxide`

```rust
let doc = Document::open(src)?;
doc.save_as(dst)?;
```

不引入 LibreOffice 或外部 CLI。转换后返回 `{ "path": "...", "format": "xlsx" }`。

同步更新 `docx/xlsx/pptx` SKILL.md：**优先** `office_read_to_markdown` / `data_query`（`.xls` 走内存临时转换，不写项目文件）；**仅当** OOXML 编辑、样式化输出或用户明确要求现代格式时才 `office_convert`。转换可能丢失版式（实测可完成但格式不保真）。不可对旧格式 `ooxml_unpack`。

### D8：转换仅在必要时进行（用户实测反馈）

- **原则**：能读就不转；减少项目内 `-converted` 新文件
- **无需转换的场景**：内容阅读/摘要、`data_query` 查 `.xls`、`office_read_to_markdown` 任意旧格式
- **必须转换的场景**：`ooxml_unpack` / `ooxml_pack` 编辑链、`excel_write` / `xlsx_recalc` 针对旧 xls、用户明确要求输出 `.docx/.xlsx/.pptx`
- **保真说明**：`office_convert` 基于 `office_oxide` IR 迁移，复杂版式/宏/VBA 可能丢失；Agent 须在回复中说明若用户关心格式

### D3：`data_query` 支持 `.xls`

在 `load_source` 增加 `Some("xls")` 分支：经 `office_oxide` 打开后写入 OS 临时目录的 `.xlsx`（`tempfile`），再用现有 `xlsx_to_dataframe` 读取；临时文件会话结束即删，**不**写入项目目录。

**备选**：要求 Agent 先 `office_convert` 再查询 — 多一步、截图场景仍会失败；弃用。

### D4：右侧栏布局

新建 `RightPanel.tsx`（或扩展 `ToolChainPanel` 容器）：

```
┌─ RightPanel (w-64) ─────────┐
│ ToolChainPanel  flex-1      │
│ ─── border-t ───            │
│ ProjectFileExplorer ~38%    │
└─────────────────────────────┘
```

`ProjectFileExplorer` 接收 `projectId`，内部维护 `currentPath`（相对项目根，`.` 表示根）。

### D5：文件浏览 IPC

| Command | 说明 |
|---------|------|
| `list_project_dir` | `(project_id, relative_path?)` → 单层 entries（name, is_dir），复用 sandbox 校验 |
| `open_project_file` | `(project_id, relative_path)` → Rust 解析绝对路径后 `open::that()` |

不在前端直接拼绝对路径，避免绕过 sandbox。

导航：点击目录 → `currentPath` 更新；`..` → `parent`；根目录隐藏 `..`。

刷新：切换 `activeProjectId` 时重置为 `.`；可选在工具链完成写文件后由父组件触发 refresh（MVP 可手动切换目录或 mount 时加载）。

### D6：安装与品牌

- `tauri.conf.json`：`productName: "DocAgent"`，`app.windows[0].title` 保持 `"Doc Agent"`
- 图标：用新 Logo 重新生成 `src-tauri/icons/*` 全套（可用 `tauri icon` CLI）
- 顶栏：`public/logo.svg` + 文字「Doc Agent」

### D7：Logo 视觉（概念 A）

- 图形：圆角矩形文档轮廓，内 2–3 条横线表示文本行，一条青蓝弧线从左下穿过文档向右上扬（象征 AI 处理流）
- 色：`#22d3ee`（cyan-400）图形 + `#0b1020` 透明底（顶栏 SVG）/ 深色底（应用图标）
- 小尺寸：仅保留文档 + 弧线，去掉细横线

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| `office_oxide` 转换丢格式/宏/VBA | 工具返回说明「转换后请用 `xlsx_recalc` / 人工抽查」；Skill 注明保真局限 |
| `-converted` 文件累积 | Agent 可识别后缀；用户可手动删除；不自动清理 |
| 已安装 `Doc Agent` 目录残留 | Release note 说明；不自动迁移 |
| 文件浏览与 `list_project_files` 重复 | 职责分离：前者单层导航，后者扁平清单供 `@` |
| `out_path` 校验过严 | 错误信息明确「Agent 转换输出须含 -converted 后缀」 |

## Migration Plan

1. 合并后新版本 Windows 安装到 `Program Files\DocAgent\`
2. 无数据库迁移
3. 回滚：还原 `productName` 与图标资源即可

## Open Questions

（无 — 用户已确认四项决策）
