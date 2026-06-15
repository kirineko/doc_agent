## MODIFIED Requirements

### Requirement: 用户确认后下载并安装更新

系统 SHALL 仅在用户确认后下载并安装更新包。下载与安装 MUST 校验 Tauri updater 签名（`pubkey`）。安装完成后，系统 MUST 提供重启应用以完成更新的路径（调用 `relaunch` 或等效流程）。自用户确认起至 `relaunch` 完成前，系统 MUST 在应用内展示可见的更新进度反馈（见 `workspace-ui`「更新下载进度遮罩」）；下载阶段 MUST 通过 `downloadAndInstall` 的 `DownloadEvent` 驱动进度；安装阶段 MUST 展示「即将重启」类文案。

#### Scenario: 用户确认更新

- **WHEN** 用户在更新 dialog 中选择确认安装
- **THEN** 系统下载对应平台更新包、校验签名、执行安装，并提示或自动重启应用
- **AND** 下载与安装期间主界面 MUST 展示全局更新进度遮罩

#### Scenario: 用户拒绝更新

- **WHEN** 用户在更新 dialog 中选择取消
- **THEN** 系统不下载更新包，用户继续使用当前版本
- **AND** MUST NOT 展示更新进度遮罩

#### Scenario: 下载或安装失败

- **WHEN** 更新包下载失败、签名校验失败或安装失败
- **THEN** 系统通过 dialog 展示可读错误信息，不崩溃，用户可继续使用当前版本
- **AND** 更新进度遮罩 MUST 关闭

#### Scenario: 下载阶段进度事件

- **WHEN** 用户确认更新且 updater 发出 `Started` / `Progress` 事件
- **THEN** 系统 MUST 累加已下载字节并在 UI 中反映下载进行中状态
- **AND** 若 `Started` 含 `contentLength`，UI MUST 展示下载百分比

#### Scenario: 安装阶段无下载事件

- **WHEN** 下载完成（`Finished`）且安装尚未结束
- **THEN** UI MUST 切换为安装阶段文案（含「即将重启」语义）
- **AND** MUST NOT 假装展示安装百分比
