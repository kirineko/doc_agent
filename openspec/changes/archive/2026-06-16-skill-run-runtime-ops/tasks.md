## 1. Spec 与 Agent 可发现性（P0）

- [x] 1.1 新增 `assets/skills/runtime/SKILL.md`（能力矩阵：引擎、normalize、API 表、polyfill、限制、示例、故障修复）
- [x] 1.2 `core/skills.rs`：注册 `runtime` skill；`index_markdown()` 含 runtime 条目
- [x] 1.3 `agent/loop_support.rs` system prompt：skill_run 前先 `skill_read runtime`
- [x] 1.4 `tools/skill.rs`：`skill_read` / `skill_run` 工具描述更新
- [x] 1.5 确认 change delta spec 覆盖 Purpose/引擎/import 语义（归档前合并主 spec）

## 2. import normalize（P0）

- [x] 2.1 `normalize.rs`：改写常见 `import … from 'pptxgenjs'|'docx'|'exceljs'|'pdf-lib'` 模式
- [x] 2.2 `normalize.rs` 单测：各 import 模式 + 无法识别时保留原样
- [x] 2.3 集成测试：含 `import PptxGenJS from 'pptxgenjs'` 的 skill_run 可成功或给出 runtime hint

## 3. Native op + fs polyfill（P1）

- [x] 3.1 `ops.rs`：`__doc_exists`、`__doc_list`（复用 `list_project_dir`）
- [x] 3.2 `mod.rs` HELPERS：`doc_exists`、`doc_list`；`fs.existsSync`、`fs.readdirSync`
- [x] 3.3 单测/集成测试：exists/list/越界；`doc_list("unpacked/...")` 可列 slide 文件

## 4. Bundle 启发式（P1）

- [x] 4.1 `bundles_for_code`：移除裸 `pptx`；实现 `needs_pptxgenjs` 库用法检测
- [x] 4.2 测试：仅 `"output.pptx"` 的 fs 脚本不加载 pptxgenjs；含 `new PptxGenJS()` 仍加载

## 5. 诊断与 SKILL 交叉引用（P2）

- [x] 5.1 `with_runtime_hint` / `build_script_error`：模块/import/fetch 错误指向 runtime 文档与白名单
- [x] 5.2 docx/pptx/xlsx/pdf SKILL.md：skill_run 段落增加「API 见 skill_read runtime」（最小 diff）

## 6. 验证

- [x] 6.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [x] 6.2 `openspec validate skill-run-runtime-ops --strict`
