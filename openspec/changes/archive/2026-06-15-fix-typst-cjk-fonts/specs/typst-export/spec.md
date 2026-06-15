## MODIFIED Requirements

### Requirement: 字体策略

模板字体栈 MUST 优先使用当前平台常见系统字体（Windows：微软雅黑、宋体、黑体；macOS：Songti SC、Heiti SC、PingFang SC 等），拉丁文 MUST 使用 `covers: "latin-in-cjk"` 与 Times New Roman 分离；并 MUST 捆绑 Noto Sans SC / Noto Serif SC（Subset Regular + Bold）作为跨平台回退，经 `TypstKitFontOptions::include_dirs` 注入，以便在无平台中文字体的环境仍可无警告编译。

#### Scenario: Windows 系统字体

- **WHEN** 在已安装微软雅黑与宋体的 Windows 上编译中文模板
- **THEN** PDF 使用相应系统字体渲染中文

#### Scenario: 无系统中文字体

- **WHEN** 在无 SimSun/微软雅黑的环境编译
- **THEN** 编译仍成功，回退至捆绑的 Noto SC 字体，且无 `unknown font family` 警告
