use super::{ToolContext, ToolError};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub const TMP_DIR: &str = ".skill-run";
pub const SCRIPT_REL: &str = ".skill-run/script.js";
pub const ERROR_REL: &str = ".skill-run/error.json";

pub fn write_temp_script(ctx: &ToolContext, code: &str) -> Result<(), ToolError> {
    let path = ctx.sandbox.resolve_for_write(SCRIPT_REL)?;
    ensure_parent(&path)?;
    fs::write(path, code).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(())
}

pub fn read_script_path(ctx: &ToolContext, path: &str) -> Result<String, ToolError> {
    let resolved = ctx.sandbox.resolve(path)?;
    fs::read_to_string(resolved).map_err(|e| ToolError::Execution(e.to_string()))
}

pub fn cleanup(ctx: &ToolContext) {
    if let Ok(path) = ctx.sandbox.resolve(TMP_DIR) {
        let _ = fs::remove_dir_all(path);
    }
}

/// 成功执行后清除上一次失败遗留的 error.json，避免误导后续修复。
pub fn clear_error(ctx: &ToolContext) {
    if let Ok(path) = ctx.sandbox.resolve(ERROR_REL) {
        let _ = fs::remove_file(path);
    }
}

pub fn has_error(ctx: &ToolContext) -> bool {
    ctx.sandbox
        .resolve(ERROR_REL)
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Turn 结束兜底清理：只要没有未修复的失败现场（error.json），就删除 `.skill-run/`。
/// style_warnings 是否被处理不影响清理 —— 脚本只在 turn 内供修复使用。
pub fn cleanup_on_turn_end(ctx: &ToolContext) {
    if !has_error(ctx) {
        cleanup(ctx);
    }
}

pub fn write_error(ctx: &ToolContext, error: &Value) -> Result<(), ToolError> {
    let path = ctx.sandbox.resolve_for_write(ERROR_REL)?;
    ensure_parent(&path)?;
    let text = serde_json::to_string_pretty(error).unwrap_or_else(|_| error.to_string());
    fs::write(path, text).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(())
}

pub fn tmp_dir_exists(ctx: &ToolContext) -> bool {
    ctx.sandbox
        .resolve(TMP_DIR)
        .map(|p| p.exists())
        .unwrap_or(false)
}

fn ensure_parent(path: &Path) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::Execution(e.to_string()))?;
    }
    Ok(())
}
