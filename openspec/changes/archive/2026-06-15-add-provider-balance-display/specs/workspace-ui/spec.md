## ADDED Requirements

### Requirement: 设置抽屉账户余额展示

系统 SHALL 在设置抽屉内、版本信息区块下方展示账户余额。仅当用户已配置 DeepSeek 和/或 Kimi API Key 时，MUST 展示对应 provider 一行；两者均未配置时 MUST NOT 展示「账户余额」区块。

每行 MUST 以 provider 名称（DeepSeek / Kimi）与右对齐的总余额展示字符串组成。余额 MUST 仅展示人民币总可用余额；加载中 MUST 显示 `…`；查询失败 MUST 显示 `—`。

余额查询 MUST 仅在用户打开设置抽屉时发起；在用户未打开设置抽屉前 MUST NOT 为展示余额而调用 `fetch_provider_balances`。

#### Scenario: 打开抽屉时查询余额

- **WHEN** 用户打开设置抽屉且已配置至少一个 DeepSeek 或 Kimi API Key
- **THEN** 系统调用 `fetch_provider_balances` 获取余额
- **AND** 在用户未打开设置抽屉前 MUST NOT 为展示余额发起该请求

#### Scenario: 均未配置时不查询余额

- **WHEN** 用户打开设置抽屉且未配置 DeepSeek 与 Kimi API Key
- **THEN** MUST NOT 调用 `fetch_provider_balances`

#### Scenario: 已配置 DeepSeek 展示一行

- **WHEN** 用户已配置 DeepSeek API Key 且余额查询成功
- **THEN** 设置抽屉「账户余额」区块可见 DeepSeek 一行及格式化后的 ¥ 金额

#### Scenario: 未配置 Key 不展示行

- **WHEN** 用户未配置 Kimi API Key
- **THEN** 设置抽屉 MUST NOT 展示 Kimi 余额行

#### Scenario: 均未配置隐藏区块

- **WHEN** 用户未配置 DeepSeek 与 Kimi API Key
- **THEN** 设置抽屉 MUST NOT 展示「账户余额」区块

#### Scenario: 查询失败显示占位符

- **WHEN** 用户已配置 Key 但余额查询失败
- **THEN** 对应 provider 行显示 `—`

#### Scenario: 加载中状态

- **WHEN** 用户打开设置抽屉且余额请求尚未完成
- **THEN** 已配置 Key 的 provider 行显示 `…` 直至请求结束
