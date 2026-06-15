## MODIFIED Requirements

### Requirement: 报告产物持久落盘于项目目录

HTML 报告的所有交付文件（HTML、CSS、JS、assets）MUST 通过 `fs_write` / `fs_patch`（或等效沙箱写工具）写入用户选定的**项目根目录**内，并持久保留于磁盘。报告产物 MUST NOT 写入 `.cache/skill-run/` 或任何仅在 turn 内存在、会被自动清理的临时目录。

#### Scenario: fs_write 写入报告

- **WHEN** Agent 调用 `fs_write` 写入 `reports/q1/index.html`
- **THEN** 文件存在于项目沙箱内，重启应用后仍可访问，且出现在项目文件浏览区

#### Scenario: 禁止临时目录

- **WHEN** Agent 按 html-report skill 生成报告
- **THEN** 产出路径段 MUST NOT 为 `.cache/skill-run` 或其子路径
