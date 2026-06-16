## Context

- `ChatPanel` 已有 textarea + 发送、`@`（`FileMentionPopup`）、键盘 `/`（`SlashCommandPopup`）、粘贴图片（`addPastedImage` → `save_upload` → `.cache/attachments/`）
- 项目文件索引由 `useProjectFiles` + `list_project_files_cmd` 驱动；Explorer 的 `currentPath` 为组件本地 state，**不与 Chat 共享**
- `open_project_file` 用 `tauri_plugin_opener::open_path` 打开**文件**，拒绝目录
- `save_upload` 仅接受图片 MIME，写入 attachment 缓存，**不能**用于项目内容导入
- `ChatPanel.tsx` ~477 行，接近组件体量软上限，需拆分工具栏

## Goals / Non-Goals

**Goals:**

- 三按钮显式暴露上传、图片、斜杠能力
- 多文件导入项目根，冲突可覆盖/另存为/取消
- 导入后 `@` 索引即时可见，输入框光标处自动插入引用
- 斜杠按钮：6 类二级菜单，与 registry 一致
- Explorer 一键打开项目根（Finder / 资源管理器）

**Non-Goals:**

- 导入到子目录、Explorer 路径同步
- 拖拽、进度条、上传后自动发送
- 修改 `slashCommands` registry 内容或键盘 `/` fuzzy 行为
- 图片写入项目目录

## Decisions

### 1. 上传目标：固定项目根

所有 `+` 导入写入 `./filename`（相对项目根）。用户若需子目录整理，通过 Explorer 新增「打开项目根」在系统文件管理器中操作。

**备选**：同步 Explorer `currentPath` — 需 lift state，MVP 排除。

### 2. IPC：`import_project_file`

单文件命令，前端多选后**顺序**循环 invoke（便于逐文件弹冲突对话框）。

```rust
#[derive(Deserialize)]
pub struct ImportProjectFileRequest {
    pub project_id: String,
    pub filename: String,      // basename only，禁止 path 分量
    pub data_base64: String,
    pub on_conflict: String,   // "overwrite" | "rename" | "skip"
}

#[derive(Serialize)]
pub struct ImportProjectFileResponse {
    pub path: String,          // 最终相对路径（可能已 rename）
    pub renamed: bool,
}
```

- `Sandbox::resolve_for_write(&filename)`，`filename` MUST NOT 含 `/`、`\`、`..`
- 任意 MIME/扩展名；大小上限 **100MB**（与 attachment 分开常量）
- `on_conflict=rename`：Rust 侧生成 `stem (1).ext`、`(2).ext` … 直至不冲突
- `skip`：返回 Err 或专用响应；前端视为用户取消该文件

**备选**：批量 IPC + 前端一次 ask — 多文件冲突 UX 不清晰，不采用。

### 3. 重名对话框（前端）

检测到 invoke 返回「文件已存在」或使用 `head/exists` 预检 — **推荐**：先 invoke 带 `on_conflict=skip` 的 exists 检查，或 dedicated `check_project_file_exists`；更简单做法是 **Rust 在 overwrite/rename 未指定且存在时返回 structured error**，前端 `ask` 三按钮：

| 按钮 | 后续 invoke |
|------|-------------|
| 覆盖 | `on_conflict=overwrite` |
| 另存为 | `on_conflict=rename` |
| 取消 | 跳过，继续下一个文件 |

多文件队列：**每冲突文件单独询问**。

### 4. 自动 `@` 插入

全部选定文件处理完成后：

1. `mergeProjectFileEntries` + `setFileRevision`
2. 在**当前光标**拼接：`formatMentionPath(path)` 空格分隔，末尾加空格
3. 不强制换行；不移动光标到末尾（保持用户在 `@` 块之后可继续输入）

提取 `buildMentionInsert(paths: string[], cursor: number, text: string)` 单测。

### 5. 图片按钮

隐藏 `<input type="file" accept="image/png,image/jpeg,image/webp,image/gif" multiple={false}>`；`onChange` 读 File → 现有 `addPastedImage(file, mime)`。vision 校验、附件上限、toast 不变。

### 6. 斜杠：双入口、一套 insert

- **键盘 `/`**：保留 `detectSlash` + `SlashCommandPopup`（fuzzy）
- **按钮 `/`**：新建 `SlashMenuFlyout` — 一级 `CATEGORY_ORDER` 标签，hover/click 展开二级命令列表（label + description）
- 共用 `insertSlashPrompt(text, cursor, commandId)`：
  - 若 `detectSlash` 活跃 → `applySlash` 替换 `/query`
  - 否则 → 在 `cursor` 插入 `prompt`，选中首个 `{{占位符}}`

Flyout 与 `@`/键盘 slash 弹层互斥；clarify / busy / initializing 时工具栏 disabled。

### 7. UI 布局

Composite 输入容器：

```
┌─────────────────────────────────────────┐
│ textarea                                 │
├─────────────────────────────────────────┤
│ [+] [🖼] [/]                    [发送]  │
└─────────────────────────────────────────┘
```

Popups（`@`、键盘 `/`、SlashMenuFlyout）锚定到**外层 relative 容器**，避免定位错位。

组件：`ChatInputToolbar.tsx`（~80 行）、`SlashMenuFlyout.tsx`（~120 行）、`useProjectImport.ts`（~100 行）。

### 8. `open_project_root`

```rust
pub fn open_project_root(state, project_id) -> Result<(), String> {
    let sandbox = ...;
    tauri_plugin_opener::open_path(sandbox.root(), None::<&str>)
}
```

Explorer `PanelSectionHeader.actions`：`[📂 打开]` + 现有 `[↻]`。macOS aria-label「在 Finder 中打开」；Windows「在文件资源管理器中打开」。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 大文件 base64 内存峰值 | 100MB 上限；后续可改 streaming（非 MVP） |
| 根目录堆积文件 | Explorer 打开根目录 + 文档说明 |
| ChatPanel 继续膨胀 | 强制拆 toolbar / hook |
| 自动 `@` 打断编辑 | 仅在用户主动点 `+` 后插入；busy/clarify 禁用按钮 |
| 另存为 `(n)` 与 Agent 引用不一致 | 响应返回最终 `path`，插入实际路径 |

## Migration Plan

纯增量功能，无数据迁移。发版后 placeholder 可补充「+ 上传文件」提示。

## Open Questions

（无 — 探索阶段已确认：根目录、自动递增另存为、光标处 `@`、任意文件、多选、6 类二级菜单。）
