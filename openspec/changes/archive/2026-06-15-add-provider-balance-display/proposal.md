## Why

用户已在「模型与密钥」中配置 DeepSeek / Kimi API Key，但无法在应用内快速确认账户是否还有可用余额，需跳转各平台控制台。在设置抽屉（版本信息旁）展示人民币总余额，可在打开面板时一次性自查，减少对话中途因余额不足失败的情况。

## What Changes

- 新增 Tauri IPC：按 provider 查询 DeepSeek / Kimi 账户余额（Rust 侧读取 Key 并调用官方 REST API）
- 设置抽屉版本区块下方展示「账户余额」：仅显示已配置 Key 的 DeepSeek、Kimi 两行
- 展示规则：仅人民币、仅总可用余额；加载中显示 `…`；查询失败显示 `—`
- 仅在用户打开设置抽屉时发起余额请求（与最新版本查询同一时机）
- **不做**：MiMo 余额（官方暂无接口）、USD 币种、赠金/充值拆分、手动刷新按钮、侧栏/模型 Drawer 展示

## Capabilities

### New Capabilities

- `provider-balance`：DeepSeek / Kimi 余额查询 IPC、API 集成、Key 门禁与响应格式化契约

### Modified Capabilities

- `workspace-ui`：设置抽屉新增账户余额展示区块及打开时查询行为

## Impact

- **Rust**：`ipc/` 新 command；`core/` 或 `agent/provider/` 余额 HTTP 客户端；`lib.rs` 注册
- **前端**：`SettingsDrawer.tsx` UI 扩展；`App.tsx` 或 hook 调用 IPC；可选 `types` 对齐
- **OpenSpec**：`workspace-ui` spec delta
- **依赖**：无新 crate（复用 `reqwest`）
- **风险**：余额 API 变更或 Key 失效时展示 `—`；不在日志中输出 Key 或 Authorization
