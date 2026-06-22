use crate::agent::types::{ClarifyAnswer, ClarifyQuestion};
use crate::core::file_locks::normalize_project_path;
use crate::core::sandbox::{Sandbox, SandboxError};
use crate::core::store::{Message, ToolCallRecord};
use std::fs;

pub const AGENTS_MD_PATH: &str = "AGENTS.md";
pub const MAX_AGENTS_MD_INJECT_CHARS: usize = 3000;
pub const MAX_AGENTS_MD_FILE_CHARS: usize = 8000;

const SECTION_PRIORITY: &[&str] = &["PPT", "Word", "Excel", "PDF", "Typst", "概述"];

/// User message starts a profile init turn (`/init` or `/init …`; leading whitespace allowed).
pub fn is_profile_init_message(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == "/init" || trimmed.starts_with("/init ")
}

/// Whether the current turn's latest `confirm_agents_md` answer was affirmative.
pub fn detect_agents_md_confirmed(history: &[Message], tool_calls: &[ToolCallRecord]) -> bool {
    let Some(start) = history.iter().rposition(|m| m.role == "user") else {
        return false;
    };
    let mut last_confirmed = false;
    for m in &history[start..] {
        if m.role != "tool" {
            continue;
        }
        let Some(tool_call_id) = m.tool_call_id.as_deref() else {
            continue;
        };
        let Some(record) = tool_calls.iter().find(|t| t.id == tool_call_id) else {
            continue;
        };
        if record.name != "clarify_ask" {
            continue;
        }
        let Ok(question) = serde_json::from_str::<ClarifyQuestion>(&record.args_json) else {
            continue;
        };
        if question.kind != "confirm_agents_md" {
            continue;
        }
        let Some(content) = m.content.as_deref() else {
            continue;
        };
        let Ok(answer) = serde_json::from_str::<ClarifyAnswer>(content) else {
            continue;
        };
        last_confirmed = answer.preview_markdown.is_some();
    }
    last_confirmed
}

pub fn is_agents_md_path(path: &str) -> bool {
    is_agents_md_relative_path(path)
}

pub fn is_agents_md_relative_path(rel: &str) -> bool {
    rel.trim()
        .replace('\\', "/")
        .eq_ignore_ascii_case(AGENTS_MD_PATH)
}

/// Whether a user path resolves to project-root `AGENTS.md` (handles `./AGENTS.md`, etc.).
pub fn targets_agents_md(sandbox: &Sandbox, user_path: &str) -> Result<bool, SandboxError> {
    let rel = normalize_project_path(sandbox, user_path)?;
    Ok(is_agents_md_relative_path(&rel))
}

pub fn guard_agents_md_write(
    sandbox: &Sandbox,
    user_path: &str,
    profile_init: bool,
    agents_md_confirmed: bool,
    content: Option<&str>,
) -> Result<(), String> {
    if !targets_agents_md(sandbox, user_path).map_err(|e| e.to_string())? {
        return Ok(());
    }
    if !profile_init {
        return Err(format!(
            "{AGENTS_MD_PATH} can only be written during /init; use /init or edit the file manually"
        ));
    }
    if !agents_md_confirmed {
        return Err(format!(
            "{AGENTS_MD_PATH} write requires confirm_agents_md approval in this /init turn"
        ));
    }
    if let Some(body) = content {
        validate_agents_md_write(body)?;
    }
    Ok(())
}

/// Read project `AGENTS.md` for system injection; returns `None` if missing or empty.
pub fn read_agents_md_for_inject(sandbox: &Sandbox) -> Option<String> {
    let path = sandbox.root().join(AGENTS_MD_PATH);
    if !path.is_file() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(truncate_agents_md_for_inject(trimmed))
}

