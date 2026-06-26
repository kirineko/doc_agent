## 1. 修复 comments.xml 写入（主因）

- [x] 1.1 重写 `comment::add_comment`：用 `quick-xml` 定位根 `<w:comments>`，处理自闭合空壳（`<w:comments/>` → 展开成对标签）与已有内容两种形态，在根部内部末尾追加构造好的 `<w:comment>`，替换当前的 `rfind("</w:comments>")` 字符串方案（`src-tauri/src/tools/ooxml/comment.rs:32`）
- [x] 1.2 构造 `<w:comment>` 片段时对批注文本做 XML 实体转义（`<` `>` `&` `"` `'`），避免破坏文档
- [x] 1.3 写入前检查目标 `w:id` 是否已存在于 comments.xml，重复则返回 `ToolError`（满足"重复 id 应被拒绝"场景）
- [x] 1.4 单元测试：对自闭合空壳、已有内容、重复 id 三种 comments.xml 形态分别验证写入结果

## 2. 新增 document.xml 锚点装配（段落级定位）

- [x] 2.1 `comment_tool` schema（`src-tauri/src/tools/ooxml/mod.rs:57-67`）增加 `paragraph_index: integer`（必填）与 `text_hint: string`（可选）；更新工具 description 说明定位语义
- [x] 2.2 `comment_handler` 解析新参数并传入 `add_comment`
- [x] 2.3 `add_comment` 增加定位逻辑：遍历 `word/document.xml` 的顶层 `<w:p>`，按 `paragraph_index` 定位；越界或命中非段落元素则报错
- [x] 2.4 在目标段落插入 `<w:commentRangeStart w:id="X"/>`（段首 run 前）、`<w:commentRangeEnd w:id="X"/>` + 含 `<w:commentReference w:id="X"/>` 的 `<w:r>`（段末 run 后），`X` 取自 `id` 参数
- [x] 2.5 若提供 `text_hint`：校验该段落纯文本包含此子串，不匹配则报错（防 agent 数错段落）
- [x] 2.6 测试：`paragraph_index` 命中、越界、`text_hint` 不匹配三种场景

## 3. 回复链（commentsExtended）与 people.xml

- [x] 3.1 当 `parent` 提供时：校验父 `w:id` 存在于 comments.xml，不存在则报错；写入 `word/commentsExtended.xml` 的 `<w15:commentEx w15:paraIdParent="<父paraId>"/>`，子条目 paraId 用既有 `wrapping_mul(0x9E37_79B9)` 算法生成
- [x] 3.2 commentsExtended.xml 随需创建（不存在则建带正确命名空间的空容器）并登记 `[Content_Types].xml` Override
- [x] 3.3 在 `word/_rels/document.xml.rels` 增补 commentsExtended 的关系（已存在则跳过）
- [x] 3.4 首次写入批注时创建 `word/people.xml`（含作者 `<w15:person>`），登记 Content_Types Override 与必要关系；重复作者不重复登记
- [x] 3.5 测试：回复链 paraIdParent 对应、父 id 不存在报错、people.xml 首次建立与重复作者去重

## 4. 验证器批注一致性规则

- [x] 4.1 在 `validate/rules/wml.rs` 新增 `wml.comment.consistency`：收集 `word/comments.xml` 的 `w:comment/@w:id` 集合 与 `word/document.xml` 的 `commentReference/@w:id` 集合，对称差非空即违规，message 列出失配 id
- [x] 4.2 在 `validate/rules/mod.rs` 分发：当被校验 part 为 `word/document.xml` 时触发该规则（此时 comments.xml 可读）
- [x] 4.3 测试：有 comment 无 commentReference、有 commentReference 无 comment、一致放行 三种场景

## 5. 端到端测试加固

- [x] 5.1 升级 `smoke_redline_comment_chain`（`src-tauri/src/tools/tests.rs:1670`）：断言改为——打包后 `word/comments.xml` 含 `<w:comment w:id="1">`、`word/document.xml` 含匹配的 `commentRangeStart w:id="1"` 与 `commentReference w:id="1"`（不再只查文件是否在 zip 列表）
- [x] 5.2 新增端到端测试：含 `paragraph_index` + `text_hint` 的批注 → 打包 → 解包校验锚点与条目对应
- [x] 5.3 新增端到端测试：parent 回复链 → 校验 commentsExtended 的 paraIdParent 归属

## 6. 文档与收尾

- [x] 6.1 更新 `comment_tool` 的工具 description，写清 `paragraph_index`（0-based 顶层段落）、`text_hint`（断言式校验）、整段粒度限制
- [x] 6.2 在本 design.md 注明 MVP 不支持表格内/跨段细粒度批注
- [x] 6.3 跑全部门禁：`cargo fmt --check && cargo clippy -- -D warnings && cargo test`；前端无需改动

## 7. Review 后续修复

