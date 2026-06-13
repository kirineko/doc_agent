## 1. 版本与密钥准备

- [x] 1.1 运行 `tauri signer generate` 生成 updater 密钥对，安全备份私钥
- [x] 1.2 将 `TAURI_SIGNING_PRIVATE_KEY`（及可选 `PASSWORD`）、`ALIYUN_ACCESS_KEY_ID`、`ALIYUN_ACCESS_KEY_SECRET`、`OSS_BUCKET`、`OSS_REGION` 写入 GitHub Secrets
- [x] 1.3 将版本号升至 `1.0.0`（`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`）
- [x] 1.4 清理 OSS 中 `0.0.0-test` 测试对象（若仍存在）

## 2. Tauri Updater 后端配置

- [x] 2.1 添加 `tauri-plugin-updater`、`tauri-plugin-process` 依赖（Rust + npm）
- [x] 2.2 在 `lib.rs` 注册 updater 与 process 插件（仅 desktop）
- [x] 2.3 配置 `tauri.conf.json`：`createUpdaterArtifacts: true`、`plugins.updater`（pubkey、endpoint、windows `installMode: passive`）
- [x] 2.4 更新 `capabilities/default.json`：添加 `updater:default`、`process:default`

## 3. Release CI（OSS + GH Release）

- [x] 3.1 `build` job 注入 `TAURI_SIGNING_PRIVATE_KEY` 环境变量
- [x] 3.2 `build` job 扩展 artifact：收集 macOS `*.app.tar.gz` + `.sig`、Windows `*-setup.exe` + `.sig`（`updater-*` artifact）
- [x] 3.3 `publish` job 下载全部 artifacts，编写脚本生成 `latest.json`（OSS URL、`darwin-aarch64` + `windows-x86_64`）
- [x] 3.4 `publish` job 安装/调用 `ossutil`，上传 `releases/<version>/` 与根 `latest.json`（`Cache-Control: no-cache`）
- [x] 3.5 保留 `softprops/action-gh-release` 上传安装包至 GitHub Release
- [x] 3.6 单平台构建失败时阻止覆盖 OSS 根 `latest.json`

## 4. 前端更新体验

- [x] 4.1 新增 `src/lib/updater.ts`：`checkForUpdates`（check → dialog → downloadAndInstall → relaunch）
- [x] 4.2 应用启动后延迟触发静默检查（避免阻塞首屏）
- [x] 4.3 `Sidebar` 增加「检查更新」入口及进行中状态
- [x] 4.4 错误场景 dialog 提示（下载失败、无网络等）

## 5. 文档与验证

- [x] 5.1 更新 README：OSS 为主下载源、GH Release 为备、1.0.0 为自动更新基线
- [x] 5.2 本地 `npm run tauri build` 验证 updater 产物与 `.sig` 生成
- [x] 5.3 跑通 CI 自检：`cargo fmt/clippy/test`、`npm run typecheck/test/build`
- [ ] 5.4 推送 tag `1.0.0` 后验证 OSS `latest.json` 匿名可访问、安装包 URL 有效
- [ ] 5.5 （可选）发布 `1.0.1` 验证应用内更新闭环
