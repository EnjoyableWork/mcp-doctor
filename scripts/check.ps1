$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$doctorScriptDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$doctorRepositoryRoot = [IO.Path]::GetFullPath((Join-Path $doctorScriptDirectory ".."))
$doctorCallerUserRoot = $env:USERPROFILE
if ([string]::IsNullOrWhiteSpace($doctorCallerUserRoot)) {
    throw "USERPROFILE must identify the caller Rust toolchain root"
}
$doctorCargoHome = if ([string]::IsNullOrWhiteSpace($env:CARGO_HOME)) {
    Join-Path $doctorCallerUserRoot ".cargo"
} else {
    $env:CARGO_HOME
}
$doctorRustupHome = if ([string]::IsNullOrWhiteSpace($env:RUSTUP_HOME)) {
    Join-Path $doctorCallerUserRoot ".rustup"
} else {
    $env:RUSTUP_HOME
}
$doctorTempParent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$doctorSyntheticRoot = Join-Path $doctorTempParent ("mcp-doctor-quality-" + [guid]::NewGuid().ToString("N"))
$doctorSyntheticPrefix = Join-Path $doctorTempParent "mcp-doctor-quality-"
$doctorSyntheticUserRoot = Join-Path $doctorSyntheticRoot "user"
$doctorLocationPushed = $false

function Assert-DoctorGate {
    param([Parameter(Mandatory = $true)][string]$Name)

    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

try {
    $doctorDirectories = @(
        (Join-Path $doctorSyntheticUserRoot ".cache"),
        (Join-Path $doctorSyntheticUserRoot ".config"),
        (Join-Path $doctorSyntheticUserRoot ".local/share"),
        (Join-Path $doctorSyntheticUserRoot ".local/state"),
        (Join-Path $doctorSyntheticUserRoot "AppData/Local"),
        (Join-Path $doctorSyntheticUserRoot "AppData/Roaming"),
        (Join-Path $doctorSyntheticRoot "runtime"),
        (Join-Path $doctorSyntheticRoot "tmp")
    )
    New-Item -ItemType Directory -Force -Path $doctorDirectories | Out-Null

    $env:APPDATA = Join-Path $doctorSyntheticUserRoot "AppData/Roaming"
    $env:CARGO_HOME = $doctorCargoHome
    $env:CARGO_INCREMENTAL = "0"
    $env:CARGO_TERM_COLOR = "never"
    $env:CFFIXED_USER_HOME = $doctorSyntheticUserRoot
    $env:HOME = $doctorSyntheticUserRoot
    $env:LANG = "C"
    $env:LC_ALL = "C"
    $env:LOCALAPPDATA = Join-Path $doctorSyntheticUserRoot "AppData/Local"
    $env:MCP_DOCTOR_TEST_MODE = "1"
    $env:MCP_DOCTOR_TEST_ROOT = $doctorSyntheticRoot
    $env:NO_COLOR = "1"
    $env:RUSTUP_HOME = $doctorRustupHome
    $env:TEMP = Join-Path $doctorSyntheticRoot "tmp"
    $env:TMP = Join-Path $doctorSyntheticRoot "tmp"
    $env:TMPDIR = Join-Path $doctorSyntheticRoot "tmp"
    $env:TZ = "UTC"
    $env:USERPROFILE = $doctorSyntheticUserRoot
    $env:XDG_CACHE_HOME = Join-Path $doctorSyntheticUserRoot ".cache"
    $env:XDG_CONFIG_HOME = Join-Path $doctorSyntheticUserRoot ".config"
    $env:XDG_DATA_HOME = Join-Path $doctorSyntheticUserRoot ".local/share"
    $env:XDG_RUNTIME_DIR = Join-Path $doctorSyntheticRoot "runtime"
    $env:XDG_STATE_HOME = Join-Path $doctorSyntheticUserRoot ".local/state"

    Push-Location $doctorRepositoryRoot
    $doctorLocationPushed = $true
    Write-Output "Running quality gates through a disposable Windows user environment."

    & cargo fmt --all -- --check
    Assert-DoctorGate "Formatting"
    & cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    Assert-DoctorGate "Clippy"
    & cargo test --workspace --all-targets --all-features --locked
    Assert-DoctorGate "Tests"

    Write-Output "Formatting, Clippy, and tests passed."
} finally {
    if ($doctorLocationPushed) {
        Pop-Location
    }

    $doctorResolvedRoot = [IO.Path]::GetFullPath($doctorSyntheticRoot)
    if (!$doctorResolvedRoot.StartsWith($doctorSyntheticPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove unexpected quality-gate path: $doctorResolvedRoot"
    }
    if (Test-Path -LiteralPath $doctorResolvedRoot -PathType Container) {
        Remove-Item -LiteralPath $doctorResolvedRoot -Recurse -Force
    }
}
