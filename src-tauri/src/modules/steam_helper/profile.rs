use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::paths;
use super::rate_limit::{self, SteamEndpoint};
use super::steeeam;

const STORE_USER_AGENT: &str = "Steeeam";

const CACHE_TTL_SECS: u64 = 24 * 60 * 60;
const CACHE_VERSION: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamProfileStats {
    pub persona_name: String,
    pub avatar_url: String,
    pub steam_id: String,
    pub profile_url: String,
    pub level: u32,
    pub xp_to_next_level: u32,
    pub total_games: u32,
    pub played_games: u32,
    pub unplayed_games: u32,
    pub total_playtime_minutes: u32,
    pub average_playtime_hours: f64,
    pub total_initial_formatted: String,
    pub total_current_formatted: String,
    pub average_price_formatted: String,
    pub price_per_hour_formatted: String,
    pub played_percent: u32,
    pub vac_bans: u32,
    pub game_bans: u32,
    pub currency: String,
    pub top_games: Vec<TopGameStat>,
    pub partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopGameStat {
    pub app_id: u32,
    pub name: String,
    pub img_url: String,
    pub playtime_minutes: u32,
    pub recent_playtime_minutes: u32,
    pub current_price_formatted: Option<String>,
}

#[derive(Debug, Clone)]
struct PriceCents {
    initial: u32,
    final_price: u32,
    formatted: String,
}

#[derive(Debug, Deserialize)]
struct PlayerSummariesResponse {
    response: PlayerSummariesInner,
}

#[derive(Debug, Deserialize)]
struct PlayerSummariesInner {
    players: Option<Vec<PlayerSummary>>,
}

#[derive(Debug, Deserialize)]
struct PlayerSummary {
    personaname: Option<String>,
    avatarfull: Option<String>,
    profileurl: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OwnedGamesResponse {
    response: OwnedGamesInner,
}

#[derive(Debug, Deserialize)]
struct OwnedGamesInner {
    games: Option<Vec<OwnedGameEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
struct OwnedGameEntry {
    appid: u32,
    name: Option<String>,
    playtime_forever: Option<u32>,
    playtime_2weeks: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct BadgesResponse {
    response: BadgesInner,
}

#[derive(Debug, Deserialize)]
struct BadgesInner {
    player_level: Option<u32>,
    player_xp_needed_to_level_up: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct BansResponse {
    players: Option<Vec<BanEntry>>,
}

#[derive(Debug, Deserialize)]
struct BanEntry {
    #[serde(rename = "NumberOfVACBans")]
    number_of_vac_bans: Option<u32>,
    #[serde(rename = "NumberOfGameBans")]
    number_of_game_bans: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct StoreAppResponse {
    success: bool,
    data: Option<StorePriceData>,
}

#[derive(Debug, Deserialize)]
struct StorePriceData {
    price_overview: Option<PriceOverview>,
}

#[derive(Debug, Deserialize)]
struct PriceOverview {
    initial: u32,
    #[serde(rename = "final")]
    final_price: u32,
    final_formatted: String,
}

fn country_config(code: &str) -> (&'static str, &'static str) {
    match code.to_lowercase().as_str() {
        "uk" => ("GBP", "uk"),
        "ca" => ("CAD", "ca"),
        "au" => ("AUD", "au"),
        "nz" => ("NZD", "nz"),
        "eu" => ("EUR", "eu"),
        _ => ("USD", "us"),
    }
}

fn format_money(cents: u64, currency: &str) -> String {
    let amount = cents as f64 / 100.0;
    match currency {
        "EUR" => format!("€{amount:.2}"),
        "GBP" => format!("£{amount:.2}"),
        "USD" => format!("${amount:.2}"),
        "CAD" => format!("CA${amount:.2}"),
        "AUD" => format!("A${amount:.2}"),
        "NZD" => format!("NZ${amount:.2}"),
        _ => format!("${amount:.2}"),
    }
}

fn store_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(STORE_USER_AGENT)
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())
}

fn cache_path() -> std::path::PathBuf {
    crate::modules::settings::data_dir().join("profile_stats_cache.json")
}

pub fn clear_profile_cache() {
    let _ = std::fs::remove_file(cache_path());
}

pub async fn fetch_profile_stats(
    api_key: &str,
    steam_id: &str,
    country_code: &str,
    force: bool,
) -> Result<SteamProfileStats, String> {
    if steam_id.trim().is_empty() {
        return Err("No Steam account detected.".into());
    }

    if force {
        clear_profile_cache();
    } else if let Some(cached) = read_cache(steam_id, country_code) {
        if cache_is_usable(&cached) {
            return Ok(cached);
        }
        clear_profile_cache();
    }

    match steeeam::fetch_profile_via_steeeam(steam_id, country_code).await {
        Ok(stats) => {
            write_cache(steam_id, country_code, &stats)?;
            return Ok(stats);
        }
        Err(steeeam_error) => {
            if api_key.trim().is_empty() {
                return Err(format!(
                    "Could not load profile stats: {steeeam_error}. Add a Steam Web API key in Settings for fallback."
                ));
            }
            eprintln!("[profile] Online enrichment failed, using local fallback: {steeeam_error}");
        }
    }

    let local = fetch_profile_local(api_key, steam_id, country_code).await?;
    if cache_is_usable(&local) {
        write_cache(steam_id, country_code, &local)?;
    }
    Ok(local)
}

async fn fetch_profile_local(
    api_key: &str,
    steam_id: &str,
    country_code: &str,
) -> Result<SteamProfileStats, String> {
    let (currency, store_cc) = country_config(country_code);
    let client = store_client()?;

    let summary_url = format!(
        "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={api_key}&steamids={steam_id}"
    );
    let summary: PlayerSummariesResponse = client
        .get(&summary_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let player = summary
        .response
        .players
        .and_then(|p| p.into_iter().next())
        .ok_or("Steam profile not found.")?;

    let owned_url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={api_key}&steamid={steam_id}&include_appinfo=1&include_played_free_games=1&include_free_sub=1&format=json"
    );
    let owned: OwnedGamesResponse = client
        .get(&owned_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let games = owned.response.games.unwrap_or_default();
    let total_games = games.len() as u32;
    let played_games = games
        .iter()
        .filter(|g| g.playtime_forever.unwrap_or(0) > 0)
        .count() as u32;
    let unplayed_games = total_games.saturating_sub(played_games);
    let total_playtime_minutes: u32 = games
        .iter()
        .map(|g| g.playtime_forever.unwrap_or(0))
        .sum();
    let average_playtime_hours = if played_games > 0 {
        (total_playtime_minutes as f64 / 60.0) / played_games as f64
    } else {
        0.0
    };
    let played_percent = if total_games > 0 {
        ((played_games as f64 / total_games as f64) * 100.0).round() as u32
    } else {
        0
    };

    let mut top_games: Vec<_> = games.clone();
    top_games.sort_by(|a, b| {
        b.playtime_forever
            .unwrap_or(0)
            .cmp(&a.playtime_forever.unwrap_or(0))
    });
    top_games.truncate(5);

    let app_ids: Vec<u32> = games.iter().map(|g| g.appid).collect();
    let (price_map, price_count) = fetch_all_prices(&client, &app_ids, store_cc).await;

    let mut total_initial: u64 = 0;
    let mut total_final: u64 = 0;

    for price in price_map.values() {
        total_initial += price.initial as u64;
        total_final += price.final_price as u64;
    }

    let average_price_cents = if price_count > 0 {
        total_initial / price_count
    } else {
        0
    };
    let price_per_hour = if total_playtime_minutes > 0 && total_final > 0 {
        (total_final as f64 / 100.0) / (total_playtime_minutes as f64 / 60.0)
    } else {
        0.0
    };

    let badges_url = format!(
        "https://api.steampowered.com/IPlayerService/GetBadges/v1/?key={api_key}&steamid={steam_id}"
    );
    let (level, xp_to_next_level) = match client.get(&badges_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(body) = resp.json::<BadgesResponse>().await {
                (
                    body.response.player_level.unwrap_or(0),
                    body.response.player_xp_needed_to_level_up.unwrap_or(0),
                )
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    };

    let bans_url = format!(
        "https://api.steampowered.com/ISteamUser/GetPlayerBans/v1/?key={api_key}&steamids={steam_id}"
    );
    let (vac_bans, game_bans) = match client.get(&bans_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(body) = resp.json::<BansResponse>().await {
                if let Some(entry) = body.players.and_then(|p| p.into_iter().next()) {
                    (
                        entry.number_of_vac_bans.unwrap_or(0),
                        entry.number_of_game_bans.unwrap_or(0),
                    )
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    };

    let stats = SteamProfileStats {
        persona_name: player.personaname.unwrap_or_else(|| "Steam User".into()),
        avatar_url: player.avatarfull.unwrap_or_default(),
        steam_id: steam_id.to_string(),
        profile_url: player
            .profileurl
            .unwrap_or_else(|| format!("https://steamcommunity.com/profiles/{steam_id}")),
        level,
        xp_to_next_level,
        total_games,
        played_games,
        unplayed_games,
        total_playtime_minutes,
        average_playtime_hours,
        total_initial_formatted: format_money(total_initial, currency),
        total_current_formatted: format_money(total_final, currency),
        average_price_formatted: format_money(average_price_cents, currency),
        price_per_hour_formatted: format_money((price_per_hour * 100.0).round() as u64, currency),
        played_percent,
        vac_bans,
        game_bans,
        currency: currency.to_string(),
        top_games: top_games
            .into_iter()
            .map(|g| {
                let price = price_map.get(&g.appid);
                TopGameStat {
                    app_id: g.appid,
                    name: g.name.unwrap_or_else(|| format!("App {}", g.appid)),
                    img_url: paths::steam_capsule_url(g.appid),
                    playtime_minutes: g.playtime_forever.unwrap_or(0),
                    recent_playtime_minutes: g.playtime_2weeks.unwrap_or(0),
                    current_price_formatted: price.map(|p| {
                        if p.final_price == 0 {
                            "Free".into()
                        } else {
                            p.formatted.clone()
                        }
                    }),
                }
            })
            .collect(),
        partial: price_count > 0 && price_count < (total_games as u64 / 4),
    };

    if cache_is_usable(&stats) {
        write_cache(steam_id, country_code, &stats)?;
    }

    Ok(stats)
}

fn cache_is_usable(stats: &SteamProfileStats) -> bool {
    if stats.total_games > 100 {
        let zero_prices = stats.total_current_formatted.contains("0.00")
            && stats.total_initial_formatted.contains("0.00");
        if zero_prices {
            return false;
        }
    }
    true
}

async fn fetch_all_prices(
    client: &reqwest::Client,
    app_ids: &[u32],
    store_cc: &str,
) -> (HashMap<u32, PriceCents>, u64) {
    let mut map = HashMap::new();

    if rate_limit::cooldown_message(SteamEndpoint::Store).is_some() {
        return (map, 0);
    }

    for chunk in app_ids.chunks(50) {
        if rate_limit::acquire(SteamEndpoint::Store).await.is_err() {
            break;
        }

        let ids = chunk
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let url = format!(
            "https://store.steampowered.com/api/appdetails?appids={ids}&filters=price_overview&cc={store_cc}"
        );

        if let Ok(resp) = client.get(&url).send().await {
            let status = resp.status().as_u16();
            if status == 403 || status == 429 || status == 503 {
                rate_limit::report_rate_limited(SteamEndpoint::Store, status);
                break;
            }
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<HashMap<String, StoreAppResponse>>().await {
                    merge_price_map(&mut map, body);
                }
            }
        }
    }

    let count = map.len() as u64;
    (map, count)
}

fn merge_price_map(map: &mut HashMap<u32, PriceCents>, body: HashMap<String, StoreAppResponse>) {
    for (id_str, entry) in body {
        if !entry.success {
            continue;
        }
        let Ok(app_id) = id_str.parse::<u32>() else {
            continue;
        };
        let Some(overview) = entry
            .data
            .as_ref()
            .and_then(|d| d.price_overview.as_ref())
        else {
            continue;
        };
        map.insert(
            app_id,
            PriceCents {
                initial: overview.initial,
                final_price: overview.final_price,
                formatted: overview.final_formatted.clone(),
            },
        );
    }
}

#[derive(Serialize, Deserialize)]
struct ProfileCacheFile {
    version: u32,
    steam_id: String,
    country_code: String,
    fetched_at: u64,
    stats: SteamProfileStats,
}

fn read_cache(steam_id: &str, country_code: &str) -> Option<SteamProfileStats> {
    let path = cache_path();
    let raw = std::fs::read_to_string(path).ok()?;
    let cache: ProfileCacheFile = serde_json::from_str(&raw).ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    if cache.version == CACHE_VERSION
        && cache.steam_id == steam_id
        && cache.country_code == country_code
        && now.saturating_sub(cache.fetched_at) < CACHE_TTL_SECS
        && cache_is_usable(&cache.stats)
    {
        Some(cache.stats)
    } else {
        None
    }
}

fn write_cache(steam_id: &str, country_code: &str, stats: &SteamProfileStats) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let cache = ProfileCacheFile {
        version: CACHE_VERSION,
        steam_id: steam_id.to_string(),
        country_code: country_code.to_string(),
        fetched_at: now,
        stats: stats.clone(),
    };
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string(&cache).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}
