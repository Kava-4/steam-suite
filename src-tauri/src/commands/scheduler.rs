use crate::modules::scheduler::SchedulerStatus;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn scheduler_get_status(state: State<'_, AppState>) -> SchedulerStatus {
    state.scheduler.lock().unwrap().status.clone()
}

#[tauri::command]
pub async fn scheduler_start(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.scheduler_runner.start(app).await
}

#[tauri::command]
pub async fn scheduler_stop(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.scheduler_runner.stop(&app).await;
    Ok(())
}

#[tauri::command]
pub fn scheduler_advance(state: State<'_, AppState>) {
    let settings = state.settings.lock().unwrap().clone();
    state
        .scheduler
        .lock()
        .unwrap()
        .advance(&settings.scheduler_tasks);
}
