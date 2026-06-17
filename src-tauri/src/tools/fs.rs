use super::{ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};
use std::fs;
use walkdir::WalkDir;

pub fn list_tool() -> ToolSpec {
    ToolSpec {
        name: "fs_list",
        description: "List files and directories under a project-relative path",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Relative path, default '.'" }
            }
        }),
        handler: list_handler,
    }
}

pub fn read_tool() -> ToolSpec {
    ToolSpec {
        name: "fs_read",
        description: "Read a UTF-8 text file inside the project",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
            "required": ["path"]
        }),
        handler: read_handler,
    }
}

pub fn write_tool() -> ToolSpec {
    ToolSpec {
        name: "fs_write",
        description: "Write UTF-8 text to a file inside the project",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        }),
        handler: write_handler,
    }
}

pub fn patch_tool() -> ToolSpec {
    ToolSpec {
        name: "fs_patch",
        description: "Apply exact substring replacements in a UTF-8 text file. \
            Prefer over fs_write when fixing a session-scoped skill_run script_path or other large files: \
            read once, patch locally, rerun skill_run with path. \
            Each edit must match exactly once unless replace_all is true. \
            Atomic: if any edit fails to match, NO changes are written and the error lists every missed edit.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "edits": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "old": {
                                "type": "string",
                                "description": "Exact substring to find"
                            },
                            "new": {
                                "type": "string",
                                "description": "Replacement text"
                            },
                            "replace_all": {
                                "type": "boolean",
                                "default": false,
                                "description": "Replace every occurrence; default replaces only when old is unique"
                            }
                        },
                        "required": ["old", "new"]
                    },
                    "minItems": 1
                }
            },
            "required": ["path", "edits"]
        }),
        handler: patch_handler,
    }
}

pub fn search_tool() -> ToolSpec {
    ToolSpec {
        name: "fs_search",
        description: "Search for files by name substring inside the project",
        parameters: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            },
            "required": ["query"]
        }),
        handler: search_handler,
    }
}

fn list_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
    let resolved = ctx.sandbox.resolve(path)?;
    let mut entries = Vec::new();
    for entry in fs::read_dir(&resolved).map_err(|e| ToolError::Execution(e.to_string()))? {
        let entry = entry.map_err(|e| ToolError::Execution(e.to_string()))?;
        let meta = entry
            .metadata()
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        entries.push(json!({
            "name": entry.file_name().to_string_lossy(),
            "is_dir": meta.is_dir(),
        }));
    }
    Ok(json!({ "entries": entries }))
}

fn read_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let resolved = ctx.sandbox.resolve(path)?;
    let content = fs::read_to_string(&resolved).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({ "content": content }))
}

fn patch_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let edits = args
        .get("edits")
        .and_then(|v| v.as_array())
        .filter(|items| !items.is_empty())
        .ok_or_else(|| ToolError::InvalidArgs("edits required".into()))?;

    let resolved = ctx.sandbox.resolve_for_write(path)?;
    let mut content =
        fs::read_to_string(&resolved).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut applied = 0u64;
    let mut missed = Vec::new();

    for edit in edits {
        let old = edit
            .get("old")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("each edit needs old".into()))?;
        let new = edit
            .get("new")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("each edit needs new".into()))?;
        if old.is_empty() {
            return Err(ToolError::InvalidArgs("edit old must not be empty".into()));
        }
        if old == new {
            return Err(ToolError::InvalidArgs(
                "edit old and new are identical".into(),
            ));
        }
        let replace_all = edit
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let count = content.matches(old).count();
        if count == 0 {
            missed.push(json!({ "old": old, "reason": "not found" }));
        } else if !replace_all && count > 1 {
            missed.push(json!({
                "old": old,
                "reason": "multiple matches",
                "matches": count,
                "hint": "include more surrounding context or set replace_all"
            }));
        } else {
            content = content.replace(old, new);
            applied += count as u64;
        }
    }

    // 全有或全无：任一 edit 未命中则不写盘，避免重试时已应用 edit 变成 not found
    if !missed.is_empty() {
        return Err(ToolError::Structured(json!({
            "error": "fs_patch not applied",
            "path": path,
            "missed": missed,
            "hint": "No changes were written. Fix the missed edits (old must match the file exactly) and retry with the full edit list."
        })));
    }

    fs::write(&resolved, &content).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({ "path": path, "applied": applied }))
}

fn write_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("content required".into()))?;
    let resolved = ctx.sandbox.resolve_for_write(path)?;
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::Execution(e.to_string()))?;
    }
    fs::write(&resolved, content).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({ "written": resolved.display().to_string() }))
}

fn search_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("query required".into()))?;
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    for entry in WalkDir::new(ctx.sandbox.root())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.contains(&query_lower) {
            let path = entry
                .path()
                .strip_prefix(ctx.sandbox.root())
                .unwrap_or(entry.path())
                .display()
                .to_string();
            matches.push(json!({ "path": path, "is_dir": entry.file_type().is_dir() }));
        }
    }
    Ok(json!({ "matches": matches }))
}
