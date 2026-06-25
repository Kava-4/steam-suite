use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SteamAccountProfile {
    pub steam_session_id: String,
    pub steam_login_secure: String,
    pub steam_machine_auth: String,
    pub steam_credentials_user: String,
    pub steam_api_key: String,
    pub steam_country_code: String,
    pub idle_game_ids: Vec<u32>,
    pub farm_game_ids: Vec<u32>,
    pub auto_idle_on_start: bool,
    pub card_farming_enabled: bool,
    pub steamgifts_cookie: String,
    pub indiegala_cookie: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub steam_api_key: String,
    pub steam_id: String,
    pub steam_session_id: String,
    pub steam_login_secure: String,
    pub steam_machine_auth: String,
    pub steam_credentials_user: String,
    pub steam_country_code: String,
    pub utility_path: String,
    #[serde(default)]
    pub saveslot_cli_path: String,
    pub idle_game_ids: Vec<u32>,
    pub farm_game_ids: Vec<u32>,
    pub auto_idle_on_start: bool,
    pub card_farming_enabled: bool,
    pub max_idle_games: u32,
    pub steamgifts_cookie: String,
    pub indiegala_cookie: String,
    pub enable_indiegala: bool,
    pub refresh_delay_minutes: u32,
    pub max_pages: u32,
    pub max_giveaway_end_hours: u32,
    pub indiegala_entry_delay: u32,
    pub indiegala_min_cost: u32,
    pub manual_select_giveaways: bool,
    pub notify_on_win: bool,
    pub auto_redeem_on_win: bool,
    pub scheduler_enabled: bool,
    pub scheduler_tasks: Vec<String>,
    pub start_with_windows: bool,
    pub minimize_to_tray_on_close: bool,
    #[serde(default)]
    pub account_profiles: HashMap<String, SteamAccountProfile>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            steam_api_key: String::new(),
            steam_id: String::new(),
            steam_session_id: String::new(),
            steam_login_secure: String::new(),
            steam_machine_auth: String::new(),
            steam_credentials_user: String::new(),
            steam_country_code: "eu".to_string(),
            utility_path: String::new(),
            saveslot_cli_path: String::new(),
            idle_game_ids: Vec::new(),
            farm_game_ids: Vec::new(),
            auto_idle_on_start: false,
            card_farming_enabled: false,
            max_idle_games: 32,
            steamgifts_cookie: String::new(),
            indiegala_cookie: String::new(),
            enable_indiegala: false,
            refresh_delay_minutes: 10,
            max_pages: 5,
            max_giveaway_end_hours: 3,
            indiegala_entry_delay: 5,
            indiegala_min_cost: 0,
            manual_select_giveaways: false,
            notify_on_win: true,
            auto_redeem_on_win: false,
            scheduler_enabled: false,
            scheduler_tasks: vec![
                "card-farming".to_string(),
                "auto-idler".to_string(),
                "giveaways".to_string(),
                "redeem".to_string(),
            ],
            start_with_windows: false,
            minimize_to_tray_on_close: true,
            account_profiles: HashMap::new(),
        }
    }
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-suite")
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    if !path.exists() {
        return AppSettings::default();
    }

    let mut settings = match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    };

    crate::modules::accounts::migrate_legacy(&mut settings);
    crate::modules::accounts::backfill_per_account_api(&mut settings);
    crate::modules::accounts::apply_active_profile(&mut settings);
    settings
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let mut to_save = settings.clone();
    crate::modules::accounts::persist_active_profile(&mut to_save);

    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&to_save).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}
