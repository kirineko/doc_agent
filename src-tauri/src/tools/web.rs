use super::{required_str_arg, ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};
use std::time::Duration;
use tavily::{SearchRequest, Tavily};

const MAX_RESULTS_CAP: i32 = 10;
const MAX_EXTRACT_URLS: usize = 5;
const CONTENT_TRUNCATE: usize = 8000;

pub fn search_tool() -> ToolSpec {
    ToolSpec {
        name: "web_search",
        description: "Search the public web for up-to-date information outside the project. \
            Returns a synthesized answer and source snippets. Use one comprehensive query when possible.",
        parameters: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "max_results": { "type": "integer", "default": 5, "description": "Max sources (1-10)" },
                "search_depth": {
                    "type": "string",
                    "enum": ["basic", "advanced"],
                    "default": "basic",
                    "description": "basic = faster; advanced = deeper results"
                }
            },
            "required": ["query"]
        }),
        handler: search_stub,
    }
}

pub fn extract_tool() -> ToolSpec {
    ToolSpec {
        name: "web_extract",
        description: "Extract readable text content from 1-5 public HTTP(S) URLs.",
        parameters: json!({
            "type": "object",
            "properties": {
                "urls": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": 5,
                    "description": "HTTP(S) URLs to extract"
                }
            },
            "required": ["urls"]
        }),
        handler: extract_stub,
    }
}

fn search_stub(_ctx: &ToolContext, _args: Value) -> Result<Value, ToolError> {
    Err(ToolError::NotImplemented)
}

fn extract_stub(_ctx: &ToolContext, _args: Value) -> Result<Value, ToolError> {
    Err(ToolError::NotImplemented)
}

pub async fn search_handler(ctx: &ToolContext<'_>, args: Value) -> Result<Value, ToolError> {
    let query = required_str_arg(&args, "query")?;
    if query.trim().is_empty() {
        return Err(ToolError::InvalidArgs("query required".into()));
    }

    let max_results = args
        .get("max_results")
        .and_then(|v| v.as_i64())
        .unwrap_or(5)
        .clamp(1, MAX_RESULTS_CAP as i64) as i32;

    let depth = args
        .get("search_depth")
        .and_then(|v| v.as_str())
        .unwrap_or("basic");
    let search_depth = match depth {
        "basic" | "advanced" => depth,
        other => {
            return Err(ToolError::InvalidArgs(format!(
                "search_depth must be basic or advanced, got {other}"
            )));
        }
    };

    let api_key = tavily_api_key(ctx)?;
    let tavily = build_client(&api_key)?;

    let request = SearchRequest::new(&api_key, query.trim())
        .include_answer(true)
        .max_results(max_results)
        .search_depth(search_depth);

    let response = tavily
        .call(&request)
        .await
        .map_err(|e| ToolError::Execution(format!("Tavily search failed: {e}")))?;

    let results: Vec<Value> = response
        .results
        .iter()
        .map(|r| {
            json!({
                "title": r.title,
                "url": r.url,
                "content": r.content,
                "score": r.score,
            })
        })
        .collect();

    Ok(json!({
        "query": response.query,
        "answer": response.answer,
        "results": results,
        "follow_up_questions": response.follow_up_questions,
    }))
}

pub async fn extract_handler(ctx: &ToolContext<'_>, args: Value) -> Result<Value, ToolError> {
    let urls_value = args
        .get("urls")
        .ok_or_else(|| ToolError::InvalidArgs("urls required".into()))?;
    let urls_array = urls_value
        .as_array()
        .ok_or_else(|| ToolError::InvalidArgs("urls must be an array".into()))?;
    if urls_array.is_empty() {
        return Err(ToolError::InvalidArgs("urls must not be empty".into()));
    }
    if urls_array.len() > MAX_EXTRACT_URLS {
        return Err(ToolError::InvalidArgs(format!(
            "urls must contain at most {MAX_EXTRACT_URLS} items"
        )));
    }

    let mut urls = Vec::with_capacity(urls_array.len());
    for item in urls_array {
        let url = item
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("each url must be a string".into()))?
            .trim();
        if url.is_empty() {
            return Err(ToolError::InvalidArgs("url must not be empty".into()));
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ToolError::InvalidArgs(format!(
                "url must start with http:// or https://, got {url}"
            )));
        }
        urls.push(url.to_string());
    }

    let api_key = tavily_api_key(ctx)?;
    let tavily = build_client(&api_key)?;
    let response = tavily
        .extract(urls.clone())
        .await
        .map_err(|e| ToolError::Execution(format!("Tavily extract failed: {e}")))?;

    let results: Vec<Value> = response
        .results
        .iter()
        .map(|item| {
            let mut entry = json!({
                "url": item.url,
                "content": item.raw_content,
            });
            truncate_field(&mut entry, "content", &item.raw_content);
            entry
        })
        .collect();

    let failed: Vec<Value> = response
        .failed_results
        .iter()
        .map(|item| json!({ "url": item.url, "error": item.error }))
        .collect();

    Ok(json!({
        "results": results,
        "failed_results": failed,
        "response_time": response.response_time,
    }))
}

