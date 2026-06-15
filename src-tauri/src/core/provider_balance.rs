use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const BALANCE_UNAVAILABLE: &str = "—";

const DEEPSEEK_BALANCE_URL: &str = "https://api.deepseek.com/user/balance";
const KIMI_BALANCE_URL: &str = "https://api.moonshot.cn/v1/users/me/balance";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProviderBalanceRow {
    pub provider: String,
    pub display: String,
}

#[derive(Debug, Deserialize)]
struct DeepSeekBalanceResponse {
    balance_infos: Vec<DeepSeekBalanceInfo>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekBalanceInfo {
    currency: String,
    total_balance: String,
}

#[derive(Debug, Deserialize)]
struct KimiBalanceResponse {
    code: i32,
    data: Option<KimiBalanceData>,
}

#[derive(Debug, Deserialize)]
struct KimiBalanceData {
    available_balance: f64,
}

pub fn format_cny_display(amount: f64) -> String {
    format!("¥{amount:.2}")
}

pub fn parse_deepseek_balance(body: &str) -> String {
    let parsed: DeepSeekBalanceResponse = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return BALANCE_UNAVAILABLE.to_string(),
    };

    for info in parsed.balance_infos {
        if info.currency == "CNY" {
            if let Ok(amount) = info.total_balance.parse::<f64>() {
                return format_cny_display(amount);
            }
            return BALANCE_UNAVAILABLE.to_string();
        }
    }

    BALANCE_UNAVAILABLE.to_string()
}

pub fn parse_kimi_balance(body: &str) -> String {
    let parsed: KimiBalanceResponse = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return BALANCE_UNAVAILABLE.to_string(),
    };

    if parsed.code != 0 {
        return BALANCE_UNAVAILABLE.to_string();
    }

    parsed
        .data
        .map(|data| format_cny_display(data.available_balance))
        .unwrap_or_else(|| BALANCE_UNAVAILABLE.to_string())
}

pub fn merge_optional_rows(
    deepseek: Option<ProviderBalanceRow>,
    kimi: Option<ProviderBalanceRow>,
) -> Vec<ProviderBalanceRow> {
    [deepseek, kimi].into_iter().flatten().collect()
}

pub fn build_balance_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
}

async fn fetch_balance_display(
    client: &reqwest::Client,
    url: &str,
    api_key: &str,
    parse: fn(&str) -> String,
) -> String {
    let response = match client.get(url).bearer_auth(api_key).send().await {
        Ok(response) if response.status().is_success() => response,
        _ => return BALANCE_UNAVAILABLE.to_string(),
    };

    let body = match response.text().await {
        Ok(body) => body,
        Err(_) => return BALANCE_UNAVAILABLE.to_string(),
    };

    parse(&body)
}

pub async fn fetch_deepseek_balance(client: &reqwest::Client, api_key: &str) -> String {
    fetch_balance_display(
        client,
        DEEPSEEK_BALANCE_URL,
        api_key,
        parse_deepseek_balance,
    )
    .await
}

pub async fn fetch_kimi_balance(client: &reqwest::Client, api_key: &str) -> String {
    fetch_balance_display(client, KIMI_BALANCE_URL, api_key, parse_kimi_balance).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_cny_display_rounds_to_two_decimals() {
        assert_eq!(format_cny_display(49.58894), "¥49.59");
        assert_eq!(format_cny_display(12.3), "¥12.30");
    }

    #[test]
    fn parse_deepseek_cny_balance() {
        let body = r#"{"is_available":true,"balance_infos":[{"currency":"CNY","total_balance":"12.34","granted_balance":"0","topped_up_balance":"12.34"}]}"#;
        assert_eq!(parse_deepseek_balance(body), "¥12.34");
    }

    #[test]
    fn parse_deepseek_no_cny_returns_unavailable() {
        let body = r#"{"balance_infos":[{"currency":"USD","total_balance":"1.00"}]}"#;
        assert_eq!(parse_deepseek_balance(body), BALANCE_UNAVAILABLE);
    }

    #[test]
    fn parse_deepseek_invalid_json_returns_unavailable() {
        assert_eq!(parse_deepseek_balance("not-json"), BALANCE_UNAVAILABLE);
    }

    #[test]
    fn parse_kimi_success() {
        let body = r#"{"code":0,"data":{"available_balance":49.58894,"voucher_balance":46.58893,"cash_balance":3.00001},"status":true}"#;
        assert_eq!(parse_kimi_balance(body), "¥49.59");
    }

    #[test]
    fn parse_kimi_error_code_returns_unavailable() {
        let body = r#"{"code":1,"data":null,"status":false}"#;
        assert_eq!(parse_kimi_balance(body), BALANCE_UNAVAILABLE);
    }

    #[test]
    fn merge_optional_rows_skips_unconfigured_providers() {
        let deepseek = ProviderBalanceRow {
            provider: "deepseek".to_string(),
            display: "¥1.00".to_string(),
        };
        assert_eq!(
            merge_optional_rows(Some(deepseek.clone()), None),
            vec![deepseek]
        );
        assert!(merge_optional_rows(None, None).is_empty());
    }
}
