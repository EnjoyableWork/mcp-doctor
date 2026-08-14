param(
    [Parameter(Mandatory = $true)]
    [string]$RunnerLabel
)

$ErrorActionPreference = "Stop"
$inventoryPath = Join-Path $PSScriptRoot "../.github/ci-tools.json"
if (-not (Test-Path -LiteralPath $inventoryPath -PathType Leaf)) {
    throw "CI tool inventory is unavailable"
}

$inventory = Get-Content -LiteralPath $inventoryPath -Raw | ConvertFrom-Json
if ($inventory.schema_version -ne "mcp-doctor.ci-tools/v1") {
    throw "CI tool inventory schema is unsupported"
}
$contracts = @($inventory.runner_contracts | Where-Object { $_.runner -eq $RunnerLabel })
if ($contracts.Count -ne 1 -or @($contracts[0].commands).Count -eq 0) {
    throw "CI tool inventory has no valid exact runner contract"
}

foreach ($command in $contracts[0].commands) {
    if ($command -notmatch '^[a-z0-9][a-z0-9-]*$') {
        throw "CI tool inventory contains an invalid command name"
    }
    if (-not (Get-Command -Name $command -ErrorAction SilentlyContinue)) {
        throw "required declared runner command is unavailable: $command"
    }
}

Write-Output "Verified declared runner commands for $RunnerLabel."
