use crate::agent::types::ModelId;
use crate::core::file_locks::FileLockRegistry;
use crate::core::sandbox::Sandbox;
use crate::core::secrets::Secrets;
use serde_json::Value;
use std::sync::Arc;
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
    pub project_id: &'a str,
    pub session_id: &'a str,
    pub turn_id: &'a str,
    pub session_title: &'a str,
    pub file_locks: Option<Arc<FileLockRegistry>>,
    pub write_gate: Option<Arc<crate::tools::runtime::write_gate::RuntimeWriteGate>>,
}

impl<'a> ToolContext<'a> {
    pub fn new(sandbox: &'a Sandbox) -> Self {
        Self::with_test_turn(sandbox, "test-project", "test-session", "test-turn", "test")
    }

    pub fn with_secrets(sandbox: &'a Sandbox, secrets: &'a Secrets) -> Self {
        Self {
            sandbox,
            secrets: Some(secrets),
            project_id: "test-project",
            session_id: "test-session",
            turn_id: "test-turn",
            session_title: "test",
            file_locks: None,
            write_gate: None,
        }
    }

    pub fn with_test_turn(
        sandbox: &'a Sandbox,
        project_id: &'a str,
        session_id: &'a str,
        turn_id: &'a str,
        session_title: &'a str,
    ) -> Self {
        Self {
            sandbox,
            secrets: None,
            project_id,
            session_id,
            turn_id,
            session_title,
            file_locks: None,
            write_gate: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_turn(
        sandbox: &'a Sandbox,
        secrets: Option<&'a Secrets>,
        project_id: &'a str,
        session_id: &'a str,
        turn_id: &'a str,
        session_title: &'a str,
        file_locks: Arc<FileLockRegistry>,
        write_gate: Option<Arc<crate::tools::runtime::write_gate::RuntimeWriteGate>>,
    ) -> Self {
        Self {
            sandbox,
            secrets,
            project_id,
            session_id,
            turn_id,
            session_title,
            file_locks: Some(file_locks),
            write_gate,
        }
    }

    pub fn with_write_gate(
        &self,
        write_gate: Option<Arc<crate::tools::runtime::write_gate::RuntimeWriteGate>>,
    ) -> Self {
        Self {
            sandbox: self.sandbox,
            secrets: self.secrets,
            project_id: self.project_id,
            session_id: self.session_id,
            turn_id: self.turn_id,
            session_title: self.session_title,
            file_locks: self.file_locks.clone(),
            write_gate,
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
                crate::tools::clarify::ask_tool(),
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
                crate::tools::pdf_render_pages::tool(),
                crate::tools::pdf_read::tool(),
                crate::tools::html_export::tool(),
                crate::tools::typst_export::typst_to_pdf_tool(),
                crate::tools::typst_export::typst_list_templates_tool(),
                crate::tools::typst_export::typst_read_template_tool(),
                crate::tools::web::search_tool(),
                crate::tools::web::extract_tool(),
                crate::tools::image_read::tool(),
            ],
        }
    }

    pub fn tools_for_model(
        &self,
        model: ModelId,
        include_web: bool,
    ) -> Vec<crate::agent::types::ToolDefinition> {
        self.tools
            .iter()
            .filter(|t| include_web || !is_web_tool(t.name))
            .filter(|t| t.name != "image_read" || model.supports_vision())
            .map(|t| {
                let (description, parameters) = if t.name == "pdf_read" {
                    (
                        crate::tools::pdf_read::description_for_model(model).to_string(),
                        crate::tools::pdf_read::parameters_schema(),
                    )
                } else {
                    (t.description.to_string(), t.parameters.clone())
                };
                crate::agent::types::ToolDefinition {
                    name: t.name.to_string(),
                    description,
                    parameters,
                    strict: if t.name == "clarify_ask" {
                        Some(true)
                    } else {
                        None
                    },
                }
            })
            .collect()
    }

    pub fn definitions(&self, include_web: bool) -> Vec<crate::agent::types::ToolDefinition> {
        self.tools_for_model(ModelId::KimiK26, include_web)
    }

    pub fn tool_names(&self) -> Vec<&'static str> {
        self.tools.iter().map(|t| t.name).collect()
    }

    pub async fn execute<R: Runtime>(
        &self,
        ctx: &ToolContext<'_>,
        app: &AppHandle<R>,
        model_id: ModelId,
        name: &str,
        args: Value,
    ) -> Result<Value, ToolError> {
        match name {
            "web_search" => crate::tools::web::search_handler(ctx, args).await,
            "web_extract" => crate::tools::web::extract_handler(ctx, args).await,
            "html_to_pdf" => crate::tools::html_export::handler(ctx, app, args).await,
            "typst_to_pdf" => crate::tools::typst_export::typst_to_pdf_handler(ctx, args).await,
            "image_read" => crate::tools::image_read::handler(ctx, args, model_id).await,
            "pdf_read" => crate::tools::pdf_read::handler(ctx, args, model_id).await,
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
