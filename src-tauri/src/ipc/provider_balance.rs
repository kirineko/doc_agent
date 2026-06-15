use crate::core::provider_balance::{self, ProviderBalanceRow};
use crate::state::AppState;
use tauri::State;

async fn query_deepseek(
    secrets: &crate::core::secrets::Secrets,
    client: &reqwest::Client,
) -> Result<Option<ProviderBalanceRow>, String> {
    if !secrets.has_api_key("deepseek").map_err(|e| e.to_string())? {
        return Ok(None);
    }

    let Some(api_key) = secrets.get_api_key("deepseek").map_err(|e| e.to_string())? else {
        return Ok(None);
    };

    let display = provider_balance::fetch_deepseek_balance(client, &api_key).await;
    Ok(Some(ProviderBalanceRow {
        provider: "deepseek".to_string(),
        display,
    }))
}

async fn query_kimi(
    secrets: &crate::core::secrets::Secrets,
    client: &reqwest::Client,
) -> Result<Option<ProviderBalanceRow>, String> {
    if !secrets.has_api_key("kimi").map_err(|e| e.to_string())? {
        return Ok(None);
    }

    let Some(api_key) = secrets.get_api_key("kimi").map_err(|e| e.to_string())? else {
        return Ok(None);
    };

    let display = provider_balance::fetch_kimi_balance(client, &api_key).await;
    Ok(Some(ProviderBalanceRow {
        provider: "kimi".to_string(),
        display,
    }))
}

#[tauri::command]
pub async fn fetch_provider_balances(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderBalanceRow>, String> {
    let client = provider_balance::build_balance_client().map_err(|e| e.to_string())?;
    let secrets = state.secrets.clone();
    let (deepseek, kimi) = tokio::join!(
        query_deepseek(&secrets, &client),
        query_kimi(&secrets, &client),
    );
    Ok(provider_balance::merge_optional_rows(deepseek?, kimi?))
}
