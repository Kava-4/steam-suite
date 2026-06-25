use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(embed_libs)]
use include_dir::{include_dir, Dir};

#[cfg(embed_libs)]
static LIBS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../libs");

const RUNTIME_LIB_NAMES: &[&str] = &[
    "SteamSuiteUtility.exe",
    "steam_api.dll",
    "Steamworks.NET.dll",
    "Newtonsoft.Json.dll",
    "SaveSlotStudio.Cli.exe",
];

pub fn resolve_bundled_file(file_name: &str, custom: &str) -> Result<PathBuf, String> {
    if !custom.trim().is_empty() {
        let path = PathBuf::from(custom.trim());
        if path.exists() {
            return Ok(path);
        }
        return Err(format!("Native helper not found at {}", path.display()));
    }

    let dir = runtime_libs_dir()?;
    let path = dir.join(file_name);
    if path.exists() {
        return Ok(path);
    }

    Err(format!(
        "{file_name} not found. Run scripts/build-release.ps1 to bundle native helpers."
    ))
}

pub fn bundled_ready(file_name: &str, custom: &str) -> bool {
    resolve_bundled_file(file_name, custom).is_ok()
}

fn runtime_libs_dir() -> Result<PathBuf, String> {
    #[cfg(embed_libs)]
    {
        return ensure_embedded_extracted();
    }

    #[cfg(not(embed_libs))]
    {
        dev_libs_dir()
    }
}

#[cfg(not(embed_libs))]
fn dev_libs_dir() -> Result<PathBuf, String> {
    if let Ok(mut exe) = std::env::current_exe() {
        exe.pop();
        let loose = exe.join("libs");
        if loose.join(RUNTIME_LIB_NAMES[0]).exists() {
            return Ok(loose);
        }
    }

    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("libs");
    if dev.join(RUNTIME_LIB_NAMES[0]).exists() {
        return Ok(dev);
    }

    Err("Native libs not found. Build helpers into libs/ or run scripts/build-release.ps1.".into())
}

#[cfg(embed_libs)]
fn ensure_embedded_extracted() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| "Could not resolve LOCALAPPDATA.".to_string())?
        .join("steam-suite")
        .join("native-libs");

    fs::create_dir_all(&base).map_err(|e| e.to_string())?;

    let token = embedded_manifest_token();
    let marker = base.join(".manifest");
    let marker_ok = fs::read_to_string(&marker)
        .map(|saved| saved.trim() == token)
        .unwrap_or(false);

    if marker_ok && runtime_files_present(&base) {
        return Ok(base);
    }

    for name in RUNTIME_LIB_NAMES {
        let file = LIBS_DIR
            .get_file(name)
            .ok_or_else(|| format!("Embedded lib missing at build time: {name}"))?;
        write_file_atomically(&base.join(name), file.contents())?;
    }

    write_file_atomically(&marker, token.as_bytes())?;
    Ok(base)
}

#[cfg(embed_libs)]
fn runtime_files_present(base: &Path) -> bool {
    RUNTIME_LIB_NAMES
        .iter()
        .all(|name| base.join(name).is_file())
}

#[cfg(embed_libs)]
fn embedded_manifest_token() -> String {
    let mut parts = vec![env!("CARGO_PKG_VERSION").to_string()];
    for name in RUNTIME_LIB_NAMES {
        if let Some(file) = LIBS_DIR.get_file(name) {
            parts.push(format!("{name}:{}", file.contents().len()));
        }
    }
    parts.join("|")
}

fn write_file_atomically(path: &Path, contents: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let temp = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&temp).map_err(|e| e.to_string())?;
        file.write_all(contents).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
    }

    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    fs::rename(&temp, path).map_err(|e| e.to_string())?;
    Ok(())
}
