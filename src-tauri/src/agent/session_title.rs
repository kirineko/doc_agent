pub const MAX_STORED_TITLE_CHARS: usize = 120;

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

fn strip_inline_markdown(line: &str) -> String {
    let mut s = line.to_string();
    for marker in ["**", "__", "~~", "`", "*"] {
        s = s.replace(marker, "");
    }
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

/// 清洗首轮 user 文本：取首行并去除 Markdown，不做展示级截断。
pub fn normalize_first_turn(text: &str) -> String {
    strip_markdown_prefix(first_line(text))
}

pub fn truncate_for_storage(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "新会话".to_string();
    }
    let count = trimmed.chars().count();
    if count <= MAX_STORED_TITLE_CHARS {
        return trimmed.to_string();
    }
    format!(
        "{}…",
        trimmed
            .chars()
            .take(MAX_STORED_TITLE_CHARS)
            .collect::<String>()
    )
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
    fn normalize_strips_markdown() {
        assert_eq!(normalize_first_turn("**报告** `draft`"), "报告 draft");
        assert_eq!(
            normalize_first_turn("参考 [文档](https://example.com)"),
            "参考 文档"
        );
    }

    #[test]
    fn truncate_for_storage_at_120_chars() {
        let long = "一".repeat(130);
        let title = truncate_for_storage(&long);
        assert!(title.chars().count() <= MAX_STORED_TITLE_CHARS + 1);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn truncate_empty_becomes_default() {
        assert_eq!(truncate_for_storage("   "), "新会话");
    }
}