pub fn truncate_agents_md_for_inject(body: &str) -> String {
    if char_len(body) <= MAX_AGENTS_MD_INJECT_CHARS {
        return body.to_string();
    }
    truncate_by_section_priority(body, MAX_AGENTS_MD_INJECT_CHARS)
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn truncate_by_section_priority(body: &str, max: usize) -> String {
    let sections = split_markdown_sections(body);
    if sections.is_empty() {
        return truncate_chars(body, max);
    }

    let mut buckets: Vec<Vec<(usize, String)>> = vec![Vec::new(); SECTION_PRIORITY.len() + 1];
    for (index, (title, content)) in sections.into_iter().enumerate() {
        let bucket = section_bucket_index(&title);
        let block = if title.is_empty() {
            content
        } else {
            format!("## {title}\n{content}")
        };
        buckets[bucket].push((index, block));
    }

    let mut out = String::new();
    for bucket in buckets {
        for (_, block) in bucket {
            let sep = if out.is_empty() { "" } else { "\n\n" };
            let candidate = format!("{out}{sep}{block}");
            if char_len(&candidate) <= max {
                out = candidate;
            } else if out.is_empty() {
                return truncate_chars(&block, max);
            } else {
                break;
            }
        }
        if char_len(&out) >= max {
            break;
        }
    }

    if out.is_empty() {
        truncate_chars(body, max)
    } else {
        out
    }
}

fn section_bucket_index(title: &str) -> usize {
    let lower = title.to_lowercase();
    for (idx, key) in SECTION_PRIORITY.iter().enumerate() {
        if lower.contains(&key.to_lowercase()) {
            return idx;
        }
    }
    SECTION_PRIORITY.len()
}

fn split_markdown_sections(body: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_title = String::new();
    let mut current_lines: Vec<&str> = Vec::new();

    for line in body.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            if !current_title.is_empty() || !current_lines.is_empty() {
                sections.push((
                    current_title.clone(),
                    current_lines.join("\n").trim().to_string(),
                ));
            }
            current_title = title.trim().to_string();
            current_lines.clear();
        } else {
            current_lines.push(line);
        }
    }
    sections.push((current_title, current_lines.join("\n").trim().to_string()));
    sections
        .into_iter()
        .filter(|(title, content)| !title.is_empty() || !content.is_empty())
        .collect()
}

fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

pub fn agents_md_inject_section(body: &str) -> String {
    format!("## 项目配置（AGENTS.md）\n{body}")
}

pub fn profile_init_system_hint() -> &'static str {
    "\n用户通过 /init 初始化或更新项目 AGENTS.md。MUST 先 skill_read profile；fs_read AGENTS.md（不存在时 exists:false，非错误）；读项目文件；结合当前会话历史澄清需求；clarify_ask 每轮 assistant 仅调用一次（逐问，禁止同轮并行多个）；含 confirm_agents_md 确认后 fs_write AGENTS.md；结束后用简短文字摘要变更。禁止继续上一文档任务；禁止非 init 流程写 AGENTS.md。\n"
}

