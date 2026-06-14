## ADDED Requirements

### Requirement: 循环内自动压缩触发

系统 SHALL 在 Agent 工具循环的**每一步开头**（构造 LLM 请求之前）检查上下文是否需要压缩：以 `token_count + pending_estimate` 与当前模型 `max_context_size` 调用压缩触发判定，命中则先执行压缩并以未归档消息重建 `working_messages`，再发起本步请求。触发点 MUST 覆盖单个 turn 内连续工具结果累加的场景，而非仅在 turn 开始时检查一次。

#### Scenario: turn 内大工具输出触发压缩

- **WHEN** 单个 turn 内连续多次工具调用使累计上下文接近模型上限
- **THEN** 在下一步构造请求前触发压缩，压缩后再发起请求，请求上下文不超过上限

#### Scenario: 未超阈值不压缩

- **WHEN** 当前 `token_count + pending_estimate` 低于触发阈值
- **THEN** 不执行压缩，直接发起本步请求

### Requirement: 工作上下文基于未归档消息重建

系统 SHALL 使 `build_working_messages` 仅纳入未归档（archived = 0）的消息构造工作上下文，从而让压缩产生的摘要消息与保留消息自然成为后续轮次的上下文基础。

#### Scenario: 压缩后续 turn 复用摘要

- **WHEN** 某 turn 已压缩历史并写入摘要，用户在后续 turn 继续对话
- **THEN** 新 turn 重建的工作上下文包含该摘要而非已归档的原始旧消息

### Requirement: 提高单 turn 工具步数上限

系统 SHALL 将单个 turn 的最大工具调用步数上限从 32 提高（目标 64），以支撑文档生成类多步流程（澄清 → 多次 skill_read/skill_run → 校验）。该上限仍作为防失控循环的保护，达到上限时行为与原「达到最大轮次保护」一致。

#### Scenario: 多步文档流程不易触顶

- **WHEN** 一个文档生成 turn 需要超过 32 步工具调用但少于新上限
- **THEN** loop 正常完成而不再因步数上限提前中断

#### Scenario: 达到新上限仍有保护

- **WHEN** 工具调用步数达到提高后的上限
- **THEN** 系统终止循环并返回「已达最大步数」提示