- [x] 7.1 `end_of_ppr` 改用深度计数定位段落 `pPr` 闭合，替换脆弱的 `find("</w:pPr>")`：当 `pPr` 内含 `<w:pPrChange>`（修订标记）嵌套的内层 `pPr` 时，旧实现会命中内层闭合标签，把 `commentRangeStart` 错误插入 `pPrChange` 内部（非法 WML）。同时删除不再使用的 `find_self_close_end`
- [x] 7.2 新增回归测试 `comment_anchor_after_ppr_with_nested_pprchange`：断言带 `pPrChange` 嵌套 `pPr` 的段落，锚点落在外层 `pPr` 闭合之后、首个 `<w:r>` 之前，且输出仍 well-formed

## 8. Review 第三轮：原子性与部件可发现性

- [x] 8.1 `add_comment` 改为完全原子：用 `comments_path.exists()` 纯检查替代会落盘的 `ensure_comments_file`，comments-less 文档在内存模板上推导；commentsExtended / people 也在 compute 阶段推导内容、commit 阶段统一写入。修复"校验失败时已建空壳 comments.xml → 重试时 `comments_created==false` 漏注册"（P2-1）
- [x] 8.2 `register_part` 在 `word/_rels/document.xml.rels` 缺失时按模板补建（含 `_rels` 目录），不再 `if exists` 静默跳过。修复"最小 DOCX 无 rels 时评论关系不登记 → Word 发现不了"（P2-2）
- [x] 8.3 删除脆弱的 `append_into_root`（`rfind` 闭合标签），统一复用健壮的 `append_comment_to_root`（重命名为通用 `append_into_root`，支持自闭合根与任意命名空间前缀）。修复"已存在的自闭合 `<w15:people/>` / `<w15:commentsEx/>` 追加报错 → 非原子且重试撞重复 id"（P2-3）
- [x] 8.4 新增回归测试：`failed_call_does_not_create_comments_then_retry_registers`、`registers_comments_when_document_rels_missing`、`append_into_root_handles_self_closing_aux_part`

## 9. Review 第四轮：校验误报与内置文档同步

- [x] 9.1 `wml.comment.consistency` 豁免回复批注：经 `commentsExtended.xml`（`paraIdParent`）链接到父批注的回复，其 `paraId` 通过 comments.xml 映射回 `w:id` 后从"缺 commentReference"方向排除；Word 真实线程文档不再被误判（P2）。新增 helper `collect_reply_comment_ids` / `collect_reply_para_ids` / `map_reply_comment_ids`（`src-tauri/src/tools/ooxml/validate/rules/wml.rs`）
- [x] 9.2 回归测试：`comment_consistency_reply_without_reference_passes`（回复无引用应放行）、`comment_consistency_non_reply_without_reference_still_fails`（非回复缺引用仍报错）
- [x] 9.3 同步内置 docx skill 文档至新 API：`assets/skills/docx/SKILL.md` 与 `editing.md` 标注 `paragraph_index` 必填、`text_hint` 可选，并改为"`docx_comment` 自动写入正文锚点，无需手动加标记"（P2，旧文档教 agent 用缺 `paragraph_index` 的过时调用会直接报错）

## 10. Review 第五轮：跨 story part 引用、命名空间声明、转义文档

- [x] 10.1 `wml.comment.consistency` 跨 story part 收集 `commentReference`：新增 `collect_reference_ids` 扫描 `word/` 下 `header*.xml` / `footer*.xml` / `footnotes.xml` / `endnotes.xml`（连同传入的 document.xml），修复"批注锚定在页眉/页脚/脚注 → 引用不在 document.xml → 被误判缺引用、阻塞 `ooxml_pack`"（P2）。回归测试 `comment_consistency_reference_in_header_passes`
- [x] 10.2 插入的 `<w:comment>` 自带 `xmlns:w` / `xmlns:w14` 声明（新增 `WML_NS` / `W14_NS` 常量）：修复"现有 comments.xml 根仅声明 `xmlns:w`（或用默认/自定义前缀）时，片段引入未声明的 `w14:` → 对合法 OOXML 输入产出非法 XML"（P2）。回归测试 `comment_entry_self_declares_namespaces`、`append_comment_into_w14less_root_keeps_w14_declared`
- [x] 10.3 修正 `SKILL.md` 批注文档：工具自身已转义 `< > & " '`，删除"text must be pre-escaped XML"误导（否则 `&amp;` 会被二次转义成 `&amp;amp;`），示例改用原始 `&` 与排版字符；`editing.md` 补同等说明（P2）

## 11. Review 第六轮：回复 paraId 与段落前缀

- [x] 11.1 回复改用父批注**真实** `w14:paraId`：新增 `find_comment_para_id` 从 comments.xml 读取父批注段落的 paraId，替换 `parent_id.wrapping_mul(...)` 推导。修复"回复 Word 原生已有批注时，推导值≠真实 paraId → `paraIdParent` 指向不存在段落 → 回复未挂到父批注下"（P2）；父批注无 paraId 时明确报错。回归测试 `finds_actual_parent_para_id`
- [x] 11.2 锚点前缀跟随段落实际命名空间前缀：新增 `element_prefix`，`insert_paragraph_anchors` 用其构造 `commentRangeStart/End`/`commentReference` 及闭合标签查找，替换硬编码 `w:` 与 `</w:p>`/`</p>`。修复"document.xml 用默认/自定义前缀（如 `w2:p`）绑定 WML 时，定位虽按 local name 命中，但拼接只认 `</w:p>` 且锚点硬编码 `w:` → 报错或产出未绑定标记"（P2）。移除已无用的 `close_tag_abs`。回归测试 `anchors_use_document_paragraph_prefix`、`anchors_handle_default_namespace_paragraph`、`element_prefix_extracts_prefix`

