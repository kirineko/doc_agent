# 设计：SemVer 兼容日历版本（CalVer）

## Context

- 当前最新发布：**1.0.1**（SemVer）；自动更新、OSS、`latest.json` 已验证。
- CI Release 触发：推送匹配 `*.*.*` 的纯数字 tag（无 `v` 前缀）。
- Tauri updater 与前端 `isNewerVersion` 均按 **major → minor → patch 数值**比较。
- SemVer 2.0 要求数字标识符 **不得含前导零**（`06`、`09` 违规）；Tauri updater 底层依赖 semver 解析。

## Goals / Non-Goals

**Goals：**

- 统一维护者发版口径：`YYYY.M.D`，与 tag / 三处版本文件 / `latest.json` 一致。
- 保证 `2026.6.14` 对 `1.0.1` 判定为更新（迁移无痛）。
- 文档与 Cursor rules 可执行、可检查。

**Non-Goals：**

- 自动化「按当天日期打 tag」脚本（可后续加 `scripts/calver-today.mjs`）。
- 同日第二次发布的第四段编码。
- 修改 `release.yml` 的 tag 匹配模式（`2026.6.14` 已满足 `*.*.*`）。

## Decisions

### D1：格式 `YYYY.M.D`（无前导零）

| 段 | 含义 | 合法示例 | 非法示例 |
|----|------|----------|----------|
| MAJOR | 公历年 | `2026` | — |
| MINOR | 月 | `6` | `06` |
| PATCH | 日 | `14` | `014`、`09`（日≤9 时写 `9` 非 `09`） |

示例：`2026.6.1`、`2026.6.14`、`2026.12.31`。

**备选 `YY.M.D`（`26.6.14`）** — 拒绝：易与「第 26 主版本」混淆。

**备选 `YYYY.MM.DD` 补零** — 拒绝：违反 SemVer 数字段规则，严格校验可能失败。

### D2：机器版本 vs 展示文案分离

- **机器**（tag、`package.json`、`tauri.conf.json`、`latest.json`、`releases/<ver>/`）：`2026.6.14`
- **展示**（GitHub Release 标题、CHANGELOG 日期标题）：`2026-06-14` 或 `Doc Agent 2026.6.14（2026-06-14）`

设置抽屉「当前版本 / 最新版本」继续显示机器版本字符串，不强制补零。

### D3：比较逻辑不改动

现有 `isNewerVersion` 与 Tauri `check()` 对 `2026.6.14` vs `1.0.1` 数值比较成立，无需新依赖。

### D4：迁移策略

- 保留历史 tag `1.0.0`、`1.0.1` 与 OSS 路径不变。
- **首个 CalVer 发版**取**实际发布日当天**的 `YYYY.M.D`（非固定示例日期）。
- CHANGELOG 增加「版本策略变更」说明。

### D5：同日多版

MVP **不支持**同日第二版。若紧急热修，维护者择一：次日发版，或接受「同日仅一发」约束。不在 PATCH 嵌入序号。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 维护者误用 `2026.06.14` | rules + README 明确禁止；发版 checklist |
| 月初/月末日 vs 月混淆 | 文档表格固定 MAJOR=年、MINOR=月、PATCH=日 |
| 2099 年后 MAJOR 四位仍可比 | 2100+ 继续用年份作 MAJOR |
| 与 SemVer「兼容性」语义脱节 | 在 `project-versioning` 中声明：本项目 CalVer 不承载 API 兼容语义 |

## Migration Plan

1. 合并本 change 的文档与 rules（无需发版）。
2. 下次功能/修复发版时，版本号改为发布日 `YYYY.M.D`，打 tag 走现有 Release CI。
3. 已装 `1.0.1` 用户收到首个 CalVer 更新提示，验证一次即可关闭迁移。

## Open Questions

- 无。首个 CalVer tag 日期由维护者在发版当日确定。
