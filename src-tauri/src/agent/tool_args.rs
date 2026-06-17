use serde_json::{json, Value};

/// Parse tool-call arguments JSON. Returns structured diagnostics on failure.
pub fn parse_tool_arguments(tool_name: &str, raw: &str) -> Result<Value, Value> {
    serde_json::from_str(raw).map_err(|err| {
        json!({
            "error": "invalid tool arguments JSON",
            "detail": err.to_string(),
            "line": err.line(),
            "column": err.column(),
            "snippet": snippet_around_line_column(raw, err.line(), err.column(), 120),
            "hint": argument_parse_hint(tool_name),
        })
    })
}

pub fn truncation_error(tool_name: &str, raw_arguments: &str) -> Value {
    json!({
        "error": "tool call truncated",
        "tool": tool_name,
        "received_argument_chars": raw_arguments.chars().count(),
        "hint": "The model output ended before tool arguments were complete. Retry with a shorter inline script, or fs_write the script to a project-relative .js file and rerun with skill_run {\"path\":\"...\"}, or fs_patch the script_path from a prior failed inline run and rerun with that path."
    })
}

fn argument_parse_hint(tool_name: &str) -> &'static str {
    match tool_name {
        "skill_run" => "If this is skill_run code, avoid embedding long double-quoted text in JSON arguments. Use single-quoted JavaScript strings, or fs_patch the script_path from the prior failed run and rerun with skill_run {\"path\":\"<script_path>\"}.",
        _ => "Ensure tool arguments are valid JSON.",
    }
}

fn snippet_around_line_column(raw: &str, line: usize, column: usize, max_len: usize) -> String {
    if line == 0 {
        return truncate(raw, max_len);
    }
    let target_line = raw.lines().nth(line.saturating_sub(1)).unwrap_or(raw);
    let col_idx = column.saturating_sub(1).min(target_line.len());
    let start = target_line
        .char_indices()
        .nth(col_idx.saturating_sub(max_len / 2))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let end = target_line
        .char_indices()
        .nth((col_idx + max_len / 2).min(target_line.len()))
        .map(|(i, _)| i)
        .unwrap_or(target_line.len());
    truncate(&target_line[start..end], max_len)
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut out = s.chars().take(max_len).collect::<String>();
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_json() {
        let v = parse_tool_arguments("fs_list", r#"{"path":"."}"#).unwrap();
        assert_eq!(v["path"], ".");
    }

    #[test]
    fn parse_invalid_skill_run_arguments() {
        let raw = r#"{"code":"p(\"简称\"广软\"），")"#;
        let err = parse_tool_arguments("skill_run", raw).unwrap_err();
        assert_eq!(err["error"], "invalid tool arguments JSON");
        assert!(err["line"].as_u64().unwrap_or(0) >= 1);
        assert!(err["column"].as_u64().unwrap_or(0) >= 1);
        assert!(err["snippet"].as_str().unwrap_or("").contains("广软"));
        assert!(err["hint"].as_str().unwrap_or("").contains("skill_run"));
    }

    #[test]
    fn truncation_error_includes_char_count() {
        let err = truncation_error("skill_run", "{\"code\":\"abc");
        assert_eq!(err["error"], "tool call truncated");
        assert_eq!(err["received_argument_chars"], 12);
    }
}
