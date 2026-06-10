# Editing Presentations

**本系统编辑已有 PPT 走 XML 流程**：`ooxml_unpack` → 编辑 XML → `ooxml_pack`。无缩略图/渲染能力。

## Template-Based Workflow

When using an existing presentation as a template:

1. **Analyze existing slides**:

   ```json
   office_read_to_markdown {"path": "template.pptx"}
   ```

   了解每页文本与占位符；布局细节需 unpack 后看 `slide{N}.xml`。

2. **Plan slide mapping**: For each content section, choose a template slide.

   ⚠️ **USE VARIED LAYOUTS** — monotonous presentations are a common failure mode. Don't default to basic title + bullet slides. Actively seek out:
   - Multi-column layouts (2-column, 3-column)
   - Image + text combinations
   - Quote or callout slides
   - Section dividers
   - Stat/number callouts

   Match content type to layout style (e.g., key points → bullet slide, team info → multi-column, testimonials → quote slide).

3. **Unpack**:

   ```json
   ooxml_unpack {"path": "template.pptx", "out_dir": "unpacked/"}
   ```

4. **Adjust slide structure**（删除/重排，见下方 Slide Operations）。**Complete all structural changes before step 5.**

5. **Edit content**: Update text in each `unpacked/ppt/slides/slide{N}.xml`.
   - 批量替换：`skill_run` + `fs.readFileSync(path, 'utf-8')` → `replace` → `fs.writeFileSync`（带 missed 检查，模式同 docx editing.md）
   - 逐处修改：`fs_read` + `fs_write`

6. **Pack**:

   ```json
   ooxml_pack {"dir": "unpacked/", "out_path": "output.pptx", "original": "template.pptx"}
   ```

   含校验与自动修复。

---

## Slide Operations

Slide order is in `unpacked/ppt/presentation.xml` → `<p:sldIdLst>`.

**Reorder**: Rearrange `<p:sldId>` elements.

**Delete**:
1. Remove the `<p:sldId>` entry from `<p:sldIdLst>`
2. 同时删除对应 `ppt/_rels/presentation.xml.rels` 中的 `<Relationship>`，以及 `[Content_Types].xml` 中该 slide 的 `<Override>`（否则打包校验可能报孤儿引用）

**Duplicate / Add（无专用工具，手工四步）**：
1. 复制 `slides/slideN.xml` → `slides/slideM.xml`（M 取未占用编号），同时复制 `slides/_rels/slideN.xml.rels` → `slideM.xml.rels`
2. `[Content_Types].xml` 增加 `<Override PartName="/ppt/slides/slideM.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>`
3. `ppt/_rels/presentation.xml.rels` 增加 `<Relationship Id="rIdX" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slideM.xml"/>`（rIdX 取未占用 id）
4. `<p:sldIdLst>` 目标位置插入 `<p:sldId id="NNN" r:id="rIdX"/>`（id 取 ≥256 且未占用的数字）

⚠️ 若源 slide 的 `.rels` 引用 notesSlide，复制时删除该 Relationship（或同步复制 notes 文件），否则引用悬空。

---

## Editing Content

For each slide:
1. Read the slide's XML (`fs_read`)
2. Identify ALL placeholder content—text, images, charts, icons, captions
3. Replace each placeholder with final content（只改 `<a:t>` 内文本，保留标签结构）

### Formatting Rules

- **Bold all headers, subheadings, and inline labels**: Use `b="1"` on `<a:rPr>`. This includes:
  - Slide titles
  - Section headers within a slide
  - Inline labels like (e.g.: "Status:", "Description:") at the start of a line
- **Never use unicode bullets (•)**: Use proper list formatting with `<a:buChar>` or `<a:buAutoNum>`
- **Bullet consistency**: Let bullets inherit from the layout. Only specify `<a:buChar>` or `<a:buNone>`.

---

## Common Pitfalls

### Template Adaptation

When source content has fewer items than the template:
- **Remove excess elements entirely** (images, shapes, text boxes), don't just clear text
- Check for orphaned visuals after clearing text content
- Run visual QA to catch mismatched counts

When replacing text with different length content:
- **Shorter replacements**: Usually safe
- **Longer replacements**: May overflow or wrap unexpectedly
- Test with visual QA after text changes
- Consider truncating or splitting content to fit the template's design constraints

**Template slots ≠ Source items**: If template has 4 team members but source has 3 users, delete the 4th member's entire group (image + text boxes), not just the text.

### Multi-Item Content

If source has multiple items (numbered lists, multiple sections), create separate `<a:p>` elements for each — **never concatenate into one string**.

**❌ WRONG** — all items in one paragraph:
```xml
<a:p>
  <a:r><a:rPr .../><a:t>Step 1: Do the first thing. Step 2: Do the second thing.</a:t></a:r>
</a:p>
```

**✅ CORRECT** — separate paragraphs with bold headers:
```xml
<a:p>
  <a:pPr algn="l"><a:lnSpc><a:spcPts val="3919"/></a:lnSpc></a:pPr>
  <a:r><a:rPr lang="en-US" sz="2799" b="1" .../><a:t>Step 1</a:t></a:r>
</a:p>
<a:p>
  <a:pPr algn="l"><a:lnSpc><a:spcPts val="3919"/></a:lnSpc></a:pPr>
  <a:r><a:rPr lang="en-US" sz="2799" .../><a:t>Do the first thing.</a:t></a:r>
</a:p>
<a:p>
  <a:pPr algn="l"><a:lnSpc><a:spcPts val="3919"/></a:lnSpc></a:pPr>
  <a:r><a:rPr lang="en-US" sz="2799" b="1" .../><a:t>Step 2</a:t></a:r>
</a:p>
<!-- continue pattern -->
```

Copy `<a:pPr>` from the original paragraph to preserve line spacing. Use `b="1"` on headers.

### Smart Quotes

Handled automatically by unpack/pack.

**When adding new text with quotes, use XML entities:**

```xml
<a:t>the &#x201C;Agreement&#x201D;</a:t>
```

| Character | Name | Unicode | XML Entity |
|-----------|------|---------|------------|
| `“` | Left double quote | U+201C | `&#x201C;` |
| `”` | Right double quote | U+201D | `&#x201D;` |
| `‘` | Left single quote | U+2018 | `&#x2018;` |
| `’` | Right single quote | U+2019 | `&#x2019;` |

### Other

- **Whitespace**: Use `xml:space="preserve"` on `<a:t>` with leading/trailing spaces
- **String replace only**: 用字符串替换编辑 XML，不要尝试解析/重排序整个文档（命名空间易被破坏）
