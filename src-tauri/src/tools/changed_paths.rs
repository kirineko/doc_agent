use serde_json::Value;

/// Extract project-relative paths changed by a successful tool invocation.
///
/// 返回项目相对 POSIX 路径（无尾部斜杠约定）。
/// 目录/文件的区分不由路径格式承载——消费侧（@ 索引、产物面板）各自处理。
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
        "image_download" => {
            // 逐个上报成功下载的本地文件，便于 @ 引用具体图片
            if let Some(items) = result.get("downloaded").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(path) = item.get("path").and_then(|v| v.as_str()) {
                        push_normalized(&mut paths, path);
                    }
                }
            }
        }
        _ => {}
    }
    // 过滤 .cache/ 下的中间产物（解包工作目录、渲染缓存等），仅保留交付物。
    // 与 core::project_files 的 @ 索引、文件树过滤口径一致。
    paths.retain(|p| !crate::core::cache_paths::is_cache_path(p));
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
        // 目录路径补尾部 / 标记，供前端区分目录与文件
        let paths = extract_changed_paths(
            "ooxml_unpack",
            &json!({ "path": "src.docx", "out_dir": "unpacked" }),
            &json!({ "parts": 12 }),
        );
        assert_eq!(paths, vec!["unpacked"]);
    }

    #[test]
    fn out_dir_has_no_trailing_slash() {
        // 目录路径不携带尾部斜杠约定，避免污染 @ 索引的 path 合并逻辑。
        // 目录/文件区分由消费侧处理。
        let paths = extract_changed_paths(
            "pdf_split",
            &json!({ "path": "src.pdf", "out_dir": "pages" }),
            &json!({ "written": ["/tmp/pages/p1.pdf"] }),
        );
        assert_eq!(paths, vec!["pages"]);
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

    #[test]
    fn image_download_reports_downloaded_file_paths() {
        let paths = extract_changed_paths(
            "image_download",
            &json!({ "urls": ["https://example.com/a.png"], "dir": "images" }),
            &json!({
                "dir": "images",
                "downloaded": [
                    { "url": "https://example.com/a.png", "path": "images/a.png" },
                    { "url": "https://example.com/b.jpg", "path": "images/b.jpg" }
                ],
                "failed": [],
                "count": 2
            }),
        );
        assert_eq!(paths, vec!["images/a.png", "images/b.jpg"]);
    }

    #[test]
    fn ooxml_unpack_cache_dir_is_filtered() {
        // ooxml_unpack 省略 out_dir 时返回 .cache/ooxml/<hash>/，属中间工作目录，不应进 changed_paths
        let paths = extract_changed_paths(
            "ooxml_unpack",
            &json!({ "path": "src.docx" }),
            &json!({ "out_dir": ".cache/ooxml/a1b2c3d4/", "parts": 12 }),
        );
        assert!(paths.is_empty());
    }

    #[test]
    fn cache_paths_are_filtered_from_skill_run() {
        // skill_run 若意外把 .cache 下的脚本/暂存路径写入 written_paths，也应被过滤
        let paths = extract_changed_paths(
            "skill_run",
            &json!({ "code": "x" }),
            &json!({
                "result": "ok",
                "written_paths": ["report.docx", ".cache/skill-run/abc/script.js"]
            }),
        );
        assert_eq!(paths, vec!["report.docx"]);
    }
}
