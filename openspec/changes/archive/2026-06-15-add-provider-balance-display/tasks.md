## 1. OpenSpec

- [x] 1.1 proposal.md / design.md / specs / tasks.md

## 2. Rust 后端

- [x] 2.1 新增 `core/provider_balance.rs`：DeepSeek / Kimi HTTP 查询、CNY 过滤、¥ 格式化、失败返回 `—`
- [x] 2.2 新增 IPC `fetch_provider_balances`：Key 门禁、并行查询、注册至 `lib.rs`
- [x] 2.3 单元测试：JSON 解析、无 Key 跳过、无 CNY / 失败 → `—`

## 3. 前端

- [x] 3.1 新增 `src/lib/providerBalance.ts`（invoke 封装与类型）
- [x] 3.2 扩展 `SettingsDrawer`：版本区块下「账户余额」、打开时查询、`…` / `—` 状态
- [x] 3.3 前端测试（可选）：格式化或 hook 行为

## 4. 验证

- [x] 4.1 `cargo test` / `npm test` / `npm run typecheck` 通过
