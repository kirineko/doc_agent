## MODIFIED Requirements

### Requirement: 字体策略

模板字体栈 MUST 优先使用当前平台常见系统字体（Windows：微软雅黑、宋体、黑体；macOS：Songti SC、Heiti SC、PingFang SC 等），拉丁文 MUST 使用 `covers: "latin-in-cjk"` 与 Times New Roman 分离；并 MUST 捆绑 Noto Sans SC / Noto Serif SC（Subset Regular + Bold）作为跨平台回退，经 `TypstKitFontOptions::include_dirs` 注入，以便在无平台中文字体的环境仍可无警告编译。

代码块（Typst `raw`）字体 MUST 由 `apply-zh-body` 与 `apply-en-body` 通过 `show raw` 规则显式钉死，MUST NOT 依赖 Typst 隐式默认或不受控的逐字形回退。该代码块字体栈 MUST 为「等宽英文 + 受控中文衬线」组合：英文/符号 MUST 优先使用等宽字体栈 `font-mono`（Consolas / Menlo / Courier New / Libertinus Mono），中文 MUST 确定性地使用衬线宋体栈 `font-serif-zh`（Windows `SimSun`、macOS `Songti SC`/`STSong`，并以捆绑 `Noto Serif SC` 跨平台兜底）。`font-mono` MUST 被 `show raw` 实际引用而非保留为未使用的死代码。

#### Scenario: Windows 系统字体

- **WHEN** 在已安装微软雅黑与宋体的 Windows 上编译中文模板
- **THEN** PDF 使用相应系统字体渲染中文

#### Scenario: 无系统中文字体

- **WHEN** 在无 SimSun/微软雅黑的环境编译
- **THEN** 编译仍成功，回退至捆绑的 Noto SC 字体，且无 `unknown font family` 警告

#### Scenario: 代码块中文使用衬线宋体而非回退书法体

- **WHEN** 用户 `.typ` 套用 `apply-zh-body` 并含中英文混排的代码块（` ``` ` 围栏）
- **THEN** 代码块英文使用 `font-mono`（如 Consolas），中文确定性使用 `font-serif-zh`（Windows `SimSun` / macOS `Songti SC`，缺失时 `Noto Serif SC`），MUST NOT 出现隶书等不受控的回退字体

#### Scenario: 代码块字体零警告编译

- **WHEN** 在已注入捆绑字体的环境编译含中文代码块的模板
- **THEN** 编译成功且 `warnings` 为空（无 `unknown font family`）
