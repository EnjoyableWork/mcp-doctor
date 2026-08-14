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
$smokeLegacy11Snapshot = Join-Path $smokeRoot 'contract-2025-11-25.json'
$smokeLegacy06Snapshot = Join-Path $smokeRoot 'contract-2025-06-18.json'
$smokeJsonArtifact = Join-Path $smokeRoot 'report-artifact.json'
$smokeJunitArtifact = Join-Path $smokeRoot 'report-artifact.xml'
$smokeLegacyScenario = Join-Path $smokeRoot 'legacy-scenario.json'
$smokeWorkflowScenario = Join-Path $smokeRoot 'workflow-scenario.json'
$smokeRejectMarker = Join-Path $smokeRoot 'reject-count.txt'
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

function Assert-LegacyActiveReport {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Report,

        [Parameter(Mandatory = $true)]
        [string]$ExpectedRevision,

        [Parameter(Mandatory = $true)]
        [int]$ExpectedRequired,

        [Parameter(Mandatory = $true)]
        [int]$ExpectedCases,

        [Parameter(Mandatory = $true)]
        [bool]$ExpectGeneration
    )

    if (
        $Report.schema_version -ne 'mcp-doctor.report/v1' -or
        $Report.protocol_revision -ne $ExpectedRevision -or
        $Report.negotiated_protocol_revision -ne $ExpectedRevision -or
        $null -ne $Report.primary_diagnosis -or
        @($Report.independent_findings).Count -ne 0 -or
        $Report.outcome -ne 'passed' -or
        $Report.exit_code -ne 0 -or
        $Report.summary.required -ne $ExpectedRequired -or
        $Report.summary.required_skipped -ne 0 -or
        $Report.summary.failed -ne 0
    ) {
        throw 'Installed legacy active diagnostic returned an unexpected report.'
    }
    foreach ($check in @($Report.checks | Where-Object requirement -eq 'required')) {
        if ($check.state -ne 'performed' -or $check.outcome -ne 'passed') {
            throw 'Installed legacy active diagnostic did not pass every required check.'
        }
    }
    $runtimeCases = @(
        $Report.checks | Where-Object { $_.id.StartsWith('runtime.tools.case[') }
    )
    if ($runtimeCases.Count -ne $ExpectedCases) {
        throw 'Installed legacy active diagnostic returned an unexpected case count.'
    }
    $generation = @($Report.checks | Where-Object id -eq 'generation.cases')
    if (($generation.Count -eq 1) -ne $ExpectGeneration) {
        throw 'Installed legacy active diagnostic returned unexpected generation evidence.'
    }
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
    $rejectStdio = @(
        $capabilities.protocol_support |
            Where-Object { $_.command -eq 'reject' -and $_.transport -eq 'stdio' }
    )
    $rejectHttp = @(
        $capabilities.protocol_support |
            Where-Object {
                $_.command -eq 'reject' -and $_.transport -eq 'streamable_http'
            }
    )
    if (
        $capabilities.schema_version -ne 'mcp-doctor.capabilities/v1' -or
        $capabilities.schema_stability -ne 'stable' -or
        $capabilities.product.name -ne 'mcp-doctor' -or
        $capabilities.product.version -ne $ExpectedVersion -or
        ($commandNames -join ',') -ne 'aggregate,break,capabilities,check,diff,inspect,reject' -or
        $inspectStdio.Count -ne 1 -or
        (@($inspectStdio[0].revisions) -join ',') -ne '2026-07-28,2025-11-25,2025-06-18' -or
        $checkHttp.Count -ne 1 -or
        (@($checkHttp[0].revisions) -join ',') -ne '2026-07-28,2025-11-25,2025-06-18' -or
        $rejectStdio.Count -ne 1 -or
        (@($rejectStdio[0].revisions) -join ',') -ne '2026-07-28' -or
        $rejectHttp.Count -ne 1 -or
        (@($rejectHttp[0].revisions) -join ',') -ne '2026-07-28' -or
        (@($capabilities.schema_versions.diagnostic_report) -join ',') -ne 'mcp-doctor.report/v1' -or
        (@($capabilities.schema_versions.scenario) -join ',') -ne 'mcp-doctor.scenario/v1alpha1,mcp-doctor.scenario/v2alpha1' -or
        (@($capabilities.schema_versions.generator) -join ',') -ne 'mcp-doctor.generator/v1' -or
        (@($capabilities.schema_versions.contract_snapshot) -join ',') -ne 'mcp-doctor.contract-snapshot/v1alpha1' -or
        (@($capabilities.schema_versions.contract_diff) -join ',') -ne 'mcp-doctor.contract-diff/v1alpha1' -or
        $capabilities.exit_semantics.version -ne 'mcp-doctor.exit/v1' -or
        ($exitCodes -join ',') -ne '0,1,2,3,4' -or
        $capabilities.platform.family -ne 'windows' -or
        $capabilities.platform.process_tree_control -ne 'job_object' -or
        $capabilities.platform.file_identity -ne 'volume_file_id' -or
        @($capabilities.limit_profiles | Where-Object hard -eq $true).Count -ne 4 -or
        (@(
            $capabilities.limit_profiles |
                Where-Object id -eq 'mcp-doctor.limits/diagnostic/v1' |
                ForEach-Object { $_.selections -join ',' }
        ) -join ',') -ne 'default,slow-start' -or
        (@(
            $capabilities.limit_profiles |
                Where-Object id -eq 'mcp-doctor.limits/diagnostic/v1' |
                ForEach-Object { $_.selectable_for -join ',' }
        ) -join ',') -ne 'break,check,inspect' -or
        $capabilities.limits.output_bytes -ne 65536
    ) {
        throw 'Installed executable returned an unexpected compiled-capability contract.'
    }

    $reportOutput = Invoke-McpDoctor `
        'inspect' '--limit-profile' 'slow-start' '--format' 'json' `
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
        $report.limits.profile -ne 'slow-start' -or
        $report.limits.startup_ms -ne 30000 -or
        $report.limits.discovery_ms -ne 30000 -or
        $report.limits.request_ms -ne 60000 -or
        $report.limits.response_ms -ne 60000 -or
        $report.limits.shutdown_grace_ms -ne 2000 -or
        $report.limits.total_ms -ne 240000 -or
        $report.limits.redirects -ne 0 -or
        $report.limits.retries -ne 0 -or
        $report.limits.concurrency -ne 1 -or
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
    $junitArtifact = Get-Content -LiteralPath $smokeJunitArtifact -Raw
    if (-not $junitArtifact.Contains('limits.profile=slow-start')) {
        throw 'Installed diagnostic JUnit artifact omitted the selected limit profile.'
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

    $rejectOutput = Invoke-McpDoctor `
        'reject' `
        '--tool' 'synthetic.reviewed' `
        '--allow-tool' 'synthetic.reviewed' `
        '--effects' 'read_only' `
        '--seed' '7529' `
        '--format' 'json' `
        '--' $resolvedFixture 'reject-success' $smokeRejectMarker
    $rejectReport = $rejectOutput | ConvertFrom-Json
    $rejectCases = @(
        $rejectReport.checks |
            Where-Object { $_.id.StartsWith('runtime.tools.case[') }
    )
    $rejectGeneration = @($rejectReport.checks | Where-Object id -eq 'generation.cases')
    if (
        $rejectReport.schema_version -ne 'mcp-doctor.report/v1' -or
        $rejectReport.protocol_revision -ne '2026-07-28' -or
        $null -ne $rejectReport.primary_diagnosis -or
        @($rejectReport.independent_findings).Count -ne 0 -or
        $rejectReport.outcome -ne 'passed' -or
        $rejectReport.exit_code -ne 0 -or
        $rejectReport.summary.failed -ne 0 -or
        $rejectCases.Count -ne 7 -or
        @($rejectCases | Where-Object {
            $_.state -eq 'performed' -and $_.outcome -eq 'passed'
        }).Count -ne 7 -or
        @($rejectCases | Where-Object {
            $_.state -eq 'skipped' -and $_.skip_reason -eq 'not_applicable'
        }).Count -ne 0 -or
        $rejectGeneration.Count -ne 1 -or
        $rejectGeneration[0].state -ne 'performed' -or
        $rejectGeneration[0].outcome -ne 'passed' -or
        (Get-Content -LiteralPath $smokeRejectMarker -Raw).Trim() -ne '7'
    ) {
        throw 'Installed current-revision reject diagnostic returned unexpected evidence.'
    }
    foreach ($privateRejectValue in @(
        'synthetic.reviewed',
        'synthetic-secret-payload-7f2c',
        'synthetic_private_mode_never_report_7f2c',
        'mcp-doctor-invalid-enum',
        'sequence',
        'secret'
    )) {
        if ($rejectOutput.Contains($privateRejectValue)) {
            throw 'Installed current-revision reject report disclosed a private value.'
        }
    }

    $workflowScenario = [ordered]@{
        schema_version = 'mcp-doctor.scenario/v2alpha1'
        steps = @(
            [ordered]@{
                id = 'installed-private-lookup'
                tool = 'synthetic.workflow.lookup'
                safety = [ordered]@{ effects = 'read_only' }
                arguments = [ordered]@{ query = 'synthetic-secret-payload-7f2c' }
                captures = [ordered]@{ resource_id = '/resource/id' }
                expect = [ordered]@{ result = 'success' }
            },
            [ordered]@{
                id = 'installed-private-read'
                tool = 'synthetic.workflow.read'
                safety = [ordered]@{ effects = 'read_only' }
                arguments = [ordered]@{ id = $null }
                argument_refs = [ordered]@{ '/id' = 'resource_id' }
                expect = [ordered]@{ result = 'success' }
            }
        )
    } | ConvertTo-Json -Depth 8
    [System.IO.File]::WriteAllText(
        $smokeWorkflowScenario,
        $workflowScenario,
        [System.Text.UTF8Encoding]::new($false)
    )
    $workflowOutput = Invoke-McpDoctor `
        'check' '--scenario' $smokeWorkflowScenario `
        '--allow-tool' 'synthetic.workflow.lookup' `
        '--allow-tool' 'synthetic.workflow.read' `
        '--format' 'json' `
        '--' $resolvedFixture 'workflow-read-only'
    $workflowReport = $workflowOutput | ConvertFrom-Json
    $workflowChecks = @(
        $workflowReport.checks |
            Where-Object { $_.id.StartsWith('runtime.workflow.step[') }
    )
    if (
        $workflowReport.schema_version -ne 'mcp-doctor.report/v1' -or
        $workflowReport.protocol_revision -ne '2026-07-28' -or
        $null -ne $workflowReport.primary_diagnosis -or
        @($workflowReport.independent_findings).Count -ne 0 -or
        $workflowReport.outcome -ne 'passed' -or
        $workflowReport.exit_code -ne 0 -or
        $workflowChecks.Count -ne 2 -or
        @($workflowChecks | Where-Object {
            $_.state -eq 'performed' -and $_.outcome -eq 'passed'
        }).Count -ne 2
    ) {
        throw 'Installed current-revision workflow diagnostic returned unexpected evidence.'
    }
    foreach ($privateWorkflowValue in @(
        'synthetic.workflow.lookup',
        'synthetic.workflow.read',
        'installed-private-lookup',
        'installed-private-read',
        'synthetic-secret-payload-7f2c',
        'resource_id'
    )) {
        if ($workflowOutput.Contains($privateWorkflowValue)) {
            throw 'Installed workflow report disclosed a private value.'
        }
    }

    $legacyScenario = [ordered]@{
        schema_version = 'mcp-doctor.scenario/v1alpha1'
        tool = 'synthetic.reviewed'
        safety = [ordered]@{ effects = 'read_only' }
        cases = @(
            [ordered]@{
                id = 'installed-legacy-case'
                arguments = [ordered]@{ sequence = 0 }
                expect = [ordered]@{ result = 'success' }
            }
        )
    } | ConvertTo-Json -Depth 8
    [System.IO.File]::WriteAllText(
        $smokeLegacyScenario,
        $legacyScenario,
        [System.Text.UTF8Encoding]::new($false)
    )

    foreach ($activeRevision in @('2025-11-25', '2025-06-18')) {
        $legacyCheckOutput = Invoke-McpDoctor `
            'check' '--protocol-version' $activeRevision `
            '--scenario' $smokeLegacyScenario `
            '--allow-tool' 'synthetic.reviewed' `
            '--format' 'json' `
            '--' $resolvedFixture 'legacy-active-success'
        $legacyCheck = $legacyCheckOutput | ConvertFrom-Json
        Assert-LegacyActiveReport `
            -Report $legacyCheck `
            -ExpectedRevision $activeRevision `
            -ExpectedRequired 8 `
            -ExpectedCases 1 `
            -ExpectGeneration $false

        $legacyBreakOutput = Invoke-McpDoctor `
            'break' '--protocol-version' $activeRevision `
            '--tool' 'synthetic.generated' `
            '--allow-tool' 'synthetic.generated' `
            '--effects' 'read_only' `
            '--cases' '2' `
            '--seed' '6027' `
            '--format' 'json' `
            '--' $resolvedFixture 'legacy-break-success' '2'
        $legacyBreak = $legacyBreakOutput | ConvertFrom-Json
        Assert-LegacyActiveReport `
            -Report $legacyBreak `
            -ExpectedRevision $activeRevision `
            -ExpectedRequired 10 `
            -ExpectedCases 2 `
            -ExpectGeneration $true
        foreach ($privateActiveValue in @(
            'synthetic.reviewed',
            'synthetic.generated',
            'installed-legacy-case',
            'synthetic-secret-payload-7f2c',
            'synthetic_private_query_never_report_7f2c',
            'synthetic_private_limit_never_report_7f2c',
            'synthetic_private_flags_never_report_7f2c'
        )) {
            if (
                $legacyCheckOutput.Contains($privateActiveValue) -or
                $legacyBreakOutput.Contains($privateActiveValue)
            ) {
                throw 'Installed legacy active report disclosed a private value.'
            }
        }
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

    $legacyCases = @(
        @{
            Revision = '2025-11-25'
            Dialect = 'draft_2020_12'
            Snapshot = $smokeLegacy11Snapshot
        },
        @{
            Revision = '2025-06-18'
            Dialect = 'ambiguous'
            Snapshot = $smokeLegacy06Snapshot
        }
    )
    foreach ($legacyCase in $legacyCases) {
        $legacyReportOutput = Invoke-McpDoctor `
            'inspect' '--format' 'json' `
            '--protocol-version' $legacyCase.Revision `
            '--snapshot' $legacyCase.Snapshot `
            '--allow-sensitive-snapshot' $legacyCase.Snapshot `
            '--' $resolvedFixture 'legacy-success'
        $legacyReport = $legacyReportOutput | ConvertFrom-Json
        if (
            $legacyReport.schema_version -ne 'mcp-doctor.report/v1' -or
            $legacyReport.protocol_revision -ne $legacyCase.Revision -or
            $legacyReport.negotiated_protocol_revision -ne $legacyCase.Revision -or
            $legacyReport.outcome -ne 'passed' -or
            $legacyReport.exit_code -ne 0
        ) {
            throw 'Installed legacy passive diagnostic returned an unexpected report.'
        }
        if (-not (Test-Path -LiteralPath $legacyCase.Snapshot -PathType Leaf)) {
            throw 'Installed legacy passive diagnostic did not create its snapshot.'
        }
        $legacySnapshot = Get-Content -LiteralPath $legacyCase.Snapshot -Raw | ConvertFrom-Json
        $legacyTools = @($legacySnapshot.catalogs.tools.contracts)
        if (
            $legacySnapshot.schema_version -ne 'mcp-doctor.contract-snapshot/v1alpha1' -or
            $legacySnapshot.protocol_revision -ne $legacyCase.Revision -or
            $legacySnapshot.negotiated_protocol_revision -ne $legacyCase.Revision -or
            -not $legacySnapshot.capabilities.tools.advertised -or
            $legacyTools.Count -ne 1 -or
            $legacyTools[0].input_schema_dialect -ne $legacyCase.Dialect -or
            $legacyTools[0].output_schema_dialect -ne $legacyCase.Dialect
        ) {
            throw 'Installed legacy snapshot returned an unexpected revision contract.'
        }
        $capabilityNames = @($legacySnapshot.capabilities.PSObject.Properties.Name)
        if ($legacyCase.Revision -eq '2025-11-25') {
            if (
                -not $legacySnapshot.capabilities.logging.advertised -or
                -not $legacySnapshot.capabilities.tasks.advertised -or
                -not $legacySnapshot.capabilities.tasks.list -or
                -not $legacySnapshot.capabilities.tasks.cancel -or
                -not $legacySnapshot.capabilities.tasks.requests_tools_call
            ) {
                throw 'Installed MCP 2025-11-25 snapshot omitted fixed capability evidence.'
            }
        }
        elseif (
            $capabilityNames -contains 'logging' -or
            $capabilityNames -contains 'tasks'
        ) {
            throw 'Installed MCP 2025-06-18 snapshot inferred absent capabilities.'
        }

        $legacyDiffOutput = Invoke-McpDoctor `
            'diff' '--format' 'json' $legacyCase.Snapshot $legacyCase.Snapshot
        $legacyDiff = $legacyDiffOutput | ConvertFrom-Json
        if (
            $legacyDiff.schema_version -ne 'mcp-doctor.contract-diff/v1alpha1' -or
            $legacyDiff.protocol_revision -ne $legacyCase.Revision -or
            $legacyDiff.outcome -ne 'unchanged' -or
            $legacyDiff.exit_code -ne 0 -or
            $legacyDiff.summary.total -ne 0 -or
            @($legacyDiff.findings).Count -ne 0
        ) {
            throw 'Installed legacy offline contract diff returned an unexpected result.'
        }
        foreach ($check in @($legacyDiff.checks)) {
            if ($check.state -ne 'performed') {
                throw 'Installed legacy diff did not perform every comparison check.'
            }
        }
    }
}
finally {
    if (Test-Path -LiteralPath $smokeRoot) {
        Remove-Item -LiteralPath $smokeRoot -Recurse -Force
    }
}
