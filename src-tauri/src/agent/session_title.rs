use crate::core::project_files::{text_contains_document_extension, DOCUMENT_EXTENSIONS};

/// 侧栏 w-72 混排路径/英文时约 18 字可完整展示
const MAX_TITLE_CHARS: usize = 18;

const FILLER_PHRASES: &[&str] = &[
    "里面", "其中", "所有", "一下", "这个", "那个", "的内容", "的数据", "的信息", "专业的",
    "目录里", "文件夹",
];

const GENERIC_OPENERS: &[&str] = &[
    "你好", "您好", "嗨", "在吗", "你是谁", "介绍一下", "hi", "hello", "hey",
];

const TASK_VERBS: &[&str] = &[
    "分析", "总结", "转换", "导出", "合并", "列出", "查看", "读取", "修改", "生成", "整理", "处理",
    "打开", "编写", "撰写", "提取", "对比", "比较", "检查", "审核", "翻译", "格式化", "查询",
];

pub fn is_default_session_title(title: &str) -> bool {
    title == "新会话" || title.starts_with("会话 ")
}

pub(crate) fn is_autotitle_eligible_user_count(user_count: usize) -> bool {
    user_count == 1 || user_count == 2
}

fn first_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
}

fn strip_markdown_prefix(line: &str) -> String {
    let mut s = line.trim();
    while s.starts_with('#') {
        s = s.trim_start_matches('#').trim();
    }
    s = s.trim_start_matches(['-', '*', '•', '>']).trim();
    strip_inline_markdown(s)
}

/// 去除行内 Markdown 标记（加粗/斜体/行内代码/删除线/链接），用于纯文本标题
fn strip_inline_markdown(line: &str) -> String {
    let mut s = line.to_string();
    for marker in ["**", "__", "~~", "`", "*"] {
        s = s.replace(marker, "");
    }
    // [文本](链接) → 文本
    while let Some(open) = s.find('[') {
        let Some(close_rel) = s[open..].find("](") else {
            break;
        };
        let close = open + close_rel;
        let Some(end_rel) = s[close..].find(')') else {
            break;
        };
        let end = close + end_rel;
        let text = s[open + 1..close].to_string();
        s.replace_range(open..=end, &text);
    }
    s.trim().to_string()
}

fn normalize_for_opener_check(text: &str) -> String {
    text.chars()
        .filter(|c| {
            !c.is_ascii_punctuation()
                && !matches!(
                    c,
                    '，' | '。' | '！' | '？' | '、' | '；' | '：' | '…' | '（' | '）' | '【' | '】'
                )
        })
        .flat_map(char::to_lowercase)
        .collect()
}

fn has_substantive_markers(text: &str) -> bool {
    if text_contains_document_extension(text) {
        return true;
    }
    if text
        .split_whitespace()
        .any(|token| token.len() >= 2 && token.contains('.'))
    {
        return true;
    }
    if text.split_whitespace().any(|token| {
        let upper = token.to_uppercase();
        upper.len() > 2
            && upper.starts_with("SK")
            && upper.chars().skip(2).all(|c| c.is_ascii_digit())
    }) {
        return true;
    }
    TASK_VERBS.iter().any(|verb| text.contains(verb))
}

pub fn is_generic_opener(text: &str) -> bool {
    let cleaned = strip_markdown_prefix(first_line(text));
    let normalized = normalize_for_opener_check(&cleaned);
    if normalized.is_empty() {
        return true;
    }
    if GENERIC_OPENERS.iter().any(|phrase| normalized == *phrase) {
        return true;
    }
    normalized.chars().count() <= 3 && !has_substantive_markers(&cleaned)
}

fn clean_question(text: &str) -> String {
    let mut s = strip_markdown_prefix(first_line(text));
    for prefix in [
        "请帮我",
        "请帮忙",
        "请",
        "帮我",
        "帮忙",
        "你好，",
        "你好,",
        "你好 ",
        "能否",
        "可以",
    ] {
        if s.starts_with(prefix) {
            s = s[prefix.len()..].trim().to_string();
        }
    }
    s
}

pub(crate) fn clean_intent(text: &str) -> String {
    compact_intent(&clean_question(text), true)
}

fn path_start_before_ext<'a>(before: &'a str) -> usize {
    before
        .rfind('/')
        .map(|i| i + 1)
        .or_else(|| {
            before.char_indices().rev().find_map(|(i, c)| {
                if c.is_whitespace() || matches!(c, '（' | '(' | '【' | '@') {
                    Some(i + c.len_utf8())
                } else {
                    None
                }
            })
        })
        .unwrap_or(0)
}

