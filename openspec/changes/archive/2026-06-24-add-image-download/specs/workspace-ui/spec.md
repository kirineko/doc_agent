## ADDED Requirements

### Requirement: 图片下载工具中文标签
系统 SHALL 为 `image_download` 提供中文工具链标签（如「下载图片」），并在工具名注册列表测试（`toolLabels.test.ts` 的 `EXPECTED_TOOLS`）中保持同步。

#### Scenario: 工具链展示中文标签
- **WHEN** 右侧工具链渲染一次 `image_download` 调用卡片
- **THEN** 卡片显示对应中文标签（非原始英文名 `image_download`）

#### Scenario: 标签注册表与后端工具一致
- **WHEN** 运行前端工具标签测试
- **THEN** `REGISTERED_TOOL_NAMES` 包含 `image_download`，与后端 `default_tools` 工具名集合一致
