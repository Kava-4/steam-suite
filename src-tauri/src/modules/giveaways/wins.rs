use super::steamgifts::{SteamgiftsService, WonGiveaway};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Default, Serialize, Deserialize)]
struct KnownWinsCache {
    codes: HashSet<String>,
}

fn cache_path(steam_id: &str) -> PathBuf {
    crate::modules::accounts::known_wins_path(steam_id)
}

fn load_known(steam_id: &str) -> KnownWinsCache {
    let path = cache_path(steam_id);
    if !path.exists() {
        return KnownWinsCache::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_known(steam_id: &str, cache: &KnownWinsCache) -> Result<(), String> {
    let path = cache_path(steam_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(cache).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

fn notify_win(app: &AppHandle, win: &WonGiveaway) {
    let _ = app
        .notification()
        .builder()
        .title("Steam Suite — Giveaway Won!")
        .body(format!("You won: {}", win.name))
        .show();
}

pub async fn process_wins(
    app: Option<&AppHandle>,
    cookie: &str,
    steam_id: &str,
    notify_on_win: bool,
    auto_redeem_on_win: bool,
) -> Result<Vec<WonGiveaway>, String> {
    if cookie.trim().is_empty() {
        return Ok(Vec::new());
    }

    let service = SteamgiftsService::new(cookie);
    let wins = service.fetch_won().await?;
    let mut known = load_known(steam_id);
    let mut new_wins = Vec::new();

    for win in &wins {
        if known.codes.insert(win.code.clone()) {
            new_wins.push(win.clone());
        }
    }

    if !new_wins.is_empty() {
        save_known(steam_id, &known)?;

        for win in &new_wins {
            if notify_on_win {
                if let Some(app) = app {
                    notify_win(app, win);
                }
            }

            if auto_redeem_on_win {
                if let Err(error) = service.claim_won(&win.code).await {
                    eprintln!("[SteamGifts] Auto-claim {}: {error}", win.name);
                }
            }
        }
    }

    Ok(new_wins)
}
