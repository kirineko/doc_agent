## ADDED Requirements

### Requirement: 多工具 streaming 占位平滑过渡

当同轮多个工具处于参数流式生成（`tool_call_stream`）阶段时，收到 `ToolCall { status: running }` MUST 仅升级对应 `index` 的 streaming 占位卡片为 running，MUST NOT 删除其他 index 的 streaming 占位。同批全部工具开始执行后，工具链卡片数量 MUST NOT 少于 streaming 阶段的占位数量（除非某工具已被同一 index 的 running 事件替换）。

#### Scenario: 三个 pdf_read 不整栏清空

- **WHEN** 右侧工具链显示三个 `pdf_read` 的「生成参数中」卡片，且本轮三个工具即将执行
- **THEN** 过渡后仍显示三张卡片且均为「执行中」或已完成，MUST NOT 出现整栏空白占位文案

#### Scenario: 按 index 就地升级

- **WHEN** 收到 `ToolCall { index: 1, status: running }` 且存在 `streaming-1` 占位
- **THEN** 该占位卡片升级为 running 并展示参数，其他 streaming 占位保持不变
