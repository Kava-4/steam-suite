use std::path::PathBuf;

use crate::modules::native_libs;

pub const CLI_EXE: &str = "SaveSlotStudio.Cli.exe";

pub fn resolve_cli_path(custom: &str) -> Result<PathBuf, String> {
    native_libs::resolve_bundled_file(CLI_EXE, custom)
}

pub fn cli_ready(custom: &str) -> bool {
    native_libs::bundled_ready(CLI_EXE, custom)
}
