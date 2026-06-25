use crate::modules::settings;
use crate::modules::steam_helper::credentials::{
    self, CredentialsStatus, LoginWindowResult,
};
use crate::modules::steam_helper::session;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn steam_get_credentials_status(state: State<'_, AppState>) -> CredentialsStatus {
    let settings = state.settings.lock().unwrap();
    let mut status = credentials::status_from_settings(&settings);

    if status.connected && status.user.is_none() {
        if let Ok(Some(account)) = session::get_active_steam_account() {
            if !account.persona_name.trim().is_empty() {
                status.user = Some(account.persona_name);
            }
        }
    }

    status
}

#[tauri::command]
pub async fn steam_refresh_credentials_user(
    state: State<'_, AppState>,
) -> Result<CredentialsStatus, String> {
    let settings = state.settings.lock().unwrap().clone();
    if settings.steam_session_id.is_empty() || settings.steam_login_secure.is_empty() {
        return Ok(credentials::status_from_settings(&settings));
    }

    let steam_id = session::resolve_steam_id(&settings.steam_id)?;
    let machine_auth = settings.steam_machine_auth.trim();
    let user = credentials::validate_session(
        &settings.steam_session_id,
        &settings.steam_login_secure,
        if machine_auth.is_empty() {
            None
        } else {
            Some(machine_auth)
        },
        &steam_id,
    )
    .await
    .unwrap_or_else(|_| {
        session::get_active_steam_account()
            .ok()
            .flatten()
            .map(|a| a.persona_name)
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_default()
    });

    if user.trim().is_empty() {
        return Err("Could not resolve display name".into());
    }

    let mut updated = settings;
    updated.steam_credentials_user = user.trim().to_string();
    settings::save_settings(&updated)?;
    *state.settings.lock().unwrap() = updated.clone();

    Ok(credentials::status_from_settings(&updated))
}

#[tauri::command]
pub async fn steam_sign_in_via_steam(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CredentialsStatus, String> {
    let steam_id = {
        let settings = state.settings.lock().unwrap();
        session::resolve_steam_id(&settings.steam_id)?
    };

    let result = credentials::open_steam_login_window(&app).await?;
    if !result.success {
        return Err(result
            .message
            .unwrap_or_else(|| "Steam login failed".into()));
    }

    let session_id = result
        .session_id
        .ok_or("Missing sessionid from Steam login")?;
    let steam_login_secure = result
        .steam_login_secure
        .ok_or("Missing steamLoginSecure from Steam login")?;

    let status = credentials::save_credentials(
        session_id,
        steam_login_secure,
        None,
        &steam_id,
    )
    .await?;

    let mut locked = state.settings.lock().unwrap();
    *locked = settings::load_settings();
    Ok(status)
}

#[tauri::command]
pub async fn steam_save_credentials(
    session_id: String,
    steam_login_secure: String,
    steam_machine_auth: Option<String>,
    state: State<'_, AppState>,
) -> Result<CredentialsStatus, String> {
    let steam_id = {
        let settings = state.settings.lock().unwrap();
        session::resolve_steam_id(&settings.steam_id)?
    };

    let status = credentials::save_credentials(
        session_id,
        steam_login_secure,
        steam_machine_auth,
        &steam_id,
    )
    .await?;

    let mut locked = state.settings.lock().unwrap();
    *locked = settings::load_settings();
    Ok(status)
}

#[tauri::command]
pub async fn steam_clear_credentials(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    credentials::clear_credentials(&app).await?;
    let mut locked = state.settings.lock().unwrap();
    *locked = settings::load_settings();
    Ok(())
}

#[tauri::command]
pub async fn steam_open_login_window(app: AppHandle) -> Result<LoginWindowResult, String> {
    credentials::open_steam_login_window(&app).await
}
