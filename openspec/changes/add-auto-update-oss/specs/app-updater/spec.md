## ADDED Requirements

### Requirement: 启动时检查更新

从 **1.0.0** 起，桌面客户端 SHALL 在应用启动后自动向配置的 updater endpoint 发起更新检查。检查 MUST 使用 HTTPS endpoint `https://doc-agent.oss-cn-guangzhou.aliyuncs.com/latest.json`。若当前版本已是最新，系统 MUST 静默结束，不得打扰用户。

#### Scenario: 启动时无可用更新

- **WHEN** 应用启动且 OSS `latest.json` 中版本不大于当前安装版本
- **THEN** 不展示更新提示，用户可正常使用应用

#### Scenario: 启动时发现新版本

- **WHEN** 应用启动且 `latest.json` 中版本大于当前安装版本
- **THEN** 系统通过 dialog 告知用户新版本号与 release notes，并询问是否立即更新

### Requirement: 用户确认后下载并安装更新

系统 SHALL 仅在用户确认后下载并安装更新包。下载与安装 MUST 校验 Tauri updater 签名（`pubkey`）。安装完成后，系统 MUST 提供重启应用以完成更新的路径（调用 `relaunch` 或等效流程）。

#### Scenario: 用户确认更新

- **WHEN** 用户在更新 dialog 中选择确认安装
- **THEN** 系统下载对应平台更新包、校验签名、执行安装，并提示或自动重启应用

#### Scenario: 用户拒绝更新

- **WHEN** 用户在更新 dialog 中选择取消
- **THEN** 系统不下载更新包，用户继续使用当前版本

#### Scenario: 下载或安装失败

- **WHEN** 更新包下载失败、签名校验失败或安装失败
- **THEN** 系统通过 dialog 展示可读错误信息，不崩溃，用户可继续使用当前版本

### Requirement: 手动检查更新

系统 SHALL 在侧栏提供「检查更新」入口，允许用户主动触发与启动检查相同的 updater 逻辑。

#### Scenario: 手动检查已是最新

- **WHEN** 用户点击「检查更新」且当前已是最新版本
- **THEN** 系统提示「当前已是最新版本」或等效文案

#### Scenario: 手动检查发现新版本

- **WHEN** 用户点击「检查更新」且存在更高版本
- **THEN** 系统展示与启动检查相同的更新确认 dialog

### Requirement: 平台与版本基线

Updater MUST 仅支持 **darwin-aarch64** 与 **windows-x86_64**。版本 **低于 1.0.0** 的构建物不含 updater 能力；自动更新能力从 **1.0.0** 首次安装包起生效。

#### Scenario: 支持的平台收到更新

- **WHEN** macOS Apple Silicon 或 Windows x86_64 客户端检查更新且 `latest.json` 包含对应 platform 条目
- **THEN** 系统使用该平台 `url` 与 `signature` 完成更新流程

#### Scenario: 1.0.0 之前版本无 updater

- **WHEN** 用户运行版本号低于 1.0.0 的安装包
- **THEN** 该构建不包含应用内自动更新功能
