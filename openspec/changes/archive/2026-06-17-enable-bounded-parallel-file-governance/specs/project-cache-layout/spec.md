## MODIFIED Requirements

### Requirement: 缓存子目录职责

系统 SHALL 将 `.cache/` 划分为下列子目录，职责 MUST NOT 混用：

| 子目录 | 职责 |
|--------|------|
| `attachments/` | 用户粘贴的图片附件；路径写入 `attachments_json` |
| `skill-run/<session_key>/` | `skill_run` 临时脚本与错误现场；按 session 隔离（8 位 hex，同会话跨 turn 路径不变） |
| `ooxml/<session_key>/<work_key>/` | `ooxml_unpack` 自动生成的 OOXML 编辑工作区；`work_key = hash(session, turn, source)`，无独立 turn 目录层 |
| `pdf/<cache_key>/` | PDF 页渲染 PNG 与 manifest；源文件或渲染参数变则失效 |
| `tmp/<session_key>/<turn_key>/` | **预留**通用 turn-scoped 临时目录（`turn_tmp_dir` helper 已定义，尚无调用方） |

#### Scenario: skill_run 写入 session-scoped 目录

- **WHEN** Agent 调用 `skill_run` 并传入 inline `code`
- **THEN** 执行前脚本保存为 `.cache/skill-run/<session_key>/script.js`

#### Scenario: OOXML 自动工作区写入 ooxml

- **WHEN** Agent 调用 `ooxml_unpack {"path":"template.docx"}` 且未传 `out_dir`
- **THEN** 解包目录位于 `.cache/ooxml/<session_key>/<work_key>/`

#### Scenario: PDF 渲染写入 pdf

- **WHEN** `pdf_render_pages` 或 `pdf_read` 触发渲染
- **THEN** 页图与 manifest 位于 `.cache/pdf/<cache_key>/`

### Requirement: 缓存边界与缺失降级

系统 SHALL 区分可重建派生缓存、系统 scratch 工作区与会话附件：

- **`pdf/`**：可重建派生缓存；删除后下次工具调用重建
- **`skill-run/<session_key>/`**：脚本恢复现场；turn 结束无 `error.json` 时删除整个 session scratch 目录；有失败现场则保留至修复或覆盖
- **`ooxml/<session_key>/`**：系统生成的 OOXML 工作区根；叶子为 `<work_key>/`，MUST 隐藏于普通文件浏览与 `@` 候选
- **`tmp/<session_key>/<turn_key>/`**：**预留**；helper 已定义，尚无生产调用；未来用于 turn 内中间产物，turn 结束可清理
- **`attachments/`**：会话附件文件；DB 仅存相对路径，不存 base64

本变更 MUST NOT 提供「清理 cache」UI 或自动 GC。

#### Scenario: cache 目录对用户浏览隐藏

- **WHEN** 用户打开项目文件浏览或 `@` 文件候选
- **THEN** `.cache/` 及其子路径 MUST NOT 出现在列表中

### Requirement: 路径常量集中定义

系统 SHALL 在 `core/cache_paths`（或等效模块）集中定义 `.cache/` 相关相对路径常量与 helper；`skill_run_tmp`、`pdf_cache`、`ooxml_unpack`、`save_upload` 与 `is_upload_attachment_path` MUST 引用该模块，禁止硬编码分散路径字符串。

#### Scenario: scratch 路径常量单一来源

- **WHEN** 实现需要写入 skill_run 脚本或 OOXML 自动工作区
- **THEN** 使用 `cache_paths` 模块导出的 helper，而非手写 `.cache/skill-run/` 或 `.cache/ooxml/`
