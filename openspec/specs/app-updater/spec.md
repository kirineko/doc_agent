# app-updater Specification

## Purpose
TBD - created by archiving change add-auto-update-oss. Update Purpose after archive.
## Requirements
### Requirement: 启动时检查更新

从 **1.0.0** 起，桌面客户端 SHALL 在应用启动后自动向配置的 updater endpoint 发起更新检查。检查 MUST 使用 HTTPS endpoint `https://doc-agent.oss-cn-guangzhou.aliyuncs.com/latest.json`。若当前版本已是最新，系统 MUST 静默结束，不得打扰用户。

#### Scenario: 启动时无可用更新

- **WHEN** 应用启动且 OSS `latest.json` 中版本不大于当前安装版本
- **THEN** 不展示更新提示，用户可正常使用应用

#### Scenario: 启动时发现新版本

- **WHEN** 应用启动且 `latest.json` 中版本大于当前安装版本
- **THEN** 系统通过 dialog 告知用户新版本号与 release notes，并询问是否立即更新

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

### Requirement: 手动检查更新

系统 SHALL 在侧栏提供「检查更新」入口，允许用户主动触发与启动检查相同的 updater 逻辑。

#### Scenario: 手动检查已是最新

- **WHEN** 用户点击「检查更新」且当前已是最新版本
- **THEN** 系统提示「当前已是最新版本」或等效文案

#### Scenario: 手动检查发现新版本

- **WHEN** 用户点击「检查更新」且存在更高版本
- **THEN** 系统展示与启动检查相同的更新确认 dialog

### Requirement: 平台与版本基线

Updater MUST 仅支持 **darwin-aarch64** 与 **windows-x86_64**。版本 **低于 1.0.0** 的构建物不含 updater 能力；自动更新能力从 **1.0.0** 首次安装包起生效。自日历版本策略生效后，后续发行版本号采用 **`YYYY.M.D`**（见 `project-versioning`），updater MUST 以数值比较判定新旧，且 MUST 支持从 SemVer 基线（如 `1.0.1`）升级至 CalVer 版本（如 `2026.6.14`）。

#### Scenario: 支持的平台可检查更新

- **WHEN** macOS Apple Silicon 或 Windows x86_64 客户端检查更新且 `latest.json` 包含对应 platform 条目
- **THEN** 更新检查与安装流程可正常执行

#### Scenario: 1.0.0 之前版本无 updater

- **WHEN** 用户运行版本号低于 1.0.0 的安装包
- **THEN** 该构建不包含应用内 updater 能力

#### Scenario: SemVer 基线升级至 CalVer

- **WHEN** 用户安装版本为 `1.0.1` 且 `latest.json` 版本为 `2026.6.14`
- **THEN** 应用内 updater SHALL 提示新版本并可完成安装

### Requirement: 启动清理 stale updater 临时文件

系统 SHALL 在每次应用启动后，于**后台线程**扫描操作系统临时目录（如 Windows `%TEMP%`、macOS `$TMPDIR`）的**顶层条目**，删除匹配 updater 产物 pattern 且**创建时间**早于 **24 小时**的文件或目录。清理 MUST NOT 阻塞 UI 线程或 setup 主线程；任何单条删除失败 MUST 静默忽略；整个清理过程 MUST 快速完成（仅扫描 temp 顶层，条目数上限 512，无重试、无用户可见错误）。无法读取创建时间的条目 MUST 跳过。

匹配 pattern MUST 严格对应 Tauri 实际产物命名（`productName` = `DocAgent`，CalVer `YYYY.M.D` 无段前导零）：

1. **Updater 临时目录**（`tauri-plugin-updater` `make_temp_dir`）：`DocAgent-{CalVer}-updater-{random}/`
2. **Updater 临时安装包**（`write_to_temp`）：`DocAgent-{CalVer}-installer.exe`
3. **NSIS 发布/更新包**（bundler）：`DocAgent_{CalVer}_x64-setup.exe`

MUST NOT 匹配泛化的 `DocAgent-*.exe`、`.msi`、或其他应用含 `-updater` 的文件名。

#### Scenario: 启动后后台清理

- **WHEN** 应用完成 `setup` 并进入主界面
- **THEN** 清理逻辑已在 detached 后台线程启动且 setup 未同步等待其完成

#### Scenario: 删除 24 小时前的 updater 目录

- **WHEN** temp 顶层存在 `DocAgent-2026.6.1-updater-abc123/` 且其创建时间早于 24 小时前
- **THEN** 系统 MUST 尝试删除该目录及其内容

#### Scenario: 保留 24 小时内产物

- **WHEN** temp 中存在当日 updater 临时目录或安装包
- **THEN** 系统 MUST NOT 删除该条目

#### Scenario: 清理失败不干扰用户

- **WHEN** 某条目删除因权限或文件占用失败
- **THEN** 应用 MUST 正常继续启动，不展示错误 dialog

#### Scenario: 扫描范围受限

- **WHEN** 执行清理
- **THEN** 仅扫描系统 temp 目录顶层条目，MUST NOT 递归扫描整个磁盘

