# retain-skill-run-scripts

## 动机

成功执行的 `skill_run` 脚本在 turn 结束时被清理，导致下一轮 Agent 按历史 `script_path` 做 `fs_read`/`fs_patch`（如修改 PPT）时文件已不存在。`skill_run.code` 每次都会覆盖 `script.js`，保留成功脚本不会累积垃圾文件。

## 纳入

- 成功执行后保留 `.cache/skill-run/<session_key>/script.js`（含纯计算脚本）
- 成功时清除 `error.json`（修复成功后清理失败现场）
- 仅在用户 **cancel turn** 时删除 session scratch 目录
- 更新 script-runtime spec 与相关 skill 文案

## 排除

- 删除会话时清理 `.cache/skill-run/`（另案）
- 会话列表排序（保持现状）
