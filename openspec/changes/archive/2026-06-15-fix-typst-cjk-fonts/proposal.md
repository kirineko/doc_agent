# Proposal: fix-typst-cjk-fonts

## 问题

`typst_to_pdf` 编译中文模板时出现 `unknown font family` 警告（Microsoft YaHei、SimSun、SimHei、Noto Sans/Serif CJK SC）。根因是字体栈引用了未捆绑的 Noto 名称，且跨平台字体名混排。

## 方案

采用 Noto **Subset** 简体（Regular + Bold，约 39 MB），构建时下载至 `src-tauri/fonts/`（不入库），经 Tauri `resources` 与 `TypstKitFontOptions::include_dirs` 注入；`fonts.typ` 回退名改为 `Noto Sans SC` / `Noto Serif SC`；按目标 OS 提供独立中文字体栈，消除跨平台警告。

## 纳入

- build.rs 下载 4 个 OTF
- 平台字体栈（macOS / Windows / fallback）
- 编译链 `include_dirs` + 启动时配置资源路径
- 测试：中文模板编译无字体警告

## 排除

- 全量 CJK SC 字体包（~80 MB）
- pyftsubset 自定义裁切
- Linux 安装包
