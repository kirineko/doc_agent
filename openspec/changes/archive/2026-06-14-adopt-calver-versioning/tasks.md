## 1. 规范与 rules

- [x] 1.1 更新 `.cursor/rules/ci-and-release.mdc`：CalVer `YYYY.M.D`、禁止前导零、发版示例
- [x] 1.2 更新 `README.md` 维护者「发版说明」章节
- [x] 1.3 在 `CHANGELOG.md` 顶部补充版本策略说明（CalVer 与历史 SemVer 并存）

## 2. 文档对齐（实现日可选）

- [x] 2.1 首个 CalVer 发版时在 CHANGELOG 增加「版本策略变更」条目（`[Unreleased]` 模板已就绪，发版日改标题即可）
- [x] 2.2 确认 `release.yml` Release 说明模板无需改动（`${{ env.VERSION }}` 已兼容 CalVer；workflow 示例已更新）

## 3. 验证

- [x] 3.1 维护者自检：对照 `project-versioning` 检查合法/非法版本示例（见 `verification.md` + `src/lib/version.test.ts`）
- [ ] 3.2 首个 CalVer 发布后：确认 `latest.json`、`releases/<ver>/`、updater 从 `1.0.1` 升级成功（见 `verification.md` 清单）
