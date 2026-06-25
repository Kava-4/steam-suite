use crate::modules::giveaways::bot::{GiveawayBotStatus, GiveawayLogEntry};
use crate::modules::giveaways::steamgifts::{PointsInfo, SteamgiftsService, WonGiveaway};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn giveaway_fetch_points(state: State<'_, AppState>) -> Result<PointsInfo, String> {
    let settings = state.settings.lock().unwrap().clone();
    if settings.steamgifts_cookie.is_empty() {
        return Err("Set your SteamGifts PHPSESSID in Settings.".into());
    }
    SteamgiftsService::new(&settings.steamgifts_cookie)
        .fetch_points()
        .await
}

#[tauri::command]
pub async fn giveaway_fetch_won(state: State<'_, AppState>) -> Result<Vec<WonGiveaway>, String> {
    let settings = state.settings.lock().unwrap().clone();
    SteamgiftsService::new(&settings.steamgifts_cookie)
        .fetch_won()
        .await
}

#[tauri::command]
pub async fn giveaway_start_bot(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.lock().unwrap().clone();
    state
        .giveaway_bot
        .start(
            settings.steamgifts_cookie,
            settings.refresh_delay_minutes,
            settings.max_pages,
            settings.max_giveaway_end_hours,
            2,
            Some(app),
            settings.steam_id,
            settings.notify_on_win,
            settings.auto_redeem_on_win,
        )
        .await
}

#[tauri::command]
pub async fn giveaway_stop_bot(state: State<'_, AppState>) -> Result<(), String> {
    state.giveaway_bot.stop().await;
    Ok(())
}

#[tauri::command]
pub async fn giveaway_get_status(state: State<'_, AppState>) -> Result<GiveawayBotStatus, String> {
    Ok(state.giveaway_bot.status().await)
}

#[tauri::command]
pub async fn giveaway_get_logs(state: State<'_, AppState>) -> Result<Vec<GiveawayLogEntry>, String> {
    Ok(state.giveaway_bot.logs().await)
}
