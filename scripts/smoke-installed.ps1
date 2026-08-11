param(
    [Parameter(Mandatory = $true)]
    [string]$Executable,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedVersion,

    [Parameter(Mandatory = $true)]
    [string]$Fixture
)

$ErrorActionPreference = 'Stop'

if ($ExpectedVersion -notmatch '^\d+\.\d+\.\d+$') {
    throw 'Expected version must be a stable semantic version.'
}

$resolvedExecutable = (Resolve-Path -LiteralPath $Executable).Path
$resolvedFixture = (Resolve-Path -LiteralPath $Fixture).Path
$smokeRoot = Join-Path ([System.IO.Path]::GetTempPath()) (
    'mcp-doctor-release-smoke-' + [guid]::NewGuid().ToString('N')
)
$smokeHome = Join-Path $smokeRoot 'home'

function Invoke-McpDoctor {
    param(
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Arguments
    )

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $resolvedExecutable
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.Environment.Clear()
    $startInfo.Environment['APPDATA'] = $smokeHome
    $startInfo.Environment['HOME'] = $smokeHome
    $startInfo.Environment['LOCALAPPDATA'] = $smokeHome
    $startInfo.Environment['NO_COLOR'] = '1'
    $startInfo.Environment['PATH'] = $env:PATH
    $startInfo.Environment['TEMP'] = $smokeRoot
    $startInfo.Environment['TMP'] = $smokeRoot
    $startInfo.Environment['TZ'] = 'UTC'
    $startInfo.Environment['USERPROFILE'] = $smokeHome

    foreach ($argument in $Arguments) {
        $startInfo.ArgumentList.Add($argument)
    }

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    if (-not $process.Start()) {
        throw 'Could not start the installed mcp-doctor executable.'
    }

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()

    if ($process.ExitCode -ne 0) {
        throw "Installed mcp-doctor command failed with exit code $($process.ExitCode)."
    }
    if (-not [string]::IsNullOrEmpty($stderr)) {
        throw 'Installed mcp-doctor command wrote unexpected stderr.'
    }

    return $stdout.TrimEnd("`r", "`n")
}

try {
    New-Item -ItemType Directory -Path $smokeHome -Force | Out-Null

    $versionOutput = Invoke-McpDoctor '--version'
    if ($versionOutput -ne "mcp-doctor $ExpectedVersion") {
        throw 'Installed executable reported an unexpected version.'
    }

    $reportOutput = Invoke-McpDoctor `
        'inspect' '--format' 'json' '--' $resolvedFixture 'catalog-valid'
    $report = $reportOutput | ConvertFrom-Json

    if (
        $report.schema_version -ne 'mcp-doctor.report/v1' -or
        $report.schema_stability -ne 'stable' -or
        $report.protocol_revision -ne '2026-07-28' -or
        $null -ne $report.primary_diagnosis -or
        @($report.independent_findings).Count -ne 0 -or
        $report.outcome -ne 'passed' -or
        $report.exit_code -ne 0 -or
        $report.summary.required -ne 5 -or
        $report.summary.required_skipped -ne 0 -or
        $report.summary.failed -ne 0
    ) {
        throw 'Installed passive diagnostic returned an unexpected report contract.'
    }

    $requiredChecks = @($report.checks | Where-Object requirement -eq 'required')
    if ($requiredChecks.Count -ne 5) {
        throw 'Installed passive diagnostic did not declare five required checks.'
    }
    foreach ($check in $requiredChecks) {
        if ($check.state -ne 'performed' -or $check.outcome -ne 'passed') {
            throw 'Installed passive diagnostic did not pass every required check.'
        }
    }

    $runtimeChecks = @($report.checks | Where-Object id -eq 'runtime.tools')
    if (
        $runtimeChecks.Count -ne 1 -or
        $runtimeChecks[0].state -ne 'skipped' -or
        $runtimeChecks[0].skip_reason -ne 'not_authorized' -or
        ($runtimeChecks[0].PSObject.Properties.Name -contains 'blocked_by')
    ) {
        throw 'Installed passive diagnostic did not preserve the no-tool-call boundary.'
    }
}
finally {
    if (Test-Path -LiteralPath $smokeRoot) {
        Remove-Item -LiteralPath $smokeRoot -Recurse -Force
    }
}
