use crate::modules::giveaways::wins;
use crate::modules::steam_helper::ops;
use crate::state::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

const FARM_MAX_MINUTES: u64 = 120;
const IDLE_RUN_MINUTES: u64 = 30;

pub struct SchedulerRunnerHandle {
    stop: Arc<AtomicBool>,
    task: Mutex<Option<JoinHandle<()>>>,
}

impl Default for SchedulerRunnerHandle {
    fn default() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            task: Mutex::new(None),
        }
    }
}

impl SchedulerRunnerHandle {
    pub async fn start(&self, app: AppHandle) -> Result<(), String> {
        let settings = app.state::<AppState>().settings.lock().unwrap().clone();
        if settings.scheduler_tasks.is_empty() {
            return Err("Add at least one task to the scheduler chain.".into());
        }

        self.stop.store(false, Ordering::SeqCst);
        {
            let mut task = self.task.lock().await;
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }

        {
            let state = app.state::<AppState>();
            let mut scheduler = state.scheduler.lock().unwrap();
            scheduler.start(&settings.scheduler_tasks);
        }

        let stop = self.stop.clone();
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            if let Err(error) = run_chain(&app_clone, stop.clone()).await {
                let state = app_clone.state::<AppState>();
                let mut scheduler = state.scheduler.lock().unwrap();
                scheduler.status.last_error = Some(error);
                scheduler.stop();
            }
        });

        *self.task.lock().await = Some(handle);
        Ok(())
    }

    pub async fn stop(&self, app: &AppHandle) {
        self.stop.store(true, Ordering::SeqCst);

        if let Some(handle) = self.task.lock().await.take() {
            handle.abort();
        }

        let state = app.state::<AppState>();
        state.giveaway_bot.stop().await;
        let settings = state.settings.lock().unwrap().clone();
        let _ = ops::stop_farm(&settings);
        let _ = ops::stop_all_idle();

        let mut scheduler = state.scheduler.lock().unwrap();
        scheduler.stop();
    }
}

async fn run_chain(app: &AppHandle, stop: Arc<AtomicBool>) -> Result<(), String> {
    loop {
        if stop.load(Ordering::SeqCst) {
            return Ok(());
        }

        let (current, tasks, running) = {
            let state = app.state::<AppState>();
            let scheduler = state.scheduler.lock().unwrap();
            let tasks = state.settings.lock().unwrap().scheduler_tasks.clone();
            (
                scheduler.status.current_task.clone(),
                tasks,
                scheduler.status.running,
            )
        };

        if !running {
            break;
        }

        let Some(task_id) = current else {
            break;
        };

        let result = execute_task(app, &task_id, stop.clone()).await;

        if stop.load(Ordering::SeqCst) {
            return Ok(());
        }

        if let Err(error) = result {
            let state = app.state::<AppState>();
            let mut scheduler = state.scheduler.lock().unwrap();
            scheduler.status.last_error = Some(format!("{task_id}: {error}"));
        }

        {
            let state = app.state::<AppState>();
            let mut scheduler = state.scheduler.lock().unwrap();
            scheduler.advance(&tasks);
        }
    }

    Ok(())
}

async fn execute_task(
    app: &AppHandle,
    task_id: &str,
    stop: Arc<AtomicBool>,
) -> Result<(), String> {
    match task_id {
        "card-farming" => task_card_farming(app, stop).await,
        "auto-idler" => task_auto_idler(app, stop).await,
        "giveaways" => task_giveaways(app, stop).await,
        "redeem" => task_redeem(app).await,
        other => Err(format!("Unknown scheduler task: {other}")),
    }
}

async fn task_card_farming(app: &AppHandle, stop: Arc<AtomicBool>) -> Result<(), String> {
    let settings = app.state::<AppState>().settings.lock().unwrap().clone();
    if settings.farm_game_ids.is_empty() {
        return Ok(());
    }

    ops::resume_farm(&settings).await?;

    let deadline = Instant::now() + Duration::from_secs(FARM_MAX_MINUTES * 60);
    loop {
        if stop.load(Ordering::SeqCst) {
            return Err("Stopped".into());
        }
        if Instant::now() >= deadline {
            break;
        }
        if ops::farm_process_count() == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }

    let settings = app.state::<AppState>().settings.lock().unwrap().clone();
    ops::stop_farm(&settings)?;
    Ok(())
}

async fn task_auto_idler(app: &AppHandle, stop: Arc<AtomicBool>) -> Result<(), String> {
    let settings = app.state::<AppState>().settings.lock().unwrap().clone();
    if settings.idle_game_ids.is_empty() {
        return Ok(());
    }

    ops::resume_idle(&settings).await?;

    let deadline = Instant::now() + Duration::from_secs(IDLE_RUN_MINUTES * 60);
    while Instant::now() < deadline {
        if stop.load(Ordering::SeqCst) {
            ops::stop_all_idle()?;
            return Err("Stopped".into());
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    ops::stop_all_idle()?;
    Ok(())
}

async fn task_giveaways(app: &AppHandle, stop: Arc<AtomicBool>) -> Result<(), String> {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();

    if settings.steamgifts_cookie.is_empty() {
        return Err("SteamGifts PHPSESSID required.".into());
    }

    state
        .giveaway_bot
        .start(
            settings.steamgifts_cookie.clone(),
            settings.refresh_delay_minutes,
            settings.max_pages,
            settings.max_giveaway_end_hours,
            2,
            Some(app.clone()),
            settings.steam_id.clone(),
            settings.notify_on_win,
            settings.auto_redeem_on_win,
        )
        .await?;

    let wait_secs = settings.refresh_delay_minutes.max(1) as u64 * 60;
    let deadline = Instant::now() + Duration::from_secs(wait_secs);
    while Instant::now() < deadline {
        if stop.load(Ordering::SeqCst) {
            state.giveaway_bot.stop().await;
            return Err("Stopped".into());
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    state.giveaway_bot.stop().await;
    Ok(())
}

async fn task_redeem(app: &AppHandle) -> Result<(), String> {
    let settings = app.state::<AppState>().settings.lock().unwrap().clone();
    wins::process_wins(
        Some(app),
        &settings.steamgifts_cookie,
        &settings.steam_id,
        settings.notify_on_win,
        settings.auto_redeem_on_win,
    )
    .await?;
    Ok(())
}

pub async fn resume_automation_on_startup(app: &AppHandle) {
    tokio::time::sleep(Duration::from_secs(3)).await;

    let settings = app.state::<AppState>().settings.lock().unwrap().clone();

    if settings.card_farming_enabled && !settings.farm_game_ids.is_empty() {
        if let Err(error) = ops::resume_farm(&settings).await {
            eprintln!("[startup] card farming resume: {error}");
        }
    }

    if settings.auto_idle_on_start && !settings.idle_game_ids.is_empty() {
        if let Err(error) = ops::resume_idle(&settings).await {
            eprintln!("[startup] auto idle: {error}");
        }
    }

    if settings.scheduler_enabled {
        let state = app.state::<AppState>();
        if let Err(error) = state.scheduler_runner.start(app.clone()).await {
            eprintln!("[startup] scheduler: {error}");
        }
    }
}
