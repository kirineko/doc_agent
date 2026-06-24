pub mod changed_paths;
pub mod clarify;
pub mod data;
pub mod excel;
pub mod fs;
pub mod html_export;
pub mod image_download;
pub mod image_read;
pub mod io_plan;
pub mod office;
pub mod ooxml;
pub mod pdf;
pub mod pdf_cache;
pub mod pdf_judge;
pub mod pdf_ops;
pub mod pdf_read;
pub mod pdf_render_pages;
pub mod pdf_text_quality;
pub mod registry;
pub mod runtime;
pub mod skill;
pub mod skill_run_tmp;
pub mod typst_export;
pub mod vision_subcall;
pub mod web;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod file_lock_integration;

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
