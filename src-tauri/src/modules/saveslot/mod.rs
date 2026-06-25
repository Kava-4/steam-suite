pub mod cli;
pub mod paths;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlotStatus {
    pub vault_root: String,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlotSnapshot {
    pub id: String,
    pub created_at: String,
    pub note: Option<String>,
    pub file_count: i32,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlotProfile {
    pub name: String,
    pub slug: String,
    pub snapshots: Vec<SaveSlotSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlotGameState {
    pub app_id: u32,
    pub name: String,
    pub steam_id64: String,
    pub vault_root: String,
    pub save_location_count: i32,
    pub profiles: Vec<SaveSlotProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlotGameSummary {
    pub app_id: u32,
    pub name: String,
    pub save_location_count: i32,
    pub in_vault: bool,
    pub has_live_saves: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlotActionResult {
    pub message: String,
}

pub fn get_status(custom_path: &str) -> Result<SaveSlotStatus, String> {
    cli::invoke_cli(custom_path, &["status"])
}

pub fn list_games_with_saves(custom_path: &str) -> Result<Vec<SaveSlotGameSummary>, String> {
    cli::invoke_cli(custom_path, &["games-with-saves"])
}

pub fn get_game_state(custom_path: &str, app_id: u32) -> Result<SaveSlotGameState, String> {
    cli::invoke_cli(custom_path, &["game-state", &app_id.to_string()])
}

pub fn create_profile(custom_path: &str, app_id: u32, name: &str) -> Result<SaveSlotActionResult, String> {
    cli::invoke_cli(
        custom_path,
        &["create-profile", &app_id.to_string(), name],
    )
}

pub fn backup(custom_path: &str, app_id: u32, profile_slug: &str) -> Result<SaveSlotActionResult, String> {
    cli::invoke_cli(
        custom_path,
        &["backup", &app_id.to_string(), profile_slug],
    )
}

pub fn restore(
    custom_path: &str,
    app_id: u32,
    profile_slug: &str,
    snapshot_id: &str,
) -> Result<SaveSlotActionResult, String> {
    cli::invoke_cli(
        custom_path,
        &[
            "restore",
            &app_id.to_string(),
            profile_slug,
            snapshot_id,
        ],
    )
}

pub fn open_vault(custom_path: &str) -> Result<SaveSlotActionResult, String> {
    cli::invoke_cli(custom_path, &["open-vault"])
}
