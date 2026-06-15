## ADDED Requirements

### Requirement: Provider 余额查询 IPC

系统 SHALL 提供 Tauri command `fetch_provider_balances`，在 Rust 后端读取已保存的 API Key 并调用 DeepSeek / Kimi 官方余额 REST API。该 command MUST NOT 将 API Key 明文返回前端或写入日志。

对 `deepseek`：MUST 调用 `GET https://api.deepseek.com/user/balance`，从响应 `balance_infos` 中选取 `currency` 为 `CNY` 的条目，取 `total_balance` 作为总可用余额。

对 `kimi`：MUST 调用 `GET https://api.moonshot.cn/v1/users/me/balance`，当 `code` 为 `0` 时取 `data.available_balance` 作为总可用余额。

若某 provider 未配置 API Key，MUST 跳过该 provider（不发起 HTTP、不包含在返回列表中）。HTTP 请求 MUST 设置 10 秒超时。

返回列表中每项 MUST 包含 `provider`（`deepseek` 或 `kimi`）与 `display`（格式化后的展示字符串）。

#### Scenario: 已配置 DeepSeek Key 且返回 CNY 余额

- **WHEN** 用户已保存 DeepSeek API Key 且接口返回含 `currency: CNY` 的 `balance_infos`
- **THEN** `fetch_provider_balances` 返回含 `provider: deepseek` 的条目
- **AND** `display` 为带 `¥` 前缀、保留两位小数的总余额字符串

#### Scenario: 已配置 Kimi Key 且返回成功

- **WHEN** 用户已保存 Kimi API Key 且接口返回 `code: 0` 与 `data.available_balance`
- **THEN** 返回含 `provider: kimi` 的条目
- **AND** `display` 为带 `¥` 前缀、保留两位小数的字符串

#### Scenario: 未配置 Key 不查询

- **WHEN** 用户未保存 DeepSeek API Key
- **THEN** 返回列表中 MUST NOT 包含 `provider: deepseek`
- **AND** MUST NOT 向 DeepSeek 发起余额 HTTP 请求

#### Scenario: 查询失败返回占位符

- **WHEN** 用户已配置 Key 但 HTTP 非 2xx、JSON 解析失败、DeepSeek 无 CNY 条目或 Kimi `code` 非 0
- **THEN** 仍返回该 provider 条目
- **AND** `display` 为 `—`

#### Scenario: 并行查询已配置 provider

- **WHEN** 用户同时配置了 DeepSeek 与 Kimi API Key
- **THEN** 单次 `fetch_provider_balances` 调用返回两条记录
- **AND** 两条 HTTP 请求 SHOULD 并行执行以降低延迟

#### Scenario: MiMo 不在范围

- **WHEN** 调用 `fetch_provider_balances`
- **THEN** MUST NOT 查询 MiMo 或返回 `provider: mimo` 条目
