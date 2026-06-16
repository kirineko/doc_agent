## ADDED Requirements

### Requirement: 启动清理 stale updater 临时文件

系统 SHALL 在每次应用启动后，于**后台线程**扫描操作系统临时目录（如 Windows `%TEMP%`、macOS `$TMPDIR`）的**顶层条目**，删除匹配 updater 产物 pattern 且**创建时间**早于 **24 小时**的文件或目录。清理 MUST NOT 阻塞 UI 线程或 setup 主线程；任何单条删除失败 MUST 静默忽略；整个清理过程 MUST 快速完成（仅扫描 temp 顶层，条目数上限 512，无重试、无用户可见错误）。无法读取创建时间的条目 MUST 跳过。

匹配 pattern MUST 严格对应 Tauri 实际产物命名（`productName` = `DocAgent`，CalVer `YYYY.M.D` 无段前导零）：

1. **Updater 临时目录**（`tauri-plugin-updater` `make_temp_dir`）：`DocAgent-{CalVer}-updater-{random}/`
2. **Updater 临时安装包**（`write_to_temp`）：`DocAgent-{CalVer}-installer.exe`
3. **NSIS 发布/更新包**（bundler）：`DocAgent_{CalVer}_x64-setup.exe`

MUST NOT 匹配泛化的 `DocAgent-*.exe`、`.msi`、或其他应用含 `-updater` 的文件名。

#### Scenario: 启动后后台清理

- **WHEN** 应用完成 `setup` 并进入主界面
- **THEN** 清理逻辑已在 detached 后台线程启动且 setup 未同步等待其完成

#### Scenario: 删除 24 小时前的 updater 目录

- **WHEN** temp 顶层存在 `DocAgent-2026.6.1-updater-abc123/` 且其创建时间早于 24 小时前
- **THEN** 系统 MUST 尝试删除该目录及其内容

#### Scenario: 保留 24 小时内产物

- **WHEN** temp 中存在当日 updater 临时目录或安装包
- **THEN** 系统 MUST NOT 删除该条目

#### Scenario: 清理失败不干扰用户

- **WHEN** 某条目删除因权限或文件占用失败
- **THEN** 应用 MUST 正常继续启动，不展示错误 dialog

#### Scenario: 扫描范围受限

- **WHEN** 执行清理
- **THEN** 仅扫描系统 temp 目录顶层条目，MUST NOT 递归扫描整个磁盘
