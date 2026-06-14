# 提案：采用 SemVer 兼容日历版本（adopt-calver-versioning）

## Why

当前使用语义化版本（`1.0.0`、`1.0.1`），版本号无法直观反映发布日期；自动更新闭环已在 `1.0.1` 验证通过，适合在此时固化版本策略，避免后续发版口径不一致。采用 **SemVer 兼容的日历版本（CalVer，`YYYY.M.D`）** 可在保留 Tauri updater / CI tag 规则的前提下，让 tag、OSS 路径与用户感知日期对齐。

## What Changes

- 定义项目正式版本格式：**`YYYY.M.D`**（年.月.日），三段均为**无前导零**的非负整数，满足 SemVer 2.0 数字段规则。
- 明确版本语义：MAJOR = 公历年，MINOR = 月（1–12），PATCH = 日（1–31）；**不**使用 `2026.06.14` 等形式。
- 更新维护者文档与 Cursor rules：发版、tag、三处版本文件同步、CHANGELOG 书写约定。
- 约定展示层：机器版本（tag / `latest.json`）用 `2026.6.14`；面向用户的 Release 标题 / CHANGELOG 可用 `2026-06-14`。
- 自下一版发版起启用日历版本；**不**回溯修改已发布的 `1.0.0` / `1.0.1` tag 与 OSS 历史路径。
- **排除**：四段版本号、同日多版补丁编码（`DDN`）、自动从日期生成 tag 的 CI 脚本（MVP 仅文档与规范）、修改 updater 比较算法（沿用现有数值比较）。

## Capabilities

### New Capabilities

- `project-versioning`：项目版本号格式、SemVer 合规约束、发版同步点与展示约定。

### Modified Capabilities

- `release-distribution`：将「1.0.0 为自动更新首个版本」扩展为「支持 CalVer tag 触发完整发布流水线」；版本目录与 `latest.json.version` 使用日历版本。
- `app-updater`：明确 updater 以数值比较 CalVer 与历史 SemVer（`2026.6.14` > `1.0.1`）；`1.0.0` 仍为 updater 能力基线。

## Impact

- **规范**：`.cursor/rules/ci-and-release.mdc`、`README.md` 发版说明、`CHANGELOG.md` 书写说明。
- **发版流程**：下一 tag 示例 `2026.6.14`（以实际发布日为准）；`package.json`、`Cargo.toml`、`tauri.conf.json`、git tag 保持一致。
- **无破坏性**：已安装 `1.0.x` 用户可通过 updater 正常升至首个 CalVer 版本。
- **OSS / GH Release**：`releases/<version>/` 目录名随新格式变化；历史 `releases/1.0.0/` 等保留。