fn shorten_embedded_paths(text: &str) -> String {
    let mut result = text.to_string();
    loop {
        let lower = result.to_lowercase();
        let mut replaced = false;
        for ext in DOCUMENT_EXTENSIONS {
            let dotted = format!(".{ext}");
            let Some(idx) = lower.find(&dotted) else {
                continue;
            };
            let ext_end = idx + dotted.len();
            let path_start = path_start_before_ext(&result[..idx]);
            if !result[path_start..idx].contains('/') {
                continue;
            }
            let basename = result[path_start..ext_end].to_string();
            result.replace_range(path_start..ext_end, &basename);
            replaced = true;
            break;
        }
        if !replaced {
            break;
        }
    }
    result
}

fn find_filename(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let mut best: Option<(usize, String)> = None;
    for ext in DOCUMENT_EXTENSIONS {
        let dotted = format!(".{ext}");
        if let Some(idx) = lower.find(&dotted) {
            let ext_end = idx + dotted.len();
            let start = path_start_before_ext(&text[..idx]);
            let candidate = text[start..ext_end].to_string();
            if best.as_ref().is_none_or(|(len, _)| candidate.chars().count() > *len) {
                best = Some((candidate.chars().count(), candidate));
            }
        }
    }
    best.map(|(_, s)| s)
}

fn strip_leading_particles(s: &str) -> String {
    let mut t = s.trim();
    for p in ["中", "的", "与", "和", "及", "在", "对", "从", "把", "将"] {
        if t.starts_with(p) {
            t = t[p.len()..].trim();
        }
    }
    t.to_string()
}

fn compact_tail_after_filename(text: &str, filename: &str) -> String {
    let lower = text.to_lowercase();
    let fname_lower = filename.to_lowercase();
    let Some(idx) = lower.find(&fname_lower) else {
        return String::new();
    };
    let rest = text[idx + filename.len()..].trim();
    let rest = strip_leading_particles(rest);
    if rest.is_empty() {
        return String::new();
    }
    let tail: String = rest.chars().take(8).collect();
    strip_leading_particles(&tail)
}

fn fit_char_budget(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    text.chars().take(max).collect()
}

fn compact_intent(text: &str, strip_fillers: bool) -> String {
    let mut s = shorten_embedded_paths(text);
    if strip_fillers {
        for phrase in FILLER_PHRASES {
            s = s.replace(phrase, "");
        }
    }
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let s = s.trim();

    if let Some(fname) = find_filename(s) {
        let verb = TASK_VERBS.iter().find(|v| s.starts_with(**v)).copied();
        let tail = compact_tail_after_filename(s, &fname);
        let base = match verb {
            Some(v) => format!("{v} {fname}"),
            None => fname.clone(),
        };
        if tail.is_empty() {
            return fit_char_budget(base.trim(), MAX_TITLE_CHARS);
        }
        let spare = MAX_TITLE_CHARS.saturating_sub(base.chars().count() + 1);
        let short_tail = fit_char_budget(&tail, spare);
        return fit_char_budget(&format!("{base} {short_tail}"), MAX_TITLE_CHARS);
    }

    fit_char_budget(s, MAX_TITLE_CHARS)
}

fn is_assistant_boilerplate(text: &str) -> bool {
    let line = clean_intent(text);
    if line.starts_with("我是") && line.contains("助手") {
        return true;
    }
    let lower = line.to_lowercase();
    lower.contains("doc-agent") || lower.contains("办公文档助手")
}

pub fn truncate_title(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "新会话".to_string();
    }
    let count = trimmed.chars().count();
    if count <= MAX_TITLE_CHARS {
        return trimmed.to_string();
    }
    format!(
        "{}…",
        trimmed.chars().take(MAX_TITLE_CHARS).collect::<String>()
    )
}

fn optional_title(text: &str) -> Option<String> {
    let title = truncate_title(text);
    (title != "新会话").then_some(title)
}

