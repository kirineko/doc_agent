## Context

`skill_run` 在独立线程（32MB 栈）运行 boa_engine，入口 `execute_script` → HELPERS → 按需 bundle → `async function main()`。Native op 仅 read/write/log。`normalize.rs` 在 eval 前改写 `require` 与 main 调用。`list_project_dir` 提供 Sandbox 单层 listing，但脚本层不可见。

**调研关键数字**：四 bundle 合计 ~2.1MB（exceljs 874KB、docx 413KB、pptxgenjs 374KB、pdf-lib 529KB）。裸 `pptx` 启发式对 OOXML 工作流脚本误加载 pptxgenjs 的成本最高。

## Goals / Non-Goals

**Goals:**

- Agent 写 skill_run 前可读到**单一、准确**的运行时能力矩阵
- Spec、system prompt、工具 schema 与 boa 实现一致
- 补齐目录/存在性 op；常见 `import` 语句 compat 改写
- 消除 pptx 路径字符串导致的 pptxgenjs 误加载

**Non-Goals:**

- 完整 ES module / TypeScript / npm 生态
- docx bundle 启发式变更
- doc_list 隐藏 unpacked（与 @ 索引对齐）——会阻碍 `unpacked/ppt/slides` listing
- heap 限制、rustyscript 回切

## Decisions

### D1：`doc_list` 复用 `list_project_dir`

`__doc_list(path?)` → `list_project_dir(root, path.unwrap_or("."))`，返回 `{ path, entries: [{ name, is_dir }] }` 的 JSON 化结构（或仅 entries 数组，实现时选简洁形态并在 runtime 文档固定）。

忽略规则：**仅** `should_skip_name`（`.` 前缀、node_modules、target、`~$`），**不** skip `unpacked/` 目录名——Agent 需进入 OOXML 工作目录列 slide XML。

### D2：`doc_exists`

`sandbox.resolve(path)` + `exists()`；不存在 → `false`；越界 → 抛错（与 read 一致）。

### D3：fs shim 映射

```text
doc_exists / fs.existsSync
doc_list   / fs.readdirSync → names only
```

runtime 文档说明：需 `is_dir` 时用 `doc_list`。

### D4：runtime SKILL.md 结构

1. 引擎与入口（boa、async main、禁止末尾 main()）
2. **自动 normalize**（require 剥离、无 main 包裹、import 改写——见 D6）
3. 文件 API 表（doc_* + fs shim）
4. 库与 bundle 加载（全局名、require 白名单、何时自动加载）
5. Polyfill 表（setTimeout 无真实延迟、Buffer、TextEncoder…）
6. 限制（无 fetch/npm/shell/TS）
7. 故障修复（`.cache/skill-run/script.js` + fs_patch + path 重跑）
8. 最小可运行示例（docx / fs-only 各一）

注册 `runtime` skill；`index_markdown` 追加一行；system prompt 在 skill_run 相关句后加「先 skill_read runtime」。

### D5：pptxgenjs bundle 启发式（仅此项）

```rust
fn needs_pptxgenjs(lower: &str) -> bool {
    lower.contains("pptxgenjs")
        || lower.contains("pptxgenjs.")
        || lower.contains("new pptxgenjs")
        // PptxGenJS 全局用法（大小写已通过 lower）
        || lower.contains("pptxgenjs()")
}
```

**移除** `lower.contains("pptx")`。

**不测 docx 收紧**：`needs_docx` 保持 `lower.contains("docx")`——调研显示 fs-only 编辑脚本通常无此子串；docx-js 脚本几乎总有 `docx.` / `Document` 等。

### D6：import 兼容 normalize（非 ES module）

在 `normalize.rs` 增加对首行/单行模式的改写（与 `rewrite_require_line` 并列）：

| 输入模式 | 输出 |
|----------|------|
| `import PptxGenJS from 'pptxgenjs'` | （空行，依赖全局 PptxGenJS）或 `const PptxGenJS = PptxGenJS` 删除 |
| `import { Document, Packer } from 'docx'` | `const { Document, Packer } = docx;` |
| `import ExcelJS from 'exceljs'` | 删除（同 require exceljs） |
| `import … from 'pdf-lib'` | `const { … } = PDFLib;`（按 destructuring 保留） |

无法识别的 import 保留原样 → 解析失败 → hint 指向 runtime 文档。

**理由**：spec 与旧 Scenario 已教 import；比仅改 spec 更抗历史 prompt 惯性。

### D7：spec 修正

MODIFIED `skill_run 执行 JavaScript`、`内置文档生成库`；ADDED op / runtime 文档 / import normalize / pptx 启发式 Scenario；更新 Purpose 段。

### D8：诊断 hint 升级

`with_runtime_hint` / `build_script_error`：

- 模块错误 → 白名单模块列表 + `skill_read runtime`
- `is not defined` + `fetch`/`import` → 明确不支持

## Risks / Trade-offs

- **[import 改写覆盖不全]** → 覆盖 spec 与 Anthropic skill 常见四种库；其余靠 hint
- **[pptx 收紧漏加载]** → 脚本必须出现 `pptxgenjs` / `PptxGenJS` / `new pptxgenjs`；hint 提示显式引用
- **[readdirSync 无 is_dir]** → 文档 + doc_list
- **[runtime 文档变长]** → 控制 ≤150 行，表格化

## Migration Plan

- 无数据迁移；纯增 API + 文档 + normalize
- 现有成功脚本行为不变（除 pptx 误加载被消除）

## Open Questions

- 无阻塞项
