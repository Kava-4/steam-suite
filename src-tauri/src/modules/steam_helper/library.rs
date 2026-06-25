use super::{cards, paths, rate_limit, utility, SteamGame};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct OwnedGamesResponse {
    response: OwnedGamesInner,
}

#[derive(Debug, Deserialize)]
struct OwnedGamesInner {
    games: Option<Vec<OwnedGame>>,
}

#[derive(Debug, Deserialize)]
struct OwnedGame {
    appid: u32,
    name: Option<String>,
    playtime_forever: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct UtilityOwnershipFile {
    games: Vec<UtilityOwnedGame>,
}

#[derive(Debug, Deserialize)]
struct UtilityOwnedGame {
    appid: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct UtilityOwnershipResult {
    success: bool,
    error: Option<String>,
    suggestion: Option<String>,
}

pub async fn fetch_owned_games(
    api_key: &str,
    steam_id_setting: &str,
    utility_path_setting: &str,
) -> Result<Vec<SteamGame>, String> {
    let steam_id = super::session::resolve_steam_id(steam_id_setting)?;
    let utility = paths::resolve_utility_path(utility_path_setting).ok();

    let mut games = if paths::is_steam_running() {
        utility
            .as_ref()
            .and_then(|path| fetch_owned_games_via_utility(path, &steam_id).ok())
    } else {
        None
    };

    if games.is_none() && !api_key.is_empty() {
        games = Some(fetch_owned_games_via_api(api_key, &steam_id).await?);
    }

    let mut games = games.ok_or_else(|| {
        if !paths::is_steam_running() {
            "Steam is not running. Launch Steam and sign in.".to_string()
        } else if utility.is_none() {
            "SteamSuiteUtility not found. Place it in libs/ or set a custom path.".to_string()
        } else {
            "Could not load your game library from Steam.".to_string()
        }
    })?;

    if !api_key.is_empty() {
        if let Ok(api_games) = fetch_owned_games_via_api(api_key, &steam_id).await {
            merge_playtime(&mut games, &api_games);
        }
    }

    // Cache only — never hit Steam Store on library load (prevents IP blocks).
    cards::apply_cached_trading_cards(&mut games);

    Ok(games)
}

pub fn fetch_owned_games_via_utility(utility_path: &Path, steam_id: &str) -> Result<Vec<SteamGame>, String> {
    let output_file = crate::modules::accounts::owned_games_path(steam_id);
    let output_str = output_file
        .to_str()
        .ok_or("Invalid cache path for owned games")?;

    let output = utility::run_utility(utility_path, &["check_ownership", output_str])?;
    let stdout = utility::utility_stdout(&output);

    if let Ok(result) = serde_json::from_str::<UtilityOwnershipResult>(&stdout) {
        if !result.success {
            let mut msg = result.error.unwrap_or_else(|| "check_ownership failed".into());
            if let Some(hint) = result.suggestion {
                msg = format!("{msg} ({hint})");
            }
            return Err(msg);
        }
    } else if !output.status.success() {
        let stderr = utility::utility_stderr(&output);
        return Err(if stderr.is_empty() {
            format!("check_ownership exited with {}", output.status)
        } else {
            stderr
        });
    }

    let raw = std::fs::read_to_string(&output_file).map_err(|e| e.to_string())?;
    let body: UtilityOwnershipFile = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    Ok(body
        .games
        .into_iter()
        .map(|g| SteamGame {
            app_id: g.appid,
            name: g.name,
            playtime_forever: 0,
            img_url: paths::steam_capsule_url(g.appid),
            has_cards: false,
            is_farming: false,
            is_idling: false,
        })
        .collect())
}

/// Local ownership cache only — no Steam network calls.
pub fn load_games_from_local_cache(steam_id: &str) -> Result<Vec<SteamGame>, String> {
    let path = crate::modules::accounts::owned_games_path(steam_id);
    if !path.exists() {
        return Err("Load your library first (Games → Refresh with Steam running).".into());
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let body: UtilityOwnershipFile = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let mut games: Vec<SteamGame> = body
        .games
        .into_iter()
        .map(|g| SteamGame {
            app_id: g.appid,
            name: g.name,
            playtime_forever: 0,
            img_url: paths::steam_capsule_url(g.appid),
            has_cards: false,
            is_farming: false,
            is_idling: false,
        })
        .collect();
    cards::apply_cached_trading_cards(&mut games);
    Ok(games)
}

async fn fetch_owned_games_via_api(api_key: &str, steam_id: &str) -> Result<Vec<SteamGame>, String> {
    rate_limit::acquire(rate_limit::SteamEndpoint::WebApi).await?;

    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={api_key}&steamid={steam_id}&include_appinfo=1&include_played_free_games=1&format=json"
    );

    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let status = response.status().as_u16();
    if status == 403 || status == 429 || status == 503 {
        rate_limit::report_rate_limited(rate_limit::SteamEndpoint::WebApi, status);
        return Err(format!("Steam Web API HTTP {status} — requests paused to protect your IP."));
    }
    if !response.status().is_success() {
        return Err(format!("Steam API HTTP {}", response.status()));
    }

    let body: OwnedGamesResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(body
        .response
        .games
        .unwrap_or_default()
        .into_iter()
        .map(|g| SteamGame {
            app_id: g.appid,
            name: g.name.unwrap_or_else(|| format!("App {}", g.appid)),
            playtime_forever: g.playtime_forever.unwrap_or(0),
            img_url: paths::steam_capsule_url(g.appid),
            has_cards: false,
            is_farming: false,
            is_idling: false,
        })
        .collect())
}

fn merge_playtime(games: &mut [SteamGame], api_games: &[SteamGame]) {
    for game in games.iter_mut() {
        if let Some(api) = api_games.iter().find(|g| g.app_id == game.app_id) {
            game.playtime_forever = api.playtime_forever;
        }
    }
}
