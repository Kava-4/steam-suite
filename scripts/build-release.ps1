# Builds native helpers, embeds them into steam-suite.exe, and produces release artifacts.
param(
    [string]$SaveSlotRoot = $env:SAVESLOT_STUDIO_ROOT,
    [switch]$SkipUtility,
    [switch]$SkipSaveSlot,
    [switch]$SkipTauri
)

$ErrorActionPreference = "Stop"

$SuiteRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$LibsDir = Join-Path $SuiteRoot "libs"
$DistDir = Join-Path $SuiteRoot "dist"
$UtilityRoot = Join-Path $SuiteRoot "tools\steam-utility"

function Resolve-SaveSlotRoot {
    param([string]$Explicit)

    if ($Explicit) {
        return (Resolve-Path $Explicit).Path
    }

    $candidates = @(
        (Join-Path $SuiteRoot "..\..\SaveSlot-Studio"),
        (Join-Path $env:USERPROFILE "SaveSlot-Studio"),
        (Join-Path $env:USERPROFILE "OneDrive\SaveSlot-Studio")
    )

    foreach ($candidate in $candidates) {
        $sln = Join-Path $candidate "SaveSlotStudio.slnx"
        if (Test-Path $sln) {
            return (Resolve-Path $candidate).Path
        }
    }

    throw "SaveSlot-Studio not found. Pass -SaveSlotRoot or set SAVESLOT_STUDIO_ROOT."
}

function Ensure-Dir([string]$Path) {
    if (-not (Test-Path $Path)) {
        New-Item -ItemType Directory -Path $Path | Out-Null
    }
}

Write-Host "==> Steam Suite release build" -ForegroundColor Cyan
Write-Host "    Root: $SuiteRoot"

Ensure-Dir $LibsDir
Ensure-Dir $DistDir

if (-not $SkipUtility) {
    if (-not (Test-Path (Join-Path $UtilityRoot "SteamUtility.csproj"))) {
        throw "Missing tools/steam-utility sources. See libs/README.md."
    }

    Write-Host "==> Building SteamSuiteUtility..." -ForegroundColor Cyan
    Push-Location $UtilityRoot
    try {
        dotnet msbuild SteamUtility.csproj /p:Configuration=Release /v:minimal
        $utilityOut = Join-Path $SuiteRoot "tools\src-tauri\libs"
        Copy-Item (Join-Path $utilityOut "SteamUtility.exe") (Join-Path $LibsDir "SteamSuiteUtility.exe") -Force
        foreach ($dll in @("steam_api.dll", "Steamworks.NET.dll", "Newtonsoft.Json.dll")) {
            Copy-Item (Join-Path $utilityOut $dll) $LibsDir -Force
        }
    }
    finally {
        Pop-Location
    }
}

if (-not $SkipSaveSlot) {
    $SaveSlotRoot = Resolve-SaveSlotRoot -Explicit $SaveSlotRoot
    Write-Host "==> Publishing SaveSlotStudio.Cli from $SaveSlotRoot" -ForegroundColor Cyan
    $cliProject = Join-Path $SaveSlotRoot "src\SaveSlotStudio.Cli\SaveSlotStudio.Cli.csproj"
    $cliPublish = Join-Path $SaveSlotRoot "publish\cli"
    dotnet publish $cliProject -c Release -o $cliPublish
    Copy-Item (Join-Path $cliPublish "SaveSlotStudio.Cli.exe") (Join-Path $LibsDir "SaveSlotStudio.Cli.exe") -Force
}

$required = @(
    "SteamSuiteUtility.exe",
    "SaveSlotStudio.Cli.exe",
    "steam_api.dll",
    "Steamworks.NET.dll",
    "Newtonsoft.Json.dll"
)
foreach ($file in $required) {
    $path = Join-Path $LibsDir $file
    if (-not (Test-Path $path)) {
        throw "Missing libs\$file - helper build failed."
    }
}

if (-not $SkipTauri) {
    Write-Host "==> Building Tauri release (single EXE with embedded libs)..." -ForegroundColor Cyan
    Push-Location $SuiteRoot
    try {
        if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
            throw "pnpm is required. Install Node 20+ and pnpm first."
        }
        if ($env:Path -notlike "*\.cargo\bin*") {
            $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
        }
        $env:CARGO_TARGET_DIR = Join-Path $SuiteRoot "src-tauri\target"
        pnpm install --frozen-lockfile
        pnpm tb
    }
    finally {
        Pop-Location
    }

    $portableExe = Join-Path $SuiteRoot "src-tauri\target\release\steam-suite.exe"
    if (-not (Test-Path $portableExe)) {
        throw "Expected release binary not found at $portableExe"
    }

    $portableOut = Join-Path $DistDir "Steam-Suite.exe"
    Copy-Item $portableExe $portableOut -Force

    $nsisDir = Join-Path $SuiteRoot "src-tauri\target\release\bundle\nsis"
    if (Test-Path $nsisDir) {
        $installer = Get-ChildItem $nsisDir -Filter "*.exe" | Select-Object -First 1
        if ($installer) {
            Copy-Item $installer.FullName (Join-Path $DistDir $installer.Name) -Force
            Write-Host "    Installer: dist\$($installer.Name)" -ForegroundColor Green
        }
    }

    $sizeMb = [math]::Round((Get-Item $portableOut).Length / 1MB, 1)
    Write-Host "    Portable:  dist\Steam-Suite.exe (${sizeMb} MB)" -ForegroundColor Green
    Write-Host "    Native helpers are embedded; first run extracts to %LOCALAPPDATA%\steam-suite\native-libs" -ForegroundColor DarkGray
}

Write-Host "==> Done." -ForegroundColor Green
