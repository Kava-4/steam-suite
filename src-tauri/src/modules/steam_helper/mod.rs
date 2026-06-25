pub mod achievements;
pub mod cards;
pub mod credentials;
pub mod idling;
pub mod inventory;
pub mod library;
pub mod ops;
pub mod paths;
pub mod profile;
pub mod rate_limit;
pub mod redeem;
pub mod session;
pub mod steeeam;
pub mod utility;

use serde::{Deserialize, Serialize};

pub use achievements::AchievementInfo;
pub use cards::CardEnrichResult;
pub use inventory::{InventoryGameSummary, InventoryItem};
pub use rate_limit::SteamRateLimitStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamClientStatus {
    pub steam_running: bool,
    pub steam_user: Option<String>,
    pub steam_id: Option<String>,
    pub utility_ready: bool,
    pub utility_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamGame {
    pub app_id: u32,
    pub name: String,
    pub playtime_forever: u32,
    pub img_url: String,
    pub has_cards: bool,
    pub is_farming: bool,
    pub is_idling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedeemResult {
    pub success: bool,
    pub message: String,
}

pub fn client_status(utility_path_setting: &str) -> SteamClientStatus {
    let utility = paths::resolve_utility_path(utility_path_setting);
    let account = session::get_active_steam_account().ok().flatten();
    SteamClientStatus {
        steam_running: paths::is_steam_running(),
        steam_user: account.as_ref().map(|a| a.persona_name.clone()),
        steam_id: account.map(|a| a.steam_id),
        utility_ready: utility.is_ok(),
        utility_path: utility.ok().map(|p| p.display().to_string()),
        error: if paths::is_steam_running() {
            None
        } else {
            Some("Steam client is not running. Launch Steam and sign in.".into())
        },
    }
}
