---
name: pptx
description: "Use this skill any time a .pptx file is involved in any way — as input, output, or both. This includes: creating slide decks, pitch decks, or presentations; reading, parsing, or extracting text from any .pptx file (even if the extracted content will be used elsewhere, like in an email or summary); editing, modifying, or updating existing presentations; combining or splitting slide files; working with templates, layouts, speaker notes, or comments. Trigger whenever the user mentions \"deck,\" \"slides,\" \"presentation,\" or references a .pptx filename, regardless of what they plan to do with the content afterward. If a .pptx file needs to be opened, created, or touched, use this skill."
license: Proprietary. LICENSE.txt has complete terms
---

# PPTX Skill

> 本系统无 shell/Python/Node 环境。所有操作通过内置工具完成：
> `office_read_to_markdown`、`ooxml_unpack`、`ooxml_pack`、`skill_run`（PptxGenJS 已内置）。**skill_run API 见 `skill_read {"skill":"runtime"}`。**

## Quick Reference

| Task | Tool / Guide |
|------|------|
| Read/analyze content | `office_read_to_markdown {"path": "presentation.pptx"}` |
| Raw XML access | `ooxml_unpack {"path": "presentation.pptx"}`（省略 `out_dir`，用返回路径） |
| Edit or create from template | `skill_read {"skill": "pptx", "doc": "editing.md"}` |
| **Create from scratch（最常用）** | `skill_read {"skill": "pptx", "doc": "pptxgenjs.md"}` → `skill_run` |

### 旧格式 `.ppt`

**默认：不转换。** 阅读、摘要、提取幻灯片文字 → `office_read_to_markdown {"path": "slides.ppt"}`（不新建文件）。

**仅在必要时** `office_convert` → `slides-converted.pptx`：用户要 `.pptx` 产物、或须 `ooxml_unpack` / 模板编辑。转换**可能丢失版式**。

### 不支持的操作

- **缩略图/渲染为图片**：无 LibreOffice；视觉校验降级为文本自检 + 结构检查（见 QA），必要时请用户用 PowerPoint/WPS 打开确认。

---

## Creating from Scratch

**Read [pptxgenjs.md](pptxgenjs.md) for full details.** Use when no template is available.

长脚本 `skill_run` 失败时，查看工具返回的 `script_path` 与错误行列号，用 `fs_patch` 局部修复后用 `skill_run {"path":"<script_path>"}` 重跑；成功后的 `script.js` 在同 session 内跨 turn 保留供后续修改重跑。

最小模板（`skill_run`，可直接复制）：

```javascript
async function main() {
  const pptx = new PptxGenJS();          // 全局构造函数，require('pptxgenjs') 也可
  pptx.layout = 'LAYOUT_16x9';
  const slide = pptx.addSlide();
  slide.addText("标题", { x: 0.5, y: 0.5, w: 9, h: 1, fontSize: 36, bold: true });
  await pptx.writeFile({ fileName: "deck.pptx" });   // 已接入沙箱
  return { ok: true };
}
// 不要在末尾调用 main()，运行时会自动调用
```

---

## Editing Workflow

**Read [editing.md](editing.md) for full details.**

1. `office_read_to_markdown` 了解模板内容与占位符
2. `ooxml_unpack` → 调整幻灯片结构 → 编辑各 `slide{N}.xml` → `ooxml_pack`

---

## Design Ideas

**Don't create boring slides.** Plain bullets on a white background won't impress anyone. Consider ideas from this list for each slide.

### Before Starting

- **Pick a bold, content-informed color palette**: The palette should feel designed for THIS topic. If swapping your colors into a completely different presentation would still "work," you haven't made specific enough choices.
- **Dominance over equality**: One color should dominate (60-70% visual weight), with 1-2 supporting tones and one sharp accent. Never give all colors equal weight.
- **Dark/light contrast**: Dark backgrounds for title + conclusion slides, light for content ("sandwich" structure). Or commit to dark throughout for a premium feel.
- **Commit to a visual motif**: Pick ONE distinctive element and repeat it — rounded image frames, icons in colored circles, thick single-side borders. Carry it across every slide.

### 中文字体（CRITICAL）

中文演示文稿 MUST 指定中文字体，否则西文字体会回退渲染中文：

```javascript
slide.addText("季度汇报", {
  x: 1, y: 1, fontSize: 28, bold: true,
  fontFace: "微软雅黑",  // 标题可用「微软雅黑」或「思源黑体」
});
// 正文同样设置 fontFace: "微软雅黑"
```

### Color Palettes

Choose colors that match your topic — don't default to generic blue. Use these palettes as inspiration:

