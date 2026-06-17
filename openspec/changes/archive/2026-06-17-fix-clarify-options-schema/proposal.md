## Why

模型调用 `clarify_ask` 时经常在 `options` 里枚举超过 6 项，触发运行时校验失败且不出澄清卡片。根因是 JSON Schema 未声明 `maxItems`，模型在 strict tool 模式下看不到上限；SKILL 文案「2–4 个选项」也与实现不一致。

## What Changes

- `clarify_ask` 的 `options` JSON Schema 增加 `minItems: 2`、`maxItems: 12`
- 运行时校验上限从 6 调整为 12（`MAX_OPTIONS`）
- clarify SKILL「提问原则」改为：**2–8 个选项 + `allow_custom` 承接「其他」**（硬上限 12，为模型偶发多枚举留余量）
- 更新 `clarify-interaction` spec 中 options 数量要求

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `clarify-interaction`：`single`/`multi` 的 `options` 合法范围为 2–7，且 tool JSON Schema MUST 声明 `minItems`/`maxItems`

## Impact

- `src-tauri/src/tools/clarify.rs`（schema + 校验常量）
- `src-tauri/assets/skills/clarify/SKILL.md`（提问原则）
- `src-tauri/src/tools/tests.rs`（校验测试）