fn tavily_api_key(ctx: &ToolContext<'_>) -> Result<String, ToolError> {
    let secrets = ctx
        .secrets
        .ok_or_else(|| ToolError::Execution("Tavily API key not configured".into()))?;
    secrets
        .get_api_key("tavily")
        .map_err(|e| ToolError::Execution(e.to_string()))?
        .ok_or_else(|| ToolError::Execution("Tavily API key not configured".into()))
}

fn build_client(api_key: &str) -> Result<Tavily, ToolError> {
    Tavily::builder(api_key)
        .timeout(Duration::from_secs(60))
        .max_retries(3)
        .build()
        .map_err(|e| ToolError::Execution(format!("failed to init Tavily client: {e}")))
}

fn truncate_field(item: &mut Value, field: &str, content: &str) {
    if content.len() <= CONTENT_TRUNCATE {
        return;
    }
    // 按字节上限截断，但必须落在 UTF-8 字符边界上，否则切片会 panic
    let mut end = CONTENT_TRUNCATE;
    while !content.is_char_boundary(end) {
        end -= 1;
    }
    item[field] = json!(format!("{}…", &content[..end]));
    item["truncated"] = json!(true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use crate::core::secrets::Secrets;
    use tempfile::tempdir;

    struct TestFixture {
        _dir: tempfile::TempDir,
        sandbox: Sandbox,
        secrets: Secrets,
    }

    impl TestFixture {
        fn new() -> Self {
            let dir = tempdir().unwrap();
            let secrets = Secrets::new(dir.path().join("config.toml"));
            let sandbox = Sandbox::new(dir.path()).unwrap();
            Self {
                _dir: dir,
                sandbox,
                secrets,
            }
        }

        fn ctx(&self) -> ToolContext<'_> {
            ToolContext::with_secrets(&self.sandbox, &self.secrets)
        }
    }

    #[tokio::test]
    async fn web_search_rejects_empty_query() {
        let fixture = TestFixture::new();
        let err = search_handler(&fixture.ctx(), json!({ "query": "   " }))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidArgs(_)));
    }

    #[tokio::test]
    async fn web_search_requires_api_key() {
        let fixture = TestFixture::new();
        let err = search_handler(&fixture.ctx(), json!({ "query": "rust programming" }))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::Execution(_)));
        assert!(err.to_string().contains("not configured"));
    }

    #[tokio::test]
    async fn web_extract_rejects_too_many_urls() {
        let fixture = TestFixture::new();
        let urls: Vec<String> = (0..6).map(|i| format!("https://example.com/{i}")).collect();
        let err = extract_handler(&fixture.ctx(), json!({ "urls": urls }))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidArgs(_)));
    }

    #[test]
    fn truncate_field_respects_utf8_char_boundary() {
        // 全中文内容（3 字节/字符），8000 不是字符边界，按字节切片会 panic
        let content = "汉".repeat(4000);
        assert!(content.len() > CONTENT_TRUNCATE);
        let mut entry = json!({ "url": "https://example.com", "content": content });
        truncate_field(&mut entry, "content", &content);
        assert_eq!(entry["truncated"], json!(true));
        let truncated = entry["content"].as_str().unwrap();
        assert!(truncated.len() <= CONTENT_TRUNCATE + '…'.len_utf8());
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn truncate_field_keeps_short_content() {
        let content = "short";
        let mut entry = json!({ "url": "https://example.com", "content": content });
        truncate_field(&mut entry, "content", content);
        assert_eq!(entry["content"], json!("short"));
        assert!(entry.get("truncated").is_none());
    }

    #[tokio::test]
    async fn web_extract_rejects_empty_urls() {
        let fixture = TestFixture::new();
        let err = extract_handler(&fixture.ctx(), json!({ "urls": [] }))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidArgs(_)));
    }
}
