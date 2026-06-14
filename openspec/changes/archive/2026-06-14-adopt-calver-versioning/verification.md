# 验证：adopt-calver-versioning

## 3.1 版本格式自检（`project-versioning`）

运行单元测试：

```bash
npm test -- src/lib/version.test.ts src/lib/updater.test.ts
```

| 版本 | 预期 |
|------|------|
| `2026.6.14` | ✅ 合法 |
| `2026.6.1` | ✅ 合法 |
| `2026.6.9` | ✅ 合法 |
| `2026.12.31` | ✅ 合法 |
| `2026.06.14` | ❌ 非法（月前导零） |
| `2026.6.09` | ❌ 非法（日前导零） |
| `1.0.1` | ❌ 非 CalVer 形状（历史 SemVer 已发布 tag 除外） |

当日 CalVer tag：

```bash
npm run calver:today
```

## 3.2 首个 CalVer 发布后检查清单

- [ ] `curl -s https://doc-agent.oss-cn-guangzhou.aliyuncs.com/latest.json | jq .version` 为当日 `YYYY.M.D`
- [ ] OSS 存在 `releases/<YYYY.M.D>/` 且含 Windows `*-setup.exe` + `.sig`
- [ ] GitHub Release 标题与 Assets 正常
- [ ] 已装 `1.0.1` 客户端：设置抽屉显示最新版本 → 更新 → 重启后版本为 CalVer
- [ ] CHANGELOG `[Unreleased]` 改为正式版本节标题
