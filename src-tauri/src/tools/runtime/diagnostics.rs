use serde_json::{json, Value};

pub fn build_script_error(code: &str, detail: &str, script_path: Option<&str>) -> Value {
    let (line, column) = extract_line_column(detail).unwrap_or((0, 0));
    let source = if line > 0 {
        get_source_line(code, line)
    } else {
        None
    };
    let mut out = json!({
        "error": classify_error(detail),
        "detail": detail,
    });
    if line > 0 {
        out["line"] = json!(line);
    }
    if column > 0 {
        out["column"] = json!(column);
    }
    if let Some(src) = source.as_deref() {
        out["source"] = json!(src);
        let quotes = quote_diagnostics(src);
        if !quotes.is_empty() {
            out["quote_diagnostics"] = Value::Array(quotes);
        }
    }
    if let Some(path) = script_path {
        out["script_path"] = json!(path);
        out["hint"] = json!(format!(
            "Use fs_patch on {path} for local fixes (not fs_write), then rerun with skill_run {{\"path\":\"{path}\"}}."
        ));
    }
    out
}

fn classify_error(detail: &str) -> &'static str {
    let lower = detail.to_lowercase();
    if lower.contains("syntax") || lower.contains("unexpected") {
        "JavaScript parse error"
    } else {
        "JavaScript runtime error"
    }
}

fn extract_line_column(detail: &str) -> Option<(usize, usize)> {
    let lower = detail.to_lowercase();
    let line = parse_number_after(&lower, "line ")?;
    let column = parse_number_after(&lower, "col ")
        .or_else(|| parse_number_after(&lower, "column "))
        .unwrap_or(0);
    Some((line, column))
}

fn parse_number_after(haystack: &str, needle: &str) -> Option<usize> {
    let idx = haystack.find(needle)?;
    let rest = &haystack[idx + needle.len()..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn get_source_line(code: &str, line: usize) -> Option<String> {
    code.lines().nth(line.saturating_sub(1)).map(str::to_string)
}

fn quote_diagnostics(line: &str) -> Vec<Value> {
    line.char_indices()
        .filter_map(|(byte_idx, ch)| {
            let name = match ch {
                '"' => "QUOTATION MARK",
                '\'' => "APOSTROPHE",
                '“' => "LEFT DOUBLE QUOTATION MARK",
                '”' => "RIGHT DOUBLE QUOTATION MARK",
                '‘' => "LEFT SINGLE QUOTATION MARK",
                '’' => "RIGHT SINGLE QUOTATION MARK",
                _ => return None,
            };
            let column = line[..byte_idx].chars().count() + 1;
            let mut item = json!({
                "column": column,
                "char": ch.to_string(),
                "code_point": format!("U+{:04X}", ch as u32),
                "name": name,
            });
            if ch == '"' {
                if let Some(obj) = item.as_object_mut() {
                    obj.insert(
                        "note".into(),
                        json!("ASCII double quote may terminate a JavaScript string delimited with \"."),
                    );
                }
            }
            Some(item)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_diagnostics_report_ascii_and_smart_quotes() {
        let line = r#"p("简称"广软"），")"#;
        let quotes = quote_diagnostics(line);
        assert!(quotes.iter().any(|q| q["code_point"] == "U+0022"));
        assert!(quotes.iter().any(|q| q["name"] == "QUOTATION MARK"));
    }

    #[test]
    fn build_script_error_includes_source_and_path() {
        let code = "async function main() {\n  p(\"简称\"广软\"），\");\n}";
        let err = build_script_error(
            code,
            "SyntaxError: unexpected identifier at line 2, col 10",
            Some(".skill-run/script.js"),
        );
        assert_eq!(err["error"], "JavaScript parse error");
        assert_eq!(err["line"], 2);
        assert_eq!(err["script_path"], ".skill-run/script.js");
        assert!(err["source"].as_str().unwrap_or("").contains("广软"));
        assert!(err.get("quote_diagnostics").is_some());
    }

    #[test]
    fn extract_line_column_parses_boa_style_message() {
        assert_eq!(
            extract_line_column("SyntaxError: foo at line 204, col 58"),
            Some((204, 58))
        );
    }
}
