## 1. OpenSpec

- [x] 1.1 proposal.md / design.md / specs / tasks.md

## 2. 更新进度状态与 updater 逻辑

- [x] 2.1 新增 `updateProgress` 模块或 hook：阶段 `idle | downloading | installing`、字节累加、`reset`
- [x] 2.2 `checkForAppUpdates` 接入 `downloadAndInstall(onEvent)`，确认后进入 downloading，`Finished` 后 installing
- [x] 2.3 `updater.test.ts`：mock `DownloadEvent` 序列，验证阶段切换与百分比计算

## 3. 全局进度 UI

- [x] 3.1 新增 `UpdateProgressOverlay`：遮罩 + 圆环进度 + 分阶段文案
- [x] 3.2 `App.tsx` 挂载 overlay，订阅共享进度状态
- [x] 3.3 `SettingsDrawer` 与全局状态对齐（保留按钮禁用 / 「更新中…」）

## 4. 验证

- [x] 4.1 `npm test` / `npm run typecheck` 通过
- [x] 4.2 手动：设置抽屉触发更新，确认下载百分比与安装文案可见后重启（需在已打包版本上验证）
