## Context

- 模型 Flyout 使用 `useAnchorPosition(triggerRef)`，`width = Math.min(320, rect.width)`，侧栏宽于 320px 时 Flyout 窄于 trigger；仅监听 `window resize`，拖拽 `react-resizable-panels` 分隔条不触发重算。
- 斜杠/`@` 弹层大量 `text-[11px]`，输入框 `text-sm`，用户感知字号不一致。
- `ProjectFileExplorer` 硬编码 macOS「Finder」文案，Windows 用户反馈错误。
- Windows 更新走 `tauri-plugin-updater`：`write_to_temp` + `ShellExecute` + `process::exit(0)`，析构不运行，temp 中 `{productName}-{version}-updater-*` 目录与 `*-setup.exe` 可能残留（上游已知行为）。
- `UpdateProgressOverlay` 文案含 `…`（如 `正在下载 v2026.6.17… 45%`），用户易误判为 UI 截断。

## Goals / Non-Goals

**Goals:**

- Flyout 水平宽度与 trigger 一致，侧栏 resize 时 Flyout 打开状态下位置/宽度同步更新
- `@`、斜杠 fuzzy 弹层、斜杠图形菜单三处弹层字号统一提升
- 文件管理器按钮跨平台中性中文
- 启动后后台一次性清理 24h 前的 updater temp 产物，毫秒级完成、失败静默
- 更新遮罩：无省略号、信息分层清晰、卡片有最小宽度

**Non-Goals:**

- 修复 tauri-plugin-updater 上游 Windows `exit(0)` 行为
- 更新中途取消、安装阶段百分比
- 平台分支文案（macOS Finder / Windows 资源管理器）
- 递归扫描整个磁盘或非 temp 目录

## Decisions

### 1. Flyout 宽度 = trigger 宽度 + ResizeObserver

```ts
// useAnchorPosition
const width = Math.max(rect.width, 240); // 去掉 320 上限
```

对 `triggerRef` 挂载 `ResizeObserver`（与 `window resize` 共用 `update()`），侧栏拖拽时重算。`ModelFlyout` 不传 `width: 320` 选项。

**备选**：Flyout 改为 `absolute` 相对侧栏容器 — 拒绝，现有 `fixed` + 锚定 design 已稳定。

### 2. 弹层字号统一方案

| 元素 | 现 | 改 |
|------|----|----|
| 分组标题 / 分类 tab / 提示行 / id / description | `text-[11px]` | `text-xs` |
| 命令 label / 文件名主行 | `text-xs` | `text-xs font-medium`（保持或 `text-sm` 仅主 label） |

三文件同 diff 规则：`SlashMenuFlyout`、`SlashCommandPopup`、`FileMentionPopup`。

### 3. 文件管理器按钮文案

统一：

```tsx
aria-label="在文件管理器中打开项目根目录"
title="在文件管理器中打开项目根目录"
```

### 4. 启动清理 updater 临时文件（Rust）

**触发**：`lib.rs` `setup` 末尾调用 `updater_cleanup::spawn_stale_cleanup()`，立即返回。

**执行模型**：

```rust
pub fn spawn_stale_cleanup() {
    std::thread::Builder::new()
        .name("updater-cleanup".into())
        .spawn(|| {
            let _ = cleanup_stale_updater_artifacts();
        })
        .ok(); // 线程创建失败也静默
}
```

- **不**在 setup 主线程做 I/O
- **不** `await`、不占用 async runtime
- 单线程、只读 temp 顶层目录、匹配即删，无重试

**扫描范围**：`std::env::temp_dir()` **仅一层** `read_dir`（目录内 updater 产物为整目录删除，不再递归子树以控时）。

**Pattern**（与 `tauri-plugin-updater` Windows 行为对齐）：

- 目录名：`DocAgent-*-updater*`（productName 来自 `CARGO_PKG_NAME` 或常量 `DocAgent`，与 bundler 一致）
- 文件：同目录下 `DocAgent*.exe` 或名称含 `-updater` 且扩展名为 `.exe` / `.msi`（保守匹配，避免误删）

**mtime 阈值**：`SystemTime::now() - 24h`；无法读 mtime 的条目跳过。

**安全/fast 上限**：

- 最多处理 512 个 temp 顶层条目，超出即停止（防止异常 temp 目录拖慢启动）
- 每条 `remove_file` / `remove_dir_all` 错误 `let _ =`，不 log 到用户可见层

**测试**：纯函数 `is_stale_updater_artifact(name, modified, cutoff)` + tempdir 集成测试，不依赖真实 `%TEMP%`。

### 5. 更新进度遮罩 UX

去掉所有 `…`：

| 阶段 | 主文案 | 副文案 |
|------|--------|--------|
| downloading + percent | 正在下载 | `v{version}` 与 `{n}%` 分两行或副行 |
| downloading 无 percent | 正在下载 | `v{version}` 或「请稍候」 |
| installing | 正在安装 | 即将重启 |

布局：

```tsx
<div className="... min-w-[17rem] text-center">
  <UpdateProgressRing />
  <p className="text-sm font-medium">{primary}</p>
  {secondary && <p className="text-xs text-fg-secondary">{secondary}</p>}
</div>
```

百分比可单独一行、`tabular-nums`，避免与版本号挤在同一行。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 24h 内用户多次更新失败，temp 仍残留 | 可接受；24h 后自动清；比永久残留好 |
| pattern 过宽误删 | 限定 DocAgent + updater 子串 + temp 顶层；单元测试 |
| 正在进行的更新安装器读 temp exe | 24h 阈值 + 新安装不会删当日文件 |
| ResizeObserver 兼容性 | 现代 WebView2/WKWebView 均支持 |
| 清理线程极端慢 temp | 512 条目上限 |

## Migration Plan

无数据迁移。发版后下次启动自动清理历史残留；UI 变更即时生效。

## Open Questions

无（用户已确认：统一文件管理器、24h pattern 清理、@ 字号同步、去省略号并优化遮罩 UX）。
