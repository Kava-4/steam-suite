use crate::modules::settings::{self, AppSettings, SteamAccountProfile};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn owned_games_path(steam_id: &str) -> PathBuf {
    if steam_id.is_empty() {
        return settings::data_dir().join("owned_games.json");
    }
    settings::data_dir().join(format!("owned_games_{steam_id}.json"))
}

pub fn known_wins_path(steam_id: &str) -> PathBuf {
    if steam_id.is_empty() {
        return settings::data_dir().join("known_wins.json");
    }
    settings::data_dir().join(format!("known_wins_{steam_id}.json"))
}

pub fn profile_from_settings(settings: &AppSettings) -> SteamAccountProfile {
    SteamAccountProfile {
        steam_session_id: settings.steam_session_id.clone(),
        steam_login_secure: settings.steam_login_secure.clone(),
        steam_machine_auth: settings.steam_machine_auth.clone(),
        steam_credentials_user: settings.steam_credentials_user.clone(),
        steam_api_key: settings.steam_api_key.clone(),
        steam_country_code: settings.steam_country_code.clone(),
        idle_game_ids: settings.idle_game_ids.clone(),
        farm_game_ids: settings.farm_game_ids.clone(),
        auto_idle_on_start: settings.auto_idle_on_start,
        card_farming_enabled: settings.card_farming_enabled,
        steamgifts_cookie: settings.steamgifts_cookie.clone(),
        indiegala_cookie: settings.indiegala_cookie.clone(),
    }
}

pub fn apply_profile_to_settings(settings: &mut AppSettings, profile: &SteamAccountProfile) {
    settings.steam_session_id = profile.steam_session_id.clone();
    settings.steam_login_secure = profile.steam_login_secure.clone();
    settings.steam_machine_auth = profile.steam_machine_auth.clone();
    settings.steam_credentials_user = profile.steam_credentials_user.clone();
    settings.steam_api_key = profile.steam_api_key.clone();
    settings.steam_country_code = if profile.steam_country_code.is_empty() {
        "eu".to_string()
    } else {
        profile.steam_country_code.clone()
    };
    settings.idle_game_ids = profile.idle_game_ids.clone();
    settings.farm_game_ids = profile.farm_game_ids.clone();
    settings.auto_idle_on_start = profile.auto_idle_on_start;
    settings.card_farming_enabled = profile.card_farming_enabled;
    settings.steamgifts_cookie = profile.steamgifts_cookie.clone();
    settings.indiegala_cookie = profile.indiegala_cookie.clone();
}

pub fn persist_active_profile(settings: &mut AppSettings) {
    let steam_id = settings.steam_id.trim();
    if steam_id.is_empty() {
        return;
    }
    let profile = profile_from_settings(settings);
    settings
        .account_profiles
        .insert(steam_id.to_string(), profile);
}

pub fn apply_active_profile(settings: &mut AppSettings) {
    let steam_id = settings.steam_id.trim();
    if steam_id.is_empty() {
        return;
    }
    let profile = settings
        .account_profiles
        .get(steam_id)
        .cloned()
        .unwrap_or_default();
    apply_profile_to_settings(settings, &profile);
}

pub fn migrate_legacy(settings: &mut AppSettings) {
    if !settings.account_profiles.is_empty() {
        return;
    }

    if settings.steam_id.is_empty() {
        if let Ok(Some(acc)) = crate::modules::steam_helper::session::get_client_active_account() {
            settings.steam_id = acc.steam_id;
        }
    }

    if settings.steam_id.is_empty() {
        return;
    }

    settings.account_profiles.insert(
        settings.steam_id.clone(),
        profile_from_settings(settings),
    );

    migrate_owned_games_file(&settings.steam_id);
}

pub fn backfill_per_account_api(settings: &mut AppSettings) {
    if settings.account_profiles.is_empty() {
        return;
    }

    let global_key = settings.steam_api_key.clone();
    let global_cc = settings.steam_country_code.clone();

    for profile in settings.account_profiles.values_mut() {
        if profile.steam_api_key.is_empty() && !global_key.is_empty() {
            profile.steam_api_key = global_key.clone();
        }
        if profile.steam_country_code.is_empty() && !global_cc.is_empty() {
            profile.steam_country_code = global_cc.clone();
        }
    }
}

pub fn migrate_owned_games_file(steam_id: &str) {
    let legacy = settings::data_dir().join("owned_games.json");
    let target = owned_games_path(steam_id);
    if legacy.exists() && !target.exists() {
        let _ = std::fs::copy(&legacy, &target);
    }

    let legacy_wins = settings::data_dir().join("known_wins.json");
    let target_wins = known_wins_path(steam_id);
    if legacy_wins.exists() && !target_wins.exists() {
        let _ = std::fs::copy(&legacy_wins, &target_wins);
    }
}

pub fn switch_account(settings: &mut AppSettings, steam_id: &str) -> Result<(), String> {
    let steam_id = steam_id.trim();
    if steam_id.is_empty() {
        return Err("Steam ID is required.".into());
    }

    let known = crate::modules::steam_helper::session::get_steam_accounts()?
        .into_iter()
        .any(|a| a.steam_id == steam_id);
    if !known {
        return Err("Account not found in local Steam login list.".into());
    }

    persist_active_profile(settings);
    settings.steam_id = steam_id.to_string();
    if !settings.account_profiles.contains_key(steam_id) {
        settings
            .account_profiles
            .insert(steam_id.to_string(), SteamAccountProfile::default());
    }
    apply_active_profile(settings);
    migrate_owned_games_file(steam_id);
    settings::save_settings(settings)
}

pub fn update_credentials(
    settings: &mut AppSettings,
    steam_id: &str,
    session_id: String,
    login_secure: String,
    machine_auth: String,
    credentials_user: String,
) -> Result<(), String> {
    persist_active_profile(settings);

    let profile = settings
        .account_profiles
        .entry(steam_id.to_string())
        .or_default();
    profile.steam_session_id = session_id;
    profile.steam_login_secure = login_secure;
    profile.steam_machine_auth = machine_auth;
    profile.steam_credentials_user = credentials_user;

    if settings.steam_id == steam_id {
        apply_active_profile(settings);
    }

    settings::save_settings(settings)
}

pub fn clear_credentials(settings: &mut AppSettings, steam_id: &str) -> Result<(), String> {
    persist_active_profile(settings);

    if let Some(profile) = settings.account_profiles.get_mut(steam_id) {
        profile.steam_session_id.clear();
        profile.steam_login_secure.clear();
        profile.steam_machine_auth.clear();
        profile.steam_credentials_user.clear();
    }

    if settings.steam_id == steam_id {
        apply_active_profile(settings);
    }

    settings::save_settings(settings)
}

pub fn ensure_account_profiles_map(settings: &mut AppSettings) {
    if settings.account_profiles.is_empty() {
        settings.account_profiles = HashMap::new();
    }
}
