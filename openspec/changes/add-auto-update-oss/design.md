# 设计：应用内自动更新 + 阿里云 OSS 分发

## Context

- 当前版本 **0.2.0**；发版流水线：`check` → 矩阵 `build`（macOS aarch64 + Windows x86_64）→ `publish` 上传 GitHub Release。
- 无 updater 插件、无代码签名、无 OSS 集成。
- 已验证 OSS Bucket `doc-agent`（`oss-cn-guangzhou`）：`public-read`、匿名 HTTPS 下载正常；本机 5MB 测试约 15 MB/s；**未接 CDN**，MVP 可 OSS 直连。
- 用户决策：首次安装走 OSS；GH Release 保留；Windows updater 用 **NSIS `.exe`**；从 **1.0.0** 启用自动更新。

## Goals / Non-Goals

**Goals：**

- 1.0.0 起，已安装用户可通过应用内 updater 获取后续版本。
- 国内用户可从 OSS 下载首次安装包（`.dmg` / `*-setup.exe`）与 updater 包。
- CI 在 GitHub Actions 完成构建与签名，自动上传 OSS 并生成 `latest.json`。
- 保留 GitHub Release 作为版本记录与备用下载源。
- 启动时自动检查更新，用户确认后下载安装并重启。

**Non-Goals：**

- 阿里云 CDN、自定义域名（Phase 2 优化）。
- macOS Developer ID 公证、Windows Authenticode（独立 milestone；无签名时 Gatekeeper/SmartScreen 仍可能拦截）。
- Linux 安装包与更新。
- Intel macOS（`x86_64-apple-darwin`）构建。
- 静默强制更新、beta/stable 多渠道、版本跳过记忆。

## Decisions

### D1：官方 `tauri-plugin-updater` + 静态 `latest.json`

使用 Tauri 2 官方插件，endpoint 指向 OSS 根路径 `latest.json`。

```json
{
  "bundle": { "createUpdaterArtifacts": true },
  "plugins": {
    "updater": {
      "pubkey": "<TAURI_SIGNER_PUBLIC_KEY>",
      "endpoints": [
        "https://doc-agent.oss-cn-guangzhou.aliyuncs.com/latest.json"
      ],
      "windows": { "installMode": "passive" }
    }
  }
}
```

**备选**：自建下载逻辑 — 拒绝，重复实现签名校验与跨平台安装。

### D2：双通道分发（OSS 主 + GitHub Release 备）

```
push tag 1.0.0
    │
    ├─ build 矩阵（mac / win）
    │     ├─ 安装包：.dmg, *-setup.exe, .msi
    │     └─ updater：.app.tar.gz + .sig, *-setup.exe + .sig
    │
    └─ publish job
          ├─ 生成 latest.json（URL 指向 OSS）
          ├─ ossutil 上传 OSS
          └─ softprops/action-gh-release（保留现有）
```

OSS 路径：

```
oss://doc-agent/
├── latest.json                          # Cache-Control: no-cache
└── releases/<version>/
    ├── DocAgent_<ver>_aarch64.dmg
    ├── DocAgent_<ver>_x64-setup.exe
    ├── DocAgent_<ver>_aarch64.app.tar.gz
    ├── DocAgent_<ver>_aarch64.app.tar.gz.sig
    ├── DocAgent_<ver>_x64-setup.exe.sig
    └── …
```

`latest.json` 中 `platforms` 仅包含 `darwin-aarch64` 与 `windows-x86_64`。

### D3：保留现有三阶段 CI，publish job 扩展 OSS 步骤

不整体替换为 `tauri-action` 矩阵直发，而是在现有 `publish` job 中：

1. 下载 `bundles-*` 与新增 `updater-*` artifacts
2. Bash 脚本读取 `.sig` 内容，组装 `latest.json`
3. `ossutil` 上传 `releases/<version>/` 与根 `latest.json`
4. 继续 `action-gh-release` 上传全部安装包

**备选**：全面改用 `tauri-action` — 拒绝，与现有 `check` 门禁与 artifact 聚合结构冲突大。

### D4：Updater 签名密钥（独立于 OS 代码签名）

```bash
npm run tauri signer generate -- -w ~/.tauri/doc-agent.key
```

