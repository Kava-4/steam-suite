use std::path::Path;

fn main() {
  println!("cargo:rustc-check-cfg=cfg(embed_libs)");

  let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  let libs = manifest_dir.join("../libs");
  let required = ["SteamSuiteUtility.exe", "SaveSlotStudio.Cli.exe"];

  let embed_ready = required
    .iter()
    .all(|name| libs.join(name).is_file());

  if embed_ready {
    println!("cargo:rustc-cfg=embed_libs");
    println!("cargo:rerun-if-changed=../libs/SteamSuiteUtility.exe");
    println!("cargo:rerun-if-changed=../libs/SaveSlotStudio.Cli.exe");
    println!("cargo:rerun-if-changed=../libs/steam_api.dll");
    println!("cargo:rerun-if-changed=../libs/Steamworks.NET.dll");
    println!("cargo:rerun-if-changed=../libs/Newtonsoft.Json.dll");
  } else {
    println!(
      "cargo:warning=libs/ helpers missing — release will not embed native tools (dev loose libs only)"
    );
  }

  tauri_build::build()
}
