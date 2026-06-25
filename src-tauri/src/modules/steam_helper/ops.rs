use super::idling::{list_running, spawn_idle, stop_all_by_source, stop_farm_idle, verify_spawn};
use super::paths;
use crate::modules::settings::{self, AppSettings};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UtilityOwnershipFile {
    games: Vec<UtilityOwnedGame>,
}

#[derive(Debug, Deserialize)]
struct UtilityOwnedGame {
    appid: u32,
    name: String,
}

pub fn game_names_for_ids(app_ids: &[u32], steam_id: &str) -> Vec<(u32, String)> {
    let cache_path = crate::modules::accounts::owned_games_path(steam_id);
    if let Ok(raw) = std::fs::read_to_string(&cache_path) {
        if let Ok(body) = serde_json::from_str::<UtilityOwnershipFile>(&raw) {
            return app_ids
                .iter()
                .map(|id| {
                    let name = body
                        .games
                        .iter()
                        .find(|g| g.appid == *id)
                        .map(|g| g.name.clone())
                        .unwrap_or_else(|| format!("App {id}"));
                    (*id, name)
                })
                .collect();
        }
    }

    app_ids
        .iter()
        .map(|id| (*id, format!("App {id}")))
        .collect()
}

pub async fn start_farm_for_ids(settings: &AppSettings, app_ids: &[u32]) -> Result<(), String> {
    if app_ids.is_empty() {
        return Err("No games selected for card farming.".into());
    }
    if !paths::is_steam_running() {
        return Err("Steam client is not running.".into());
    }

    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    let max = settings.max_idle_games.max(1).min(32) as usize;
    let games: Vec<_> = game_names_for_ids(app_ids, &settings.steam_id)
        .into_iter()
        .take(max)
        .collect();
    let ids: Vec<u32> = games.iter().map(|(id, _)| *id).collect();

    for (app_id, name) in &games {
        spawn_idle(&utility, *app_id, name, "farm")?;
    }
    verify_spawn(&ids).await?;

    let mut locked = settings::load_settings();
    locked.farm_game_ids = ids;
    locked.card_farming_enabled = true;
    settings::save_settings(&locked)?;
    Ok(())
}

pub async fn resume_farm(settings: &AppSettings) -> Result<(), String> {
    if settings.farm_game_ids.is_empty() {
        return Ok(());
    }
    start_farm_for_ids(settings, &settings.farm_game_ids).await
}

pub async fn start_idle_for_ids(settings: &AppSettings, app_ids: &[u32]) -> Result<(), String> {
    if app_ids.is_empty() {
        return Ok(());
    }
    if !paths::is_steam_running() {
        return Err("Steam client is not running.".into());
    }

    let utility = paths::resolve_utility_path(&settings.utility_path)?;
    let games = game_names_for_ids(app_ids, &settings.steam_id);

    for (app_id, name) in &games {
        spawn_idle(&utility, *app_id, name, "idle")?;
    }

    let ids: Vec<u32> = games.iter().map(|(id, _)| *id).collect();
    verify_spawn(&ids).await?;
    Ok(())
}

pub async fn resume_idle(settings: &AppSettings) -> Result<(), String> {
    start_idle_for_ids(settings, &settings.idle_game_ids).await
}

pub fn stop_farm(settings: &AppSettings) -> Result<(), String> {
    stop_farm_idle()?;
    let mut locked = settings::load_settings();
    locked.card_farming_enabled = false;
    settings::save_settings(&locked)?;
    let _ = settings;
    Ok(())
}

pub fn stop_all_idle() -> Result<(), String> {
    stop_all_by_source("idle")
}

pub fn farm_process_count() -> usize {
    list_running()
        .map(|p| p.iter().filter(|proc| proc.source == "farm").count())
        .unwrap_or(0)
}
