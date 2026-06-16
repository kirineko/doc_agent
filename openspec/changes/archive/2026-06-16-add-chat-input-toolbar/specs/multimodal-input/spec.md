## ADDED Requirements

### Requirement: 工具按钮选择图片附件

除剪贴板粘贴外，系统 SHALL 支持用户通过 Chat 输入区**图片**按钮，从文件对话框选择单张图片（`image/png`、`image/jpeg`、`image/webp`、`image/gif`），行为 MUST 与粘贴图片一致：vision 模型校验、写入 `.cache/attachments/`、展示可删除缩略图 chip、发送时随消息提交；**MUST NOT** 写入项目根目录或出现在 `@` 文件索引中。

#### Scenario: 按钮选图成功

- **WHEN** 当前模型 `supports_vision=true` 且用户通过图片按钮选择 PNG
- **THEN** 展示附件 chip，与粘贴 PNG 行为一致

#### Scenario: 非 vision 模型提示

- **WHEN** 当前模型 `supports_vision=false` 且用户通过图片按钮选择图片
- **THEN** 不添加 chip，展示与粘贴相同的 toast 提示切换模型

#### Scenario: 图片按钮与项目导入分离

- **WHEN** 用户通过图片按钮添加附件
- **THEN** 项目根目录与 `@` 索引无新增条目
