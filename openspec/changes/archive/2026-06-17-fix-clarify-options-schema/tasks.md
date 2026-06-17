## 1. Schema 与校验

- [x] 1.1 `clarify.rs`：`MAX_OPTIONS = 12`；`options` schema 增加 `minItems: 2`、`maxItems: 12`；更新 tool description
- [x] 1.2 补充/更新测试：10 项通过、13 项拒绝

## 2. 文档与 spec

- [x] 2.1 更新 `assets/skills/clarify/SKILL.md` 提问原则（2–6 + allow_custom）
- [x] 2.2 更新 `openspec/changes/fix-clarify-options-schema/specs/clarify-interaction/spec.md` delta

## 3. 验证

- [x] 3.1 `cargo test` clarify 相关用例