## 12. Review 第七轮：默认命名空间下的属性前缀

- [x] 12.1 默认命名空间文档的元素可无前缀，但属性必须带 `w:`：`insert_paragraph_anchors` 拆分 `pfx`(元素) 与 `apfx`(属性)；当 `pfx` 为空时 `apfx` 固定为 `w:`，产出 `<commentRangeStart w:id="…"/>` 而非无效的 `<commentRangeStart id="…"/>`（XML 默认命名空间不作用于属性，Word 无法识别裸 `id`/`val`）（P2）。更新回归测试 `anchors_handle_default_namespace_paragraph`

## 13. Review 第八轮：部件注册、xmlns 绑定、w15 自声明、回复校验

- [x] 13.1 `register_part` 不再仅在新文件时调用：成功写入批注后始终注册 `comments.xml` / `people.xml`，有回复时始终注册 `commentsExtended.xml`（idempotent），修复"orphan comments shell 缺 Content Types/rels → Word 发现不了"（P2）。回归测试 `registers_comments_when_orphan_shell_lacks_rels`
- [x] 13.2 默认命名空间文档在段落 open tag 注入 `xmlns:w`（`inject_w_namespace_decl`），使 `w:id`/`w:val` 属性有合法绑定作用域；修复"裸 `w:` 前缀未声明 → namespace-invalid"（P2）。更新 `anchors_handle_default_namespace_paragraph`
- [x] 13.3 `commentsExtended`/`people` 插入片段自带 `xmlns:w15`（`W15_NS` 常量），与 `format_comment_entry` 对齐（P2）。回归测试 `comment_ex_entry_self_declares_w15_namespace`
- [x] 13.4 回复豁免前先验证 `paraIdParent` 指向 comments.xml 中已有 paraId（`collect_comment_para_ids` + `collect_verified_reply_para_ids`），修复"stale commentsExtended 误豁免 → 无锚点批注通过 pack"（P2）。回归测试 `comment_consistency_stale_para_id_parent_still_fails`

## 14. Review 第九轮：自闭合 rels 与可达 story part

- [x] 14.1 `register_part` 写 rels 时复用 `append_into_root`，修复"已有 `<Relationships .../>` 空壳时 `rfind("</Relationships>")` 静默 no-op → 关系未写入"（P2）。回归测试 `registers_part_when_rels_is_self_closing_shell`
- [x] 14.2 `collect_reference_ids` 仅统计 `document.xml.rels` 引用的 header/footer/footnotes/endnotes（`collect_referenced_story_targets` + `STORY_REL_TYPES`），忽略磁盘上 stale 未引用 story 文件（P2）。更新 `comment_consistency_reference_in_header_passes`（补 rels）；新增 `comment_consistency_unreferenced_header_does_not_count`

## 15. Review 第十轮：headerReference 交叉校验与 prefixed OPC 根

- [x] 15.1 header/footer story 须同时满足 rels **且** `document.xml` 中有匹配的 `headerReference`/`footerReference`（`collect_used_header_footer_rids`）；footnotes/endnotes 仍仅看 rels（P2）。更新 `comment_consistency_reference_in_header_passes`（补 headerReference）；新增 `comment_consistency_stale_header_rel_without_header_reference_fails`
- [x] 15.2 `register_part` 写 `[Content_Types].xml` 时复用 `append_into_root`，修复 prefixed 根（如 `<ct:Types>`）或自闭合 shell 时 `rfind("</Types>")` 静默失败（P2）。回归测试 `registers_part_with_prefixed_content_types_root`

## 16. Review 第十一轮：prefixed OPC 子元素

- [x] 16.1 `register_part` 插入 `[Content_Types].xml` 的 `Override` 时，读取 root QName 前缀（如 `ct:`）并生成同前缀的 `<ct:Override>`；无前缀 root 保持 `<Override>`（默认命名空间生效）。修复 prefixed root 无默认命名空间时裸 `Override` 不在 OPC content-types namespace 内的问题（P2）。收紧回归测试 `registers_part_with_prefixed_content_types_root`
- [x] 16.2 `register_part` 插入 `document.xml.rels` 的 `Relationship` 时，读取 root QName 前缀（如 `rel:`）并生成同前缀的 `<rel:Relationship>`；无前缀 root 保持 `<Relationship>`。修复 prefixed rels root 无默认命名空间时裸 `Relationship` 不在 OPC relationships namespace 内的问题（P2）。新增回归测试 `registers_part_with_prefixed_relationships_root`
