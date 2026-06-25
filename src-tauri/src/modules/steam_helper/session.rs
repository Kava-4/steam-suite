use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamAccount {
    pub steam_id: String,
    pub persona_name: String,
    pub account_name: String,
    pub most_recent: i64,
    pub is_active: bool,
}

pub fn login_users_path() -> Result<PathBuf, String> {
    let steam_dir = steamlocate::SteamDir::locate().map_err(|e| e.to_string())?;
    Ok(steam_dir.path().join("config").join("loginusers.vdf"))
}

pub fn get_steam_accounts() -> Result<Vec<SteamAccount>, String> {
    let path = login_users_path()?;
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    parse_login_users(&content)
}

pub fn get_client_active_account() -> Result<Option<SteamAccount>, String> {
    Ok(get_steam_accounts()?
        .into_iter()
        .find(|a| a.is_active))
}

pub fn get_active_steam_account() -> Result<Option<SteamAccount>, String> {
    get_client_active_account()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamAccountContext {
    pub selected_steam_id: String,
    pub selected_persona_name: Option<String>,
    pub client_active_steam_id: Option<String>,
    pub client_active_persona_name: Option<String>,
    pub client_mismatch: bool,
    pub accounts: Vec<SteamAccount>,
}

pub fn build_account_context(selected_steam_id: &str) -> Result<SteamAccountContext, String> {
    let accounts = get_steam_accounts()?;
    let client_active = accounts.iter().find(|a| a.is_active).cloned();
    let selected = accounts
        .iter()
        .find(|a| a.steam_id == selected_steam_id)
        .cloned();

    let client_active_steam_id = client_active.as_ref().map(|a| a.steam_id.clone());
    let client_mismatch = client_active_steam_id
        .as_ref()
        .map(|id| id != selected_steam_id)
        .unwrap_or(false);

    Ok(SteamAccountContext {
        selected_steam_id: selected_steam_id.to_string(),
        selected_persona_name: selected.map(|a| a.persona_name),
        client_active_steam_id,
        client_active_persona_name: client_active.map(|a| a.persona_name),
        client_mismatch,
        accounts,
    })
}

pub fn resolve_steam_id(settings_id: &str) -> Result<String, String> {
    if !settings_id.trim().is_empty() {
        return Ok(settings_id.trim().to_string());
    }
    get_active_steam_account()?
        .map(|a| a.steam_id)
        .ok_or_else(|| "No Steam account found. Sign in to the Steam client.".into())
}

fn parse_login_users(content: &str) -> Result<Vec<SteamAccount>, String> {
    let mut users: HashMap<String, (String, String, i64, bool)> = HashMap::new();
    let mut current_id = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(id) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            if id.len() == 17 && id.chars().all(|c| c.is_ascii_digit()) {
                current_id = id.to_string();
                users.entry(current_id.clone()).or_insert_with(|| {
                    (String::new(), String::new(), 0, false)
                });
            }
        }

        if current_id.is_empty() {
            continue;
        }

        let entry = users.get_mut(&current_id).unwrap();

        if trimmed.contains("\"PersonaName\"") {
            if let Some(value) = extract_vdf_value(trimmed) {
                entry.0 = value;
            }
        } else if trimmed.contains("\"AccountName\"") {
            if let Some(value) = extract_vdf_value(trimmed) {
                entry.1 = value;
            }
        } else if trimmed.contains("\"Timestamp\"") {
            if let Some(value) = extract_vdf_value(trimmed) {
                entry.2 = value.parse().unwrap_or(0);
            }
        } else if trimmed.contains("\"MostRecent\"") {
            if let Some(value) = extract_vdf_value(trimmed) {
                entry.3 = value == "1";
            }
        }
    }

    let mut accounts: Vec<SteamAccount> = users
        .into_iter()
        .filter(|(_, (persona, account, _, _))| !persona.is_empty() || !account.is_empty())
        .map(|(steam_id, (persona_name, account_name, most_recent, is_active))| SteamAccount {
            steam_id,
            persona_name: if persona_name.is_empty() {
                account_name.clone()
            } else {
                persona_name
            },
            account_name,
            most_recent,
            is_active,
        })
        .collect();

    accounts.sort_by(|a, b| {
        b.is_active
            .cmp(&a.is_active)
            .then(b.most_recent.cmp(&a.most_recent))
    });

    Ok(accounts)
}

fn extract_vdf_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}
