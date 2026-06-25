use serde::Deserialize;
use serde_json::{json, Value};

use super::paths;
use super::profile::SteamProfileStats;
use super::profile::TopGameStat;

const STEEEAM_GAMEDATA: &str = "https://steeeam.vercel.app/api/gamedata";
const STEEEAM_STEAMINFO: &str = "https://steeeam.vercel.app/api/steaminfo";

#[derive(Debug, Deserialize)]
struct GamedataResponse {
    success: bool,
    error: Option<String>,
    #[serde(rename = "userGameData")]
    user_game_data: Option<SteeamGameData>,
}

#[derive(Debug, Deserialize)]
struct SteaminfoResponse {
    success: bool,
    error: Option<String>,
    #[serde(rename = "userSummary")]
    user_summary: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct SteeamGameData {
    #[serde(rename = "topFiveGames")]
    top_five_games: Option<Vec<SteeamTopGame>>,
    #[serde(rename = "topFiveGameDetails")]
    top_five_game_details: Option<Vec<SteeamGameDetails>>,
    totals: Option<SteeamTotals>,
    #[serde(rename = "playCount")]
    play_count: Option<SteeamPlayCount>,
    #[serde(rename = "userXP")]
    user_xp: Option<SteeamUserXp>,
}

#[derive(Debug, Deserialize)]
struct SteeamTopGame {
    game: SteeamGameRef,
    minutes: u32,
    #[serde(rename = "recentMinutes")]
    recent_minutes: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SteeamGameRef {
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SteeamGameDetails {
    id: Option<u32>,
    name: Option<String>,
    #[serde(rename = "isFree")]
    is_free: Option<bool>,
    #[serde(rename = "priceOverview")]
    price_overview: Option<SteeamPriceOverview>,
}

#[derive(Debug, Deserialize)]
struct SteeamPriceOverview {
    #[serde(rename = "final_formatted")]
    final_formatted: Option<String>,
    #[serde(rename = "final")]
    final_price: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SteeamTotals {
    #[serde(rename = "totalInitialFormatted")]
    total_initial_formatted: Option<String>,
    #[serde(rename = "totalFinalFormatted")]
    total_final_formatted: Option<String>,
    #[serde(rename = "averageGamePrice")]
    average_game_price: Option<String>,
    #[serde(rename = "averagePlaytime")]
    average_playtime: Option<String>,
    #[serde(rename = "totalGames")]
    total_games: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SteeamPlayCount {
    #[serde(rename = "playedCount")]
    played_count: Option<u32>,
    #[serde(rename = "unplayedCount")]
    unplayed_count: Option<u32>,
    #[serde(rename = "totalPlaytime")]
    total_playtime: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SteeamUserXp {
    #[serde(rename = "xpRemaining")]
    xp_remaining: Option<u32>,
    level: Option<u32>,
}

fn currency_code(country_code: &str) -> &'static str {
    match country_code.to_lowercase().as_str() {
        "uk" => "GBP",
        "ca" => "CAD",
        "au" => "AUD",
        "nz" => "NZD",
        "eu" => "EUR",
        _ => "USD",
    }
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    let field = value.get(key)?;
    match field {
        Value::String(s) => Some(s.trim().to_string()),
        Value::Array(items) => items
            .first()
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn json_u32(value: &Value, key: &str) -> Option<u32> {
    let field = value.get(key)?;
    match field {
        Value::Number(n) => n.as_u64().map(|v| v as u32),
        Value::String(s) => s.parse().ok(),
        Value::Array(items) => items
            .first()
            .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .map(|v| v as u32),
        _ => None,
    }
}

fn parse_steaminfo_summary(summary: &Value, fallback_steam_id: &str) -> (String, String, String, u32, u32) {
    let persona = json_string(summary, "nickname")
        .or_else(|| json_string(summary, "personaName"))
        .unwrap_or_else(|| "Steam User".into());

    let avatar = summary
        .get("avatar")
        .and_then(|a| a.get("large"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| json_string(summary, "avatarFull"))
        .unwrap_or_default();

    let profile_url = json_string(summary, "url")
        .unwrap_or_else(|| format!("https://steamcommunity.com/profiles/{fallback_steam_id}"));

    let bans = summary.get("bans");
    let vac_bans = bans
        .and_then(|b| json_u32(b, "vacBans"))
        .or_else(|| json_u32(summary, "vacBans"))
        .unwrap_or(0);
    let game_bans = bans
        .and_then(|b| json_u32(b, "gameBans"))
        .or_else(|| json_u32(summary, "gameBans"))
        .unwrap_or(0);

    (persona, avatar, profile_url, vac_bans, game_bans)
}

pub async fn fetch_profile_via_steeeam(
    steam_id: &str,
    country_code: &str,
) -> Result<SteamProfileStats, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|e| e.to_string())?;

    let id = steam_id.trim();
    let currency = country_code.to_lowercase();

    let gamedata_fut = client
        .post(STEEEAM_GAMEDATA)
        .json(&json!({ "id": id, "currency": currency }))
        .send();

    let steaminfo_fut = client
        .post(STEEEAM_STEAMINFO)
        .json(&json!({ "id": id }))
        .send();

    let (gamedata_resp, steaminfo_resp) =
        tokio::try_join!(gamedata_fut, steaminfo_fut).map_err(|e| e.to_string())?;

    let gamedata: GamedataResponse = gamedata_resp
        .json()
        .await
        .map_err(|e| format!("Steeam gamedata parse error: {e}"))?;

    if !gamedata.success {
        return Err(gamedata
            .error
            .unwrap_or_else(|| "Steeam gamedata failed".into()));
    }

    let game_data = gamedata
        .user_game_data
        .ok_or("Steeam returned no game data")?;

    let steaminfo: SteaminfoResponse = steaminfo_resp
        .json()
        .await
        .map_err(|e| format!("Steeam steaminfo parse error: {e}"))?;

    if !steaminfo.success {
        return Err(steaminfo
            .error
            .unwrap_or_else(|| "Steeam steaminfo failed".into()));
    }

    let summary = steaminfo
        .user_summary
        .ok_or("Steeam returned no user summary")?;

    let (persona_name, avatar_url, profile_url, vac_bans, game_bans) =
        parse_steaminfo_summary(&summary, id);

    let totals = game_data.totals.unwrap_or_default();
    let play = game_data.play_count.unwrap_or_default();
    let xp = game_data.user_xp.unwrap_or_default();

    let total_games = totals.total_games.unwrap_or(0);
    let played_games = play.played_count.unwrap_or(0);
    let total_playtime_minutes = play.total_playtime.unwrap_or(0);
    let total_final_formatted = totals
        .total_final_formatted
        .clone()
        .unwrap_or_else(|| "—".into());
    let total_initial_formatted = totals
        .total_initial_formatted
        .clone()
        .unwrap_or_else(|| "—".into());
    let average_price_formatted = totals
        .average_game_price
        .clone()
        .unwrap_or_else(|| "—".into());

    let price_per_hour = compute_price_per_hour(
        &total_final_formatted,
        total_playtime_minutes,
        currency_code(country_code),
    );

    let details = game_data.top_five_game_details.unwrap_or_default();
    let top_games = game_data
        .top_five_games
        .unwrap_or_default()
        .into_iter()
        .map(|entry| {
            let app_id = entry.game.id;
            let detail = find_game_detail(&details, app_id, &entry.game.name);
            TopGameStat {
                app_id,
                name: entry.game.name,
                img_url: paths::steam_capsule_url(app_id),
                playtime_minutes: entry.minutes,
                recent_playtime_minutes: entry.recent_minutes.unwrap_or(0),
                current_price_formatted: Some(format_top_game_price(detail)),
            }
        })
        .collect();

    let played_percent = if total_games > 0 {
        ((played_games as f64 / total_games as f64) * 100.0).round() as u32
    } else {
        0
    };

    let average_playtime_hours = totals
        .average_playtime
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(SteamProfileStats {
        persona_name,
        avatar_url,
        steam_id: id.to_string(),
        profile_url,
        level: xp.level.unwrap_or(0),
        xp_to_next_level: xp.xp_remaining.unwrap_or(0),
        total_games,
        played_games,
        unplayed_games: play.unplayed_count.unwrap_or(0),
        total_playtime_minutes,
        average_playtime_hours,
        total_initial_formatted,
        total_current_formatted: total_final_formatted,
        average_price_formatted,
        price_per_hour_formatted: price_per_hour,
        played_percent,
        vac_bans,
        game_bans,
        currency: currency_code(country_code).to_string(),
        top_games,
        partial: false,
    })
}

fn find_game_detail<'a>(
    details: &'a [SteeamGameDetails],
    app_id: u32,
    name: &str,
) -> Option<&'a SteeamGameDetails> {
    details
        .iter()
        .find(|d| d.id == Some(app_id))
        .or_else(|| {
            details
                .iter()
                .find(|d| d.name.as_deref() == Some(name))
        })
}

/// Same logic as steeeam TopFiveGames/GameDetails.tsx
fn format_top_game_price(detail: Option<&SteeamGameDetails>) -> String {
    let Some(detail) = detail else {
        return "Free".into();
    };

    if let Some(overview) = detail.price_overview.as_ref() {
        if let Some(formatted) = overview
            .final_formatted
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            return formatted.clone();
        }

        let cents = overview.final_price.unwrap_or(0);
        if cents > 0 {
            return format!("${:.2}", cents as f64 / 100.0);
        }
    }

    if detail.is_free.unwrap_or(false) {
        return "Free".into();
    }

    "Free".into()
}

impl Default for SteeamTotals {
    fn default() -> Self {
        Self {
            total_initial_formatted: None,
            total_final_formatted: None,
            average_game_price: None,
            average_playtime: None,
            total_games: None,
        }
    }
}

impl Default for SteeamPlayCount {
    fn default() -> Self {
        Self {
            played_count: None,
            unplayed_count: None,
            total_playtime: None,
        }
    }
}

impl Default for SteeamUserXp {
    fn default() -> Self {
        Self {
            xp_remaining: None,
            level: None,
        }
    }
}

fn compute_price_per_hour(
    total_final_formatted: &str,
    total_playtime_minutes: u32,
    currency: &str,
) -> String {
    if total_playtime_minutes == 0 {
        return format_money(0, currency);
    }

    let amount: f64 = total_final_formatted
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect::<String>()
        .replace(',', ".")
        .parse()
        .unwrap_or(0.0);

    if amount <= 0.0 {
        return total_final_formatted.to_string();
    }

    let per_hour = amount / (total_playtime_minutes as f64 / 60.0);
    format_money((per_hour * 100.0).round() as u64, currency)
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
