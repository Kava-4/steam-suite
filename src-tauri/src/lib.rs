mod commands;
mod modules;
mod state;

use modules::scheduler::resume_automation_on_startup;
use state::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, RunEvent,
};
use tauri_plugin_autostart::MacosLauncher;

pub const STARTUP_ARG: &str = "--startup";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tauri::command]
fn get_app_version() -> String {
    APP_VERSION.to_string()
}

#[tauri::command]
fn is_startup_launch() -> bool {
    std::env::args().any(|arg| arg == STARTUP_ARG)
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("Missing app icon");

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Steam Suite")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg(windows)]
fn setup_window_chrome(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE,
        DWM_WINDOW_CORNER_PREFERENCE,
    };
    use windows::Win32::UI::Controls::MARGINS;

    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let hwnd = HWND(hwnd.0);

    unsafe {
        let round = DWM_WINDOW_CORNER_PREFERENCE(2); // DWMWCP_ROUND
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &round as *const _ as *const _,
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        );

        let margins = MARGINS {
            cxLeftWidth: 1,
            cxRightWidth: 1,
            cyTopHeight: 1,
            cyBottomHeight: 1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
    }
}

#[cfg(not(windows))]
fn setup_window_chrome(_window: &tauri::WebviewWindow) {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![STARTUP_ARG]),
        ))
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            is_startup_launch,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::credentials::steam_get_credentials_status,
            commands::credentials::steam_refresh_credentials_user,
            commands::credentials::steam_sign_in_via_steam,
            commands::credentials::steam_save_credentials,
            commands::credentials::steam_clear_credentials,
            commands::credentials::steam_open_login_window,
            commands::steam::steam_get_status,
            commands::steam::steam_get_accounts,
            commands::steam::steam_get_account_context,
            commands::steam::steam_switch_account,
            commands::steam::steam_detect_account,
            commands::steam::steam_get_profile_stats,
            commands::steam::steam_get_games,
            commands::steam::steam_get_rate_limit_status,
            commands::steam::steam_reset_rate_limit,
            commands::steam::steam_enrich_trading_cards,
            commands::steam::steam_get_running_processes,
            commands::steam::steam_start_idle,
            commands::steam::steam_stop_idle,
            commands::steam::steam_start_farm,
            commands::steam::steam_stop_farm,
            commands::steam::steam_redeem_key,
            commands::steam::steam_get_achievements,
            commands::steam::steam_unlock_achievement,
            commands::steam::steam_lock_achievement,
            commands::steam::steam_toggle_achievement,
            commands::steam::steam_unlock_all_achievements,
            commands::steam::steam_lock_all_achievements,
            commands::steam::steam_get_inventory,
            commands::steam::steam_get_inventory_games,
            commands::giveaways::giveaway_fetch_points,
            commands::giveaways::giveaway_fetch_won,
            commands::giveaways::giveaway_start_bot,
            commands::giveaways::giveaway_stop_bot,
            commands::giveaways::giveaway_get_status,
            commands::giveaways::giveaway_get_logs,
            commands::scheduler::scheduler_get_status,
            commands::scheduler::scheduler_start,
            commands::scheduler::scheduler_stop,
            commands::scheduler::scheduler_advance,
            commands::saveslot::saveslot_get_status,
            commands::saveslot::saveslot_list_games_with_saves,
            commands::saveslot::saveslot_get_game_state,
            commands::saveslot::saveslot_create_profile,
            commands::saveslot::saveslot_backup,
            commands::saveslot::saveslot_restore,
            commands::saveslot::saveslot_open_vault,
        ])
        .setup(|app| {
            build_tray(app.handle())?;

            if let Some(window) = app.get_webview_window("main") {
                setup_window_chrome(&window);
            }

            if is_startup_launch() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                resume_automation_on_startup(&handle).await;
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                if let Some(window) = app_handle.get_webview_window("main") {
                    api.prevent_exit();
                    let _ = window.hide();
                }
            }
        });
}
