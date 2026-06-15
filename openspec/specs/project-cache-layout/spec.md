# project-cache-layout Specification

## Purpose
TBD - created by archiving change unify-project-cache-layout. Update Purpose after archive.
## Requirements
### Requirement: 统一项目缓存根目录

系统 SHALL 在项目沙箱根目录下使用 **唯一** 点目录 `.cache/` 作为所有内部缓存与附件的父目录。项目根 MUST NOT 再新建 `.uploads/` 或 `.skill-run/`。`.cache/` 及其子目录 MUST 沿用现有点目录隐藏规则，不出现在 `@` 候选与项目文件浏览列表。

#### Scenario: 项目根仅一个点目录

- **WHEN** 用户通过粘贴图片、`skill_run` 或 `pdf_read` 触发内部写入
- **THEN** 新文件仅写入 `.cache/attachments/`、`.cache/skill-run/` 或 `.cache/pdf/` 下
- **AND** 项目根不再出现新的 `.uploads/` 或 `.skill-run/` 目录

#### Scenario: cache 目录对用户浏览隐藏

- **WHEN** 用户打开项目文件浏览或 `@` 文件候选
- **THEN** `.cache/` 及其子路径 MUST NOT 出现在列表中

### Requirement: 缓存子目录职责

系统 SHALL 将 `.cache/` 划分为下列子目录，职责 MUST NOT 混用：

| 子目录 | 职责 |
|--------|------|
| `attachments/` | 用户粘贴的图片附件；路径写入 `attachments_json` |
| `skill-run/` | `skill_run` 临时脚本与错误现场；turn 内自动清理 |
| `pdf/<cache_key>/` | PDF 页渲染 PNG 与 manifest；源文件或渲染参数变则失效 |

#### Scenario: 附件写入 attachments

- **WHEN** vision 模型下用户粘贴 PNG 并保存
- **THEN** 文件位于 `.cache/attachments/<uuid>.<ext>`

#### Scenario: skill_run 写入 skill-run

- **WHEN** Agent 调用 `skill_run` 并传入 inline `code`
- **THEN** 执行前脚本保存为 `.cache/skill-run/script.js`

#### Scenario: PDF 渲染写入 pdf

- **WHEN** `pdf_render_pages` 或 `pdf_read` 触发渲染
- **THEN** 页图与 manifest 位于 `.cache/pdf/<cache_key>/`

### Requirement: 缓存边界与缺失降级

系统 SHALL 区分可重建派生缓存与会话附件，并在附件文件缺失时降级而不中断会话：

- **`pdf/`** 与 **`skill-run/`**：视为可重建或临时现场；删除后由下次工具调用重建或不再保留失败现场
- **`attachments/`**：视为会话附件文件；DB 仅存相对路径，不存 base64

当 `attachments_json` 指向的文件不存在时：

- UI MUST 展示「无法加载」占位，保留消息文本
- Agent 重建上下文时 MUST 静默跳过缺失附件（不报错、不中断 turn）
- 当轮新发送的附件若文件缺失 MUST 返回明确错误（与现有 `encode_attachment_data_url` 行为一致）

本变更 MUST NOT 提供「清理 cache」UI 或自动 GC。

#### Scenario: 附件文件缺失时 UI 降级

- **WHEN** 历史消息含 `attachments_json` 指向 `.cache/attachments/missing.png` 且磁盘无该文件
- **THEN** 消息气泡展示「无法加载」缩略图占位，文本内容仍可见

#### Scenario: 附件文件缺失时 Agent 静默跳过

- **WHEN** Agent turn 重建上下文且某条 user 消息的附件文件已不存在
- **THEN** 该条消息以文本（及空 `image_urls`）送入 LLM，turn 不因缺失附件失败

#### Scenario: 本变更不提供清 cache UI

- **WHEN** 用户打开应用设置或项目菜单
- **THEN** 不存在「清理项目 cache」或等效批量删除 `.cache/` 的入口

### Requirement: 路径常量集中定义

系统 SHALL 在 `core/cache_paths`（或等效模块）集中定义 `.cache/` 相关相对路径常量；`skill_run_tmp`、`pdf_cache`、`save_upload` 与 `is_upload_attachment_path` MUST 引用该模块，禁止硬编码分散路径字符串。

#### Scenario: 常量单一来源

- **WHEN** 实现需要写入 skill_run 脚本或用户附件
- **THEN** 使用 `cache_paths` 模块导出的常量而非字面量 `.skill-run/` 或 `.uploads/`

### Requirement: 不迁移历史目录

系统 MUST NOT 读取、写入或删除项目根下遗留的 `.uploads/` 与 `.skill-run/` 目录；本变更不提供路径 fallback 或 DB 路径迁移。

#### Scenario: 旧路径不被新代码使用

- **WHEN** 用户项目根仍存在 `.uploads/` 或 `.skill-run/`
- **THEN** 新粘贴图片与 `skill_run` 仍写入 `.cache/` 下对应子目录
- **AND** 系统不自动迁移或删除旧目录内容

