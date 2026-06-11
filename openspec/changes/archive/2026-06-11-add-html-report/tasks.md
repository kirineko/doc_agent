# 实施任务：HTML 报告与导出

每组附说明；实现与 `design.md` / specs 冲突时先更新 artifact，再改代码。

## 1. Spike 与依赖

- [x] 1.1 Spike：Tauri 2 隐藏 `WebviewWindow` + 加载沙箱 `file://` HTML + macOS 打印 PDF 最小可行路径（记录所需 `Cargo.toml` feature）
- [x] 1.2 Spike：Windows WebView2 `PrintToPdf`（或等价 API）可行性确认；失败则在 design.md 记录降级方案后再实施

## 2. html-report skill

- [x] 2.1 编写 `src-tauri/assets/skills/html-report/SKILL.md`：目录约定（`reports/<name>/`）、`fs_write` 工作流、打印 CSS 模板、禁止 `.skill-run`/框架/build、与 `data_query`/`html_to_pdf` 可选衔接、系统浏览器预览说明
- [x] 2.2 `core/skills.rs` 注册 `html-report`（`SKILLS` 数组 + `include_str!`）
- [x] 2.3 `tools/skill.rs`：`skill_read` 校验列表加入 `html-report`；错误信息枚举包含该项
- [x] 2.4 Agent system prompt 组装处：skill 索引与强制 `skill_read` 文案覆盖 HTML 报告（见 `document-skills` spec delta）

## 3. html_to_pdf 工具实现

- [x] 3.1 新建 `src-tauri/src/tools/html_export/`（`mod.rs` + `print.rs`）：路径解析（`.html` 或目录 `index.html`）、沙箱校验、`out_path` 父目录创建
- [x] 3.2 实现 WebView 生命周期：创建隐藏窗口 → 加载本地 URL（base 为 HTML 所在目录）→ 等待 load → 打印 PDF → 销毁窗口；默认超时 30s
- [x] 3.3 支持参数 `page_size`（A4/Letter）、`landscape`、`margin_mm`；返回 `{ path, pages }`
- [x] 3.4 `tools/mod.rs` 挂载 `html_export`；`registry.rs` 注册 `html_to_pdf`（ToolSpec + stub handler 或占位）
- [x] 3.5 `loop_runner.rs` + `registry.execute`：为 `html_to_pdf` 增加 async 分支并传入 `AppHandle`（对齐 `web_search` 模式）
- [x] 3.6 `changed_paths.rs`：成功导出时记录 `out_path`
- [x] 3.7 前端 `toolLabels.ts` / `toolLabels.test.ts`：增加 `html_to_pdf` 标签

## 4. 测试与验收

- [x] 4.1 测试 fixture：用 `fs_write` 在项目内写入含 `styles.css` 引用的 sample HTML（表格+中文）
- [x] 4.2 `html_to_pdf` 冒烟：sample HTML fixture 覆盖目录入口与相对 CSS；真实 PDF 导出需桌面 WebView 手验（CI 覆盖错误分支与参数校验）
- [x] 4.3 错误分支：越界路径、目录无 `index.html`、不存在 HTML → 明确错误且不产生空 PDF
- [x] 4.4 `skill_read {"skill":"html-report"}` 返回全文；启动枚举含五项 skill
- [x] 4.5 `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test`；`npm run typecheck` + `npm test`（若改前端标签）
