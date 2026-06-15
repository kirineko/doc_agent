## 字体资源

构建时从 noto-cjk 仓库下载 Subset OTF（SC）：

| 文件 | 用途 |
|------|------|
| NotoSansSC-Regular.otf | 中文无衬线正文 |
| NotoSansSC-Bold.otf | 中文标题 |
| NotoSerifSC-Regular.otf | 中文衬线正文 |
| NotoSerifSC-Bold.otf | 中文衬线加粗 |

存放 `src-tauri/fonts/`（gitignore），Tauri bundle `resources: { "fonts/": "fonts/" }`。

## 引擎

```text
TypstEngine::builder()
  .search_fonts_with(
    TypstKitFontOptions::default()
      .include_dirs(font_search_paths())
  )
```

`font_search_paths()` 与 PDFium 相同模式：`DOC_AGENT_FONTS_DIR` → `.app/Contents/Resources/fonts` → `CARGO_MANIFEST_DIR/fonts`。

## 平台字体栈（虚拟路径 `/doc-agent/typst/common/fonts-stack.typ`）

- **macOS**：Songti SC / Heiti SC / PingFang SC + Noto SC
- **Windows**：SimSun / 微软雅黑 / SimHei + Noto SC
- **其他（CI）**：仅 Noto SC + Libertinus

拉丁文使用 `covers: "latin-in-cjk"` 与 Times New Roman。

## 字体族名

Subset OTF 注册名为 `Noto Sans SC`、`Noto Serif SC`（非 `Noto Sans CJK SC`）。
