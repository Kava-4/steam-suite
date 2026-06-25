use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::modules::settings::{self, AppSettings};
use super::session;

lazy_static! {
    static ref DROPDOWN: Regex = Regex::new(
        r#"<div\s+class="popup_block_new"\s+id="account_dropdown"\s+style="display:\s*none;"#
    )
    .unwrap();
    static ref PERSONA: Regex = Regex::new(
        r#"<a\s+href="https://steamcommunity\.com/(id|profiles)/[^"]*"\s+data-miniprofile="\d+">([^<]+)\s*"#
    )
    .unwrap();
    static ref PULLDOWN: Regex = Regex::new(
        r#"id="account_pulldown"[^>]*>([^<]+)<"#
    )
    .unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialsStatus {
    pub connected: bool,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginWindowResult {
    pub success: bool,
    pub session_id: Option<String>,
    pub steam_login_secure: Option<String>,
    pub message: Option<String>,
}

pub fn status_from_settings(settings: &AppSettings) -> CredentialsStatus {
    let connected = !settings.steam_session_id.is_empty()
        && !settings.steam_login_secure.is_empty();

    let stored_user = settings.steam_credentials_user.trim();
    let user = if connected {
        if !stored_user.is_empty() {
            Some(stored_user.to_string())
        } else {
            session::get_active_steam_account()
                .ok()
                .flatten()
                .map(|a| a.persona_name)
                .filter(|name| !name.trim().is_empty())
        }
    } else {
        None
    };

    CredentialsStatus { connected, user }
}

pub fn build_cookie_header(settings: &AppSettings, steam_id: &str) -> Option<String> {
    if settings.steam_session_id.is_empty() || settings.steam_login_secure.is_empty() {
        return None;
    }

    let sid = &settings.steam_session_id;
    let sls = &settings.steam_login_secure;
    let sma = settings.steam_machine_auth.trim();

    Some(if sma.is_empty() {
        format!("sessionid={sid}; steamLoginSecure={sls}")
    } else {
        format!(
            "sessionid={sid}; steamLoginSecure={sls}; steamparental={sma}; steamMachineAuth{steam_id}={sma}"
        )
    })
}

pub async fn validate_session(
    session_id: &str,
    steam_login_secure: &str,
    steam_machine_auth: Option<&str>,
    steam_id: &str,
) -> Result<String, String> {
    if !steam_login_secure.starts_with(steam_id) {
        return Err("Credentials do not match the detected Steam account.".into());
    }

    let cookie_value = match steam_machine_auth.filter(|v| !v.is_empty()) {
        Some(sma) => format!(
            "sessionid={session_id}; steamLoginSecure={steam_login_secure}; steamparental={sma}; steamMachineAuth{steam_id}={sma}"
        ),
        None => format!("sessionid={session_id}; steamLoginSecure={steam_login_secure}"),
    };

    let client = Client::new();
    let response = client
        .get("https://steamcommunity.com/?l=english")
        .header("Cookie", cookie_value)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let html = response.text().await.map_err(|e| e.to_string())?;

    if DROPDOWN.is_match(&html) {
        if let Some(name) = extract_persona_name(&html) {
            return Ok(name);
        }

        if let Ok(Some(account)) = session::get_active_steam_account() {
            if !account.persona_name.trim().is_empty() {
                return Ok(account.persona_name);
            }
        }

        return Err("Logged in but could not read your Steam display name.".into());
    }

    Err("Not logged in.".into())
}

fn extract_persona_name(html: &str) -> Option<String> {
    if let Some(captures) = PERSONA.captures(html) {
        let name = captures[2].trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }

    if let Some(captures) = PULLDOWN.captures(html) {
        let name = captures[1].trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }

    None
}

pub async fn save_credentials(
    session_id: String,
    steam_login_secure: String,
    steam_machine_auth: Option<String>,
    steam_id: &str,
) -> Result<CredentialsStatus, String> {
    let user = validate_session(
        &session_id,
        &steam_login_secure,
        steam_machine_auth.as_deref(),
        steam_id,
    )
    .await?;

    let mut settings = settings::load_settings();
    crate::modules::accounts::update_credentials(
        &mut settings,
        steam_id,
        session_id,
        steam_login_secure,
        steam_machine_auth.unwrap_or_default(),
        user.trim().to_string(),
    )?;

    Ok(CredentialsStatus {
        connected: true,
        user: Some(user.trim().to_string()),
    })
}

pub async fn clear_credentials(app: &AppHandle) -> Result<(), String> {
    delete_login_window_cookies(app).await?;

    let mut settings = settings::load_settings();
    let steam_id = settings.steam_id.clone();
    if steam_id.is_empty() {
        settings.steam_session_id.clear();
        settings.steam_login_secure.clear();
        settings.steam_machine_auth.clear();
        settings.steam_credentials_user.clear();
        return settings::save_settings(&settings);
    }

    crate::modules::accounts::clear_credentials(&mut settings, &steam_id)
}

pub async fn open_steam_login_window(app: &AppHandle) -> Result<LoginWindowResult, String> {
    let window = tauri::webview::WebviewWindowBuilder::new(
        app,
        "steam-login",
        tauri::WebviewUrl::External(
            "https://steamcommunity.com/login/home/?goto="
                .parse()
                .map_err(|e| format!("Invalid URL: {e}"))?,
        ),
    )
    .title("Steam Login")
    .inner_size(800.0, 700.0)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    if let Some(found) = read_steam_cookies(&window)? {
        let _ = window.close();
        return Ok(LoginWindowResult {
            success: true,
            session_id: Some(found.0),
            steam_login_secure: Some(found.1),
            message: None,
        });
    }

    window.show().map_err(|e| e.to_string())?;

    let window_clone = window.clone();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            let _ = tx.try_send(());
        }
    });

    let start = std::time::Instant::now();
    loop {
        if rx.try_recv().is_ok() {
            return Ok(LoginWindowResult {
                success: false,
                session_id: None,
                steam_login_secure: None,
                message: Some("Login window closed".into()),
            });
        }

        if start.elapsed() > Duration::from_secs(300) {
            let _ = window_clone.close();
            return Ok(LoginWindowResult {
                success: false,
                session_id: None,
                steam_login_secure: None,
                message: Some("Login timed out".into()),
            });
        }

        if window_clone.get_webview("steam-login").is_none() {
            return Ok(LoginWindowResult {
                success: false,
                session_id: None,
                steam_login_secure: None,
                message: Some("Login window closed".into()),
            });
        }

        if let Some(found) = read_steam_cookies(&window_clone)? {
            let _ = window_clone.close();
            return Ok(LoginWindowResult {
                success: true,
                session_id: Some(found.0),
                steam_login_secure: Some(found.1),
                message: None,
            });
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

fn read_steam_cookies(
    window: &tauri::WebviewWindow,
) -> Result<Option<(String, String)>, String> {
    let webview = window
        .get_webview("steam-login")
        .ok_or("Steam login webview not ready")?;
    let cookies = webview
        .cookies_for_url(
            tauri::Url::parse("https://steamcommunity.com/")
                .map_err(|e| format!("Invalid URL: {e}"))?,
        )
        .map_err(|e| e.to_string())?;

    let mut session_id = None;
    let mut steam_login_secure = None;

    for cookie in cookies {
        if cookie.name() == "sessionid" {
            session_id = Some(cookie.value().to_string());
        }
        if cookie.name() == "steamLoginSecure" {
            steam_login_secure = Some(cookie.value().to_string());
        }
    }

    Ok(match (session_id, steam_login_secure) {
        (Some(sid), Some(sls)) => Some((sid, sls)),
        _ => None,
    })
}

pub async fn delete_login_window_cookies(app: &AppHandle) -> Result<(), String> {
    let window = tauri::webview::WebviewWindowBuilder::new(
        app,
        "steam-logout",
        tauri::WebviewUrl::External(
            "https://steamcommunity.com/"
                .parse()
                .map_err(|e| format!("Invalid URL: {e}"))?,
        ),
    )
    .title("Steam Logout")
    .inner_size(1.0, 1.0)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    tokio::time::sleep(Duration::from_millis(1500)).await;

    if let Some(webview) = window.get_webview("steam-logout") {
        let cookies = webview.cookies().map_err(|e| e.to_string())?;
        for cookie in cookies {
            webview.delete_cookie(cookie).map_err(|e| e.to_string())?;
        }
    }

    let _ = window.close();
    Ok(())
}