pub fn validate_agents_md_write(content: &str) -> Result<(), String> {
    if char_len(content) > MAX_AGENTS_MD_FILE_CHARS {
        return Err(format!(
            "AGENTS.md exceeds {MAX_AGENTS_MD_FILE_CHARS} characters; compress before writing"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_agents(sandbox: &Sandbox, content: &str) {
        fs::write(sandbox.root().join(AGENTS_MD_PATH), content).unwrap();
    }

    #[test]
    fn read_missing_returns_none() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path().to_str().unwrap()).unwrap();
        assert!(read_agents_md_for_inject(&sandbox).is_none());
    }

    #[test]
    fn read_existing_injects_content() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path().to_str().unwrap()).unwrap();
        write_agents(&sandbox, "# 项目\n\n## PPT\n深色商务");
        let body = read_agents_md_for_inject(&sandbox).unwrap();
        assert!(body.contains("深色商务"));
    }

    #[test]
    fn truncates_with_section_priority() {
        let ppt = "P".repeat(2000);
        let word = "W".repeat(2000);
        let body = format!("## Word\n{word}\n\n## PPT\n{ppt}");
        let truncated = truncate_agents_md_for_inject(&body);
        assert!(char_len(&truncated) <= MAX_AGENTS_MD_INJECT_CHARS);
        let ppt_pos = truncated.find("## PPT");
        let word_pos = truncated.find("## Word");
        assert!(ppt_pos.is_some());
        if let (Some(p), Some(w)) = (ppt_pos, word_pos) {
            assert!(
                p < w,
                "PPT section should appear before Word when truncating"
            );
        }
    }

    #[test]
    fn targets_agents_md_normalizes_relative_paths() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path().to_str().unwrap()).unwrap();
        assert!(targets_agents_md(&sandbox, "./AGENTS.md").unwrap());
        assert!(targets_agents_md(&sandbox, "AGENTS.md").unwrap());
        assert!(!targets_agents_md(&sandbox, "./notes.md").unwrap());
    }

    #[test]
    fn guard_rejects_agents_md_outside_init() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path().to_str().unwrap()).unwrap();
        let err =
            guard_agents_md_write(&sandbox, "./AGENTS.md", false, false, Some("# x")).unwrap_err();
        assert!(err.contains("AGENTS.md"));
    }

    #[test]
    fn guard_rejects_init_without_confirm() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path().to_str().unwrap()).unwrap();
        let err =
            guard_agents_md_write(&sandbox, "AGENTS.md", true, false, Some("# x")).unwrap_err();
        assert!(err.contains("confirm_agents_md"));
    }

    #[test]
    fn is_profile_init_message_detects_prefix() {
        assert!(is_profile_init_message("/init"));
        assert!(is_profile_init_message("/init 固化PPT"));
        assert!(is_profile_init_message(" /init"));
        assert!(is_profile_init_message("  /init 固化PPT"));
        assert!(!is_profile_init_message("/initialize"));
    }

    #[test]
    fn detect_agents_md_confirmed_uses_latest_confirm_agents_md_only() {
        use crate::core::store::{Message, ToolCallRecord};

        let mk_msg = |role: &str, content: Option<&str>, tool_call_id: Option<&str>| Message {
            id: String::new(),
            session_id: "s".into(),
            role: role.into(),
            content: content.map(str::to_string),
            reasoning_content: None,
            tool_call_id: tool_call_id.map(str::to_string),
            seq: 0,
            created_at: String::new(),
            archived: false,
            attachments_json: None,
        };
        let mk_call = |id: &str, args: &str| ToolCallRecord {
            id: id.into(),
            message_id: String::new(),
            name: "clarify_ask".into(),
            args_json: args.into(),
            result_json: None,
            status: "done".into(),
            duration_ms: 0,
            created_at: String::new(),
        };
        let confirm_args = serde_json::json!({
            "id": "q1",
            "kind": "confirm_agents_md",
            "prompt": "确认",
            "preview_markdown": "# v1"
        })
        .to_string();
        let confirm_answer = serde_json::json!({
            "question_id": "q1",
            "selected": ["confirm"],
            "display_text": "确认继续",
            "preview_markdown": "# v1"
        });
        let reject_answer = serde_json::json!({
            "question_id": "q2",
            "selected": [],
            "display_text": "改一下 PPT 配色",
            "custom": "改一下 PPT 配色"
        });
        let history = vec![
            mk_msg("user", Some("/init"), None),
            mk_msg("tool", Some(&confirm_answer.to_string()), Some("call1")),
            mk_msg("tool", Some(&reject_answer.to_string()), Some("call2")),
        ];
        let tool_calls = vec![
            mk_call("call1", &confirm_args),
            mk_call(
                "call2",
                &serde_json::json!({
                    "id": "q2",
                    "kind": "confirm_agents_md",
                    "prompt": "确认",
                    "preview_markdown": "# v2"
                })
                .to_string(),
            ),
        ];
        assert!(!detect_agents_md_confirmed(&history, &tool_calls));
    }

    #[test]
    fn detect_agents_md_confirmed_ignores_prior_turn() {
        use crate::core::store::{Message, ToolCallRecord};

        let mk_msg = |role: &str, content: Option<&str>, tool_call_id: Option<&str>| Message {
            id: String::new(),
            session_id: "s".into(),
            role: role.into(),
            content: content.map(str::to_string),
            reasoning_content: None,
            tool_call_id: tool_call_id.map(str::to_string),
            seq: 0,
            created_at: String::new(),
            archived: false,
            attachments_json: None,
        };
        let mk_call = |id: &str| {
            ToolCallRecord {
            id: id.into(),
            message_id: String::new(),
            name: "clarify_ask".into(),
            args_json: "{\"id\":\"q1\",\"kind\":\"confirm_agents_md\",\"prompt\":\"确认\",\"preview_markdown\":\"# old\"}".into(),
            result_json: None,
            status: "done".into(),
            duration_ms: 0,
            created_at: String::new(),
        }
        };
        let old_answer = serde_json::json!({
            "question_id": "q1",
            "selected": ["confirm"],
            "display_text": "确认继续",
            "preview_markdown": "# old"
        });
        let history = vec![
            mk_msg("user", Some("/init"), None),
            mk_msg("tool", Some(&old_answer.to_string()), Some("old")),
            mk_msg("assistant", Some("done"), None),
            mk_msg("user", Some("/init again"), None),
        ];
        assert!(!detect_agents_md_confirmed(&history, &[mk_call("old")]));
    }

    #[test]
    fn inject_section_heading() {
        let section = agents_md_inject_section("hello");
        assert!(section.contains("## 项目配置（AGENTS.md）"));
        assert!(section.contains("hello"));
    }
}
