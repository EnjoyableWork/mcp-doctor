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
$smokeSnapshot = Join-Path $smokeRoot 'contract.json'
$smokeJsonArtifact = Join-Path $smokeRoot 'report-artifact.json'
$smokeJunitArtifact = Join-Path $smokeRoot 'report-artifact.xml'
$smokeAggregate = Join-Path $smokeRoot 'aggregate.json'

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

    $capabilitiesOutput = Invoke-McpDoctor 'capabilities' '--format' 'json'
    $capabilities = $capabilitiesOutput | ConvertFrom-Json
    $commandNames = @($capabilities.commands | ForEach-Object name)
    $exitCodes = @($capabilities.exit_semantics.codes | ForEach-Object code)
    $inspectStdio = @(
        $capabilities.protocol_support |
            Where-Object { $_.command -eq 'inspect' -and $_.transport -eq 'stdio' }
    )
    $checkHttp = @(
        $capabilities.protocol_support |
            Where-Object {
                $_.command -eq 'check' -and $_.transport -eq 'streamable_http'
            }
    )
    if (
        $capabilities.schema_version -ne 'mcp-doctor.capabilities/v1' -or
        $capabilities.schema_stability -ne 'stable' -or
        $capabilities.product.name -ne 'mcp-doctor' -or
        $capabilities.product.version -ne $ExpectedVersion -or
        ($commandNames -join ',') -ne 'aggregate,break,capabilities,check,diff,inspect' -or
        $inspectStdio.Count -ne 1 -or
        (@($inspectStdio[0].revisions) -join ',') -ne '2026-07-28,2025-11-25,2025-06-18' -or
        $checkHttp.Count -ne 1 -or
        (@($checkHttp[0].revisions) -join ',') -ne '2026-07-28' -or
        (@($capabilities.schema_versions.diagnostic_report) -join ',') -ne 'mcp-doctor.report/v1' -or
        (@($capabilities.schema_versions.scenario) -join ',') -ne 'mcp-doctor.scenario/v1alpha1' -or
        (@($capabilities.schema_versions.generator) -join ',') -ne 'mcp-doctor.generator/v1' -or
        $capabilities.exit_semantics.version -ne 'mcp-doctor.exit/v1' -or
        ($exitCodes -join ',') -ne '0,1,2,3,4' -or
        $capabilities.platform.family -ne 'windows' -or
        $capabilities.platform.process_tree_control -ne 'job_object' -or
        $capabilities.platform.file_identity -ne 'volume_file_id' -or
        @($capabilities.limit_profiles | Where-Object hard -eq $true).Count -ne 4 -or
        $capabilities.limits.output_bytes -ne 65536
    ) {
        throw 'Installed executable returned an unexpected compiled-capability contract.'
    }

    $reportOutput = Invoke-McpDoctor `
        'inspect' '--format' 'json' `
        '--json-report' $smokeJsonArtifact `
        '--junit-report' $smokeJunitArtifact `
        '--snapshot' $smokeSnapshot `
        '--allow-sensitive-snapshot' $smokeSnapshot `
        '--' $resolvedFixture 'catalog-valid'
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

    if (-not (Test-Path -LiteralPath $smokeJsonArtifact -PathType Leaf)) {
        throw 'Installed diagnostic did not create the JSON report artifact.'
    }
    $artifactReport = Get-Content -LiteralPath $smokeJsonArtifact -Raw | ConvertFrom-Json
    if (
        $artifactReport.schema_version -ne $report.schema_version -or
        $artifactReport.outcome -ne $report.outcome -or
        $artifactReport.exit_code -ne $report.exit_code -or
        @($artifactReport.checks).Count -ne @($report.checks).Count
    ) {
        throw 'Installed diagnostic JSON artifact diverged from stdout.'
    }
    if (-not (Test-Path -LiteralPath $smokeJunitArtifact -PathType Leaf)) {
        throw 'Installed diagnostic did not create the JUnit report artifact.'
    }

    $aggregateOutput = Invoke-McpDoctor `
        'aggregate' '--format' 'json' '--output' $smokeAggregate $smokeJsonArtifact
    $aggregate = $aggregateOutput | ConvertFrom-Json
    if (-not (Test-Path -LiteralPath $smokeAggregate -PathType Leaf)) {
        throw 'Installed offline diagnostic aggregate did not create its artifact.'
    }
    $aggregateArtifact = Get-Content -LiteralPath $smokeAggregate -Raw | ConvertFrom-Json
    if (
        $aggregate.schema_version -ne 'mcp-doctor.aggregate/v1' -or
        $aggregate.schema_stability -ne 'stable' -or
        $aggregate.outcome -ne 'passed' -or
        $aggregate.exit_code -ne 0 -or
        $aggregate.summary.members -ne 1 -or
        $aggregate.summary.passed -ne 1 -or
        $aggregate.summary.failed -ne 0 -or
        $aggregate.summary.incomplete -ne 0 -or
        @($aggregate.members).Count -ne 1 -or
        $aggregate.members[0].ordinal -ne 0 -or
        $aggregate.members[0].report.schema_version -ne 'mcp-doctor.report/v1' -or
        $aggregate.members[0].report.outcome -ne 'passed' -or
        $aggregateArtifact.schema_version -ne $aggregate.schema_version -or
        $aggregateArtifact.outcome -ne $aggregate.outcome
    ) {
        throw 'Installed offline diagnostic aggregate returned inconsistent evidence.'
    }
    [xml]$junitArtifact = Get-Content -LiteralPath $smokeJunitArtifact -Raw
    $junitRoot = $junitArtifact.DocumentElement
    if (
        $null -eq $junitRoot -or
        $junitRoot.LocalName -ne 'testsuites' -or
        $junitRoot.GetAttribute('name') -ne 'mcp-doctor' -or
        [int]$junitRoot.GetAttribute('tests') -ne @($report.checks).Count -or
        -not $junitRoot.InnerText.Contains('report_outcome=passed') -or
        -not $junitRoot.InnerText.Contains('exit_code=0')
    ) {
        throw 'Installed diagnostic JUnit artifact omitted required evidence.'
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

    if (-not (Test-Path -LiteralPath $smokeSnapshot -PathType Leaf)) {
        throw 'Installed passive diagnostic did not create the acknowledged snapshot.'
    }
    $snapshot = Get-Content -LiteralPath $smokeSnapshot -Raw | ConvertFrom-Json
    if (
        $snapshot.schema_version -ne 'mcp-doctor.contract-snapshot/v1alpha1' -or
        $snapshot.protocol_revision -ne '2026-07-28' -or
        -not $snapshot.capabilities.tools.advertised -or
        @($snapshot.catalogs.tools.contracts).Count -ne 2 -or
        @($snapshot.catalogs.tools.correlation).Count -ne 2 -or
        @($snapshot.catalogs.prompts.contracts).Count -ne 1 -or
        @($snapshot.catalogs.resources.contracts).Count -ne 1 -or
        @($snapshot.catalogs.resource_templates.contracts).Count -ne 1
    ) {
        throw 'Installed passive diagnostic returned an unexpected snapshot contract.'
    }

    $diffOutput = Invoke-McpDoctor `
        'diff' '--format' 'json' $smokeSnapshot $smokeSnapshot
    $diff = $diffOutput | ConvertFrom-Json
    if (
        $diff.schema_version -ne 'mcp-doctor.contract-diff/v1alpha1' -or
        $diff.protocol_revision -ne '2026-07-28' -or
        $diff.outcome -ne 'unchanged' -or
        $diff.exit_code -ne 0 -or
        $diff.summary.total -ne 0 -or
        @($diff.findings).Count -ne 0
    ) {
        throw 'Installed offline contract diff returned an unexpected result.'
    }
    foreach ($check in @($diff.checks)) {
        if ($check.state -ne 'performed') {
            throw 'Installed offline contract diff did not perform every comparison check.'
        }
    }
}
finally {
    if (Test-Path -LiteralPath $smokeRoot) {
        Remove-Item -LiteralPath $smokeRoot -Recurse -Force
    }
}
