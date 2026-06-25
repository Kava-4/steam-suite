use std::path::PathBuf;

use crate::modules::native_libs;

pub const UTILITY_EXE: &str = "SteamSuiteUtility.exe";

pub fn resolve_utility_path(custom: &str) -> Result<PathBuf, String> {
    native_libs::resolve_bundled_file(UTILITY_EXE, custom)
}

pub fn utility_ready(custom: &str) -> bool {
    native_libs::bundled_ready(UTILITY_EXE, custom)
}

#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn steam_capsule_url(app_id: u32) -> String {
    format!(
        "https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/header.jpg"
    )
}

pub fn is_steam_running() -> bool {
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    system.processes().values().any(|p| {
        let name = p.name().to_string_lossy().to_ascii_lowercase();
        name == "steam.exe" || name == "steam"
    })
}