- 公钥写入 `tauri.conf.json` 的 `plugins.updater.pubkey`
- 私钥存 GitHub Secret `TAURI_SIGNING_PRIVATE_KEY`
- **私钥丢失则无法向已安装用户推送更新**，须安全备份

构建时 `TAURI_SIGNING_PRIVATE_KEY` 注入 `build` 矩阵各 job。

### D5：各平台 updater 产物

| 平台 | 首次安装 | Updater 下载 |
|------|----------|--------------|
| macOS aarch64 | `.dmg` | `*.app.tar.gz` + `.sig` |
| Windows x86_64 | `*-setup.exe`（主）、`.msi`（备） | `*-setup.exe` + `.sig` |

`build` job 在 `upload-artifact` 时额外收集 updater 目录产物。

### D6：前端更新流程

```text
App mount（延迟 3s 或 idle）
  └─ check() from @tauri-apps/plugin-updater
       ├─ 无更新 → 静默结束
       └─ 有更新 → dialog 确认
            ├─ 取消 → 结束
            └─ 确认 → downloadAndInstall（进度回调）
                 └─ relaunch()

Sidebar「检查更新」→ 同一 check 逻辑，手动触发
```

复用已有 `tauri-plugin-dialog`；新增 `tauri-plugin-process` 用于 `relaunch()`。

### D7：版本基线 1.0.0

- 实现本 change 时同步将 `version` 升至 `1.0.0`
- Tag 格式保持仓库规范：`1.0.0`（无 `v` 前缀）
- **1.0.0 之前版本无 updater**，用户需从 OSS/GH 手动安装 1.0.0 获得后续自动更新能力

### D8：OSS 与 CDN

MVP 使用 OSS 外网 endpoint，已验证满足 HTTPS 与公共读要求。CDN 作为 Phase 2：当全国用户反馈慢或流量成本上升时再接入；接入时 `latest.json` 必须配置不缓存。

### D9：GitHub Secrets

| Secret | 用途 |
|--------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | 构建签名 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码（可选） |
| `ALIYUN_ACCESS_KEY_ID` | OSS 上传（RAM 子账号） |
| `ALIYUN_ACCESS_KEY_SECRET` | OSS 上传 |
| `OSS_BUCKET` | `doc-agent` |
| `OSS_REGION` | `oss-cn-guangzhou`（ossutil `-e` 用 `oss-cn-guangzhou.aliyuncs.com`） |

RAM 子账号最小权限：`doc-agent` Bucket 的 `PutObject`、`ListObjects`、`GetObject`。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| updater 私钥丢失 | 安全备份；文档注明轮换流程 |
| `latest.json` 被 CDN/浏览器缓存（未来） | 上传 `--cache-control no-cache`；CDN 规则单独配置 |
| 只上传 `.dmg` 漏传 `.tar.gz` | CI 分 `updater-*` artifact 清单验收 |
| macOS 无公证导致更新后 Gatekeeper 警告 | 提案外 milestone；README 说明 |
| 1.0.0 前用户无法自动升级 | README/OSS 提供 1.0.0 基线包下载说明 |
| OSS 测试文件残留 | 发版前清理 `0.0.0-test` 对象 |
| Windows 安装前应用自动退出 | 利用 `on_before_exit` 或依赖 Tauri 默认行为；避免更新中发起 Agent 任务 |

## Migration Plan

1. 生成 updater 密钥对，配置 GitHub Secrets 与 `tauri.conf.json` pubkey
2. 合并本 change，版本升至 1.0.0
3. 推送 tag `1.0.0` 触发 Release workflow
4. 验证 OSS `latest.json` 与安装包 URL 可匿名访问
5. 从 OSS 安装 1.0.0，模拟发布 `1.0.1` tag，验证应用内更新闭环
6. 更新 README：OSS 下载链接为主，GH Release 为备

回滚：关闭 `plugins.updater` endpoint 或 revert CI OSS 步骤；已安装 1.0.0 用户需手动重装。

## Open Questions

- （已决）OSS 广州、无 CDN MVP、GH Release 保留、Windows NSIS `.exe`、1.0.0 基线
- （实现时）更新提示文案与 i18n — 中文为主，与现有 UI 一致即可
- （后续）macOS 公证 + Windows 签名 milestone 名称与优先级
