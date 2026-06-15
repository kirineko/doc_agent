## ADDED Requirements

### Requirement: 更新下载进度遮罩

系统 SHALL 在用户确认安装更新后，于 App 根级展示全局更新进度遮罩，覆盖启动静默检查与设置抽屉手动更新等所有调用 `checkForAppUpdates` 的路径。遮罩 MUST 阻止用户与主界面交互，直至更新失败关闭遮罩或应用 `relaunch`。遮罩 MUST 包含圆环式进度指示器与状态文案。

下载阶段（`downloading`）：

- 若 updater 提供总大小（`contentLength`），MUST 展示圆环进度与百分比，文案含目标版本号（如「正在下载 v{version}… {n}%」）
- 若无总大小，MUST 展示旋转圆环与「正在下载更新…」或等效文案

安装阶段（`installing`）：

- MUST 展示「正在安装，即将重启…」或等效文案
- MUST NOT 展示安装百分比

#### Scenario: 启动更新确认后展示遮罩

- **WHEN** 启动静默检查发现新版本且用户在 dialog 中确认更新
- **THEN** 主界面 MUST 展示全局更新进度遮罩
- **AND** 遮罩 MUST 在下载完成前保持可见

#### Scenario: 设置抽屉触发更新展示遮罩

- **WHEN** 用户在设置抽屉点击「更新」并确认安装
- **THEN** 全局更新进度遮罩 MUST 可见
- **AND** 设置抽屉「更新」按钮 MUST 处于禁用或「更新中…」状态

#### Scenario: 有总大小时展示百分比

- **WHEN** 下载开始且 `DownloadEvent Started` 含 `contentLength`
- **THEN** 遮罩 MUST 展示圆环进度与 0–100% 数值

#### Scenario: 无总大小时旋转指示

- **WHEN** 下载开始但无 `contentLength`
- **THEN** 遮罩 MUST 展示旋转圆环与「正在下载…」文案
- **AND** MUST NOT 展示虚假百分比

#### Scenario: 安装阶段文案

- **WHEN** 下载事件 `Finished` 且安装尚未完成
- **THEN** 遮罩文案 MUST 切换为安装阶段（含即将重启语义）

#### Scenario: 失败关闭遮罩

- **WHEN** 更新下载或安装失败
- **THEN** 遮罩 MUST 关闭
- **AND** 用户 MUST 可继续操作主界面

## MODIFIED Requirements

### Requirement: 设置抽屉检查更新入口

系统 SHALL 在顶栏提供设置入口，点击后从右侧滑出设置抽屉；抽屉内 MUST 以简洁文案展示「当前版本」与「最新版本」两行信息，且 MUST 仅在用户打开抽屉时请求 `latest.json` 获取最新版本号。当最新版本高于当前版本时，抽屉内 SHALL 提供「更新」入口触发安装流程。

#### Scenario: 设置抽屉展示版本信息

- **WHEN** 用户打开设置抽屉
- **THEN** 抽屉内可见当前版本与最新版本两行信息

#### Scenario: 打开抽屉时查询最新版本

- **WHEN** 用户打开设置抽屉
- **THEN** 系统通过 Tauri 后端请求 updater manifest 获取最新版本号
- **AND** 在用户未打开抽屉前 MUST NOT 为展示版本信息而发起该请求

#### Scenario: 更新进行中反馈

- **WHEN** 用户触发更新且下载或安装尚未完成
- **THEN** 更新入口展示进行中状态（禁用或「更新中…」），防止重复触发
- **AND** 全局更新进度遮罩 MUST 同步展示（见「更新下载进度遮罩」）
