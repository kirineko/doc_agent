## 1. 实现代码块字体规则

- [x] 1.1 在 `common/fonts.typ` 的 `apply-zh-body` 内新增 `show raw: set text(font: (..font-mono, ..font-serif-zh))`
- [x] 1.2 在 `common/fonts.typ` 的 `apply-en-body` 内新增同一 `show raw` 规则
- [x] 1.3 确认 `font-mono` 已被实际引用（不再是死代码），保留其定义

## 2. 测试与验证

- [x] 2.1 在 `typst_export` 编译测试中新增「含中英文混排代码块」的用例，断言 `warnings` 为空
- [x] 2.2 验证 fallback 字体栈（CI/Linux）下含中文代码块仍零 `unknown font family` 警告
- [x] 2.3 本地编译一个含中文代码块的样例 `.typ`，目视确认中文为宋体、英文为等宽体，无隶书等回退

## 3. 文档与一致性

- [x] 3.1 视情况在 `syntax/typst-guide.md` 第 16 节补充说明代码块中文已由模板钉死为宋体（如手册导出一致性测试涉及则同步）
- [x] 3.2 运行 `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test` 确认通过
