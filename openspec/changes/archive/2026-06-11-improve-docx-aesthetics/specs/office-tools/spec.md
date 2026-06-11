# office-tools Spec Delta

## REMOVED Requirements

### Requirement: Word 文档生成
**Reason**: `word_create` 的 markdown → office_oxide 转换路径产出无样式、无中文字体配置的低质量文档，且作为「一步到位」捷径系统性地诱导模型绕过高质量的 skill_run + docx-js 路径。生成 Word 收敛到唯一路径以保证美观性。
**Migration**: 生成 Word 文档统一使用 `skill_read {"skill":"docx"}` 获取规范后，经 `skill_run` + 内置 docx 库生成；最小模板见 docx SKILL.md「Creating New Documents」章节。历史会话中的 `word_create` 调用记录仅作展示，不受影响。
