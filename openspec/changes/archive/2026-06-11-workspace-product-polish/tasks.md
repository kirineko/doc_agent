## 1. 旧版 Office 转换与 data_query

- [x] 1.1 在 `tools/office.rs` 实现 `office_convert`（`office_oxide` `save_as`、默认 `{stem}-converted.{ext}`、`out_path` 须含 `-converted`、已存在则报错）
- [x] 1.2 注册 `office_convert` 到 `tools/registry.rs` 并补充 handler 单元测试
- [x] 1.3 扩展 `data_query` 的 `load_source` 支持 `.xls`（临时 xlsx + 现有 calamine 路径）并加测试
- [x] 1.4 更新 `ooxml/unpack.rs` 错误文案指向 `office_convert`
- [x] 1.5 更新 `assets/skills/docx|xlsx|pptx/SKILL.md` 旧格式说明

## 2. 文件浏览 IPC

- [x] 2.1 在 `core/project_files.rs` 或新模块实现单层 `list_project_dir`（复用 sandbox 与忽略规则）
- [x] 2.2 实现 `open_project_file` IPC（sandbox 校验 + `tauri-plugin-opener`）
- [x] 2.3 在 `lib.rs` 注册新 commands 并补充 IPC/路径校验测试

## 3. 右侧栏与顶栏 UI

- [x] 3.1 实现 `ProjectFileExplorer`（单层列表、目录导航、`..`、双击打开文件）
- [x] 3.2 组合 `RightPanel`（上工具链 + 下文件浏览）并替换 `App.tsx` 中的 `ToolChainPanel`
- [x] 3.3 顶栏加入 `public/logo.svg` 与 Logo 展示

## 4. 品牌与安装路径

- [x] 4.1 设计并导出 Logo（文档 + AI 弧线，青蓝主色），生成 `public/logo.svg`
- [x] 4.2 用 `tauri icon` 或等价流程更新 `src-tauri/icons/*`
- [x] 4.3 修改 `tauri.conf.json`：`productName` → `DocAgent`，`title` 保持 `Doc Agent`

## 5. 验证

- [x] 5.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [x] 5.2 `npm run typecheck && npm test && npm run build`
