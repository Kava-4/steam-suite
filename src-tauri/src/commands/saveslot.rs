use crate::modules::saveslot::{
    self, SaveSlotActionResult, SaveSlotGameState, SaveSlotGameSummary, SaveSlotStatus,
};
use crate::state::AppState;
use tauri::State;

fn cli_path(state: &State<'_, AppState>) -> String {
    state.settings.lock().unwrap().saveslot_cli_path.clone()
}

#[tauri::command]
pub fn saveslot_get_status(state: State<'_, AppState>) -> Result<SaveSlotStatus, String> {
    saveslot::get_status(&cli_path(&state))
}

#[tauri::command]
pub fn saveslot_list_games_with_saves(
    state: State<'_, AppState>,
) -> Result<Vec<SaveSlotGameSummary>, String> {
    saveslot::list_games_with_saves(&cli_path(&state))
}

#[tauri::command]
pub fn saveslot_get_game_state(
    app_id: u32,
    state: State<'_, AppState>,
) -> Result<SaveSlotGameState, String> {
    saveslot::get_game_state(&cli_path(&state), app_id)
}

#[tauri::command]
pub fn saveslot_create_profile(
    app_id: u32,
    name: String,
    state: State<'_, AppState>,
) -> Result<SaveSlotActionResult, String> {
    saveslot::create_profile(&cli_path(&state), app_id, &name)
}

#[tauri::command]
pub fn saveslot_backup(
    app_id: u32,
    profile_slug: String,
    state: State<'_, AppState>,
) -> Result<SaveSlotActionResult, String> {
    saveslot::backup(&cli_path(&state), app_id, &profile_slug)
}

#[tauri::command]
pub fn saveslot_restore(
    app_id: u32,
    profile_slug: String,
    snapshot_id: String,
    state: State<'_, AppState>,
) -> Result<SaveSlotActionResult, String> {
    saveslot::restore(&cli_path(&state), app_id, &profile_slug, &snapshot_id)
}

#[tauri::command]
pub fn saveslot_open_vault(state: State<'_, AppState>) -> Result<SaveSlotActionResult, String> {
    saveslot::open_vault(&cli_path(&state))
}
