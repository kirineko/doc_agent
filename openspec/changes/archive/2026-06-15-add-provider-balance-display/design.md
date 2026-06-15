## Context

- 设置抽屉（`SettingsDrawer`）已有「打开时拉取最新版本」模式（`fetch_latest_release_version` IPC）。
- API Key 存于 Rust `Secrets`（`config.toml`），前端仅通过 `has_api_key` 获知是否已配置，不可将 Key 传入 WebView。
- DeepSeek chat base：`https://api.deepseek.com`；Kimi chat base：`https://api.moonshot.cn`（与现有 provider 一致）。
- MiMo 官方暂无余额 REST API，本变更不覆盖。

## Goals / Non-Goals

**Goals:**

- 用户打开设置抽屉时，对已配置 Key 的 DeepSeek / Kimi 并行查询余额
- UI 仅展示人民币总可用余额；失败显示 `—`；加载显示 `…`
- Key 未配置：不请求、不展示对应行；两者均未配置时不显示「账户余额」区块

**Non-Goals:**

- MiMo / Tavily 余额
- USD 或多币种并列
- 赠金、充值、代金券拆分
- 手动刷新、定时轮询、低余额告警
- 在「模型与密钥」Drawer 或侧栏展示

## Decisions

### 1. Rust 侧聚合 IPC（非前端直连）

新增 `fetch_provider_balances` Tauri command，在 Rust 内：

1. `has_api_key("deepseek")` / `has_api_key("kimi")` 门禁
2. `get_api_key` 读取 Key（不返回给前端）
3. 并行 HTTP GET（`tokio::join!`）
4. 解析并格式化为 UI 字符串

**理由**：Key 不出后端；两家响应结构不同，集中解析更易测。

**备选**：前端按 provider 多次 invoke — 拒绝，重复 gate 且类型分散。

### 2. HTTP 端点与字段

| Provider | URL | 展示字段 |
|----------|-----|----------|
| DeepSeek | `GET https://api.deepseek.com/user/balance` | `balance_infos` 中 `currency == "CNY"` 的 `total_balance` |
| Kimi | `GET https://api.moonshot.cn/v1/users/me/balance` | `data.available_balance`（`code === 0`） |

Header：`Authorization: Bearer <key>`。超时 10s（与 `fetch_latest_release_version` 一致）。

### 3. 展示格式化

- 前缀 `¥`，金额保留 2 位小数
- DeepSeek `total_balance` 为字符串 → parse 后 format，parse 失败 → `—`
- Kimi 为 number → format 2 位
- 成功但 DeepSeek 无 CNY 条目 → `—`
- HTTP 非 2xx、JSON 解析失败、Kimi `code !== 0` → 该 provider 行显示 `—`（仍返回行，因 Key 已配置）

### 4. IPC 返回类型

```rust
#[derive(Serialize)]
pub struct ProviderBalanceRow {
    pub provider: String,  // "deepseek" | "kimi"
    pub display: String,   // "¥12.34" | "—"
}
```

仅包含已配置 Key 的 provider。前端用 `providerLabel` 渲染标签。

### 5. 模块布局

- `src-tauri/src/core/provider_balance.rs`：HTTP + 解析 + format（可单测）
- `src-tauri/src/ipc/provider_balance.rs` 或 `ipc/mod.rs` 内 command：读 Secrets、调 core
- 前端：`SettingsDrawer` 在 `open` 时 invoke；抽 `fetchProviderBalances()` 至 `src/lib/providerBalance.ts`（可选）

### 6. UI 结构

版本 `section` 下方新增 `section`（`config-surface` 同款）：

- 标题「账户余额」
- 每行：`DeepSeek` / `Kimi` + 右对齐 `display`
- `rows.length === 0` 且非 loading → 不渲染整块

打开抽屉时与版本查询并行发起；各自独立 loading 状态（或余额区统一 `…`）。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| Key 写入日志 | 禁止 log Authorization；错误信息不含 Key |
| API 结构变更 | 解析失败 → `—`；单测覆盖 JSON 样例 |
| 余额延迟 | 接受平台侧延迟；不做轮询 |
| DeepSeek 仅 USD 账户 | 无 CNY 条目 → `—`，符合「仅人民币」约定 |
| 401 暴露 Key 失效 | 显示 `—`，用户可在模型 Drawer 更新 Key |

## Migration Plan

无数据迁移。发版后即生效；未配置 Key 用户无 UI 变化。

## Open Questions

无（展示规则已在 explore 阶段确认：仅 CNY 总余额，失败 `—`）。
