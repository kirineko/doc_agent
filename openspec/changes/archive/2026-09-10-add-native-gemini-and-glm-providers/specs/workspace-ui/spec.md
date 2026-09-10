## MODIFIED Requirements

### Requirement: 非 vision 粘贴 Toast

当用户在非 vision 模型下粘贴图片时，系统 SHALL 展示非阻塞 toast，文案说明需切换至当前可新建的视觉模型。

#### Scenario: DeepSeek 下粘贴图片

- **WHEN** 会话模型为 DeepSeek Flash 且用户粘贴图片
- **THEN** 保存附件并展示缩略图，不出现非视觉 toast

#### Scenario: 非视觉模型下粘贴图片

- **WHEN** 会话模型为 MiMo v2.5 Pro 且用户粘贴图片
- **THEN** 出现 toast 且不插入附件
