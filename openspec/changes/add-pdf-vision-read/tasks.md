## 1. PDF 渲染与缓存

- [x] 1.1 在 `tools/pdf.rs` 实现 `render_pages`（PDFium → PNG、dpi、pages 范围解析）
- [x] 1.2 实现 `cache_key` 计算（`DefaultHasher` + source 元数据 + dpi + pages_spec）
- [x] 1.3 实现 `.cache/pdf/<cache_key>/` manifest 读写与命中检测
- [x] 1.4 新增 `pdf_render_pages` 工具 spec + handler + 注册
- [x] 1.5 单元测试：首次渲染、缓存命中、源文件变更失效、页文件缺失强制重渲

## 2. vision 子调用共享层

- [x] 2.1 抽取 `vision_subcall(paths, prompt, model_id)` helper（多图编码、子调用、usage 隔离）
- [x] 2.2 **BREAKING** 重构 `image_read`：仅 `paths`（1–4），调用 helper
- [x] 2.3 更新 `toolLabels`、tool 描述与 `image_read` 测试

## 3. pdf_read

- [x] 3.1 新增 `tools/pdf_read.rs`：mode 门控、text 分支（委托 `pdf::extract_text`）、vision 分支
- [x] 3.2 vision 分支：渲染（含缓存）→ `chunks(4)` → helper 分批 → 合并 Markdown
- [x] 3.3 非 vision auto 返回文本 / 显式 vision 报错；扫描件 text 空错误文案
- [x] 3.4 注册 `pdf_read`；`registry.rs` 与 `tools_for_model` 测试

## 4. 文档与 skill

- [x] 4.1 更新 `assets/skills/pdf/SKILL.md` 与 `reference.md`（pdf_read、渲染缓存、双路径）
- [x] 4.2 更新 `office_read_to_markdown` 工具描述（PDF 与 pdf_read 分工，行为不变）

## 5. 集成测试

- [x] 5.1 使用 `reference/pdf/` 样例：text 提取基线、vision 路径 smoke（Mock 或可选 real key）
- [x] 5.2 缓存命中：同 PDF 连续两次 `pdf_render_pages`，第二次 `cache_hit: true`
- [x] 5.3 4 页 PDF 单批 vision（选择题样例页数=4）
- [x] 5.4 非 vision 会话 `pdf_read` 无 mode 走 auto 返回文本；`mode=vision` 拒绝

## 6. 验收（手动）

- [ ] 6.1 Kimi K2.6：`pdf_read` 默认 auto 解析 `reference/pdf` 样例，vision 结果优于纯 text
- [ ] 6.2 DeepSeek：`pdf_read` 无 mode（auto）返回文本；`mode=text` 可用；`mode=vision` 报错
- [ ] 6.3 重复解析同一 PDF，第二次明显跳过渲染（工具返回 `cache_hit: true`）
- [ ] 6.4 `image_read` 使用 `paths` 一次读 2–4 张 cache 页图
