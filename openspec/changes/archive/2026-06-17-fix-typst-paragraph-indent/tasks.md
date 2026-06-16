## 1. 主题与 apply-zh-body

- [x] 1.1 在 `common/tokens.typ` 的 `make-theme(...)` 增加 `cjk-paragraph-indent: false` 参数，写入主题 dict
- [x] 1.2 在 `common/fonts.typ` 的 `apply-zh-body` 移除默认 `first-line-indent`；当 `theme.cjk-paragraph-indent == true` 时条件设置 `first-line-indent: indent-cjk`

## 2. 场景模板修补

- [x] 2.1 `paper/paper-zh.typ`：参考文献区块去掉 `#pad(left: indent-cjk)[…]`，改为普通条目列表
- [x] 2.2 `paper/paper-en.typ`：参考文献区块去掉 `#pad(left: 2em)[…]`，改为普通条目列表
- [x] 2.3 确认 8 个场景模板仍零 warning 编译（`cargo test` 模板相关用例）

## 3. 语法手册

- [x] 3.1 新增/修订「段落与标题规范」：`=` 标题、禁止伪标题、禁止滥用 `#pad` / 重复 `#set par(first-line-indent)`
- [x] 3.2 修订 §3 `#set` 与 `#show`：强调 `apply-zh-body` 已含段落规则，移除与默认策略矛盾的 `first-line-indent` 示例
- [x] 3.3 修订 §22 主题：文档 `cjk-paragraph-indent` 参数与用法示例
- [x] 3.4 §23 常见错误表增加「伪标题」「滥用 `#pad`」两行
- [x] 3.5 同步 §0.2 exports 表（若 `make-theme` 签名变化）；**不**添加 `#outline(depth: …)` 限制

## 4. 测试

- [x] 4.1 新增或扩展测试：默认 `apply-zh-body` 不设置 `first-line-indent`；`cjk-paragraph-indent: true` 时设置 `indent-cjk`
- [x] 4.2 确认 handbook 可编译示例与 exports 一致性测试通过（`guide_tests.rs`）
- [x] 4.3 本地跑 `cargo test`（typst_export 相关）+ 模板零 warning 用例全部 green

## 5. 验收

- [x] 5.1 用 report-zh 模板编译样例 PDF，目视确认标题/正文左缘一致、无偶发首行缩进
- [x] 5.2 用 `make-theme(cjk-paragraph-indent: true)` 编译同内容，确认首行缩进恢复
