use super::steamgifts::{within_end_window, SteamgiftsService};
use super::wins;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GiveawayBotStatus {
    pub running: bool,
    pub points: u32,
    pub current_page: u32,
    pub source: String,
    pub countdown_label: String,
    pub countdown_seconds: u32,
    pub last_message: String,
    pub entries_today: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GiveawayLogEntry {
    pub timestamp: String,
    pub message: String,
}

pub struct GiveawayBotHandle {
    stop: Arc<AtomicBool>,
    task: Mutex<Option<JoinHandle<()>>>,
    status: Arc<Mutex<GiveawayBotStatus>>,
    logs: Arc<Mutex<Vec<GiveawayLogEntry>>>,
}

impl Default for GiveawayBotHandle {
    fn default() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            task: Mutex::new(None),
            status: Arc::new(Mutex::new(GiveawayBotStatus::default())),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl GiveawayBotHandle {
    pub async fn status(&self) -> GiveawayBotStatus {
        self.status.lock().await.clone()
    }

    pub async fn logs(&self) -> Vec<GiveawayLogEntry> {
        self.logs.lock().await.clone()
    }

    pub async fn is_running(&self) -> bool {
        self.status.lock().await.running
    }

    pub async fn start(
        &self,
        cookie: String,
        refresh_delay_minutes: u32,
        max_pages: u32,
        max_giveaway_end_hours: u32,
        entry_delay_secs: u32,
        app: Option<AppHandle>,
        steam_id: String,
        notify_on_win: bool,
        auto_redeem_on_win: bool,
    ) -> Result<(), String> {
        if cookie.trim().is_empty() {
            return Err("PHPSESSID cookie is required.".into());
        }

        self.stop.store(false, Ordering::SeqCst);
        {
            let mut task = self.task.lock().await;
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }

        let stop = self.stop.clone();
        let status = self.status.clone();
        let logs = self.logs.clone();
        let app_handle = app;
        let notify = notify_on_win;
        let auto_claim = auto_redeem_on_win;

        let steam_id_for_wins = steam_id.clone();

        let handle = tokio::spawn(async move {
            {
                let mut s = status.lock().await;
                s.running = true;
                s.last_message = "Bot started".into();
            }

            let service = SteamgiftsService::new(&cookie);
            let mut current_page = 1u32;
            let max_pages = max_pages.max(1);
            let refresh_delay = refresh_delay_minutes.max(1) * 60;

            loop {
                if stop.load(Ordering::SeqCst) {
                    break;
                }

                match service.fetch_search_page(current_page).await {
                    Ok((mut points, xsrf, giveaways)) => {
                        {
                            let mut s = status.lock().await;
                            s.points = points;
                            s.current_page = current_page;
                            s.source = "steamgifts".into();
                        }

                        push_log(&logs, format!("[SteamGifts] Page {current_page} — {points}P"))
                            .await;

                        for giveaway in giveaways {
                            if stop.load(Ordering::SeqCst) {
                                break;
                            }

                            if giveaway.is_entered {
                                continue;
                            }

                            if !within_end_window(&giveaway, max_giveaway_end_hours) {
                                continue;
                            }

                            if points < giveaway.cost {
                                push_log(
                                    &logs,
                                    format!(
                                        "[SteamGifts] Not enough points for {}",
                                        giveaway.name
                                    ),
                                )
                                .await;
                                break;
                            }

                            tokio::time::sleep(std::time::Duration::from_secs(
                                entry_delay_secs.max(2) as u64,
                            ))
                            .await;

                            if stop.load(Ordering::SeqCst) {
                                break;
                            }

                            match service
                                .enter_giveaway(&giveaway.code, &xsrf)
                                .await
                            {
                                Ok(()) => {
                                    points = points.saturating_sub(giveaway.cost);
                                    push_log(
                                        &logs,
                                        format!("[SteamGifts] Entered: {}", giveaway.name),
                                    )
                                    .await;
                                    let mut s = status.lock().await;
                                    s.entries_today += 1;
                                    s.last_message =
                                        format!("Entered {}", giveaway.name);
                                }
                                Err(error) => {
                                    push_log(
                                        &logs,
                                        format!(
                                            "[SteamGifts] Failed {}: {error}",
                                            giveaway.name
                                        ),
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        push_log(&logs, format!("[SteamGifts] Error: {error}")).await;
                        {
                            let mut s = status.lock().await;
                            s.last_message = error;
                        }
                    }
                }

                if stop.load(Ordering::SeqCst) {
                    break;
                }

                if current_page < max_pages {
                    current_page += 1;
                    continue;
                }

                current_page = 1;
                push_log(
                    &logs,
                    format!("Waiting {refresh_delay_minutes} min for refresh"),
                )
                .await;

                if let Some(ref app_handle) = app_handle {
                    let _ = wins::process_wins(
                        Some(app_handle),
                        &cookie,
                        &steam_id_for_wins,
                        notify,
                        auto_claim,
                    )
                    .await;
                }

                for remaining in (0..refresh_delay).rev() {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    {
                        let mut s = status.lock().await;
                        s.countdown_label = "refresh".into();
                        s.countdown_seconds = remaining;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }

            {
                let mut s = status.lock().await;
                s.running = false;
                s.countdown_label.clear();
                s.countdown_seconds = 0;
                s.last_message = "Bot stopped".into();
            }
            push_log(&logs, "Bot stopped".to_string()).await;
        });

        *self.task.lock().await = Some(handle);
        Ok(())
    }

    pub async fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.task.lock().await.take() {
            handle.abort();
        }
        let mut s = self.status.lock().await;
        s.running = false;
        s.last_message = "Stopping...".into();
    }
}

async fn push_log(logs: &Arc<Mutex<Vec<GiveawayLogEntry>>>, message: String) {
    let timestamp = chrono_lite_now();
    let mut entries = logs.lock().await;
    entries.push(GiveawayLogEntry { timestamp, message });
    if entries.len() > 200 {
        let drain = entries.len() - 200;
        entries.drain(0..drain);
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}
