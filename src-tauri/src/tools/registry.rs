use crate::core::sandbox::Sandbox;
use serde_json::Value;
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
    #[error("not implemented")]
    NotImplemented,
}

pub struct ToolContext<'a> {
    pub sandbox: &'a Sandbox,
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

impl ToolRegistry {
    pub fn default_tools() -> Self {
        Self {
            tools: vec![
                crate::tools::fs::list_tool(),
                crate::tools::fs::read_tool(),
                crate::tools::fs::write_tool(),
                crate::tools::fs::search_tool(),
                crate::tools::office::read_markdown_tool(),
                crate::tools::word::create_tool(),
                crate::tools::word::edit_tool(),
                crate::tools::excel::read_tool(),
                crate::tools::excel::write_tool(),
                crate::tools::skill::run_tool(),
            ],
        }
    }

    pub fn definitions(&self) -> Vec<crate::agent::types::ToolDefinition> {
        self.tools
            .iter()
            .map(|t| crate::agent::types::ToolDefinition {
                name: t.name.to_string(),
                description: t.description.to_string(),
                parameters: t.parameters.clone(),
            })
            .collect()
    }

    pub fn execute(&self, ctx: &ToolContext, name: &str, args: Value) -> Result<Value, ToolError> {
        let tool = self
            .tools
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| ToolError::Unknown(name.to_string()))?;

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (tool.handler)(ctx, args))) {
            Ok(result) => result,
            Err(_) => Err(ToolError::Execution(format!("tool {name} panicked"))),
        }
    }
}
