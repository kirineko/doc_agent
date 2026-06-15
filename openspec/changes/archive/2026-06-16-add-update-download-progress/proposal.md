## Why

更新包体积持续增长，用户确认安装后 `downloadAndInstall()` 期间主界面无任何反馈；尤其在启动时静默检查发现新版本并确认后，用户常经历数秒「黑屏」感后应用突然重启。Tauri updater 插件已支持 `DownloadEvent` 进度回调，但当前实现未接入，与 `add-auto-update-oss` design D6 不一致。

## What Changes

- 在用户确认更新后，于 App 根级展示全局更新进度遮罩（`UpdateProgressOverlay`），覆盖启动静默更新与设置抽屉手动更新两条路径
- 下载阶段：圆环进度指示器；若响应含 `contentLength` 则展示百分比，否则展示旋转圆环 + 「正在下载…」
- 安装阶段：下载完成后切换文案为「正在安装，即将重启…」，直至 `relaunch`
- `checkForAppUpdates` 接入 `downloadAndInstall(onEvent)`，通过共享状态或回调上报 `downloading` / `installing` 阶段与字节进度
- 设置抽屉「更新」按钮保留禁用与「更新中…」文案，与全局遮罩并存
- 失败时关闭遮罩，沿用现有 error dialog
- **不做**：下载取消、后台静默下载、替换原生确认 dialog、安装阶段百分比（插件无事件）

## Capabilities

### New Capabilities

无

### Modified Capabilities

- `app-updater`：用户确认后下载过程须可见进度反馈；安装重启前须有明确文案
- `workspace-ui`：全局更新进度遮罩 UI、分阶段文案与圆环进度展示

## Impact

- **前端**：`src/lib/updater.ts`（进度回调与阶段状态）；`src/hooks/useUpdateProgress.ts` 或等效；`src/components/UpdateProgressOverlay.tsx`；`App.tsx` 挂载 overlay；`SettingsDrawer.tsx` 与现有 `updating` 状态对齐
- **Rust**：无变更
- **测试**：`updater.test.ts` 补充 `DownloadEvent` 序列与阶段切换；overlay 组件测试（可选）
- **OpenSpec**：`app-updater`、`workspace-ui` spec delta
