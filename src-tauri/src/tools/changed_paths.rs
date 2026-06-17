use serde_json::Value;

/// Extract project-relative paths changed by a successful tool invocation.
pub fn extract_changed_paths(tool_name: &str, args: &Value, result: &Value) -> Vec<String> {
    let mut paths = Vec::new();
    match tool_name {
        "fs_write" | "fs_patch" | "excel_write" => {
            push_arg_path(&mut paths, args, "path");
        }
        "ooxml_unpack" => {
            if args.get("out_dir").and_then(|v| v.as_str()).is_some() {
                push_arg_path(&mut paths, args, "out_dir");
            } else {
                push_result_path(&mut paths, result, "out_dir");
            }
        }
        "ooxml_pack" => {
            push_arg_path(&mut paths, args, "out_path");
        }
        "docx_accept_changes" => {
            // out_path 缺省时工具就地覆写 path
            if args.get("out_path").and_then(|v| v.as_str()).is_some() {
                push_arg_path(&mut paths, args, "out_path");
            } else {
                push_arg_path(&mut paths, args, "path");
            }
        }
        "office_convert" => {
            push_result_path(&mut paths, result, "path");
        }
        "pdf_merge" | "pdf_split" | "pdf_rotate" | "pdf_delete_pages" => {
            push_arg_path(&mut paths, args, "out_path");
            push_arg_path(&mut paths, args, "out_dir");
        }
        "html_to_pdf" | "typst_to_pdf" => {
            push_arg_path(&mut paths, args, "out_path");
        }
        "docx_extract_table" => {
            push_arg_path(&mut paths, args, "out_dir");
        }
        "excel_normalize" | "data_query" => {
            push_arg_path(&mut paths, args, "out_path");
        }
        "skill_run" => {
            // runtime 将 __doc_write 写入的相对路径放入 result.written_paths
            if let Some(items) = result.get("written_paths").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(path) = item.as_str() {
                        push_normalized(&mut paths, path);
                    }
                }
            }
        }
        _ => {}
    }
    dedupe_paths(paths)
}

fn push_arg_path(out: &mut Vec<String>, args: &Value, key: &str) {
    if let Some(path) = args.get(key).and_then(|v| v.as_str()) {
        push_normalized(out, path);
    }
}

fn push_result_path(out: &mut Vec<String>, result: &Value, key: &str) {
    if let Some(path) = result.get(key).and_then(|v| v.as_str()) {
        push_normalized(out, path);
    }
}

fn push_normalized(out: &mut Vec<String>, path: &str) {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return;
    }
    out.push(trimmed.replace('\\', "/"));
}

fn dedupe_paths(paths: Vec<String>) -> Vec<String> {
    let mut unique = paths;
    unique.sort();
    unique.dedup();
    unique
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fs_write_uses_path_arg() {
        let paths = extract_changed_paths(
            "fs_write",
            &json!({ "path": "notes/todo.md", "content": "x" }),
            &json!({ "written": "/tmp/notes/todo.md" }),
        );
        assert_eq!(paths, vec!["notes/todo.md"]);
    }

    #[test]
    fn ooxml_unpack_uses_out_dir_only() {
        let paths = extract_changed_paths(
            "ooxml_unpack",
            &json!({ "path": "src.docx", "out_dir": "unpacked" }),
            &json!({ "parts": 12 }),
        );
        assert_eq!(paths, vec!["unpacked"]);
    }

    #[test]
    fn ooxml_pack_uses_out_path_arg_not_absolute_result() {
        let paths = extract_changed_paths(
            "ooxml_pack",
            &json!({ "dir": "unpacked", "out_path": "filled.docx", "original": "src.docx" }),
            &json!({ "path": "/tmp/project/filled.docx" }),
        );
        assert_eq!(paths, vec!["filled.docx"]);
    }

    #[test]
    fn accept_changes_falls_back_to_path_when_no_out_path() {
        let paths = extract_changed_paths(
            "docx_accept_changes",
            &json!({ "path": "draft.docx" }),
            &json!({ "path": "/tmp/project/draft.docx" }),
        );
        assert_eq!(paths, vec!["draft.docx"]);
    }

    #[test]
    fn fs_patch_uses_path_arg() {
        let paths = extract_changed_paths(
            "fs_patch",
            &json!({ "path": "notes/todo.md", "edits": [{ "old": "a", "new": "b" }] }),
            &json!({ "applied": 1, "missed": [] }),
        );
        assert_eq!(paths, vec!["notes/todo.md"]);
    }

    #[test]
    fn xlsx_recalc_does_not_report_changed_paths() {
        let paths = extract_changed_paths(
            "xlsx_recalc",
            &json!({ "path": "book.xlsx" }),
            &json!({ "errors": [], "warnings": [] }),
        );
        assert!(paths.is_empty());
    }

    #[test]
    fn skill_run_reads_written_paths_from_result() {
        let paths = extract_changed_paths(
            "skill_run",
            &json!({ "code": "async function main() {}" }),
            &json!({ "result": "ok", "written_paths": ["out.xlsx", "out.xlsx", "sub\\a.txt"] }),
        );
        assert_eq!(paths, vec!["out.xlsx", "sub/a.txt"]);
    }
}
