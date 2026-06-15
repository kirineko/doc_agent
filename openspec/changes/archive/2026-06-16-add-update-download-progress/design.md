## Context

- 更新流程：`check()` → 原生 `ask` dialog → `downloadAndInstall()` → `relaunch()`（`src/lib/updater.ts`）。
- 入口：`useAppUpdater`（启动 3s 后静默检查）、`SettingsDrawer`「更新」按钮（手动 `checkForAppUpdates("manual")`）。
- `@tauri-apps/plugin-updater` `downloadAndInstall(onEvent)` 事件：`Started`（`contentLength?`）、`Progress`（`chunkLength`）、`Finished`；安装阶段无进度事件。
- 现有 `workspace-ui` 仅要求设置抽屉按钮 loading，无全局进度 UI。
- 项目已有圆环式 SVG 参考：`ContextUsageIndicator`（小尺寸上下文占用）。

## Goals / Non-Goals

**Goals:**

- 用户确认更新后，无论从何入口触发，主界面均展示不可忽略的全局进度反馈
- 下载阶段尽可能展示百分比；无总大小时仍展示旋转圆环与状态文案
- 安装阶段明确提示「即将重启」，消除 100% 后数秒无反馈
- 失败时关闭遮罩，不阻塞继续使用

**Non-Goals:**

- 下载中途取消
- 替换 Tauri 原生确认 / 错误 dialog
- 安装阶段进度百分比
- Rust 侧自定义下载逻辑或 IPC
- 侧栏单独进度入口

## Decisions

### 1. App 根级全局遮罩（非仅 SettingsDrawer）

`App.tsx` 挂载 `UpdateProgressOverlay`，`z-index` 高于设置抽屉（`z-50`），建议 `z-[60]` 或与 `ImagePreviewOverlay`（`z-[70]`）同级但低于其（更新与图片预览不应并发）。

**理由**：启动静默更新确认后 dialog 关闭，用户仍见主界面；仅改抽屉无法覆盖该路径。

**备选**：仅在 SettingsDrawer 内展示 — 拒绝，无法解决启动更新路径。

### 2. 共享更新进度状态

新增轻量模块（如 `src/lib/updateProgress.ts`）或 hook `useUpdateProgress`：

```ts
type UpdatePhase = "idle" | "downloading" | "installing";

interface UpdateProgressState {
  phase: UpdatePhase;
  version?: string;
  downloadedBytes: number;
  totalBytes?: number; // Started.contentLength
}
```

`checkForAppUpdates` 在确认后 `setPhase("downloading")`，在 `downloadAndInstall` 回调中累加字节；`Finished` 或进入 `await` 安装前 `setPhase("installing")`；`relaunch` 前或 `catch` 时 `reset`。

订阅方：`UpdateProgressOverlay`、`SettingsDrawer`（可选同步 `updating`）。

**理由**：`updater.ts` 保持可测；UI 与逻辑解耦。

### 3. `downloadAndInstall(onEvent)` 累加逻辑

```ts
let downloaded = 0;
await update.downloadAndInstall((event) => {
  if (event.event === "Started") {
    setTotal(event.data.contentLength);
  } else if (event.event === "Progress") {
    downloaded += event.data.chunkLength;
    setDownloaded(downloaded);
  } else if (event.event === "Finished") {
    setPhase("installing");
  }
});
```

`percent`：`totalBytes` 存在时 `Math.min(100, round(downloaded / totalBytes * 100))`；否则 `undefined`。

### 4. UpdateProgressOverlay 视觉

- 居中卡片：`panel` / `config-surface` 风格，与设置抽屉一致
- 圆环：`svg` + `stroke-dasharray`，直径约 40–48px；有百分比时按弧度填充；无百分比时 `animate-spin` 或脉冲
- 文案：
  - downloading + percent：`正在下载 v{version}… {n}%`
  - downloading 无 percent：`正在下载更新…`
  - installing：`正在安装，即将重启…`
- 遮罩：`bg-black/45`，阻止点击穿透；更新期间不可关闭（无取消按钮）

参考 `ContextUsageIndicator` 圆环 stroke 方式，放大尺寸并支持 `dashoffset` 动画。

### 5. 与 SettingsDrawer 协作

保留 `handleUpdate` 内 `setUpdating(true/false)`；`finally` 中重置。全局 `phase === idle` 时抽屉按钮恢复。遮罩为主反馈，按钮文案「更新中…」为辅。

### 6. 错误与边界

- `downloadAndInstall` 抛错：`reset()` → 现有 `message` error dialog
- 用户拒绝确认：不进入 downloading，`phase` 保持 `idle`
- `contentLength` 缺失：仅圆环旋转，不显示 `%`
- 静默模式失败：仍弹 error dialog；手动模式 `throw` 行为不变

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| OSS 未返回 Content-Length | 无百分比仍显示旋转圆环与文案 |
| 安装阶段仍有几秒无数值进度 | 独立 installing 文案，用户预期「即将重启」 |
| overlay 与 Agent 任务并发 | 更新前用户已确认；遮罩阻止误操作；Agent 进行中触发更新概率低 |
| 测试环境无 Tauri updater | mock `DownloadEvent` 驱动状态与 overlay |

## Migration Plan

无数据迁移。发版后所有更新入口自动获得进度 UI。

## Open Questions

无（探索阶段已确认：全局 overlay + 圆环 + 分阶段文案）。
