# 更新说明

本文件记录各版本的用户可见变更。安装包见 [GitHub Releases](https://github.com/kirineko/doc_agent/releases)；国内用户亦可从阿里云 OSS 下载（见 README）。

**版本号策略（自首个 CalVer 发版起）**：正式版本采用 **SemVer 兼容日历版本 `YYYY.M.D`**（年.月.日，各段无前导零，如 `2026.6.14`）。`1.0.0` / `1.0.1` 为历史 SemVer，保留不改动。

---

## [2026.6.23] — 2026-06-23

本版本带来项目级 Agent 配置、并行文件占用提示、澄清流程修复，PPT OOXML 斜杠命令与手动上下文压缩，以及右侧「构建产物」面板。

### 构建产物面板（BL-007 MVP）

- **Tab 切换**：右侧上半区新增「工具调用链 / 构建产物」Tab；徽标显示本轮去重产物数；切换 Tab 不影响上下分栏布局与折叠状态
- **本轮产物列表**：从 `tool_result.changed_paths` 按 turn 累积、按路径去重，标注来源工具中文名；新消息发送时清空；按 session 内存保留（切换会话可恢复，与工具调用链一致）；刷新/重启后不持久化
- **打开与定位**：每项支持「打开」（默认程序）与「定位」（系统文件管理器）；目录与文件统一支持
- **中间产物过滤**：后端 `extract_changed_paths` 过滤 `.cache/` 路径，产物列表不展示 OOXML 解包等中间目录
- **手动压缩不清空产物**：`/compact` 使用独立 `busy_compact` 路径，不丢失当前 turn 产物列表
- **Non-Goals（留待后续）**：diff 预览、撤销/回滚、历史会话产物持久化

### PPT OOXML 精准修改与手动上下文压缩

- **斜杠 `ppt:edit-ooxml`**：OOXML 解包 → 编辑 `ppt/slides/slide{N}.xml` → 回包；`ppt:edit` 明确为 PptxGenJS 脚本路径
- **`/compact` 命令**：手动触发上下文压缩（`compact_session` IPC），不写 `/compact` 用户消息；与自动压缩共享摘要管线
- **压缩 UX**：进行中提示（自动「正在压缩较早的对话历史…」/ 手动「请稍候…」）；完成后按 `trigger` 区分自动/手动文案；进行中提示不自动 5 秒消失
- **后端**：`compaction_started` / `context_compacted(trigger)` 事件；手动压缩占用全局 run slot；UTF-8 安全截断修复摘要输入 panic

### 项目级 AGENTS.md 与 `/init`

- **每 turn 自动注入**：项目根存在非空 `AGENTS.md` 时，system 提示词追加 `## 项目配置（AGENTS.md）` 段（≤3000 字符）；手写或外部编辑后下一轮即生效，无需重新初始化
- **`/init` 斜杠命令**：真发送用户消息并占一轮 turn；通过内置 `profile` skill + clarify 生成或更新 `AGENTS.md`；可选尾部说明（如 `/init 固化PPT风格`）
- **`confirm_agents_md` 澄清题型**：init 流程中展示拟写入的 Markdown 全文预览，确认后再 `fs_write`
- **写入门禁**：非 init turn 禁止 Agent 写 `AGENTS.md`；init turn 须先通过 `confirm_agents_md` 确认
- **UI 指示**：Chat 区显示项目 `AGENTS.md` 状态（缺失 / 已加载）；clarify pending 时禁止 `/init`
- **`fs_read AGENTS.md`**：文件不存在时返回 `exists: false`（非错误），便于 Agent 判断

### 需求澄清（clarify）

- **与 AGENTS.md 协作**：clarify skill 先读项目配置，跳过已在 `AGENTS.md` 中写明的样式/规范，只问本次任务仍缺的信息
- **同轮多问降级**：同一 assistant 轮次内第二个及以后的 `clarify_ask` 返回 `deferred` 提示（非红色失败），引导下一轮单独提问
- **resume 失败不再假死**：澄清答案已提交但 LLM 调用失败（如余额不足、API 繁忙）时，不再错误恢复已消费的澄清卡片；输入框与停止按钮可正常使用

### 工具链与并行文件占用

- **file_busy 友好提示**：工具链面板识别文件占用错误，展示简短中文说明（含占用会话信息）
- **工具错误默认折叠**：工具调用失败时「错误详情」默认折叠，减少技术 JSON 刷屏

### 维护与规范（开发者）

- **project-backlog**：OpenSpec 维护待办与优先级（BL-006 等项目级配置已标记完成）
- **AGENTS.md / Codex skills**：仓库贡献约定与 OpenSpec 工作流 skill 镜像至 `.codex/skills/`

### Chat 输入区焦点策略

- **回合结束 refocus**：Agent 回合结束、composer 从 disabled 恢复可编辑时，自动聚焦 textarea，无需手动点击即可继续输入（不再因焦点停在侧栏而跳过）
- **切换会话 refocus**：侧栏切换或新建会话后自动聚焦输入框
- **Overlay 抑制**：Settings/Credentials 抽屉、图片预览、斜杠/@ 弹层、Model Flyout、更新遮罩打开，或未选项目、composer 不可编辑时，不自动抢焦点
- **IME 守卫**：composer 内使用输入法组合输入时，按 Enter 确认候选词不再误触发发送（`isComposing` / `keyCode === 229` 时不拦截按键）

---

## [2026.6.19] — 2026-06-19

本版本带来有界并行与文件治理、OOXML pack 门禁结构校验，并让 `skill_run` 成功脚本跨 turn 保留以便续改交付物。

### skill_run 脚本跨 turn 保留

- **保留成功脚本**：`skill_run` 成功执行后保留 `.cache/skill-run/<session_key>/script.js`，下一轮可直接 `fs_read` / `fs_patch` 续改（如修改已生成的 PPT），无需重写全量脚本
- **失败现场保留**：执行失败保留 `script.js` 并写 `error.json`；成功（含 path 重跑修复成功）后清除 `error.json`
- **清理时机**：session scratch 目录仅在用户 **cancel turn** 时删除；turn 正常结束或达到 max tool steps 不再清理

### 有界并行与文件治理

- **全局并行**：应用内最多 3 个 running turn（跨 project/session）；第 4 个 `send_message` / `resume_turn` 在写入消息前拒绝；clarify 等待不占名额
- **同 project 并行**：移除「同项目单 turn」互斥；多会话可同时处理不同文件，写同一路径时后者 `file_busy`
- **文件锁**：`FileLockRegistry` + `ToolIoPlan`；工具执行前申请 Read/Write/SubtreeWrite；`skill_run` runtime 动态写兜底锁
- **缓存路径**：`skill_run` scratch 迁至 `.cache/skill-run/<session_key>/`（同会话跨 turn 路径不变）；`ooxml_unpack` 省略 `out_dir` 时自动生成 `.cache/ooxml/<session_key>/<work_key>/`
- **前端**：全局 3 并行满额提示、文件占用错误展示、后台 session 完成时刷新侧栏/文件区而不覆盖当前消息
- **Skills / prompt**：禁止固定 `unpacked/` 与手写 `.cache/` 路径；必须使用工具返回的 `out_dir` / `script_path`

### OOXML 结构校验（pack 门禁）

- **`ooxml_pack` 结构规则**：解包目录回包前新增 well-formed + 参照 bundled XSD 的结构规则校验（零 native / 无 XSD 引擎）；覆盖 OPC（`opc.ct.*`、`opc.rels.*`、`pkg.rels.01`）、Word（`wml.*`）、PowerPoint（`pml.*`）、Excel（`sml.*`）
- **错误格式**：`{part}:{line} [{rule_id} {xsd_ref}] {message}`，便于 Agent 按规则 ID 自修 XML
- **实现**：`validate.rs` 拆为 `validate/` 模块（well-formed、rules、roundtrip）；352 项 Rust 测试全绿

---

## [2026.6.18] — 2026-06-18

本版本新增 Agent turn 停止与同项目互斥、按会话运行态可视化，完善 skill_run 运行时文档与 Native API，并优化更新遮罩与右侧面板折叠体验。

### Turn 停止与会话运行态

- **停止按钮**：Agent 执行中可在输入区点击「停止」；进入 stopping 状态后等待当前工具结束（最长约 35 秒），随后 emit `turn_cancelled` 并对未完成 tool call 补写 cancelled result
- **按会话运行态**：前端按 session 维护 `idle` / `running` / `stopping` 与流式缓冲；切换会话不再清除后台 session 的 running 进度
- **侧栏指示**：running / stopping 会话在侧栏显示 spinner 指示，可点击切换查看后台任务
- **同项目互斥**：同一项目内最多一个 session 处于 running；其他 session 发送或 clarify resume 时被拒绝并提示正在运行的会话标题
- **SSE / 压缩可取消**：流式 LLM 与上下文压缩摘要请求监听 cancel 信号，stop 后不再追加 token
- **后端**：新增 `TurnRegistry`、`cancel_turn` IPC、`turn_cancelled` 事件；clarify 暂停时 unregister，不算 running

### skill_run 运行时

- **runtime 文档**：新增内置 `skill_read {"skill":"runtime"}` 能力矩阵（引擎、normalize、API 表、polyfill、限制与示例）；system prompt 与 skill 索引要求编写/修复 skill_run 前先读 runtime
- **Native API**：新增 `doc_exists` / `doc_list` 与 `fs.existsSync` / `fs.readdirSync`（沙箱校验；list 复用项目目录单层语义，支持 `unpacked/...` 列 slide 文件）
- **import normalize**：常见 `import … from 'pptxgenjs'|'docx'|'exceljs'|'pdf-lib'`（含 `import * as` 与无分号写法）执行前改写成全局/require 等价
- **Bundle 收紧**：移除裸子串 `pptx` 触发 pptxgenjs；仅含 `"output.pptx"` 等路径字符串的 OOXML fs 脚本不再误加载 ~374KB bundle
- **诊断**：脚本错误 hint 指向 `skill_read runtime` 与白名单；docx/pptx/xlsx/pdf SKILL 交叉引用 runtime 文档
- **Spec 修正**：`script-runtime` 归档合并为 boa_engine + JavaScript only + 全局/require 语义

### 界面

- **更新遮罩**：下载进度圆环与安装文案优化；启动静默更新与设置手动更新路径一致
- **弹层定位**：`@` / `/` 弹层与斜杠 flyout 锚点定位修正，避免溢出视口
- **更新临时文件清理**：应用启动时清理 updater 遗留临时目录
- **右侧面板**：工具链与文件区折叠互斥；拖拽展开时自动取消另一区折叠

---

## [2026.6.17] — 2026-06-17

本版本新增聊天输入工具栏、斜杠命令任务模板、@ 文件引用体验优化、可拖拽调整的工作区布局、LLM 会话自动标题，以及密钥/模型配置 UX 重构。

### Typst 中文段落缩进

- **默认无首行缩进**：`apply-zh-body` 不再全局设置 `first-line-indent`，Agent 生成的 PDF 标题与正文左缘一致，避免偶发缩进混乱
- **主题可选恢复**：`make-theme(cjk-paragraph-indent: true)` 一行开启传统两字首行缩进
- **手册规范**：语法手册新增段落/标题 Agent 规范（必须用 `=` 写标题、禁止伪标题与滥用 `#pad`）
- **模板修补**：paper 中英文模板参考文献去掉硬编码 `#pad(left: …)`

### 聊天输入工具栏

- **三按钮**：输入框底栏新增 **+**（上传文件到项目根）、**图片**（选择图片附件）、**/**（任务模板图形菜单）；clarify / busy / initializing 时与输入框一并禁用
- **文件导入**：多选本地文件写入项目根（100MB 上限）；重名时逐文件询问覆盖 / 另存为（自动 `文件名 (n).ext` 递增）/ 取消；导入期间锁定输入区，完成后在光标处插入 `@路径` 并刷新索引
- **图片按钮**：`accept` 过滤 PNG/JPEG/WebP/GIF，逻辑等同粘贴（写入 `.cache/attachments/`，不进项目目录与 `@` 列表）；非 vision 模型时按钮禁用
- **斜杠图形菜单**：六类二级 flyout（general / word / ppt / excel / pdf / web），选中填入 prompt 并选中首个 `{{占位符}}`，不自动发送；键盘 `/` fuzzy 弹层保持不变
- **文件浏览**：项目文件区标题栏新增「在文件管理器中打开」项目根目录；图标与侧栏统一为 SVG
- **体验修复**：附件缩略图预览缓存，避免弹层/流式输出时历史图片闪烁；`/` 按钮支持 toggle 开关菜单

### 密钥与模型配置

- **Header 双入口**：顶栏新增「密钥」按钮（与「设置」并列），打开「密钥与服务」Drawer，集中配置 DeepSeek / Kimi / MiMo / Tavily API Key；启动即可配置，不依赖是否已选项目
- **模型 Flyout**：侧栏左下模型 trigger 改为锚定 Flyout（优先向上展开），含 Provider segmented、可滚动模型列表与 sticky 思考区；移除原右侧全高「模型与密钥」Drawer
- **零 Key 弱提醒**：未配置任一 LLM Key 时每次启动显示 Header 弱提醒条与密钥按钮 amber dot；关闭当次提醒后下次启动仍显示
- **发送拦截**：缺 Key 发送时打开密钥 Drawer 并高亮对应 Provider，不再打开模型 Flyout
- **Web 搜索开关**：侧栏改为单行开关 + 状态摘要；`web_search_enabled` 偏好与 Tavily Key 分离（关开关不清 Key）；保存 Tavily Key 后自动开启搜索

### 会话自动标题

- **两轮策略**：第 1 轮写入清洗后的首条 user 文本（最长 120 字入库）；第 2 轮用当前会话模型、非思考模式异步 LLM 总结前两轮对话并覆盖标题（每会话仅一次）；第 3 轮及以后不再自动改名
- **历史会话**：升级时对已有 ≥2 条 user 消息的会话标记 `autotitle_llm_done`，避免误触发 LLM 补跑
- **侧栏展示**：完整标题入库，CSS ellipsis 按侧栏宽度截断，hover tooltip 显示全文
- **事件**：LLM 标题完成后 emit `session_title_updated`，前端刷新会话列表

### 工作区布局

- **三栏可拖拽**：左侧栏、会话区、右侧栏支持水平拖拽调整宽度，默认比例 20% / 60% / 20%
- **右侧上下分栏**：工具调用链与项目文件支持垂直拖拽，默认 60% / 40%；标题栏可折叠 / 展开
- **布局持久化**：分栏比例写入 `localStorage`，重启后恢复；损坏缓存自动清除并回退默认
- **恢复默认布局**：设置抽屉提供「恢复默认布局」，清除已保存的面板布局
- **分割条**：默认几乎不可见，hover / 拖拽时高亮；禁用双击重置

### 斜杠命令

- **`/` 任务模板**：内置 22 条静态命令，覆盖 general / Word / PPT / Excel / PDF / Web；命令 id 使用 `category:action`（如 `word:edit`），general 类无前缀（如 `read`）
- **分组弹层**：fzf 模糊搜索、分类分组、↑↓/Enter/Tab 选择；选中后填入 20–100 字 prompt 模板（`{{占位符}}`），自动选中首个占位符，不自动发送
- **与 @ 互斥**：mention 优先；澄清 / busy / initializing 时不展示斜杠弹层
- **Esc / 空匹配**：Esc 关闭弹层；无匹配时 Enter 不发送
- **键盘滚动**：↑↓ 切换分组时以分组标题为锚点滚动，避免「通用」等标题被卷出可视区

### @ 文件引用

- **分层浏览**：空 `@` 仅根目录；`@docs/` 进入子目录；全局搜索按父目录分组，主行文件名 + 灰色路径
- **键盘**：Tab 对目录进入子级，Enter 确认引用；Esc 仅关闭弹层，不删除 `@` 及已输入内容
- **索引**：后端返回 `modified_ms` 与 `is_dir`，按修改时间降序；目录内筛选按文件名匹配
- **路径引号**：含空格或解析终止符（如括号、中文标点）的路径自动加引号

### 后端

- **`list_project_files_cmd`**：条目含 `path`、`is_dir`、`modified_ms`，供 @ 弹层排序与展示

---

## [2026.6.16] — 2026-06-16

本版本改进应用更新下载反馈、Typst 编译诊断与模板体系，并优化工具调用链滚动体验。

### 应用更新

- **下载进度遮罩**：用户确认更新后展示全局 `UpdateProgressOverlay`；下载阶段圆环进度（有 `contentLength` 时显示百分比，否则旋转指示 +「正在下载…」）；安装阶段文案「正在安装，即将重启…」
- **双路径覆盖**：启动静默检查与设置抽屉手动更新均接入 `downloadAndInstall(onEvent)` 进度回调
- **失败处理**：沿用现有 error dialog，遮罩自动关闭

### Typst PDF 导出

- **结构化编译诊断**：`typst_to_pdf` 失败时返回 `error_type`、文件路径、行列、`snippet`、`message`、`hints` 与 `fix_guidance`；由 Typst `Span` 还原可读位置，引导 Agent 优先 `fs_patch` 局部修改
- **编译警告回传**：成功与失败均将 warnings（字体回退、弃用语法等）随工具结果返回 Agent，不再仅输出到 stderr
- **设计 token 体系**：新增 `common/tokens.typ`（字号/间距/行距/页边距/字体角色）；`make-theme(...)` 支持受控主题覆盖
- **模板重构**：`common/{fonts,page,exam,lecture}.typ` 与 8 个场景模板统一消费 token，消除硬编码魔数，零警告编译
- **手册校验**：新增测试校验 `typst-guide.md` 可编译示例与 `common/*.typ` 公开 API 一致性；同步修订手册

### 界面

- **工具调用链贴底滚动**：右侧工具调用链在新工具追加时自动滚至最新项；用户手动上滑查看历史工具时暂停自动滚动

---

## [2026.6.15] — 2026-06-15

本版本新增 Typst PDF 离线导出、PDF 智能读取与多模态能力，统一项目缓存目录，并改进 Agent 工具链稳定性。

### Typst PDF 导出

- **typst_to_pdf**：嵌入 `typst-as-lib` 离线编译沙箱内 `.typ`（或含 `main.typ` 的目录）为 PDF；60 秒超时，临时文件 + staging 写入，超时或不成功不覆盖最终 `out_path`
- **typst_list_templates / typst_read_template**：列出并读取内置资源——通用语法手册 `syntax/typst-guide`、report/exam/paper/lecture 各中英场景模板（共 9 项）
- **内置模板**：`common/fonts.typ`（中英字体栈）、`common/page.typ`、`common/exam.typ`（`calc-item` 自动递增计算题号）；虚拟路径 `#import "/doc-agent/typst/..."`
- **中文字体**：构建时捆绑 Noto Sans/Serif SC（Subset Regular + Bold，约 40 MB）；优先平台系统字体（Windows 宋体/雅黑，macOS 宋体/黑体），无系统字时回退 Noto，中文模板编译无 `unknown font family` 警告
- **Agent 约束**：同一会话首次使用 Typst 能力前 MUST 读取 `syntax/typst-guide`；系统提示与工具描述已同步
- **与 html_to_pdf 分工**：公式密集、版式严谨文档优先 Typst；图文 HTML 报告仍用 `html_to_pdf`

### CI

- **Noto 字体缓存**：PR / Release workflow 缓存 `src-tauri/fonts/`，减少重复下载（`build.rs` 变更时自动失效）

### 设置与账户

- **账户余额**：设置抽屉（版本信息下）展示已配置 API Key 的 DeepSeek、Kimi 人民币总可用余额；打开抽屉时查询，加载中 `…`、失败 `—`；MiMo 暂无官方余额接口不展示

### 项目缓存目录

- **BREAKING**：统一项目沙箱缓存至 `.cache/` 单根目录
  - 用户粘贴图片：`.uploads/` → `.cache/attachments/`
  - `skill_run` 临时脚本：`.skill-run/` → `.cache/skill-run/`
  - PDF 渲染缓存：仍为 `.cache/pdf/`（不变）
- 新增 `core/cache_paths` 集中路径常量；不迁移或读写旧 `.uploads/` / `.skill-run/` 目录
- 附件文件缺失时 UI 显示「无法加载」、Agent 重建上下文静默跳过（既有行为，见 spec）

### PDF 智能读取

- **pdf_read**：统一 PDF 理解入口，仅传 `path`（可选 `pages`、`dpi`）；先按页 PDFium 提取，vision 模型经硬规则与代表页图文 Judge 决定返回文本或全量 vision；返回 `resolved`（`text` | `vision`）与 `judge` 元数据（样本页、verdict、method 等）
- **pdf_render_pages**：将 PDF 指定页渲染为 PNG，写入 `.cache/pdf/<cache_key>/`；源文件与参数未变时 `cache_hit: true` 跳过重复渲染
- **vision 分批**：全量 vision 每批最多 4 页，与 `image_read` 多图上限一致；共享 `vision_subcall` helper，子调用 usage 不计入会话 token
- **image_read（BREAKING）**：参数改为 `paths`（1–4 张），移除单张 `path`；可读 `.cache/pdf/` 页图
- **工具分工**：一般读 PDF 用 `pdf_read`；仅需 PDFium 快速纯文本时用 `office_read_to_markdown`；pdf skill 文档已同步

### Agent 稳定性

- **tool call id 规范化**：流式 tool call 预生成 `call_{uuid}`；持久化前规范化空 id、批内重复及与 DB 冲突，修复 Kimi 等 Provider 返回空/重复 id 导致的 `UNIQUE constraint` 错误
- **同轮 pdf_read 并行**：单轮最多 3 个 `pdf_read` 并发执行，tool result 按原序写入；其他工具仍串行
- **工具链 UI**：`ToolCall` 事件携带 `index`；执行前 broadcast 全部 `running`，streaming 占位按 index 就地升级，避免多 `pdf_read` 时工具栏闪空

### 多模态与模型

- **图片输入**：vision 模型（Kimi K2.6、MiMo v2.5）支持粘贴图片发送；可仅发图无文字；附件写入项目 `.cache/attachments/` 并持久化，历史消息展示缩略图
- **image_read 工具**：vision 模型可调用 `image_read` 读取项目内图片并返回文本描述（MiMo / DeepSeek 非 vision 模型不暴露该工具）
- **MiMo Provider**：新增小米 MiMo v2.5、MiMo v2.5 Pro、MiMo v2.5 Pro Ultraspeed（1M 上下文）；侧栏「模型与密钥」Drawer 统一配置三 Provider 的 API Key
- **模型目录 IPC**：`list_models` 暴露 vision / effort / 上下文上限；新建会话默认沿用上次模型配置

### 界面

- **模型与密钥 Drawer**：模型选择、思考配置、API Key 从侧栏迁入右侧 Drawer；侧栏仅保留摘要与视觉能力标识
- **附件预览**：输入区 chip 与消息缩略图支持点击放大；IPC 读取附件避免裂图
- **上下文占用**：切换会话、空会话、历史会话均显示占用比例（无历史为 0%）；流式 `context_usage` 实时更新
- **非 vision 粘贴提示**：DeepSeek / MiMo Pro 等模型粘贴图片时 toast 引导切换 vision 模型

### 上下文与压缩

- **多模态压缩策略**：pending 估算与压缩摘要仅统计文本，不展开图片 base64；保留 tail 的 `attachments_json` 原样；`image_read` 子调用 usage 不计入会话 token

---

## [2026.6.14] — 2026-06-14

首个 CalVer 正式版本（自本版起 tag 与安装包版本号为 `YYYY.M.D`）。相较 `1.0.1`，本版本新增会话上下文自动压缩，并改进侧栏会话排序体验。

### 上下文管理与压缩

- **自动上下文压缩**：接近模型上限时自动摘要较早历史并归档，保留最近轮次；对齐 kimi/deepy 双触发（比例 85% + 预留空间）与 tool-call 配对保护
- **Token 用量采集**：流式响应解析 `usage`（`stream_options.include_usage`），结合 pending 字符估算驱动压缩判定；各模型暴露上下文上限（DeepSeek 1M、Kimi 256K、Mock 100K 便于测试）
- **持久化与重建**：压缩摘要以 `user` 角色写入 DB，旧消息标记 `archived`；后续 turn 仅从「摘要 + 未归档消息」重建上下文
- **循环内触发**：工具循环每一步开头检查压缩（非仅 turn 开始），覆盖单轮内大工具输出累加场景
- **工具步数上限**：单 turn 最大工具循环步数由 32 提升至 64
- **压缩失败兜底**：摘要 LLM 失败时截断最旧非保留消息，避免 turn 因超限彻底卡死

### 界面

- **上下文占用指示**：会话标题栏展示图标 + 占用比例（≥70% 琥珀、≥85% 红色）；压缩完成后一次性轻提示
- **压缩后同步**：收到 `context_compacted` 后立即刷新消息列表，已归档内容不再显示；压缩摘要不在聊天气泡中展示
- **侧栏会话拖动排序**：支持 drag handle 拖动重排；**懒激活**——未拖动前仍按 `updated_at` 自动排序，首次拖动后按项目写入 `localStorage` 持久化手动序

### 发布

- **Windows 安装包**：仅 NSIS（`*-setup.exe`）；移除 MSI——CalVer `YYYY.M.D` 的 major 超过 WiX 255 上限，MSI 打包会失败

### 版本与发布

- **CalVer 版本号**：应用版本由 `1.0.1` 切换为 `2026.6.14`（`YYYY.M.D`，无前导零）
- **历史版本保留**：`1.0.0` / `1.0.1` 的 tag 与 OSS 路径不变；自本版起 Release tag 与 `latest.json` 使用 CalVer

---

## [1.0.1] — 2026-06-14

### 自动更新验证

- **发版验证**：用于验证 1.0.0 → 1.0.1 应用内自动更新闭环

### 界面

- **设置抽屉**：「检查更新」移至顶栏设置抽屉，展示当前/最新版本；打开抽屉时查询 OSS `latest.json`
- **版本查询修复**：通过 Tauri 后端拉取 manifest，避免 dev/生产环境 CORS 导致拿不到最新版本

---

## [1.0.0] — 2026-06-14

### 自动更新与分发

- **应用内自动更新**：启动时检查新版本，侧栏提供「检查更新」；确认后下载安装并重启（自 1.0.0 起生效，此前版本需手动安装本版基线包）
- **国内下载加速**：Release 产物同步上传阿里云 OSS（广州），`latest.json` 供 updater 使用；GitHub Releases 保留为备用渠道

---

## [0.2.0] — 2026-06-12

相较 0.1.0，本版本聚焦工作区体验、文档生成质量、数据分析与需求澄清，共 9 项功能交付。

### 工作区与界面

- **右侧文件浏览区**：在项目目录内浏览、打开文件；支持 Finder 风格返回行与 `⌂` 面包屑导航（不再显示难懂的 `..`）
- **明暗主题切换**：顶栏一键切换深色 / 浅色（Notion 风格浅色模式），偏好写入 `localStorage` 重启后保留
- **会话与侧栏优化**：懒创建会话、发送提示、侧栏控件重组；会话标题由模型自动生成，失败时二次重试
- **文件变更同步**：Agent 写入 / 修改文件后，文件浏览区与 `@` 文件选择器即时刷新（基于 `changed_paths`，无需轮询）
- **品牌与安装**：自定义 Logo / 图标；Windows 安装路径调整为 `DocAgent`（窗口标题仍为 Doc Agent）

### 文档与 Agent 能力

- **需求澄清（clarify）**：新增 `clarify` skill 与 `clarify_ask` 工具；模糊需求时 Agent 可暂停当前轮次、以结构化单选 / 多选 / 文本收集用户输入，确认「创作简报」后继续生成或编辑
- **HTML 报告**：新增 `html-report` skill；可在项目目录生成静态 HTML 报告（表格、样式、打印 CSS）；可选 `html_to_pdf` 导出 PDF（macOS / Windows）
- **Word 生成质量**：移除低质量捷径 `word_create`；统一走 `skill_read(docx)` + `skill_run` + docx-js；补充中文排版与样式指引；新增 docx 样式 lint
- **skill_run 容错**：失败脚本保留在 `.skill-run/`；新增 `fs_patch` 局部修复；错误定位到行列号；本轮结束自动清理临时脚本
- **旧版 Office**：`office_convert` 将 `.doc` / `.xls` / `.ppt` 转为现代格式；`.xls` 可直接 `data_query` 分析，无需先另存

### Excel 与数据分析

- **不规则表格预处理**：`excel_describe` 侦察合并单元格、表头行与结构警告；`excel_normalize` 清洗为规整 CSV
- **data_query 增强**：列名归一化与错误提示优化；不规则 Excel 建议先 describe → normalize 再查询
- **工具链中文标签**：前端工具调用展示补全中文名称

### 联网与 PDF

- **联网搜索（可选）**：配置 Tavily API Key 后可使用 `web_search` / `web_extract` 检索与提取网页内容

### 修复

- **流式消息展示**：多轮工具调用时，每一步 assistant 回复独立展示，不再混在同一流式框中

### 破坏性变更

- **`word_create` 已移除**：由 Markdown 一步生成 Word 的路径已删除；请使用 `skill_read` + `skill_run` 生成 `.docx`。历史会话中的调用记录仍可查看，新会话不再提供该工具。

---

## [0.1.0] — 2026-06-10

首个公开发布版本。

- 以项目文件夹为边界的多会话本地 AI 助手（Tauri 2 · Rust · React）
- Word / Excel / PPT / PDF 阅读、生成与 OOXML 编辑工具链
- 内置 docx / pdf / pptx / xlsx Document Skills 与 `skill_run` 运行时
- DeepSeek V4 Flash / Pro、Kimi K2.6；思考模式与流式 Markdown
- `@` 引用文件、智能推荐问（需 DeepSeek Key）
- Windows（x86_64）与 macOS（Apple Silicon）安装包
