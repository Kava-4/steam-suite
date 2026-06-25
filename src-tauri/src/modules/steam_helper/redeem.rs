use super::rate_limit::{self, SteamEndpoint};
use super::RedeemResult;
use crate::modules::settings::AppSettings;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RegisterKeyResponse {
    success: Option<u8>,
    #[serde(rename = "purchase_result_details")]
    purchase_result_details: Option<u32>,
    #[serde(rename = "purchase_receipt_info")]
    purchase_receipt_info: Option<serde_json::Value>,
}

pub async fn redeem_product_key(settings: &AppSettings, key: &str) -> Result<RedeemResult, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Enter a product key.".into());
    }

    if settings.steam_session_id.is_empty() || settings.steam_login_secure.is_empty() {
        return Err("Steam web session required. Sign in via Settings → Credentials.".into());
    }

    rate_limit::acquire(SteamEndpoint::Store).await?;

    let client = reqwest::Client::new();
    let cookie = format!(
        "sessionid={}; steamLoginSecure={}",
        settings.steam_session_id, settings.steam_login_secure
    );

    let response = client
        .post("https://store.steampowered.com/account/ajaxregisterkey/")
        .header("Cookie", &cookie)
        .header("Referer", "https://store.steampowered.com/account/registerkey")
        .header("Origin", "https://store.steampowered.com")
        .form(&[
            ("key", key),
            ("sessionid", settings.steam_session_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status().as_u16();
    if status == 403 || status == 429 || status == 503 {
        rate_limit::report_rate_limited(SteamEndpoint::Store, status);
        return Err(
            rate_limit::cooldown_message(SteamEndpoint::Store)
                .unwrap_or_else(|| format!("Steam Store HTTP {status}.")),
        );
    }

    if !response.status().is_success() {
        return Err(format!("Steam HTTP {}", response.status()));
    }

    let body: RegisterKeyResponse = response.json().await.map_err(|e| e.to_string())?;

    match body.success.unwrap_or(0) {
        1 => Ok(RedeemResult {
            success: true,
            message: "Key redeemed successfully.".into(),
        }),
        2 => Ok(RedeemResult {
            success: true,
            message: "You already own this product.".into(),
        }),
        9 => Ok(RedeemResult {
            success: false,
            message: "Invalid product key.".into(),
        }),
        13 => Ok(RedeemResult {
            success: false,
            message: "This key cannot be redeemed in your region.".into(),
        }),
        14 => Ok(RedeemResult {
            success: false,
            message: "Too many redemption attempts. Try again later.".into(),
        }),
        15 => Ok(RedeemResult {
            success: false,
            message: "Key already used by another account.".into(),
        }),
        _ => {
            let detail = body
                .purchase_result_details
                .map(|d| format!(" (code {d})"))
                .unwrap_or_default();
            Ok(RedeemResult {
                success: false,
                message: format!("Redemption failed{detail}."),
            })
        }
    }
}
