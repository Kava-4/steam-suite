use std::path::Path;
use std::process::Output;

use super::paths::CREATE_NO_WINDOW;

pub fn run_utility(utility_path: &Path, args: &[&str]) -> Result<Output, String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let mut command = std::process::Command::new(utility_path);
    command.args(args);

    if let Some(dir) = utility_path.parent() {
        command.current_dir(dir);
    }

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    command.output().map_err(|e| e.to_string())
}

pub fn utility_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn utility_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}
