## ADDED Requirements

### Requirement: 设置抽屉检查更新入口

系统 SHALL 在顶栏提供设置入口，点击后从右侧滑出设置抽屉；抽屉内 MUST 以简洁文案展示「当前版本」与「最新版本」两行信息，且 MUST 仅在用户打开抽屉时请求 `latest.json` 获取最新版本号。当最新版本高于当前版本时，抽屉内 SHALL 提供「更新」入口触发安装流程。

#### Scenario: 设置抽屉展示版本信息

- **WHEN** 用户打开设置抽屉
- **THEN** 抽屉内可见当前版本与最新版本两行信息

#### Scenario: 打开抽屉时查询最新版本

- **WHEN** 用户打开设置抽屉
- **THEN** 系统通过 Tauri 后端请求 updater manifest 获取最新版本号
- **AND** 在用户未打开抽屉前 MUST NOT 为展示版本信息而发起该请求

#### Scenario: 更新进行中反馈

- **WHEN** 用户触发更新且请求尚未完成
- **THEN** 更新入口展示进行中状态（如禁用或 loading），防止重复触发
