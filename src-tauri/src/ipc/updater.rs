use serde::Deserialize;
use std::time::Duration;

const MANIFEST_URL: &str = "https://doc-agent.oss-cn-guangzhou.aliyuncs.com/latest.json";

#[derive(Debug, Deserialize)]
struct LatestManifest {
    version: String,
}

#[tauri::command]
pub async fn fetch_latest_release_version() -> Result<Option<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(MANIFEST_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let manifest: LatestManifest = response.json().await.map_err(|e| e.to_string())?;
    let version = manifest.version.trim();
    if version.is_empty() {
        Ok(None)
    } else {
        Ok(Some(version.to_string()))
    }
}
