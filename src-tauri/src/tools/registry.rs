use crate::core::sandbox::Sandbox;
use crate::core::secrets::Secrets;
use serde_json::Value;
use tauri::{AppHandle, Runtime};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("unknown tool: {0}")]
    Unknown(String),
    #[error("invalid args: {0}")]
    InvalidArgs(String),
    #[error("sandbox error: {0}")]
    Sandbox(#[from] crate::core::sandbox::SandboxError),
    #[error("execution error: {0}")]
    Execution(String),
    #[error("structured tool error")]
    Structured(Value),
    #[error("not implemented")]
    NotImplemented,
}

impl ToolError {
    pub fn to_json_value(&self) -> Value {
        match self {
            Self::Structured(v) => v.clone(),
            other => serde_json::json!({ "error": other.to_string() }),
        }
    }
}

pub struct ToolContext<'a> {
    pub sandbox: &'a Sandbox,
    pub secrets: Option<&'a Secrets>,
}

impl<'a> ToolContext<'a> {
    pub fn new(sandbox: &'a Sandbox) -> Self {
        Self {
            sandbox,
            secrets: None,
        }
    }

    pub fn with_secrets(sandbox: &'a Sandbox, secrets: &'a Secrets) -> Self {
        Self {
            sandbox,
            secrets: Some(secrets),
        }
    }
}

#[derive(Clone)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
    pub handler: fn(&ToolContext, Value) -> Result<Value, ToolError>,
}

#[derive(Clone)]
pub struct ToolRegistry {
    tools: Vec<ToolSpec>,
}

fn is_web_tool(name: &str) -> bool {
    name.starts_with("web_")
}

impl ToolRegistry {
    pub fn default_tools() -> Self {
        Self {
            tools: vec![
                crate::tools::fs::list_tool(),
                crate::tools::fs::read_tool(),
                crate::tools::fs::write_tool(),
                crate::tools::fs::patch_tool(),
                crate::tools::fs::search_tool(),
                crate::tools::office::read_markdown_tool(),
                crate::tools::office::convert_tool(),
                crate::tools::excel::read_tool(),
                crate::tools::excel::write_tool(),
                crate::tools::skill::read_tool(),
                crate::tools::skill::run_tool(),
                crate::tools::ooxml::unpack_tool(),
                crate::tools::ooxml::pack_tool(),
                crate::tools::ooxml::comment_tool(),
                crate::tools::ooxml::accept_changes_tool(),
                crate::tools::data::extract_docx_tool(),
                crate::tools::data::describe_tool(),
                crate::tools::data::normalize_tool(),
                crate::tools::data::query_tool(),
                crate::tools::data::recalc_tool(),
                crate::tools::pdf_ops::merge_tool(),
                crate::tools::pdf_ops::split_tool(),
                crate::tools::pdf_ops::rotate_tool(),
                crate::tools::pdf_ops::delete_pages_tool(),
                crate::tools::html_export::tool(),
                crate::tools::web::search_tool(),
                crate::tools::web::extract_tool(),
            ],
        }
    }

    pub fn definitions(&self, include_web: bool) -> Vec<crate::agent::types::ToolDefinition> {
        self.tools
            .iter()
            .filter(|t| include_web || !is_web_tool(t.name))
            .map(|t| crate::agent::types::ToolDefinition {
                name: t.name.to_string(),
                description: t.description.to_string(),
                parameters: t.parameters.clone(),
            })
            .collect()
    }

    pub async fn execute<R: Runtime>(
        &self,
        ctx: &ToolContext<'_>,
        app: &AppHandle<R>,
        name: &str,
        args: Value,
    ) -> Result<Value, ToolError> {
        match name {
            "web_search" => crate::tools::web::search_handler(ctx, args).await,
            "web_extract" => crate::tools::web::extract_handler(ctx, args).await,
            "html_to_pdf" => crate::tools::html_export::handler(ctx, app, args).await,
            _ => {
                let tool = self
                    .tools
                    .iter()
                    .find(|t| t.name == name)
                    .ok_or_else(|| ToolError::Unknown(name.to_string()))?;

                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (tool.handler)(ctx, args)
                })) {
                    Ok(result) => result,
                    Err(_) => Err(ToolError::Execution(format!("tool {name} panicked"))),
                }
            }
        }
    }
}
