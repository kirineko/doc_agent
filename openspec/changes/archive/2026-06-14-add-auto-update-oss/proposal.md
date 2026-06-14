# 提案：应用内自动更新 + 阿里云 OSS 分发（add-auto-update-oss）

## Why

国内用户访问 GitHub Releases 不稳定，无法可靠获取安装包与更新；同时 Doc Agent 即将发布 **1.0.0**，需要从该版本起提供应用内自动更新能力，降低用户手动下载、重装成本。构建仍使用 GitHub Actions，**分发与 updater endpoint 迁移至阿里云 OSS（广州）**，GitHub Release 保留作版本记录与海外备用渠道。

## What Changes

- 集成 **Tauri v2 官方 `tauri-plugin-updater`**，从 **1.0.0** 起客户端支持检查、下载、安装更新并重启。
- 配置 **updater 签名密钥对**（minisign），构建时生成 `.sig` 与 updater 产物。
- **Release CI** 改造：矩阵构建后上传安装包与 updater 包至 OSS，生成并发布 `latest.json`；同步保留 GitHub Release Assets。
- **OSS 目录**：`latest.json`（根路径）+ `releases/<version>/`（安装包与 updater 包）。
- **Updater endpoint**：`https://doc-agent.oss-cn-guangzhou.aliyuncs.com/latest.json`（已验证公共 HTTPS 可达，MVP 不依赖 CDN）。
- 前端：应用启动时静默检查更新，发现新版本后通过 **dialog** 询问用户是否安装；侧栏提供「检查更新」入口。
- 版本号升至 **1.0.0**（`package.json`、`src-tauri/Cargo.toml`、`tauri.conf.json` 与 tag 一致）。
- **排除**：CDN / 自定义域名、动态更新服务器、beta 渠道、macOS 公证 / Windows Authenticode（另立 milestone）、Linux 安装包。

## Capabilities

### New Capabilities

- `app-updater`：应用内更新检查、用户确认、下载安装、重启及错误处理契约。
- `release-distribution`：GitHub Actions 构建签名、OSS 上传、`latest.json` 生成与双通道（OSS + GH Release）发布契约。

### Modified Capabilities

- `workspace-ui`：侧栏增加「检查更新」入口及更新进行中的状态展示。

## Impact

- **Rust**：`tauri-plugin-updater`、`tauri-plugin-process`；`lib.rs` 注册插件；`capabilities/default.json` 权限。
- **前端**：`@tauri-apps/plugin-updater`、`@tauri-apps/plugin-process`；新增 `src/lib/updater.ts`（或等效模块）；`App.tsx` / `Sidebar` 集成。
- **配置**：`tauri.conf.json`（`createUpdaterArtifacts`、`plugins.updater`）。
- **CI**：`.github/workflows/release.yml`（签名环境变量、updater 产物收集、OSS 上传、`latest.json` 生成）。
- **Secrets（GitHub）**：`TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（可选）、`ALIYUN_ACCESS_KEY_ID`、`ALIYUN_ACCESS_KEY_SECRET`、`OSS_BUCKET`、`OSS_REGION`。
- **基础设施**：阿里云 OSS Bucket `doc-agent`（`oss-cn-guangzhou`，公共读）。
- **风险**：updater 私钥丢失将无法向已安装用户推送更新；1.0.0 之前版本不含 updater，需用户手动安装 1.0.0 基线包。
