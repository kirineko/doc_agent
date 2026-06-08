use super::{ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};

pub fn run_tool() -> ToolSpec {
    ToolSpec {
        name: "skill_run",
        description: "Run a registered Document Skill by name (reserved for future script runtime)",
        parameters: json!({
            "type": "object",
            "properties": {
                "skill": { "type": "string" },
                "input": { "type": "object" }
            },
            "required": ["skill"]
        }),
        handler: run_handler,
    }
}

fn run_handler(_ctx: &ToolContext, _args: Value) -> Result<Value, ToolError> {
    Err(ToolError::NotImplemented)
}
