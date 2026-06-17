use super::{ToolContext, ToolError, ToolSpec};
use crate::agent::types::{ClarifyOption, ClarifyQuestion};
use serde_json::{json, Map, Value};

const MIN_OPTIONS: usize = 2;
const MAX_OPTIONS: usize = 12;

pub fn ask_tool() -> ToolSpec {
    ToolSpec {
        name: "clarify_ask",
        description: "Ask the user one structured clarification question. \
            Use this during the clarify skill flow instead of plain text questions. \
            The agent loop pauses and waits for the user's answer. \
            kind MUST be one of: single, multi, text, confirm_brief. \
            single/multi require 2-12 options (prefer 2-8 plus allow_custom for 其他); \
            confirm_brief requires brief.",
        parameters: json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "kind": {
                    "type": "string",
                    "enum": ["single", "multi", "text", "confirm_brief"]
                },
                "prompt": { "type": "string" },
                "description": { "type": "string" },
                "options": {
                    "type": "array",
                    "minItems": MIN_OPTIONS,
                    "maxItems": MAX_OPTIONS,
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "label": { "type": "string" },
                            "hint": { "type": "string" }
                        },
                        "required": ["id", "label"]
                    }
                },
                "allow_custom": { "type": "boolean", "default": true },
                "custom_label": { "type": "string" },
                "custom_placeholder": { "type": "string" },
                "min_selections": { "type": "integer" },
                "max_selections": { "type": "integer" },
                "brief": {
                    "type": "object",
                    "description": "confirm_brief 必填。顶层扁平字段（文档类型、主题/目标、受众/场景、结构、排版风格、样式要点、特殊要求），每个值 MUST 为 string，禁止嵌套 object",
                    "additionalProperties": {
                        "type": "string"
                    }
                }
            },
            "required": ["id", "kind", "prompt"]
        }),
        handler: ask_handler,
    }
}

fn ask_handler(_ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let question = parse_question(args)?;
    serde_json::to_value(question).map_err(|e| ToolError::Execution(e.to_string()))
}

pub fn parse_question(args: Value) -> Result<ClarifyQuestion, ToolError> {
    let id = required_non_empty(&args, "id")?;
    let kind = required_non_empty(&args, "kind")?;
    let prompt = required_non_empty(&args, "prompt")?;
    let description = optional_string(&args, "description");
    let allow_custom = args
        .get("allow_custom")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let options = parse_options(args.get("options"))?;
    let custom_label = optional_string(&args, "custom_label");
    let custom_placeholder = optional_string(&args, "custom_placeholder");
    let min_selections = optional_usize(&args, "min_selections")?;
    let max_selections = optional_usize(&args, "max_selections")?;
    let brief = match args.get("brief") {
        None => None,
        Some(value) => Some(normalize_brief(value.clone())?),
    };

    match kind.as_str() {
        "single" | "multi" => validate_options(&kind, &options)?,
        "text" => {}
        "confirm_brief" => {
            if !options.is_empty() {
                validate_options(&kind, &options)?;
            }
            let Some(brief) = &brief else {
                return Err(invalid("confirm_brief requires brief object"));
            };
            let Value::Object(map) = brief else {
                return Err(invalid("confirm_brief requires brief object"));
            };
            if map.is_empty() {
                return Err(invalid("confirm_brief brief must not be empty"));
            }
        }
        _ => {
            return Err(invalid(
                "kind must be one of: single, multi, text, confirm_brief",
            ))
        }
    }

    if matches!(kind.as_str(), "multi") {
        if let (Some(min), Some(max)) = (min_selections, max_selections) {
            if min > max {
                return Err(invalid("min_selections must be <= max_selections"));
            }
        }
        if let Some(max) = max_selections {
            if max > options.len() {
                return Err(invalid("max_selections must not exceed options length"));
            }
        }
    }

    Ok(ClarifyQuestion {
        id,
        kind,
        prompt,
        description,
        options,
        allow_custom,
        custom_label,
        custom_placeholder,
        min_selections,
        max_selections,
        brief,
    })
}

fn required_non_empty(args: &Value, key: &str) -> Result<String, ToolError> {
    let value = args
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| invalid(format!("{key} required")))?;
    Ok(value.to_string())
}

fn optional_string(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn optional_usize(args: &Value, key: &str) -> Result<Option<usize>, ToolError> {
    match args.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_u64()
            .map(|v| v as usize)
            .ok_or_else(|| invalid(format!("{key} must be a positive integer")))
            .map(Some),
    }
}

fn parse_options(value: Option<&Value>) -> Result<Vec<ClarifyOption>, ToolError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let items = value
        .as_array()
        .ok_or_else(|| invalid("options must be an array"))?;
    let mut options = Vec::with_capacity(items.len());
    for item in items {
        let id = required_non_empty(item, "id")?;
        let label = required_non_empty(item, "label")?;
        let hint = optional_string(item, "hint");
        options.push(ClarifyOption { id, label, hint });
    }
    Ok(options)
}

fn validate_options(kind: &str, options: &[ClarifyOption]) -> Result<(), ToolError> {
    if options.len() < MIN_OPTIONS || options.len() > MAX_OPTIONS {
        return Err(invalid(format!(
            "{kind} requires {MIN_OPTIONS}-{MAX_OPTIONS} options"
        )));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> ToolError {
    ToolError::InvalidArgs(message.into())
}

/// LLM 有时会把字段包在 `{ "创作简报": { ... } }` 里；规范为扁平 string map。
fn normalize_brief(brief: Value) -> Result<Value, ToolError> {
    let mut map = brief
        .as_object()
        .cloned()
        .ok_or_else(|| invalid("brief must be an object"))?;

    if map.len() == 1 {
        if let Some((key, Value::Object(inner))) = map.iter().next() {
            if is_brief_wrapper_key(key) {
                map = inner.clone();
            }
        }
    }

    let mut flat = Map::new();
    for (key, value) in map {
        let text = brief_value_to_text(value)?;
        if text.trim().is_empty() {
            continue;
        }
        flat.insert(key, Value::String(text));
    }

    if flat.is_empty() {
        return Err(invalid("confirm_brief brief must not be empty"));
    }
    Ok(Value::Object(flat))
}

fn is_brief_wrapper_key(key: &str) -> bool {
    matches!(
        key.trim(),
        "创作简报"
            | "【创作简报】"
            | "brief"
            | "Brief"
            | "summary"
            | "Summary"
            | "创作简报摘要"
            | "creation_brief"
    )
}

fn brief_value_to_text(value: Value) -> Result<String, ToolError> {
    match value {
        Value::String(text) => Ok(text),
        Value::Number(num) => Ok(num.to_string()),
        Value::Bool(flag) => Ok(flag.to_string()),
        Value::Null => Ok(String::new()),
        Value::Array(items) => {
            let parts: Result<Vec<_>, _> = items.into_iter().map(brief_value_to_text).collect();
            Ok(parts?
                .into_iter()
                .filter(|s| !s.trim().is_empty())
                .collect::<Vec<_>>()
                .join("、"))
        }
        Value::Object(nested) => {
            if nested.is_empty() {
                return Ok(String::new());
            }
            let lines: Result<Vec<_>, _> = nested
                .into_iter()
                .map(|(k, v)| brief_value_to_text(v).map(|text| format!("{k}：{text}")))
                .collect();
            Ok(lines?.join("\n"))
        }
    }
}
