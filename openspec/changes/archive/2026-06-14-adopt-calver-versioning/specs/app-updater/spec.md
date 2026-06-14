## MODIFIED Requirements

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
