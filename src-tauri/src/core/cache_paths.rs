//! Project-local cache directory layout under `.cache/`.

use std::hash::{Hash, Hasher};

pub const CACHE_ROOT: &str = ".cache";
pub const ATTACHMENTS_DIR: &str = ".cache/attachments";
pub const SKILL_RUN_DIR: &str = ".cache/skill-run";
pub const PDF_CACHE_ROOT: &str = ".cache/pdf";
pub const OOXML_WORK_DIR: &str = ".cache/ooxml";
/// Reserved for future turn-scoped scratch (not wired up yet).
pub const TURN_TMP_DIR: &str = ".cache/tmp";

pub fn attachment_rel_path(stored_name: &str) -> String {
    format!("{ATTACHMENTS_DIR}/{stored_name}")
}

pub fn is_attachment_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let trimmed = normalized.trim_start_matches("./");
    trimmed.starts_with(&format!("{ATTACHMENTS_DIR}/")) && !trimmed.contains("..")
}

pub fn is_cache_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let trimmed = normalized.trim_start_matches("./");
    trimmed == CACHE_ROOT
        || trimmed.starts_with(&format!("{CACHE_ROOT}/")) && !trimmed.contains("..")
}

pub fn safe_path_segment(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn short_hash(parts: &[&str]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for part in parts {
        part.hash(&mut hasher);
    }
    format!("{:08x}", (hasher.finish() & 0xFFFF_FFFF) as u32)
}

/// Short filesystem-safe cache segment (8 hex chars).
pub fn cache_key(parts: &[&str]) -> String {
    short_hash(parts)
}

/// Per-session scratch workspace (stable `session_key` across turns in the same chat session).
pub fn skill_run_dir(session_id: &str) -> String {
    format!("{SKILL_RUN_DIR}/{}", cache_key(&[session_id]))
}

pub fn skill_run_script(session_id: &str) -> String {
    format!("{}/script.js", skill_run_dir(session_id))
}

pub fn skill_run_error(session_id: &str) -> String {
    format!("{}/error.json", skill_run_dir(session_id))
}

/// Reserved helper: `.cache/tmp/<session_key>/<turn_key>/`.
/// No production caller yet; turn-end cleanup and IO plan are not implemented.
pub fn turn_tmp_dir(session_id: &str, turn_id: &str) -> String {
    format!(
        "{TURN_TMP_DIR}/{}/{}",
        cache_key(&[session_id]),
        cache_key(&[session_id, turn_id])
    )
}

pub fn ooxml_work_dir(session_id: &str, turn_id: &str, source_path: &str) -> String {
    format!(
        "{OOXML_WORK_DIR}/{}/{}",
        cache_key(&[session_id]),
        cache_key(&[session_id, turn_id, source_path])
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_path_validation() {
        assert!(is_attachment_path(".cache/attachments/a.png"));
        assert!(is_attachment_path("./.cache/attachments/a.png"));
        assert!(!is_attachment_path(".uploads/a.png"));
        assert!(!is_attachment_path("../.cache/attachments/a.png"));
    }

    #[test]
    fn cache_path_validation() {
        assert!(is_cache_path(".cache/pdf/key"));
        assert!(is_cache_path(".cache/ooxml/a1/b2"));
        assert!(!is_cache_path("docs/a.pdf"));
    }

    #[test]
    fn safe_segment_rejects_dotdot() {
        let seg = safe_path_segment("../evil");
        assert!(!seg.contains('/'));
        assert!(!seg.contains('.'));
    }

    #[test]
    fn turn_scoped_dirs_differ_by_turn() {
        let a = ooxml_work_dir("s1", "t1", "a.docx");
        let b = ooxml_work_dir("s1", "t2", "a.docx");
        assert_ne!(a, b);
        assert!(a.matches('/').count() <= 3);
        assert!(!a.contains(".."));
    }

    #[test]
    fn ooxml_work_dir_uses_hash_not_filename_stem() {
        let ascii = ooxml_work_dir("s1", "t1", "report.docx");
        let chinese = ooxml_work_dir("s1", "t1", "1. AI智能.docx");
        assert!(!chinese.contains("___"));
        assert!(!chinese.contains("AI"));
        assert_eq!(ascii.len(), chinese.len());
    }

    #[test]
    fn skill_run_paths_are_session_scoped() {
        let same_session = skill_run_script("sess-1");
        assert_eq!(same_session, skill_run_script("sess-1"));
        assert_ne!(same_session, skill_run_script("sess-2"));
        assert!(same_session.starts_with(".cache/skill-run/"));
        assert!(same_session.ends_with("/script.js"));
        assert!(
            same_session.len() <= 40,
            "path should be short: {same_session}"
        );
    }
}
