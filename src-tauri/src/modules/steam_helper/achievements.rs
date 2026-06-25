use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::utility::{run_utility, utility_stderr, utility_stdout};
use crate::modules::settings::data_dir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub hidden: bool,
    pub percent: f32,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
struct AchievementFile {
    achievements: Option<Vec<AchievementRaw>>,
}

#[derive(Debug, Deserialize)]
struct AchievementRaw {
    id: String,
    name: String,
    description: String,
    #[serde(rename = "iconNormal")]
    icon_normal: Option<String>,
    achieved: Option<bool>,
    hidden: Option<bool>,
    percent: Option<f32>,
}

pub fn cache_dir_for(steam_id: &str) -> PathBuf {
    data_dir().join("cache").join(steam_id)
}

pub fn achievement_file_path(steam_id: &str, app_id: u32) -> PathBuf {
    // SteamSuiteUtility writes to {cache_root}/{steam_id}/achievement_data/{app_id}.json
    cache_dir_for(steam_id)
        .join(steam_id)
        .join("achievement_data")
        .join(format!("{app_id}.json"))
}

pub fn load_cached(steam_id: &str, app_id: u32) -> Result<Vec<AchievementInfo>, String> {
    let path = achievement_file_path(steam_id, app_id);
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    parse_achievement_file(&raw, app_id)
}

pub fn fetch_achievement_data(
    utility_path: &Path,
    steam_id: &str,
    app_id: u32,
    refetch: bool,
) -> Result<Vec<AchievementInfo>, String> {
    if !refetch {
        let cached = load_cached(steam_id, app_id)?;
        if !cached.is_empty() {
            return Ok(cached);
        }
    }

    let cache_root = cache_dir_for(steam_id);
    fs::create_dir_all(&cache_root).map_err(|e| e.to_string())?;
    let cache_str = cache_root.to_string_lossy().to_string();

    let output = run_utility(
        utility_path,
        &[
            "get_achievement_data",
            &app_id.to_string(),
            &cache_str,
        ],
    )?;

    let stdout = utility_stdout(&output);
    if stdout.contains("\"error\"") {
        return Err(stdout);
    }

    if !output.status.success() {
        let stderr = utility_stderr(&output);
        if !stderr.is_empty() {
            return Err(stderr);
        }
    }

    let path = achievement_file_path(steam_id, app_id);
    if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return parse_achievement_file(&raw, app_id);
    }

    Err(format!(
        "Achievement data file was not created at {}.",
        path.display()
    ))
}

fn parse_achievement_file(raw: &str, app_id: u32) -> Result<Vec<AchievementInfo>, String> {
    let file: AchievementFile = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    Ok(file
        .achievements
        .unwrap_or_default()
        .into_iter()
        .map(|a| AchievementInfo {
            id: a.id.clone(),
            name: a.name,
            description: a.description,
            unlocked: a.achieved.unwrap_or(false),
            hidden: a.hidden.unwrap_or(false),
            percent: a.percent.unwrap_or(0.0),
            icon: achievement_icon_url(app_id, a.icon_normal.as_deref()),
        })
        .collect())
}

pub fn achievement_icon_url(app_id: u32, icon: Option<&str>) -> String {
    match icon {
        Some(raw) if !raw.trim().is_empty() => {
            let hash = raw.trim();
            if hash.starts_with("http://") || hash.starts_with("https://") {
                return hash.to_string();
            }
            let file = if hash.ends_with(".jpg") || hash.ends_with(".png") {
                hash.to_string()
            } else {
                format!("{hash}.jpg")
            };
            format!(
                "https://cdn.steamstatic.com/steamcommunity/public/images/apps/{app_id}/{file}"
            )
        }
        _ => String::new(),
    }
}

pub fn unlock_achievement(utility_path: &Path, app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_action(utility_path, &["unlock_achievement", &app_id.to_string(), achievement_id])
}

pub fn lock_achievement(utility_path: &Path, app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_action(utility_path, &["lock_achievement", &app_id.to_string(), achievement_id])
}

pub fn toggle_achievement(utility_path: &Path, app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_action(utility_path, &["toggle_achievement", &app_id.to_string(), achievement_id])
}

pub fn unlock_all(utility_path: &Path, app_id: u32) -> Result<String, String> {
    run_action(utility_path, &["unlock_all_achievements", &app_id.to_string()])
}

pub fn lock_all(utility_path: &Path, app_id: u32) -> Result<String, String> {
    run_action(utility_path, &["lock_all_achievements", &app_id.to_string()])
}

fn run_action(utility_path: &Path, args: &[&str]) -> Result<String, String> {
    let output = run_utility(utility_path, args)?;
    Ok(utility_stdout(&output))
}
