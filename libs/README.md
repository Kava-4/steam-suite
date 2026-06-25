# libs/

Native helpers used during **development**. Release builds embed these into `Steam-Suite.exe`.

## Contents (after build)

| File | Purpose |
|------|---------|
| `SteamSuiteUtility.exe` | Idler, achievements, inventory helper |
| `SaveSlotStudio.Cli.exe` | Save backup/restore CLI (self-contained) |
| `steam_api.dll` | Steamworks native API |
| `Steamworks.NET.dll` | .NET bindings |
| `Newtonsoft.Json.dll` | JSON parsing |

DLLs must stay beside `SteamSuiteUtility.exe` when using loose `libs/` in dev.

## Build everything

```powershell
cd steam-suite
pnpm release
```

This compiles the native helper from `tools/steam-utility/`, publishes the SaveSlot CLI (auto-detects sources or uses `SAVESLOT_STUDIO_ROOT`), embeds `libs/` into the Tauri binary, and writes **`dist/Steam-Suite.exe`**.

First launch extracts helpers to:

`%LOCALAPPDATA%\steam-suite\native-libs\`

## Manual helper build (dev only)

```powershell
cd tools\steam-utility
dotnet msbuild SteamUtility.csproj /p:Configuration=Release

$out = "..\src-tauri\libs"
$dst = "..\..\libs"
Copy-Item "$out\SteamUtility.exe" "$dst\SteamSuiteUtility.exe" -Force
Copy-Item "$out\steam_api.dll" $dst -Force
Copy-Item "$out\Steamworks.NET.dll" $dst -Force
Copy-Item "$out\Newtonsoft.Json.dll" $dst -Force
```

Requires **.NET Framework 4.8** targeting pack.

## SaveSlot CLI (dev only)

```powershell
$env:SAVESLOT_STUDIO_ROOT = "C:\path\to\SaveSlot-Studio"
dotnet publish "$env:SAVESLOT_STUDIO_ROOT\src\SaveSlotStudio.Cli\SaveSlotStudio.Cli.csproj" `
  -c Release -o "$env:SAVESLOT_STUDIO_ROOT\publish\cli"
Copy-Item "$env:SAVESLOT_STUDIO_ROOT\publish\cli\SaveSlotStudio.Cli.exe" libs\ -Force
```

Publish is **self-contained single-file** — only the `.exe` is needed.

## Runtime lookup

Tauri resolves helpers in this order:

1. Custom path in Settings  
2. `libs/` next to the app executable  
3. Project `libs/` (dev mode)