pub fn summarize_session_title(user_text: &str, assistant_text: Option<&str>) -> Option<String> {
    let user_intent = clean_intent(user_text);
    if !is_generic_opener(user_text)
        && !is_generic_opener(&user_intent)
        && user_intent.chars().count() >= 2
    {
        return optional_title(&user_intent);
    }

    if let Some(assistant) = assistant_text {
        let line = compact_intent(&clean_question(assistant), false);
        if !is_generic_opener(assistant)
            && !is_assistant_boilerplate(assistant)
            && line.chars().count() >= 4
        {
            return optional_title(&line);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_default_titles() {
        assert!(is_default_session_title("新会话"));
        assert!(is_default_session_title("会话 1"));
        assert!(!is_default_session_title("课程资料概览"));
    }

    #[test]
    fn autotitle_eligible_only_first_two_rounds() {
        assert!(is_autotitle_eligible_user_count(1));
        assert!(is_autotitle_eligible_user_count(2));
        assert!(!is_autotitle_eligible_user_count(3));
    }

    #[test]
    fn generic_openers_are_detected() {
        assert!(is_generic_opener("你好"));
        assert!(is_generic_opener("在吗？"));
        assert!(is_generic_opener("Hello"));
        assert!(!is_generic_opener("分析 SK1002 课程归档资料"));
    }

    #[test]
    fn prefers_user_intent_over_assistant() {
        let title = summarize_session_title(
            "请帮我看看这个目录里有什么文件",
            Some("目录中包含课程大纲、考核方案与归档资料。"),
        );
        assert_eq!(title.as_deref(), Some("看看有什么文件"));
    }

    #[test]
    fn skips_generic_user_greeting() {
        let title = summarize_session_title("你好", Some("我是 **doc-agent**，你的办公文档助手。"));
        assert_eq!(title, None);
    }

    #[test]
    fn substantive_user_question() {
        let title = summarize_session_title("分析 SK1002 课程归档资料", None);
        assert_eq!(title.as_deref(), Some("分析 SK1002 课程归档资料"));
    }

    #[test]
    fn second_round_user_only() {
        let title = summarize_session_title("分析 SK1002 归档资料", None);
        assert_eq!(title.as_deref(), Some("分析 SK1002 归档资料"));
    }

    #[test]
    fn second_round_still_generic() {
        let title = summarize_session_title("在吗", None);
        assert_eq!(title, None);
    }

    #[test]
    fn strips_inline_markdown_from_titles() {
        let title = summarize_session_title("看下 `tasks.md`", None);
        assert_eq!(title.as_deref(), Some("tasks.md"));

        let title = summarize_session_title("参考 [文档](https://example.com) 修改", None);
        assert_eq!(title.as_deref(), Some("参考 文档 修改"));
    }

    #[test]
    fn compacts_path_and_filename_for_sidebar() {
        let title = summarize_session_title(
            "列出normalized/课程负责人.csv中软件工程专业的负责人",
            None,
        );
        let t = title.expect("expected title");
        assert!(t.chars().count() <= MAX_TITLE_CHARS);
        assert!(t.contains("课程负责人.csv"));
        assert!(!t.contains("normalized/"));
        assert!(t.contains("列出"));
    }

    #[test]
    fn truncates_long_titles_to_max_chars() {
        let title = truncate_title("这是一段非常非常非常非常非常非常长的会话标题内容还需要更长");
        assert!(title.chars().count() <= MAX_TITLE_CHARS + 1);
        assert_ne!(
            title,
            "这是一段非常非常非常非常非常非常长的会话标题内容还需要更长"
        );
    }

    #[test]
    fn assistant_fallback_when_user_generic_but_assistant_has_substance() {
        let title = summarize_session_title(
            "你好",
            Some("目录中包含课程大纲与考核方案。"),
        );
        assert_eq!(title.as_deref(), Some("目录中包含课程大纲与考核方案。"));
    }

    #[test]
    fn double_prefix_strip_leaves_generic_intent_returns_none() {
        // "请能否在吗" 经双重剥离: "请" → "能否在吗" → "在吗"
        // 原文不是短泛化词，但剥离后是 → 应返回 None
        let title = summarize_session_title("请能否在吗", None);
        assert_eq!(title, None);
    }

    #[test]
    fn double_prefix_strip_leaves_substantive_intent_returns_title() {
        // "请能否分析 SK1002" 剥离后 "分析 SK1002" 仍有实质内容
        let title = summarize_session_title("请能否分析 SK1002", None);
        assert!(title.is_some());
        let t = title.unwrap();
        assert!(t.contains("SK1002") || t.contains("分析"), "title: {t}");
    }

    #[test]
    fn polite_opener_followed_by_greeting_is_none() {
        // 带礼貌前缀的泛化开场，用户只是打招呼
        let title = summarize_session_title("你好，在吗？", None);
        assert_eq!(title, None);
    }

    #[test]
    fn polite_opener_followed_by_task_uses_task_as_title() {
        // "你好，帮我分析报告.docx" → 礼貌开头 + 实质任务
        let title = summarize_session_title("你好，帮我分析报告.docx", None);
        assert!(title.is_some(), "expected a title but got None");
        let t = title.unwrap();
        assert!(t.contains("报告") || t.contains("分析"), "title: {t}");
    }
}
