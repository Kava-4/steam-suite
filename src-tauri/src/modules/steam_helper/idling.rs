use super::paths::CREATE_NO_WINDOW;
use serde::{Deserialize, Serialize};
use std::process::Child;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct ProcessInfo {
    pub child: Child,
    pub app_id: u32,
    pub pid: u32,
    pub name: String,
    pub source: String,
}

lazy_static::lazy_static! {
    pub static ref SPAWNED_PROCESSES: Arc<Mutex<Vec<ProcessInfo>>> =
        Arc::new(Mutex::new(Vec::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningIdleProcess {
    pub app_id: u32,
    pub pid: u32,
    pub name: String,
    pub source: String,
}

pub fn cleanup_dead_processes() -> Result<(), String> {
    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    let mut i = 0;
    while i < processes.len() {
        match processes[i].child.try_wait() {
            Ok(Some(_)) | Err(_) => {
                processes.remove(i);
            }
            Ok(None) => {
                i += 1;
            }
        }
    }
    Ok(())
}

pub fn spawn_idle(
    utility_path: &std::path::Path,
    app_id: u32,
    name: &str,
    source: &str,
) -> Result<u32, String> {
    cleanup_dead_processes()?;

    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let mut command = std::process::Command::new(utility_path);
    command.args(["idle", &app_id.to_string(), name]);

    if let Some(dir) = utility_path.parent() {
        command.current_dir(dir);
    }

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let child = command.spawn().map_err(|e| e.to_string())?;
    let pid = child.id();

    SPAWNED_PROCESSES
        .lock()
        .map_err(|e| e.to_string())?
        .push(ProcessInfo {
            child,
            app_id,
            pid,
            name: name.to_string(),
            source: source.to_string(),
        });

    Ok(pid)
}

pub fn stop_idle(app_id: u32) -> Result<(), String> {
    let pid = {
        let processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
        processes
            .iter()
            .find(|p| p.app_id == app_id)
            .map(|p| p.pid)
            .ok_or_else(|| format!("No idle process for app {app_id}"))?
    };

    kill_pid(pid)?;
    if let Ok(mut processes) = SPAWNED_PROCESSES.lock() {
        processes.retain(|p| p.app_id != app_id);
    }
    Ok(())
}

pub fn stop_farm_idle() -> Result<(), String> {
    stop_all_by_source("farm")
}

pub fn stop_all_by_source(source: &str) -> Result<(), String> {
    let pids: Vec<u32> = {
        let processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
        processes
            .iter()
            .filter(|p| p.source == source)
            .map(|p| p.pid)
            .collect()
    };

    for pid in pids {
        let _ = kill_pid(pid);
    }

    if let Ok(mut processes) = SPAWNED_PROCESSES.lock() {
        processes.retain(|p| p.source != source);
    }
    Ok(())
}

pub fn list_running() -> Result<Vec<RunningIdleProcess>, String> {
    cleanup_dead_processes()?;
    let processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    Ok(processes
        .iter()
        .map(|p| RunningIdleProcess {
            app_id: p.app_id,
            pid: p.pid,
            name: p.name.clone(),
            source: p.source.clone(),
        })
        .collect())
}

fn kill_pid(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!("Failed to kill PID {pid}"));
        }
    }

    #[cfg(not(windows))]
    {
        let _ = pid;
        return Err("Steam idling is only supported on Windows".into());
    }

    Ok(())
}

pub fn kill_all_utility_processes() -> Result<u32, String> {
    cleanup_dead_processes()?;
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut killed = 0u32;
    for process in system.processes().values() {
        let name = process.name().to_string_lossy().to_ascii_lowercase();
        if name == "steamsuiteutility.exe" || name == "steamsuiteutility" {
            let pid = process.pid().as_u32();
            if kill_pid(pid).is_ok() {
                killed += 1;
            }
        }
    }

    if let Ok(mut processes) = SPAWNED_PROCESSES.lock() {
        processes.clear();
    }

    Ok(killed)
}

pub async fn verify_spawn(app_ids: &[u32]) -> Result<(), String> {
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    for app_id in app_ids {
        let alive = processes.iter_mut().any(|p| {
            p.app_id == *app_id && p.child.try_wait().ok().flatten().is_none()
        });
        if !alive {
            return Err(format!("Failed to start idling for app {app_id}"));
        }
    }
    Ok(())
}
