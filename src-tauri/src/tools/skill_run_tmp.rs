use super::{ToolContext, ToolError};
use crate::core::cache_paths::{skill_run_dir, skill_run_error, skill_run_script};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub fn script_rel(ctx: &ToolContext) -> Result<String, ToolError> {
    require_turn(ctx)?;
    Ok(skill_run_script(ctx.session_id))
}

pub fn write_temp_script(ctx: &ToolContext, code: &str) -> Result<(), ToolError> {
    let rel = script_rel(ctx)?;
    let path = ctx.sandbox.resolve_for_write(&rel)?;
    ensure_parent(&path)?;
    fs::write(path, code).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(())
}

pub fn read_script_path(ctx: &ToolContext, path: &str) -> Result<String, ToolError> {
    let resolved = ctx.sandbox.resolve(path)?;
    fs::read_to_string(resolved).map_err(|e| ToolError::Execution(e.to_string()))
}

pub fn cleanup(ctx: &ToolContext) {
    if let Ok(dir) = turn_dir_path(ctx) {
        if dir.is_dir() {
            let _ = fs::remove_dir_all(dir);
        }
    }
}

pub fn clear_error(ctx: &ToolContext) {
    if let Ok(rel) = error_rel(ctx) {
        if let Ok(path) = ctx.sandbox.resolve_for_write(&rel) {
            let _ = fs::remove_file(path);
        }
    }
}

pub fn has_error(ctx: &ToolContext) -> bool {
    error_rel(ctx)
        .ok()
        .and_then(|rel| ctx.sandbox.resolve(&rel).ok())
        .map(|p| p.exists())
        .unwrap_or(false)
}

pub fn cleanup_on_turn_end(ctx: &ToolContext) {
    if has_error(ctx) {
        return;
    }
    cleanup(ctx);
}

pub fn write_error(ctx: &ToolContext, error: &Value) -> Result<(), ToolError> {
    let rel = error_rel(ctx)?;
    let path = ctx.sandbox.resolve_for_write(&rel)?;
    ensure_parent(&path)?;
    let text = serde_json::to_string_pretty(error).unwrap_or_else(|_| error.to_string());
    fs::write(path, text).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(())
}

pub fn tmp_dir_exists(ctx: &ToolContext) -> bool {
    turn_dir_path(ctx)
        .ok()
        .map(|p| p.is_dir())
        .unwrap_or(false)
}

fn error_rel(ctx: &ToolContext) -> Result<String, ToolError> {
    require_turn(ctx)?;
    Ok(skill_run_error(ctx.session_id))
}

fn require_turn(ctx: &ToolContext) -> Result<(), ToolError> {
    if ctx.session_id.is_empty() || ctx.turn_id.is_empty() {
        return Err(ToolError::Execution(
            "skill_run scratch workspace requires turn context".into(),
        ));
    }
    Ok(())
}

fn turn_dir_path(ctx: &ToolContext) -> Result<PathBuf, ToolError> {
    require_turn(ctx)?;
    let rel = skill_run_dir(ctx.session_id);
    let script = ctx.sandbox.resolve_for_write(&format!("{rel}/script.js"))?;
    script
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| ToolError::Execution("invalid skill_run scratch path".into()))
}

fn ensure_parent(path: &Path) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::Execution(e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use tempfile::tempdir;

    #[test]
    fn script_path_stable_across_turns_in_same_session() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx_turn_a = ToolContext::with_test_turn(&sandbox, "p", "sess-1", "turn-a", "t");
        let ctx_turn_b = ToolContext::with_test_turn(&sandbox, "p", "sess-1", "turn-b", "t");

        let path_a = script_rel(&ctx_turn_a).unwrap();
        let path_b = script_rel(&ctx_turn_b).unwrap();
        assert_eq!(path_a, path_b);
        assert_ne!(path_a, script_rel(&ToolContext::with_test_turn(
            &sandbox, "p", "sess-2", "turn-a", "t"
        ))
        .unwrap());
    }

    #[test]
    fn cleanup_on_turn_end_removes_session_scratch_dir() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::with_test_turn(&sandbox, "p", "sess-1", "turn-1", "t");

        write_temp_script(&ctx, "async function main() {}").unwrap();
        let scratch_dir = turn_dir_path(&ctx).unwrap();
        assert!(scratch_dir.exists());

        fs::remove_file(scratch_dir.join("script.js")).unwrap();
        assert!(scratch_dir.is_dir());

        cleanup_on_turn_end(&ctx);
        assert!(!scratch_dir.exists());
    }
}
