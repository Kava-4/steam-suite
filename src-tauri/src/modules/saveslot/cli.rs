use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::path::Path;
use std::process::Output;

use super::paths::resolve_cli_path;
use crate::modules::steam_helper::paths::CREATE_NO_WINDOW;
use crate::modules::steam_helper::utility::{utility_stderr, utility_stdout};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliEnvelope<T> {
    success: bool,
    error: Option<String>,
    data: Option<T>,
}

pub fn run_cli(cli_path: &Path, args: &[&str]) -> Result<Output, String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let mut command = std::process::Command::new(cli_path);
    command.args(args);

    if let Some(dir) = cli_path.parent() {
        command.current_dir(dir);
    }

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    command.output().map_err(|e| e.to_string())
}

pub fn invoke_cli<T: DeserializeOwned>(custom_path: &str, args: &[&str]) -> Result<T, String> {
    let cli_path = resolve_cli_path(custom_path)?;
    let output = run_cli(&cli_path, args)?;

    if !output.status.success() {
        let stderr = utility_stderr(&output);
        let stdout = utility_stdout(&output);
        if let Ok(envelope) = serde_json::from_str::<CliEnvelope<serde_json::Value>>(&stdout) {
            if let Some(error) = envelope.error {
                return Err(error);
            }
        }
        if !stderr.is_empty() {
            return Err(stderr);
        }
        if !stdout.is_empty() {
            return Err(stdout);
        }
        return Err(format!(
            "SaveSlot CLI exited with code {:?}",
            output.status.code()
        ));
    }

    parse_cli_response(&output)
}

fn parse_cli_response<T: DeserializeOwned>(output: &Output) -> Result<T, String> {
    let stdout = utility_stdout(output);
    if stdout.is_empty() {
        let stderr = utility_stderr(output);
        return Err(if stderr.is_empty() {
            "SaveSlot CLI produced no output.".to_string()
        } else {
            stderr
        });
    }

    let envelope: CliEnvelope<T> =
        serde_json::from_str(&stdout).map_err(|e| format!("Invalid SaveSlot CLI JSON: {e}"))?;

    if envelope.success {
        envelope
            .data
            .ok_or_else(|| "SaveSlot CLI succeeded but returned no data.".to_string())
    } else {
        Err(envelope
            .error
            .unwrap_or_else(|| "Unknown SaveSlot CLI error.".to_string()))
    }
}
