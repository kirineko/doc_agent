## MODIFIED Requirements

### Requirement: 日历版本触发发布流水线

Release 流水线 MUST 由符合 `*.*.*` 模式的纯数字 git tag 触发（无 `v` 前缀）。自日历版本策略生效起，tag 与 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 中的 `version` MUST 使用 **`YYYY.M.D`** 格式（见 `project-versioning`）。`publish` job MUST 将产物上传至 `releases/<version>/`，且 `latest.json` 的 `version` 字段与 tag 一致。

#### Scenario: CalVer tag 触发完整流水线

- **WHEN** 推送 tag `2026.6.14` 且双平台构建成功
- **THEN** 执行 check、双平台 build、OSS 上传、GitHub Release 发布，且 `latest.json` 中 `version` 为 `2026.6.14`

#### Scenario: OSS 版本目录使用 CalVer

- **WHEN** tag `2026.6.14` 的 publish job 完成
- **THEN** OSS 存在 `releases/2026.6.14/` 下各平台安装包与 updater 包，且对象可匿名 HTTPS 访问

## REMOVED Requirements

### Requirement: 1.0.0 为自动更新首个版本

**Reason**：自动更新基线已在 `1.0.0` / `1.0.1` 落地；版本策略升级为 CalVer，不再将 `1.0.0` 固定为唯一发版版本号。

**Migration**：`1.0.0` 仍为 **updater 能力**首次生效的最低安装基线（见 `app-updater`）；后续发版版本号改用 `YYYY.M.D`，由 `project-versioning` 约束。
