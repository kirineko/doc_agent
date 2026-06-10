const MAX_TITLE_CHARS: usize = 24;

pub fn is_default_session_title(title: &str) -> bool {
    title == "新会话" || title.starts_with("会话 ")
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

pub fn summarize_session_title(user_text: &str, assistant_text: Option<&str>) -> String {
    if let Some(assistant) = assistant_text {
        let line = clean_question(assistant);
        if line.chars().count() >= 4 {
            return truncate_title(&line);
        }
    }
    truncate_title(&clean_question(user_text))
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
    fn summarizes_from_assistant_first() {
        let title = summarize_session_title(
            "请帮我看看这个目录里有什么文件",
            Some("目录中包含课程大纲、考核方案与归档资料。"),
        );
        assert_eq!(title, "目录中包含课程大纲、考核方案与归档资料。");
    }

    #[test]
    fn falls_back_to_user_question() {
        let title = summarize_session_title("分析 SK1002 课程归档资料", None);
        assert_eq!(title, "分析 SK1002 课程归档资料");
    }

    #[test]
    fn strips_inline_markdown_from_titles() {
        let title = summarize_session_title(
            "你好",
            Some("我是 **doc-agent**，你的办公文档助手。"),
        );
        assert_eq!(title, "我是 doc-agent，你的办公文档助手。");

        let title = summarize_session_title("看下 `tasks.md`", None);
        assert_eq!(title, "看下 tasks.md");

        let title = summarize_session_title("参考 [文档](https://example.com) 修改", None);
        assert_eq!(title, "参考 文档 修改");
    }

    #[test]
    fn truncates_long_titles() {
        let title = truncate_title("这是一段非常非常非常非常非常非常长的会话标题内容还需要更长");
        assert!(title.chars().count() <= MAX_TITLE_CHARS + 1);
        assert_ne!(
            title,
            "这是一段非常非常非常非常非常非常长的会话标题内容还需要更长"
        );
    }
}
