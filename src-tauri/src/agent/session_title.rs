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
    s = s.trim_start_matches(['-', '*', '•']).trim();
    s.to_string()
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
    fn truncates_long_titles() {
        let title = truncate_title("这是一段非常非常非常非常非常非常长的会话标题内容还需要更长");
        assert!(title.chars().count() <= MAX_TITLE_CHARS + 1);
        assert_ne!(title, "这是一段非常非常非常非常非常非常长的会话标题内容还需要更长");
    }
}
