## 决策

### D1. 硬上限 12、推荐 2–8

- **硬上限**：`MAX_OPTIONS = 12`，JSON Schema `maxItems: 12`
- **推荐写法**：SKILL 指导模型列出 2–8 个实质选项，用 `allow_custom: true` 承接「其他」
- **余量**：硬上限高于推荐值，模型偶发枚举 9–10 项（如 PPT 内容板块）仍可通过，避免不出澄清卡片

### D2. 不增加运行时自动截断

校验失败仍返回结构化错误，由模型 retry；schema 约束应足够降低失败率。

## 不变

- `minItems: 2` 不变
- `strict: true` 对 `clarify_ask` 不变
- 每轮仅一个 pending clarify 不变
