## Context

- PDF 文本提取：`tools/pdf.rs` + `office_read_to_markdown`，扫描件返回空文本错误
- Vision：`image_read` 单次 1 图、vision 子调用、tool result 纯文本；用户附件上限 4 张/消息
- 项目内隐藏目录：`.uploads/`（用户图）、`.skill-run/`（skill_run 脚本）；**尚无** `.cache/`
- 文件索引：`should_skip_name` 跳过所有 `.` 开头目录，`.cache` 不出现在 `@` 与浏览区
- 样例：`reference/pdf/` 三份 2–4 页 PDF（gitignore 的 `reference/`，本地/CI 测试用）

## Goals / Non-Goals

**Goals:**

- `pdf_render_pages`：PDFium → PNG，默认 dpi=150，页码 1-based，可选 `pages` 范围
- `.cache/pdf/<cache_key>/` + `manifest.json`；源文件或渲染参数未变则**跳过重复渲染**
- `pdf_read`：默认 `mode=auto`（未传 mode 等价）；先 PDFium 文本，vision 模型再渲染+vision；`mode=text`/`mode=vision` 走显式单路径
- vision 路径：按 4 页分批，每批调用多图 vision（扩展后的 `image_read` / 共享 helper）
- `image_read` 统一为 `paths`（1–4），与附件上限一致

**Non-Goals:**

- 大学数学 skill、题目标注 schema、组卷、试卷排版
- OCR 专用引擎、PDF 内嵌图片提取
- vision 批结果 JSON 缓存（仅缓存渲染；prompt 变化时重跑 vision）
- 将 `.skill-run/` 迁入 `.cache/skill-run/`
- 用户可配 dpi / batch_size UI

## Decisions

### D1. 缓存目录 `.cache/pdf/<cache_key>/`

```
.cache/pdf/<cache_key>/
  manifest.json
  page_001.png
  page_002.png
```

`cache_key = hex16(DefaultHasher(source_rel_path | size | mtime_secs | dpi | pages_spec))`

命中条件：manifest 存在且字段匹配，且全部 `page_NNN.png` 存在。否则清空该 key 目录后重渲。

manifest 字段：`version`, `source_path`, `source_size`, `source_mtime_secs`, `dpi`, `pages_spec`, `page_count`, `pages[]`, `created_at`。

返回 `cache_hit: true|false` 供工具链展示。

### D2. `pdf_read` 模式门控

| 调用 | 行为 |
|------|------|
| 无 `mode` / `mode=auto` | 先 PDFium 文本 → 若 `supports_vision` 再渲染+分批 vision；否则有文本则返回文本 |
| `mode=text` | 仅 PDFium 文本（任意模型） |
| `mode=vision` | 仅渲染+vision（须 `supports_vision=true`） |

无 `mode` 与 `mode=auto` 行为 MUST 完全一致。auto 在 vision 模型上 MUST NOT 因文本已足够而跳过 vision。

文本为空（扫描件）时：`mode=text` 报错；`mode=auto` 在非 vision 模型上报错并提示切换 vision 模型；`mode=auto`/`mode=vision` 在 vision 模型上走 vision 路径。

### D3. Vision 分批：固定 4，复用多图 `image_read`

每批最多 4 页 PNG → 一次 vision 子调用（`paths` 长度 1–4）。prompt 模板说明页序与页码范围。

N 页 PDF 需 `ceil(N/4)` 次子调用；受 `MAX_TOOL_STEPS=64` 约束（实用上限约 256 页/turn，MVP 不单独限制）。

批结果以纯文本并入 `pdf_read` 返回值；可选将每批摘要写入 `.cache/pdf/<key>/vision_batch_NN.txt` 便于调试（非必须持久化到 DB）。

### D4. `image_read` API（BREAKING）

```json
{ "paths": ["a.png", "b.png"], "prompt": "..." }
```

- `paths` 必填，长度 1–4
- 每张图：`sandbox.resolve` + 大小 ≤50MB；**不限** `.uploads/`（可读 `.cache/pdf/`）
- 返回 `{ "text", "paths", "count" }`
- 无单张 `path` 参数（新工具无外部兼容负担）

### D5. 共享 `vision_subcall` helper

从 `image_read` 抽出：编码多图 → 构建 `ChatRequest` → 子调用 → 返回文本。`image_read` handler 与 `pdf_read` 内部分批均调用，避免重复逻辑。子调用 usage 仍不计入 `sessions.last_token_count`。

### D6. 工具注册

| 工具 | 注册 |
|------|------|
| `pdf_render_pages` | 始终 |
| `pdf_read` | 始终 |
| `image_read` | 仅 `supports_vision`（现有规则） |

Agent 可直接 `pdf_read`；高级场景可先 `pdf_render_pages` 再手动 `image_read`。

### D7. `office_read_to_markdown` 与 PDF

MVP：PDF 分支行为不变（纯文本），文档引导 Agent 对公式/扫描件优先 `pdf_read`。可选实现：PDF + 无 mode 时内部转 `pdf_read`——**本变更采用文档引导 + `pdf_read` 为主入口**，避免隐式 vision 成本。

## Risks / Trade-offs

- **[Risk] 渲染 + 多批 vision 成本高** → 渲染缓存命中；纯文本试探用 `office_read_to_markdown`，vision 模型读 PDF 默认只传 path
- **[Risk] vision 输出不稳定** → 返回纯 Markdown 文本 + 页码标记，不强制 JSON schema
- **[Risk] 缓存目录膨胀** → 按 `cache_key` 隔离；MVP 无自动清理（与 `.skill-run` 同理，点目录可手动删）
- **[Risk] dpi 过低公式糊** → 默认 150，manifest 含 dpi，调高参数自然失效旧缓存
- **[Trade-off] breaking `image_read`** → 同步改 tool 描述、测试、toolLabels；无外部 API 消费者

## Migration Plan

- 部署后：既有会话无迁移；`image_read` 调用方仅为 Agent，靠 tool 描述与 skill 文档更新
- 回滚：移除新工具注册即可；`.cache/pdf` 残留无害

## Open Questions

- （已关闭）缓存目录名 → `.cache/pdf`
- （已关闭）批大小 → 固定 4
- （已关闭）`image_read` → 仅 `paths`
