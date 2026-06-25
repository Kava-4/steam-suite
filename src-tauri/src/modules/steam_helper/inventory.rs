use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::modules::settings::data_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub marketable: bool,
    pub tradable: bool,
    pub icon_url: String,
    pub market_hash_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryGameSummary {
    pub app_id: u32,
    #[serde(default = "default_inventory_context")]
    pub context_id: u32,
    pub name: String,
    pub item_count: u32,
}

fn default_inventory_context() -> u32 {
    2
}

#[derive(Debug, Serialize, Deserialize)]
struct InventoryIndexCache {
    steam_id: String,
    fetched_at: u64,
    games: Vec<InventoryGameSummary>,
}

const INDEX_TTL_SECS: u64 = 6 * 3600;
const PAGE_SIZE: u32 = 500;

fn index_cache_path(steam_id: &str) -> PathBuf {
    data_dir().join(format!("inventory_index_{steam_id}.json"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn inventory_icon_url(icon_path: &str) -> String {
    let path = icon_path.trim();
    if path.is_empty() {
        return String::new();
    }
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    format!(
        "https://community.cloudflare.steamstatic.com/economy/image/{path}/96fx96f"
    )
}

pub async fn fetch_inventory(
    steam_id: &str,
    app_id: u32,
    context_id: u32,
    cookie_header: Option<&str>,
) -> Result<Vec<InventoryItem>, String> {
    if steam_id.is_empty() {
        return Err("No Steam account detected.".into());
    }

    let client = reqwest::Client::new();
    let mut all_items = Vec::new();
    let mut start_assetid: Option<String> = None;

    loop {
        let mut url = format!(
            "https://steamcommunity.com/inventory/{steam_id}/{app_id}/{context_id}?l=english&count={PAGE_SIZE}"
        );
        if let Some(ref cursor) = start_assetid {
            url.push_str(&format!("&start_assetid={cursor}"));
        }

        let mut request = client.get(&url);
        if let Some(cookie) = cookie_header {
            request = request.header("Cookie", cookie);
        }

        let response = request.send().await.map_err(|e| e.to_string())?;
        let status = response.status();
        if status.as_u16() == 403 {
            return Err(
                "Inventory blocked (HTTP 403). Set credentials in Settings or make inventory public."
                    .into(),
            );
        }
        if !status.is_success() {
            return Err(format!("Inventory HTTP {status}"));
        }

        let body: InventoryResponse = response.json().await.map_err(|e| e.to_string())?;
        if body.success != Some(1) {
            return Err(body
                .error
                .unwrap_or_else(|| "Inventory is private or unavailable.".into()));
        }

        let descriptions = body.descriptions.unwrap_or_default();
        let page_items: Vec<InventoryItem> = body
            .assets
            .unwrap_or_default()
            .into_iter()
            .filter_map(|asset| map_asset(asset, &descriptions))
            .collect();
        all_items.extend(page_items);

        if body.more_items == Some(1) {
            start_assetid = body.last_assetid;
            if start_assetid.is_none() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        } else {
            break;
        }
    }

    Ok(all_items)
}

pub async fn discover_inventory_games(
    steam_id: &str,
    cookie_header: Option<&str>,
) -> Result<Vec<InventoryGameSummary>, String> {
    let url = format!(
        "https://steamcommunity.com/profiles/{steam_id}/inventory/?l=english"
    );
    let client = reqwest::Client::new();
    let mut request = client.get(&url);
    if let Some(cookie) = cookie_header {
        request = request.header("Cookie", cookie);
    }

    let response = request.send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    if status.as_u16() == 403 {
        return Err(
            "Inventory blocked (HTTP 403). Set credentials in Settings or make inventory public."
                .into(),
        );
    }
    if !status.is_success() {
        return Err(format!("Inventory page HTTP {status}"));
    }

    let html = response.text().await.map_err(|e| e.to_string())?;
    parse_inventory_contexts(&html)
}

fn parse_inventory_contexts(html: &str) -> Result<Vec<InventoryGameSummary>, String> {
    use std::collections::HashMap;

    let json = extract_app_context_json(html)
        .ok_or("Could not read inventory games. Make inventory public or add credentials.")?;

    #[derive(Debug, Deserialize)]
    struct InventoryContextInfo {
        asset_count: u32,
        id: String,
    }

    #[derive(Debug, Deserialize)]
    struct AppContextEntry {
        appid: u32,
        name: String,
        #[serde(rename = "rgContexts")]
        contexts: HashMap<String, InventoryContextInfo>,
    }

    let raw: HashMap<String, AppContextEntry> =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let mut summaries = Vec::new();
    for app in raw.into_values() {
        for ctx in app.contexts.into_values() {
            if ctx.asset_count == 0 {
                continue;
            }
            let context_id = ctx.id.parse::<u32>().unwrap_or(2);
            summaries.push(InventoryGameSummary {
                app_id: app.appid,
                context_id,
                name: app.name.clone(),
                item_count: ctx.asset_count,
            });
        }
    }

    if summaries.is_empty() {
        return Err("No inventory found on this account.".into());
    }

    summaries.sort_by(|a, b| b.item_count.cmp(&a.item_count));
    Ok(summaries)
}

fn extract_app_context_json(html: &str) -> Option<String> {
    const PREFIX: &str = "var g_rgAppContextData = ";
    let start = html.find(PREFIX)? + PREFIX.len();
    let rest = &html[start..];
    let brace_start = rest.find('{')?;
    let slice = &rest[brace_start..];
    let mut depth = 0usize;
    for (i, ch) in slice.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(slice[..=i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

pub async fn fetch_inventory_count(
    steam_id: &str,
    app_id: u32,
    context_id: u32,
    cookie_header: Option<&str>,
) -> Result<u32, String> {
    let url = format!(
        "https://steamcommunity.com/inventory/{steam_id}/{app_id}/{context_id}?l=english&count=1"
    );
    let client = reqwest::Client::new();
    let mut request = client.get(&url);
    if let Some(cookie) = cookie_header {
        request = request.header("Cookie", cookie);
    }

    let response = request.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Ok(0);
    }

    let body: InventoryResponse = response.json().await.map_err(|e| e.to_string())?;
    if body.success != Some(1) {
        return Ok(0);
    }
    Ok(body.total_inventory_count.unwrap_or(0))
}

pub async fn fetch_inventory_games(
    steam_id: &str,
    games: &[(u32, String)],
    cookie_header: Option<&str>,
    force: bool,
) -> Result<Vec<InventoryGameSummary>, String> {
    if !force {
        if let Some(cached) = read_index_cache(steam_id) {
            return Ok(cached);
        }
    }

    if let Ok(discovered) = discover_inventory_games(steam_id, cookie_header).await {
        write_index_cache(steam_id, &discovered)?;
        return Ok(discovered);
    }

    let mut summaries = Vec::new();

    if let Ok(count) = fetch_inventory_count(steam_id, 753, 6, cookie_header).await {
        if count > 0 {
            summaries.push(InventoryGameSummary {
                app_id: 753,
                context_id: 6,
                name: "Steam".into(),
                item_count: count,
            });
        }
    }

    for (app_id, name) in games {
        if *app_id == 753 {
            continue;
        }
        if let Ok(count) = fetch_inventory_count(steam_id, *app_id, 2, cookie_header).await {
            if count > 0 {
                summaries.push(InventoryGameSummary {
                    app_id: *app_id,
                    context_id: 2,
                    name: name.clone(),
                    item_count: count,
                });
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(350)).await;
    }

    summaries.sort_by(|a, b| b.item_count.cmp(&a.item_count));
    if summaries.is_empty() {
        return Err("No inventory found. Make inventory public or add credentials in Settings.".into());
    }
    write_index_cache(steam_id, &summaries)?;
    Ok(summaries)
}

fn map_asset(asset: InventoryAsset, descriptions: &[InventoryDescription]) -> Option<InventoryItem> {
    let instance = asset.instanceid.as_deref().unwrap_or("0");
    let desc = descriptions.iter().find(|d| {
        d.classid == asset.classid
            && (d.instanceid == instance || d.instanceid == "0" || instance == "0")
    })?;

    Some(InventoryItem {
        id: asset.assetid,
        name: desc.name.clone().unwrap_or_else(|| "Unknown".into()),
        marketable: desc.marketable == Some(1),
        tradable: desc.tradable == Some(1),
        icon_url: desc
            .icon_url
            .as_deref()
            .map(inventory_icon_url)
            .unwrap_or_default(),
        market_hash_name: desc.market_hash_name.clone(),
    })
}

fn read_index_cache(steam_id: &str) -> Option<Vec<InventoryGameSummary>> {
    let path = index_cache_path(steam_id);
    let raw = std::fs::read_to_string(path).ok()?;
    let cache: InventoryIndexCache = serde_json::from_str(&raw).ok()?;
    let now = now_secs();
    if cache.steam_id == steam_id && now.saturating_sub(cache.fetched_at) < INDEX_TTL_SECS {
        Some(cache.games)
    } else {
        None
    }
}

fn write_index_cache(steam_id: &str, games: &[InventoryGameSummary]) -> Result<(), String> {
    let path = index_cache_path(steam_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let cache = InventoryIndexCache {
        steam_id: steam_id.to_string(),
        fetched_at: now_secs(),
        games: games.to_vec(),
    };
    let json = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
struct InventoryResponse {
    success: Option<u8>,
    error: Option<String>,
    more_items: Option<u8>,
    last_assetid: Option<String>,
    total_inventory_count: Option<u32>,
    assets: Option<Vec<InventoryAsset>>,
    descriptions: Option<Vec<InventoryDescription>>,
}

#[derive(Debug, Deserialize)]
struct InventoryAsset {
    assetid: String,
    classid: String,
    instanceid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InventoryDescription {
    classid: String,
    instanceid: String,
    name: Option<String>,
    marketable: Option<u8>,
    tradable: Option<u8>,
    #[serde(rename = "icon_url")]
    icon_url: Option<String>,
    market_hash_name: Option<String>,
}
