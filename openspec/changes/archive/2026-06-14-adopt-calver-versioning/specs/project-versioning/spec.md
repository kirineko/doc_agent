## ADDED Requirements

### Requirement: 日历版本格式 YYYY.M.D

项目正式版本号 MUST 采用 **`YYYY.M.D`** 三段数字格式，其中 **MAJOR** 为四位公历年，**MINOR** 为月（1–12），**PATCH** 为日（1–31）。各段 MUST 为不含前导零的非负整数，以符合 SemVer 2.0 对数字标识符的约束。

#### Scenario: 合法日历版本示例

- **WHEN** 维护者为 2026 年 6 月 14 日发布定版
- **THEN** 版本号 MUST 为 `2026.6.14`

#### Scenario: 月初日与个位数月

- **WHEN** 发布日期为 2026 年 6 月 9 日
- **THEN** 版本号 MUST 为 `2026.6.9`，且 MUST NOT 使用 `2026.6.09` 或 `2026.06.9`

#### Scenario: 禁止前导零

- **WHEN** 维护者尝试使用 `2026.06.14` 作为 tag 或 `package.json` 版本
- **THEN** 该版本号视为不符合本规范，不得用于发布

### Requirement: 版本文件与 git tag 同步

发版时，下列位置的 `version` MUST 与 git tag 完全一致（无 `v` 前缀）：

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- git tag（触发 Release CI）

#### Scenario: 发版前三处一致

- **WHEN** 维护者推送 tag `2026.6.14`
- **THEN** 上述三处配置文件中的版本字段均为 `2026.6.14`

### Requirement: 版本新旧比较

系统判定「是否有新版本」时，MUST 按 MAJOR、MINOR、PATCH 依次作**数值**比较。日历版本 MUST 能正确判定高于历史 SemVer 版本（例如 `2026.6.14` 高于 `1.0.1`）。

#### Scenario: CalVer 高于历史 SemVer

- **WHEN** 当前安装版本为 `1.0.1` 且 `latest.json` 中版本为 `2026.6.14`
- **THEN** 系统 SHALL 判定存在可安装的新版本

#### Scenario: 同格式日期递增

- **WHEN** 当前安装版本为 `2026.6.14` 且 `latest.json` 中版本为 `2026.6.15`
- **THEN** 系统 SHALL 判定存在可安装的新版本

#### Scenario: 非更高版本不更新

- **WHEN** 当前安装版本为 `2026.6.15` 且 `latest.json` 中版本为 `2026.6.14`
- **THEN** 系统 MUST NOT 提供升级安装

### Requirement: 展示层日期格式

面向用户的 Release 标题、CHANGELOG 章节标题 MAY 使用 ISO 日期 `YYYY-MM-DD` 或括号注释标注发布日；机器可读版本字符串 MUST 仍为 `YYYY.M.D`，不得在 tag 或 `latest.json` 中使用补零日历格式。

#### Scenario: CHANGELOG 与 tag 区分

- **WHEN** 发布版本 `2026.6.14`
- **THEN** CHANGELOG 可使用 `## [2026.6.14] — 2026-06-14` 等形式，且 tag 仍为 `2026.6.14`

### Requirement: 历史版本保留

已发布的 SemVer tag（含 `1.0.0`、`1.0.1`）及对应 OSS 路径 MUST 保留且不得覆盖。日历版本策略自**下一版发版**起生效，不追溯改写历史 tag。

#### Scenario: 历史 OSS 路径可访问

- **WHEN** 用户访问 `releases/1.0.1/` 下历史安装包 URL
- **THEN** 该路径 MUST 继续有效（对象未被迁移或删除）
