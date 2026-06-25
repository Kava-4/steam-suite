use crate::modules::steam_helper::cards;
use crate::modules::settings;
use crate::modules::steam_helper::achievements::{
    self, AchievementInfo,
};
use crate::modules::steam_helper::idling::{
    list_running, spawn_idle, stop_farm_idle, stop_idle, verify_spawn, RunningIdleProcess,
};
use crate::modules::steam_helper::inventory;
use crate::modules::steam_helper::library;
use crate::modules::steam_helper::paths;
use crate::modules::steam_helper::credentials;
use crate::modules::steam_helper::profile::{self, SteamProfileStats};
use crate::modules::steam_helper::session::{self, SteamAccount, SteamAccountContext};
use crate::modules::steam_helper::{client_status, CardEnrichResult, InventoryGameSummary, InventoryItem, RedeemResult, SteamClientStatus, SteamGame, SteamRateLimitStatus};
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct GameInput {
    #[serde(rename = "appId")]
    pub app_id: u32,
    pub name: String,
}

#[tauri::command]
pub fn steam_get_status(state: State<'_, AppState>) -> SteamClientStatus {
    let settings = state.settings.lock().unwrap();
    client_status(&settings.utility_path)
}

#[tauri::command]
pub async fn steam_get_profile_stats(
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<SteamProfileStats, String> {
    let settings = state.settings.lock().unwrap().clone();
    let steam_id = session::resolve_steam_id(&settings.steam_id)?;
    profile::fetch_profile_stats(
        &settings.steam_api_key,
        &steam_id,
        &settings.steam_country_code,
        force.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub fn steam_get_accounts() -> Result<Vec<SteamAccount>, String> {
    session::get_steam_accounts()
}

#[tauri::command]
pub fn steam_get_account_context(state: State<'_, AppState>) -> Result<SteamAccountContext, String> {
    let settings = state.settings.lock().unwrap();
    let steam_id = session::resolve_steam_id(&settings.steam_id)?;
    session::build_account_context(&steam_id)
}

#[tauri::command]
pub fn steam_switch_account(
    steam_id: String,
    state: State<'_, AppState>,
) -> Result<SteamAccountContext, String> {
    let mut settings = state.settings.lock().unwrap();
    crate::modules::accounts::switch_account(&mut settings, &steam_id)?;
    session::build_account_context(&settings.steam_id)
}

#[tauri::command]
pub fn steam_detect_account(state: State<'_, AppState>) -> Result<Option<SteamAccount>, String> {
    let account = session::get_active_steam_account()?;
    if let Some(ref acc) = account {
        let mut settings = state.settings.lock().unwrap();
        if settings.steam_id.is_empty() {
            settings.steam_id = acc.steam_id.clone();
            settings::save_settings(&settings)?;
        }
    }
    Ok(account)
}

#[tauri::command]
pub async fn steam_get_games(state: State<'_, AppState>) -> Result<Vec<SteamGame>, String> {
    let settings = state.settings.lock().unwrap().clone();
    let mut games = library::fetch_owned_games(
        &settings.steam_api_key,
        &settings.steam_id,
        &settings.utility_path,
    )
    .await?;

    if let Ok(running) = list_running() {
        for game in &mut games {
            game.is_idling = running.iter().any(|p| p.app_id == game.app_id && p.source == "idle");
            game.is_farming = running
                .iter()
                .any(|p| p.app_id == game.app_id && p.source == "farm");
        }
    }

    Ok(games)
}

#[tauri::command]
pub fn steam_get_rate_limit_status() -> SteamRateLimitStatus {
    crate::modules::steam_helper::rate_limit::rate_limit_status()
}

#[tauri::command]
pub fn steam_reset_rate_limit() -> Result<(), String> {
    crate::modules::steam_helper::rate_limit::reset_cooldowns()
}

#[tauri::command]
pub async fn steam_enrich_trading_cards(
    max_count: Option<u32>,
    state: State<'_, AppState>,
) -> Result<CardEnrichResult, String> {
    let settings = state.settings.lock().unwrap().clone();
    let steam_id = session::resolve_steam_id(&settings.steam_id)?;
    let mut games = library::load_games_from_local_cache(&steam_id)?;
    let limit = max_count.unwrap_or(20).min(20) as usize;
    cards::enrich_trading_cards_limited(&mut games, limit).await
}

#[tauri::command]
pub fn steam_get_running_processes() -> Result<Vec<RunningIdleProcess>, String> {
    list_running()
}

#[tauri::command]
pub async fn steam_start_idle(
    app_id: u32,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.lock().unwrap().clone();
    if !paths::is_steam_running() {
        return Err("Steam client is not running.".into());
    }
    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    spawn_idle(&utility, app_id, &name, "idle")?;
    verify_spawn(&[app_id]).await?;

    let mut locked = state.settings.lock().unwrap();
    if !locked.idle_game_ids.contains(&app_id) {
        locked.idle_game_ids.push(app_id);
        settings::save_settings(&locked)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn steam_stop_idle(app_id: u32) -> Result<(), String> {
    stop_idle(app_id)
}

#[tauri::command]
pub async fn steam_start_farm(
    games: Vec<GameInput>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.lock().unwrap().clone();
    if !paths::is_steam_running() {
        return Err("Steam client is not running.".into());
    }
    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    let max = settings.max_idle_games.max(1).min(32) as usize;
    let selected: Vec<_> = games.into_iter().take(max).collect();
    if selected.is_empty() {
        return Err("Select at least one game to farm.".into());
    }

    let app_ids: Vec<u32> = selected.iter().map(|g| g.app_id).collect();
    for game in &selected {
        spawn_idle(&utility, game.app_id, &game.name, "farm")?;
    }
    verify_spawn(&app_ids).await?;

    let mut locked = state.settings.lock().unwrap();
    locked.farm_game_ids = app_ids;
    locked.card_farming_enabled = true;
    settings::save_settings(&locked)?;
    Ok(())
}

#[tauri::command]
pub async fn steam_stop_farm(state: State<'_, AppState>) -> Result<(), String> {
    stop_farm_idle()?;
    let mut locked = state.settings.lock().unwrap();
    locked.card_farming_enabled = false;
    settings::save_settings(&locked)?;
    Ok(())
}

#[tauri::command]
pub fn steam_get_achievements(
    app_id: u32,
    refetch: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<AchievementInfo>, String> {
    let settings = state.settings.lock().unwrap().clone();
    let steam_id = session::resolve_steam_id(&settings.steam_id)?;
    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    achievements::fetch_achievement_data(
        &utility,
        &steam_id,
        app_id,
        refetch.unwrap_or(false),
    )
}

#[tauri::command]
pub fn steam_unlock_achievement(
    app_id: u32,
    achievement_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    achievements::unlock_achievement(&utility, app_id, &achievement_id)
}

#[tauri::command]
pub fn steam_lock_achievement(
    app_id: u32,
    achievement_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    achievements::lock_achievement(&utility, app_id, &achievement_id)
}

#[tauri::command]
pub fn steam_toggle_achievement(
    app_id: u32,
    achievement_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    achievements::toggle_achievement(&utility, app_id, &achievement_id)
}

#[tauri::command]
pub fn steam_unlock_all_achievements(
    app_id: u32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    achievements::unlock_all(&utility, app_id)
}

#[tauri::command]
pub fn steam_lock_all_achievements(
    app_id: u32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    achievements::lock_all(&utility, app_id)
}

#[tauri::command]
pub async fn steam_get_inventory_games(
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<InventoryGameSummary>, String> {
    let settings = state.settings.lock().unwrap().clone();
    let steam_id = session::resolve_steam_id(&settings.steam_id)?;
    let cookie = credentials::build_cookie_header(&settings, &steam_id);

    let games = library::load_games_from_local_cache(&steam_id).unwrap_or_default();
    let pairs: Vec<(u32, String)> = games
        .iter()
        .map(|g| (g.app_id, g.name.clone()))
        .collect();

    if pairs.is_empty() {
        return Err("Load your library first (Games → Refresh with Steam running).".into());
    }

    inventory::fetch_inventory_games(
        &steam_id,
        &pairs,
        cookie.as_deref(),
        force.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub async fn steam_get_inventory(
    app_id: u32,
    context_id: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<InventoryItem>, String> {
    let settings = state.settings.lock().unwrap().clone();
    let steam_id = session::resolve_steam_id(&settings.steam_id)?;
    let cookie = credentials::build_cookie_header(&settings, &steam_id);
    inventory::fetch_inventory(
        &steam_id,
        app_id,
        context_id.unwrap_or(2),
        cookie.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn steam_redeem_key(key: String, state: State<'_, AppState>) -> Result<RedeemResult, String> {
    let settings = state.settings.lock().unwrap().clone();
    crate::modules::steam_helper::redeem::redeem_product_key(&settings, &key).await
}