| Theme | Primary | Secondary | Accent |
|-------|---------|-----------|--------|
| **Midnight Executive** | `1E2761` (navy) | `CADCFC` (ice blue) | `FFFFFF` (white) |
| **Forest & Moss** | `2C5F2D` (forest) | `97BC62` (moss) | `F5F5F5` (cream) |
| **Coral Energy** | `F96167` (coral) | `F9E795` (gold) | `2F3C7E` (navy) |
| **Warm Terracotta** | `B85042` (terracotta) | `E7E8D1` (sand) | `A7BEAE` (sage) |
| **Ocean Gradient** | `065A82` (deep blue) | `1C7293` (teal) | `21295C` (midnight) |
| **Charcoal Minimal** | `36454F` (charcoal) | `F2F2F2` (off-white) | `212121` (black) |
| **Teal Trust** | `028090` (teal) | `00A896` (seafoam) | `02C39A` (mint) |
| **Berry & Cream** | `6D2E46` (berry) | `A26769` (dusty rose) | `ECE2D0` (cream) |
| **Sage Calm** | `84B59F` (sage) | `69A297` (eucalyptus) | `50808E` (slate) |
| **Cherry Bold** | `990011` (cherry) | `FCF6F5` (off-white) | `2F3C7E` (navy) |

### For Each Slide

**Every slide needs a visual element** — image, chart, icon, or shape. Text-only slides are forgettable.

**Layout options:**
- Two-column (text left, illustration on right)
- Icon + text rows (icon in colored circle, bold header, description below)
- 2x2 or 2x3 grid (image on one side, grid of content blocks on other)
- Half-bleed image (full left or right side) with content overlay

**Data display:**
- Large stat callouts (big numbers 60-72pt with small labels below)
- Comparison columns (before/after, pros/cons, side-by-side options)
- Timeline or process flow (numbered steps, arrows)

**Visual polish:**
- Icons in small colored circles next to section headers
- Italic accent text for key stats or taglines

### Typography

**Choose an interesting font pairing** — don't default to Arial. Pick a header font with personality and pair it with a clean body font.

| Header Font | Body Font |
|-------------|-----------|
| Georgia | Calibri |
| Arial Black | Arial |
| Calibri | Calibri Light |
| Cambria | Calibri |
| Trebuchet MS | Calibri |
| Impact | Arial |
| Palatino | Garamond |
| Consolas | Calibri |

| Element | Size |
|---------|------|
| Slide title | 36-44pt bold |
| Section header | 20-24pt bold |
| Body text | 14-16pt |
| Captions | 10-12pt muted |

### Spacing

- 0.5" minimum margins
- 0.3-0.5" between content blocks
- Leave breathing room—don't fill every inch

### Avoid (Common Mistakes)

- **Don't repeat the same layout** — vary columns, cards, and callouts across slides
- **Don't center body text** — left-align paragraphs and lists; center only titles
- **Don't skimp on size contrast** — titles need 36pt+ to stand out from 14-16pt body
- **Don't default to blue** — pick colors that reflect the specific topic
- **Don't mix spacing randomly** — choose 0.3" or 0.5" gaps and use consistently
- **Don't style one slide and leave the rest plain** — commit fully or keep it simple throughout
- **Don't create text-only slides** — add images, icons, charts, or visual elements; avoid plain title + bullets
- **Don't forget text box padding** — when aligning lines or shapes with text edges, set `margin: 0` on the text box or offset the shape to account for padding
- **Don't use low-contrast elements** — icons AND text need strong contrast against the background; avoid light text on light backgrounds or dark text on dark backgrounds
- **NEVER use accent lines under titles** — these are a hallmark of AI-generated slides; use whitespace or background color instead

---

## QA (Required)

**Assume there are problems. Your job is to find them.**

Your first render is almost never correct. Approach QA as a bug hunt, not a confirmation step. If you found zero issues on first inspection, you weren't looking hard enough.

### Content QA

```json
office_read_to_markdown {"path": "output.pptx"}
```

Check the markdown output for:

- Missing content, typos, wrong slide order
- **Leftover placeholder text**（模板场景必查）：`xxxx`、`lorem`、`【】` 占位、"this slide layout" 等
- 数字/日期/人名等关键事实与源材料一致

### Layout QA（无渲染环境的降级方案）

本系统无法把幻灯片渲染成图片。改用代码审查方式自检坐标：

- **越界**：`x + w` 不得超过版面宽（16x9 为 10"，WIDE 为 13.33"）；`y + h` 不得超过版面高（5.625" / 7.5"）
- **重叠**：同一 slide 中各元素的矩形区域是否相交（尤其页脚条与正文、卡片与卡片）
- **边距**：元素距边缘 ≥ 0.5"，块间距 ≥ 0.3"
- **文字容量**：长文本配窄文本框会溢出；中文按 ~1.8 字符宽估算
- 最后告知用户用 PowerPoint/WPS 打开确认视觉效果

### Verification Loop

1. Generate slides → Content QA → Layout QA
2. **List issues found** (if none found, look again more critically)
3. Fix issues → re-check affected slides — one fix often creates another problem
4. **Do not declare success until you've completed at least one fix-and-verify cycle.**

---

## doc-agent 系统约束

- **PptxGenJS 保存**：只用 `await pptx.writeFile({ fileName })`（已接入沙箱）；勿用 `fs` 或 stream。
- **缩略图/渲染**：不可用；按上方 Layout QA 降级。
- **图片**：只支持 base64 `data:` 形式（无网络/无本地 path 加载），见 pptxgenjs.md Images 一节。
