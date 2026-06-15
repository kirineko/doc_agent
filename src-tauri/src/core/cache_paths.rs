//! Project-local cache directory layout under `.cache/`.

pub const CACHE_ROOT: &str = ".cache";
pub const ATTACHMENTS_DIR: &str = ".cache/attachments";
pub const SKILL_RUN_DIR: &str = ".cache/skill-run";
pub const SKILL_RUN_SCRIPT: &str = ".cache/skill-run/script.js";
pub const SKILL_RUN_ERROR: &str = ".cache/skill-run/error.json";
pub const PDF_CACHE_ROOT: &str = ".cache/pdf";

pub fn attachment_rel_path(stored_name: &str) -> String {
    format!("{ATTACHMENTS_DIR}/{stored_name}")
}

pub fn is_attachment_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let trimmed = normalized.trim_start_matches("./");
    trimmed.starts_with(&format!("{ATTACHMENTS_DIR}/")) && !trimmed.contains("..")
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
    fn attachment_rel_path_format() {
        assert_eq!(
            attachment_rel_path("uuid.png"),
            ".cache/attachments/uuid.png"
        );
    }
}
