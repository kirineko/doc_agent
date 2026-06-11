## 1. 会话自动标题（Rust）

- [x] 1.1 `session_title.rs`：`MAX_TITLE_CHARS` 改为 16；新增 `is_generic_opener`、`clean_intent`；`summarize_session_title` 改为 `Option<String>`（用户优先、过滤助手寒暄）
- [x] 1.2 `loop_runner.rs`：`maybe_autotitle_session` 支持 `user_count == 2` 重试窗口；第二轮仅传 `user_text`、不传助手文本；`None` 时不写库
- [x] 1.3 扩展 `session_title` 单元测试：泛化开场跳过、第二轮重试、16 字截断、用户优先于助手、第三轮不触发（通过 loop 或纯函数边界测试）

## 2. 项目文件导航（前端）

- [x] 2.1 `ProjectFileExplorer.tsx`：移除标题栏角落 `..` 按钮
- [x] 2.2 非根目录时列表首项渲染「返回上级」（Finder 风格，样式与文件项区分）
- [x] 2.3 路径行实现可点击面包屑：`⌂` 为根（`aria-label`/`title` = 项目根目录），中间段可跳转，当前段高亮；过长路径 truncate
- [x] 2.4 根目录仅显示 `⌂`，无返回列表项

## 3. 验证

- [x] 3.1 `cd src-tauri && cargo test`（含 session_title）
- [x] 3.2 `npm run typecheck && npm test`
- [x] 3.3 手动走通：「你好」→ 保持新会话 → 第二条实质提问 → 标题更新；文件浏览进入子目录 → 列表返回与面包屑 `⌂` 跳转
