param([Parameter(Mandatory = $true)][string]$QualityScript)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

foreach ($doctorFailGate in @($false, $true)) {
    $global:doctorFixtureState = @{ Calls = 0; Root = $null; Fail = $doctorFailGate }
    if (Test-Path -LiteralPath "Env:NO_COLOR") {
        Remove-Item -LiteralPath "Env:NO_COLOR"
    }
    Set-Item -LiteralPath "Env:CARGO_TERM_COLOR" -Value ""
    $doctorBefore = [Environment]::GetEnvironmentVariables("Process")
    $doctorBeforeLocation = (Get-Location).Path

    function global:cargo {
        $global:doctorFixtureState.Calls += 1
        if ($env:MCP_DOCTOR_TEST_MODE -cne "1" -or
            !(Test-Path -LiteralPath $env:HOME -PathType Container) -or
            !(Test-Path -LiteralPath $env:TEMP -PathType Container)) {
            throw "The quality gate did not receive its disposable environment"
        }
        $global:doctorFixtureState.Root = $env:MCP_DOCTOR_TEST_ROOT
        $global:LASTEXITCODE = if ($global:doctorFixtureState.Fail -and $global:doctorFixtureState.Calls -eq 2) {
            23
        } else {
            0
        }
    }

    $doctorFailed = $false
    try {
        & $QualityScript
    } catch {
        if (!$doctorFailGate -or $_.Exception.Message -cne "Clippy failed with exit code 23") {
            throw "The quality gate failed outside the selected synthetic failure"
        }
        $doctorFailed = $true
    }
    if ($doctorFailed -ne $doctorFailGate -or
        $global:doctorFixtureState.Calls -ne $(if ($doctorFailGate) { 2 } else { 3 })) {
        throw "The quality gate did not follow the selected success or failure path"
    }

    $doctorAfter = [Environment]::GetEnvironmentVariables("Process")
    if ($doctorBefore.Count -ne $doctorAfter.Count) {
        throw "The quality gate changed caller environment membership"
    }
    foreach ($doctorEnvironmentName in $doctorBefore.Keys) {
        if (!$doctorAfter.Contains($doctorEnvironmentName) -or
            $doctorAfter[$doctorEnvironmentName] -cne $doctorBefore[$doctorEnvironmentName]) {
            throw "The quality gate changed a caller environment value"
        }
    }
    if ((Get-Location).Path -cne $doctorBeforeLocation -or
        $null -eq $global:doctorFixtureState.Root -or
        (Test-Path -LiteralPath $global:doctorFixtureState.Root)) {
        throw "The quality gate did not restore location and remove its disposable root"
    }
}

Write-Output "Quality environment restoration passed."
