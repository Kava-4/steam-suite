use super::rate_limit::{self, SteamEndpoint};
use super::SteamGame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const TRADING_CARD_CATEGORY_ID: u64 = 29;
const CACHE_MAX_AGE_SECS: u64 = 30 * 24 * 3600;
const BATCH_SIZE: usize = 20;
/// Hard cap per user-initiated scan — never hammer Steam on library load.
pub const MAX_ENRICH_PER_SCAN: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardEnrichResult {
    pub updated: u32,
    pub remaining_uncached: u32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct TradingCardsCache {
    entries: HashMap<String, CachedEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedEntry {
    has_cards: bool,
    updated_at: u64,
}

fn cache_path() -> PathBuf {
    crate::modules::settings::data_dir().join("trading_cards_cache.json")
}

fn load_cache() -> TradingCardsCache {
    let path = cache_path();
    if !path.exists() {
        return TradingCardsCache::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_cache(cache: &TradingCardsCache) -> Result<(), String> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(cache).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_stale(entry: &CachedEntry, now: u64) -> bool {
    now.saturating_sub(entry.updated_at) > CACHE_MAX_AGE_SECS
}

/// Disk cache only — safe to call on every library load.
pub fn apply_cached_trading_cards(games: &mut [SteamGame]) {
    let cache = load_cache();
    for game in games.iter_mut() {
        game.has_cards = cache
            .entries
            .get(&game.app_id.to_string())
            .map(|e| e.has_cards)
            .unwrap_or(false);
    }
}

pub fn count_uncached(games: &[SteamGame]) -> u32 {
    let cache = load_cache();
    let now = now_secs();
    games
        .iter()
        .filter(|g| {
            cache
                .entries
                .get(&g.app_id.to_string())
                .map(|e| is_stale(e, now))
                .unwrap_or(true)
        })
        .count() as u32
}

async fn batch_has_trading_cards(app_ids: &[u32]) -> Result<HashMap<u32, bool>, String> {
    if app_ids.is_empty() {
        return Ok(HashMap::new());
    }

    rate_limit::acquire(SteamEndpoint::Store).await?;

    let ids = app_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let url = format!(
        "https://store.steampowered.com/api/appdetails?appids={ids}&filters=categories"
    );

    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let status = response.status().as_u16();

    if status == 403 || status == 429 || status == 503 {
        rate_limit::report_rate_limited(SteamEndpoint::Store, status);
        return Err(rate_limit::cooldown_message(SteamEndpoint::Store).unwrap_or_else(|| {
            format!("Steam Store blocked requests (HTTP {status}). Try again later.")
        }));
    }

    if !response.status().is_success() {
        return Ok(HashMap::new());
    }

    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let mut result = HashMap::new();

    for app_id in app_ids {
        let key = app_id.to_string();
        let has_cards = body
            .get(&key)
            .and_then(|v| v.get("data"))
            .and_then(|data| data.get("categories"))
            .and_then(|c| c.as_array())
            .map(|categories| {
                categories.iter().any(|cat| {
                    cat.get("id")
                        .and_then(|id| id.as_u64())
                        .map(|id| id == TRADING_CARD_CATEGORY_ID)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        result.insert(*app_id, has_cards);
    }

    Ok(result)
}

/// User-initiated, rate-limited scan. Never called automatically on library load.
pub async fn enrich_trading_cards_limited(
    games: &mut [SteamGame],
    max_fetch: usize,
) -> Result<CardEnrichResult, String> {
    if games.is_empty() {
        return Ok(CardEnrichResult {
            updated: 0,
            remaining_uncached: 0,
            message: "No games in library.".into(),
        });
    }

    if let Some(msg) = rate_limit::cooldown_message(SteamEndpoint::Store) {
        apply_cached_trading_cards(games);
        return Ok(CardEnrichResult {
            updated: 0,
            remaining_uncached: count_uncached(games),
            message: msg,
        });
    }

    let max_fetch = max_fetch.min(MAX_ENRICH_PER_SCAN);
    let mut cache = load_cache();
    let now = now_secs();
    let mut dirty = false;

    let mut to_fetch: Vec<u32> = games
        .iter()
        .filter(|g| {
            cache
                .entries
                .get(&g.app_id.to_string())
                .map(|e| is_stale(e, now))
                .unwrap_or(true)
        })
        .map(|g| g.app_id)
        .collect();

    to_fetch.sort_unstable();
    to_fetch.dedup();
    to_fetch.truncate(max_fetch);

    let total_uncached_before = count_uncached(games);
    let mut updated = 0u32;

    for chunk in to_fetch.chunks(BATCH_SIZE) {
        match batch_has_trading_cards(chunk).await {
            Ok(batch) => {
                for (app_id, has_cards) in batch {
                    cache.entries.insert(
                        app_id.to_string(),
                        CachedEntry {
                            has_cards,
                            updated_at: now,
                        },
                    );
                    updated += 1;
                    dirty = true;
                }
            }
            Err(error) => {
                if dirty {
                    let _ = save_cache(&cache);
                }
                apply_cached_trading_cards(games);
                return Err(error);
            }
        }
    }

    if dirty {
        save_cache(&cache)?;
    }

    apply_cached_trading_cards(games);

    let remaining = count_uncached(games);
    let message = if updated == 0 {
        if total_uncached_before == 0 {
            "All games already cached.".into()
        } else {
            "No new card data fetched. Use Scan again later (rate-limited).".into()
        }
    } else if remaining > 0 {
        format!(
            "Scanned {updated} game(s). {remaining} still uncached — run Scan again when ready (max {MAX_ENRICH_PER_SCAN} per scan)."
        )
    } else {
        format!("Scanned {updated} game(s). Card cache is up to date.")
    };

    Ok(CardEnrichResult {
        updated,
        remaining_uncached: remaining,
        message,
    })
}
