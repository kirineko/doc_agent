# release-distribution Specification

## Purpose
TBD - created by archiving change add-auto-update-oss. Update Purpose after archive.
## Requirements
### Requirement: 构建时生成 updater 签名产物

Release 构建 MUST 启用 `createUpdaterArtifacts: true`，并在持有 `TAURI_SIGNING_PRIVATE_KEY` 的环境下完成 Tauri build。构建产物 MUST 包含：

- macOS aarch64：`*.app.tar.gz` 与同名 `.sig`
- Windows x86_64：`*-setup.exe` 与同名 `.sig`
- 首次安装包：`.dmg`、`*-setup.exe`、`.msi`

#### Scenario: macOS 构建产出 updater 包

- **WHEN** macOS 矩阵 job 成功完成 `tauri build`
- **THEN** artifact 中包含 `*.app.tar.gz` 及其 `.sig` 文件

#### Scenario: Windows 构建产出 updater 包

- **WHEN** Windows 矩阵 job 成功完成 `tauri build`
- **THEN** artifact 中包含 `*-setup.exe` 及其 `.sig` 文件

### Requirement: 发布至阿里云 OSS

`publish` job MUST 将安装包与 updater 包上传至 Bucket `doc-agent`（地域 `oss-cn-guangzhou`），路径为 `releases/<version>/`。根路径 MUST 上传 `latest.json`，且 `latest.json` MUST 设置 `Cache-Control: no-cache`。

#### Scenario: 版本目录上传成功

- **WHEN** tag `1.0.0` 触发 publish job 且各平台构建成功
- **THEN** OSS 存在 `releases/1.0.0/` 下各平台安装包与 updater 包，且对象可匿名 HTTPS 访问

#### Scenario: latest.json 指向 OSS URL

- **WHEN** publish job 生成 `latest.json`
- **THEN** 其中 `platforms.*.url` MUST 以 `https://doc-agent.oss-cn-guangzhou.aliyuncs.com/releases/<version>/` 为前缀，且 `signature` 为对应 `.sig` 文件内容（非 URL）

### Requirement: latest.json 平台完整性

`latest.json` MUST 包含且仅包含当前发版矩阵支持的平台键：`darwin-aarch64` 与 `windows-x86_64`。任一平台构建失败时，publish job MUST NOT 发布不完整 `latest.json` 至 OSS 根路径。

#### Scenario: 双平台成功时发布 latest.json

- **WHEN** macOS 与 Windows 构建均成功
- **THEN** `latest.json` 同时包含 `darwin-aarch64` 与 `windows-x86_64` 条目且字段完整

#### Scenario: 单平台失败阻止发布

- **WHEN** 任一矩阵平台构建失败
- **THEN** publish job 不向 OSS 根路径覆盖 `latest.json`

### Requirement: 保留 GitHub Release

在 OSS 上传的同时，系统 MUST 继续将构建产物上传至 GitHub Release（与现有 `softprops/action-gh-release` 行为一致），作为备用下载渠道与版本记录。

#### Scenario: GitHub Release 含安装包

- **WHEN** tag 触发 release 且构建成功
- **THEN** GitHub Release Assets 包含 `.dmg`、Windows 安装包及关联构建产物

### Requirement: 日历版本触发发布流水线

Release 流水线 MUST 由符合 `*.*.*` 模式的纯数字 git tag 触发（无 `v` 前缀）。自日历版本策略生效起，tag 与 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 中的 `version` MUST 使用 **`YYYY.M.D`** 格式（见 `project-versioning`）。`publish` job MUST 将产物上传至 `releases/<version>/`，且 `latest.json` 的 `version` 字段与 tag 一致。

#### Scenario: CalVer tag 触发完整流水线

- **WHEN** 推送 tag `2026.6.14` 且双平台构建成功
- **THEN** 执行 check、双平台 build、OSS 上传、GitHub Release 发布，且 `latest.json` 中 `version` 为 `2026.6.14`

#### Scenario: OSS 版本目录使用 CalVer

- **WHEN** tag `2026.6.14` 的 publish job 完成
- **THEN** OSS 存在 `releases/2026.6.14/` 下各平台安装包与 updater 包，且对象可匿名 HTTPS 访问

