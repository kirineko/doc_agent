pub mod changed_paths;
pub mod data;
pub mod excel;
pub mod fs;
pub mod html_export;
pub mod office;
pub mod ooxml;
pub mod pdf;
pub mod pdf_ops;
pub mod registry;
pub mod runtime;
pub mod skill;
pub mod skill_run_tmp;
pub mod web;

#[cfg(test)]
mod tests;

pub use registry::{ToolContext, ToolError, ToolRegistry, ToolSpec};

pub(crate) fn required_str_arg(
    args: &serde_json::Value,
    key: &str,
) -> Result<String, registry::ToolError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| registry::ToolError::InvalidArgs(format!("{key} required")))
}

pub(crate) fn ensure_parent_dir(path: &std::path::Path) -> Result<(), registry::ToolError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| registry::ToolError::Execution(e.to_string()))?;
    }
    Ok(())
}
