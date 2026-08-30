use std::fmt::{self, Write as _};
use std::io;
use std::process::ExitCode;

use serde::{Deserialize, Deserializer, Serialize, de};

use super::limits::{DiagnosticLimitProfile, DiagnosticLimits, LimitValues};
use super::model::{
    CheckId, CheckOutcome, CheckResult, Finding, FindingCode, FindingEvidence,
    GeneratedCaseReproduction, Location, Requirement, RuleViolation, Severity, StructuralInput,
};
use super::protocol::{KnownRevision, ProtocolSelectionEvidence, SupportedRevision};
use super::redaction::REDACTION_MARKER;

pub(crate) const REPORT_SCHEMA_VERSION: &str = "mcp-doctor.report/v1";
pub(crate) const MARKDOWN_REPORT_VERSION: &str = "mcp-doctor.markdown/v1";
pub(crate) const BADGE_REPORT_VERSION: &str = "mcp-doctor.badge/v1";
const AGGREGATE_REPORT_BYTES: u64 = 8_388_608;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReportFormat {
    Human,
    Json,
    Junit,
    Markdown,
    Badge,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReportArtifactFormat {
    Json,
    Junit,
    Markdown,
    Badge,
}

impl ReportArtifactFormat {
    const fn report_format(self) -> ReportFormat {
        match self {
            Self::Json => ReportFormat::Json,
            Self::Junit => ReportFormat::Junit,
            Self::Markdown => ReportFormat::Markdown,
            Self::Badge => ReportFormat::Badge,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct ReportRequest {
    stdout: ReportFormat,
    json_artifact: bool,
    junit_artifact: bool,
    markdown_artifact: bool,
    badge_artifact: bool,
}

impl ReportRequest {
    pub(crate) const fn new(
        stdout: ReportFormat,
        json_artifact: bool,
        junit_artifact: bool,
        markdown_artifact: bool,
        badge_artifact: bool,
    ) -> Self {
        Self {
            stdout,
            json_artifact,
            junit_artifact,
            markdown_artifact,
            badge_artifact,
        }
    }

    pub(crate) const fn stdout_only(stdout: ReportFormat) -> Self {
        Self::new(stdout, false, false, false, false)
    }
}

pub(crate) struct RenderedReportArtifact {
    pub(crate) format: ReportArtifactFormat,
    pub(crate) output: String,
}

pub(crate) struct RenderedReports {
    pub(crate) stdout: String,
    pub(crate) artifacts: Vec<RenderedReportArtifact>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ReportRenderError {
    SizeLimitExceeded { maximum: u64 },
    AggregateSizeLimitExceeded { maximum: u64 },
    RenderFailed,
}

impl fmt::Display for ReportRenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SizeLimitExceeded { maximum } => write!(
                formatter,
                "diagnostic report exceeded the configured {maximum}-byte output limit"
            ),
            Self::AggregateSizeLimitExceeded { maximum } => write!(
                formatter,
                "requested diagnostic reports exceeded the configured {maximum}-byte aggregate output limit"
            ),
            Self::RenderFailed => {
                formatter.write_str("requested diagnostic reports could not be rendered safely")
            }
        }
    }
}

struct BoundedOutput {
    output: Vec<u8>,
    maximum: usize,
    declared_maximum: u64,
    exceeded: bool,
}

impl BoundedOutput {
    fn for_report(report: &DiagnosticReport) -> Self {
        let declared_maximum = report.limits().values().report_bytes;
        Self {
            output: Vec::new(),
            maximum: usize::try_from(declared_maximum).unwrap_or(usize::MAX),
            declared_maximum,
            exceeded: false,
        }
    }

    fn push(&mut self, value: char) {
        self.write_char(value)
            .expect("the bounded report writer records limit failures");
    }

    fn push_str(&mut self, value: &str) {
        self.append(value.as_bytes());
    }

    fn finish(self) -> Result<String, ReportRenderError> {
        if self.exceeded {
            Err(ReportRenderError::SizeLimitExceeded {
                maximum: self.declared_maximum,
            })
        } else {
            Ok(String::from_utf8(self.output)
                .expect("typed report serialization must produce valid UTF-8"))
        }
    }

    fn append(&mut self, value: &[u8]) {
        if self.exceeded {
            return;
        }
        let Some(length) = self.output.len().checked_add(value.len()) else {
            self.exceeded = true;
            return;
        };
        if length > self.maximum {
            self.exceeded = true;
            return;
        }
        self.output.extend_from_slice(value);
    }
}

impl fmt::Write for BoundedOutput {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.append(value.as_bytes());
        Ok(())
    }
}

impl io::Write for BoundedOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.append(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum OverallOutcome {
    Passed,
    Failed,
    Incomplete,
}

impl OverallOutcome {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Incomplete => "incomplete",
        }
    }

    pub(super) const fn exit_status(self) -> ExitStatus {
        match self {
            Self::Passed => ExitStatus::Success,
            Self::Failed => ExitStatus::DiagnosticFailure,
            Self::Incomplete => ExitStatus::Incomplete,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum ExitStatus {
    Success = 0,
    DiagnosticFailure = 1,
    InvocationError = 2,
    Incomplete = 3,
    InternalError = 4,
}

impl ExitStatus {
    pub(crate) const fn code(self) -> u8 {
        self as u8
    }

    pub(crate) const fn meaning(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::DiagnosticFailure => "unsuccessful_result",
            Self::InvocationError => "invalid_invocation_or_input",
            Self::Incomplete => "incomplete_evidence",
            Self::InternalError => "internal_or_output_failure",
        }
    }
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        Self::from(status.code())
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
pub(super) struct SeverityCounts {
    info: usize,
    warning: usize,
    error: usize,
    critical: usize,
}

impl SeverityCounts {
    fn observe(&mut self, severity: Severity) {
        match severity {
            Severity::Info => self.info += 1,
            Severity::Warning => self.warning += 1,
            Severity::Error => self.error += 1,
            Severity::Critical => self.critical += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
pub(super) struct ReportSummary {
    checks: usize,
    required: usize,
    optional: usize,
    performed: usize,
    skipped: usize,
    passed: usize,
    warned: usize,
    incomplete: usize,
    failed: usize,
    required_skipped: usize,
    findings: SeverityCounts,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct DiagnosticReport {
    revision: SupportedRevision,
    negotiated_revision: Option<KnownRevision>,
    protocol_selection: Option<ProtocolSelectionEvidence>,
    limit_profile: DiagnosticLimitProfile,
    limits: DiagnosticLimits,
    checks: Vec<CheckResult>,
    primary_diagnosis: Option<Diagnosis>,
    independent_findings: Vec<FindingReference>,
    summary: ReportSummary,
    outcome: OverallOutcome,
    exit_status: ExitStatus,
}

impl DiagnosticReport {
    pub(super) fn new(
        revision: SupportedRevision,
        limits: DiagnosticLimits,
        mut checks: Vec<CheckResult>,
    ) -> Result<Self, ReportContractError> {
        if checks.is_empty() {
            return Err(ReportContractError::NoChecks);
        }

        checks.sort_by_key(CheckResult::id);
        for pair in checks.windows(2) {
            if pair[0].id() == pair[1].id() {
                return Err(ReportContractError::DuplicateCheck(pair[0].id()));
            }
        }

        for check in &checks {
            if let Some(finding) = check.findings().and_then(|findings| {
                findings
                    .iter()
                    .find(|finding| finding.revision() != revision)
            }) {
                return Err(ReportContractError::FindingRevisionMismatch {
                    check: check.id(),
                    finding: finding.revision(),
                    report: revision,
                });
            }
        }

        let finding_count = checks
            .iter()
            .filter_map(CheckResult::findings)
            .map(<[Finding]>::len)
            .sum::<usize>();
        let maximum_findings =
            usize::try_from(limits.values().report_findings).unwrap_or(usize::MAX);
        if finding_count > maximum_findings {
            return Err(ReportContractError::TooManyFindings {
                observed: finding_count,
                maximum: maximum_findings,
            });
        }

        let summary = summarize(&checks);
        let outcome = if summary.failed > 0 {
            OverallOutcome::Failed
        } else if summary.incomplete > 0
            || summary.required == 0
            || summary.performed == 0
            || summary.required_skipped > 0
        {
            OverallOutcome::Incomplete
        } else {
            OverallOutcome::Passed
        };
        let (primary_diagnosis, independent_findings) = classify_findings(&checks);

        for check in &checks {
            let Some(reason) = check.skip_reason() else {
                continue;
            };
            if !reason.is_causal() {
                continue;
            }
            let Some(diagnosis) = primary_diagnosis.as_ref() else {
                return Err(ReportContractError::CausalSkipWithoutDiagnosis(check.id()));
            };
            if check.id() <= diagnosis.check() {
                return Err(ReportContractError::CausalSkipPrecedesDiagnosis {
                    check: check.id(),
                    diagnosis: diagnosis.check(),
                });
            }
        }

        let exit_status = outcome.exit_status();
        Ok(Self {
            revision,
            negotiated_revision: None,
            protocol_selection: None,
            limit_profile: DiagnosticLimitProfile::Default,
            limits,
            checks,
            primary_diagnosis,
            independent_findings,
            summary,
            outcome,
            exit_status,
        })
    }

    pub(super) fn with_exit_status(mut self, exit_status: ExitStatus) -> Self {
        self.exit_status = exit_status;
        self
    }

    pub(super) fn with_negotiated_revision(mut self, revision: KnownRevision) -> Self {
        self.negotiated_revision = Some(revision);
        self
    }

    pub(super) fn with_protocol_selection(mut self, selection: ProtocolSelectionEvidence) -> Self {
        self.protocol_selection = Some(selection);
        self
    }

    pub(super) fn with_limit_profile(mut self, profile: DiagnosticLimitProfile) -> Self {
        self.limit_profile = profile;
        self.limits = profile.limits();
        self
    }

    pub(super) const fn revision(&self) -> SupportedRevision {
        self.revision
    }

    pub(super) const fn negotiated_revision(&self) -> Option<KnownRevision> {
        self.negotiated_revision
    }

    pub(super) const fn protocol_selection(&self) -> Option<ProtocolSelectionEvidence> {
        self.protocol_selection
    }

    pub(super) const fn limit_profile(&self) -> DiagnosticLimitProfile {
        self.limit_profile
    }

    pub(super) const fn limits(&self) -> DiagnosticLimits {
        self.limits
    }

    pub(super) fn checks(&self) -> &[CheckResult] {
        &self.checks
    }

    pub(super) fn primary_diagnosis(&self) -> Option<&Diagnosis> {
        self.primary_diagnosis.as_ref()
    }

    pub(super) fn independent_findings(&self) -> &[FindingReference] {
        &self.independent_findings
    }

    pub(super) const fn summary(&self) -> ReportSummary {
        self.summary
    }

    pub(super) const fn outcome(&self) -> OverallOutcome {
        self.outcome
    }

    pub(super) const fn exit_status(&self) -> ExitStatus {
        self.exit_status
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct FindingReference {
    check: CheckId,
    code: FindingCode,
    location: Location,
}

impl FindingReference {
    fn new(check: CheckId, finding: &Finding) -> Self {
        Self {
            check,
            code: finding.code(),
            location: finding.location().clone(),
        }
    }

    pub(super) const fn check(&self) -> CheckId {
        self.check
    }

    pub(super) const fn code(&self) -> FindingCode {
        self.code
    }

    pub(super) const fn location(&self) -> &Location {
        &self.location
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct Diagnosis {
    check: CheckId,
    findings: Vec<FindingReference>,
}

impl Diagnosis {
    pub(super) const fn check(&self) -> CheckId {
        self.check
    }

    pub(super) fn findings(&self) -> &[FindingReference] {
        &self.findings
    }
}

fn classify_findings(checks: &[CheckResult]) -> (Option<Diagnosis>, Vec<FindingReference>) {
    let independent_findings = checks
        .iter()
        .flat_map(|check| {
            check
                .findings()
                .unwrap_or_default()
                .iter()
                .filter(|finding| finding.is_independent_safety())
                .map(|finding| FindingReference::new(check.id(), finding))
        })
        .collect::<Vec<_>>();

    let primary = checks
        .iter()
        .find_map(|check| {
            let findings = check
                .findings()?
                .iter()
                .filter(|finding| {
                    finding.severity().is_failure()
                        && !finding.is_incomplete()
                        && !finding.is_independent_safety()
                })
                .map(|finding| FindingReference::new(check.id(), finding))
                .collect::<Vec<_>>();
            (!findings.is_empty()).then_some(Diagnosis {
                check: check.id(),
                findings,
            })
        })
        .or_else(|| {
            checks.iter().find_map(|check| {
                let findings = check
                    .findings()?
                    .iter()
                    .filter(|finding| finding.severity().is_failure() && !finding.is_incomplete())
                    .map(|finding| FindingReference::new(check.id(), finding))
                    .collect::<Vec<_>>();
                (!findings.is_empty()).then_some(Diagnosis {
                    check: check.id(),
                    findings,
                })
            })
        })
        .or_else(|| {
            checks.iter().find_map(|check| {
                let findings = check
                    .findings()?
                    .iter()
                    .filter(|finding| finding.is_incomplete())
                    .map(|finding| FindingReference::new(check.id(), finding))
                    .collect::<Vec<_>>();
                (!findings.is_empty()).then_some(Diagnosis {
                    check: check.id(),
                    findings,
                })
            })
        });

    (primary, independent_findings)
}

fn summarize(checks: &[CheckResult]) -> ReportSummary {
    let mut summary = ReportSummary {
        checks: checks.len(),
        ..ReportSummary::default()
    };

    for check in checks {
        match check.requirement() {
            Requirement::Required => summary.required += 1,
            Requirement::Optional => summary.optional += 1,
        }
        if let Some(findings) = check.findings() {
            summary.performed += 1;
            for finding in findings {
                summary.findings.observe(finding.severity());
            }
            match check.outcome().expect("performed checks have an outcome") {
                CheckOutcome::Passed => summary.passed += 1,
                CheckOutcome::Warning => summary.warned += 1,
                CheckOutcome::Incomplete => summary.incomplete += 1,
                CheckOutcome::Failed => summary.failed += 1,
            }
        } else {
            summary.skipped += 1;
            if check.requirement() == Requirement::Required {
                summary.required_skipped += 1;
            }
        }
    }

    summary
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ReportContractError {
    NoChecks,
    DuplicateCheck(CheckId),
    TooManyFindings {
        observed: usize,
        maximum: usize,
    },
    FindingRevisionMismatch {
        check: CheckId,
        finding: SupportedRevision,
        report: SupportedRevision,
    },
    CausalSkipWithoutDiagnosis(CheckId),
    CausalSkipPrecedesDiagnosis {
        check: CheckId,
        diagnosis: CheckId,
    },
}

impl fmt::Display for ReportContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoChecks => formatter.write_str("a diagnostic report must contain a check"),
            Self::DuplicateCheck(check) => write!(formatter, "duplicate check: {check}"),
            Self::TooManyFindings { observed, maximum } => write!(
                formatter,
                "report contains {observed} findings; maximum is {maximum}"
            ),
            Self::FindingRevisionMismatch {
                check,
                finding,
                report,
            } => write!(
                formatter,
                "finding revision {finding} for {check} does not match report revision {report}"
            ),
            Self::CausalSkipWithoutDiagnosis(check) => write!(
                formatter,
                "causally skipped check {check} has no failing diagnosis"
            ),
            Self::CausalSkipPrecedesDiagnosis { check, diagnosis } => write!(
                formatter,
                "causally skipped check {check} does not follow diagnosis {diagnosis}"
            ),
        }
    }
}

pub(super) fn render_report(
    report: &DiagnosticReport,
    format: ReportFormat,
) -> Result<String, ReportRenderError> {
    match format {
        ReportFormat::Human => HumanReporter::try_render(report),
        ReportFormat::Json => JsonReporter::render(report),
        ReportFormat::Junit => JunitReporter::render(report),
        ReportFormat::Markdown => MarkdownReporter::render(report),
        ReportFormat::Badge => BadgeReporter::render(report),
    }
}

pub(super) fn render_reports(
    report: &DiagnosticReport,
    request: ReportRequest,
) -> Result<RenderedReports, ReportRenderError> {
    render_reports_with_limit(report, request, AGGREGATE_REPORT_BYTES)
}

fn render_reports_with_limit(
    report: &DiagnosticReport,
    request: ReportRequest,
    aggregate_maximum: u64,
) -> Result<RenderedReports, ReportRenderError> {
    if internal_test_render_failure() {
        return Err(ReportRenderError::RenderFailed);
    }

    let stdout = render_report(report, request.stdout)?;
    let mut aggregate_bytes = u64::try_from(stdout.len()).unwrap_or(u64::MAX);
    if aggregate_bytes > aggregate_maximum {
        return Err(ReportRenderError::AggregateSizeLimitExceeded {
            maximum: aggregate_maximum,
        });
    }
    let mut artifacts = Vec::with_capacity(4);
    for (requested, format) in [
        (request.json_artifact, ReportArtifactFormat::Json),
        (request.junit_artifact, ReportArtifactFormat::Junit),
        (request.markdown_artifact, ReportArtifactFormat::Markdown),
        (request.badge_artifact, ReportArtifactFormat::Badge),
    ] {
        if !requested {
            continue;
        }
        let output = render_report(report, format.report_format())?;
        aggregate_bytes = aggregate_bytes
            .checked_add(u64::try_from(output.len()).unwrap_or(u64::MAX))
            .ok_or(ReportRenderError::AggregateSizeLimitExceeded {
                maximum: aggregate_maximum,
            })?;
        if aggregate_bytes > aggregate_maximum {
            return Err(ReportRenderError::AggregateSizeLimitExceeded {
                maximum: aggregate_maximum,
            });
        }
        artifacts.push(RenderedReportArtifact { format, output });
    }

    Ok(RenderedReports { stdout, artifacts })
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_render_failure() -> bool {
    std::env::var_os("MCP_DOCTOR_TEST_MODE").as_deref() == Some(std::ffi::OsStr::new("1"))
        && std::env::var_os("MCP_DOCTOR_INTERNAL_TEST_REPORT_RENDER_FAILURE").as_deref()
            == Some(std::ffi::OsStr::new("1"))
}

#[cfg(not(feature = "internal-test-fixtures"))]
const fn internal_test_render_failure() -> bool {
    false
}

pub(super) struct HumanReporter;

impl HumanReporter {
    pub(super) fn render(report: &DiagnosticReport) -> String {
        Self::try_render(report).expect("the synthetic human report must fit its output limit")
    }

    fn try_render(report: &DiagnosticReport) -> Result<String, ReportRenderError> {
        let mut output = BoundedOutput::for_report(report);
        writeln!(
            output,
            "mcp-doctor report · MCP {} · {REPORT_SCHEMA_VERSION}",
            report.revision()
        )
        .expect("the bounded report writer records limit failures");
        if let Some(negotiated) = report.negotiated_revision() {
            writeln!(
                output,
                "protocol selection · selected {} · negotiated {}",
                report.revision(),
                negotiated.as_str()
            )
            .expect("the bounded report writer records limit failures");
        }
        if let Some(selection) = report.protocol_selection() {
            write!(
                output,
                "protocol negotiation · mode={} · path={}",
                selection.mode().as_str(),
                selection.path().as_str()
            )
            .expect("the bounded report writer records limit failures");
            if let Some(selected) = selection.selected_revision() {
                write!(output, " · selected={selected}")
                    .expect("the bounded report writer records limit failures");
            } else {
                output.push_str(" · selected=none");
            }
            writeln!(
                output,
                " · process_launches={} · lifecycle_requests={} · lifecycle_notifications={} · fallbacks={}",
                selection.process_launches(),
                selection.lifecycle_requests(),
                selection.lifecycle_notifications(),
                selection.fallbacks()
            )
            .expect("the bounded report writer records limit failures");
        }
        output.push('\n');

        write_human_diagnosis(&mut output, report);
        output.push('\n');
        write_human_limits(
            &mut output,
            report.limit_profile(),
            report.limits().values(),
        );
        output.push('\n');

        for check in report.checks() {
            if let Some(outcome) = check.outcome() {
                writeln!(
                    output,
                    "{:<5} {:<22} {}",
                    outcome.human_label(),
                    check.id().as_str(),
                    check.requirement().as_str()
                )
                .expect("the bounded report writer records limit failures");
                write_human_reproduction(&mut output, check.reproduction());

                for finding in check.findings().expect("performed check findings exist") {
                    writeln!(
                        output,
                        "      {}  {}",
                        finding.code().as_str(),
                        finding.severity().as_str()
                    )
                    .expect("the bounded report writer records limit failures");
                    writeln!(output, "      Where: {}", finding.location())
                        .expect("the bounded report writer records limit failures");
                    writeln!(output, "      What: {}", finding.code().title())
                        .expect("the bounded report writer records limit failures");
                    writeln!(output, "      Why: {}", finding.impact())
                        .expect("the bounded report writer records limit failures");
                    write_human_evidence(&mut output, finding);
                    writeln!(output, "      Expected: {}", finding.expectation())
                        .expect("the bounded report writer records limit failures");
                    writeln!(output, "      Fix: {}", finding.remediation())
                        .expect("the bounded report writer records limit failures");
                    writeln!(output, "      Reference: {}", finding.reference())
                        .expect("the bounded report writer records limit failures");
                }
            } else {
                let reason = check.skip_reason().expect("skipped check has a reason");
                writeln!(
                    output,
                    "SKIP  {:<22} {} · {}{}",
                    check.id().as_str(),
                    check.requirement().as_str(),
                    reason.description(),
                    human_blocked_by(report, reason)
                )
                .expect("the bounded report writer records limit failures");
                write_human_reproduction(&mut output, check.reproduction());
            }
        }

        let summary = report.summary();
        output.push('\n');
        writeln!(
            output,
            "{} failed · {} incomplete · {} warned · {} passed · {} skipped · outcome {} · exit {}",
            summary.failed,
            summary.incomplete,
            summary.warned,
            summary.passed,
            summary.skipped,
            report.outcome().as_str(),
            report.exit_status().code()
        )
        .expect("the bounded report writer records limit failures");
        output.finish()
    }
}

fn write_human_limits(
    output: &mut BoundedOutput,
    profile: DiagnosticLimitProfile,
    values: LimitValues,
) {
    writeln!(output, "LIMITS · profile={}", profile.as_str())
        .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  time · startup_ms={} · discovery_ms={} · request_ms={} · response_ms={} · shutdown_grace_ms={} · total_ms={}",
        values.startup_ms,
        values.discovery_ms,
        values.request_ms,
        values.response_ms,
        values.shutdown_grace_ms,
        values.total_ms
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  io · message_bytes={} · stdout_bytes={} · stderr_bytes={} · aggregate_output_bytes={} · message_count={}",
        values.message_bytes,
        values.stdout_bytes,
        values.stderr_bytes,
        values.aggregate_output_bytes,
        values.message_count
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  network · endpoint_bytes={} · resolution_addresses={} · resolution_count={} · trust_bytes={} · trust_certificates={}",
        values.endpoint_bytes,
        values.resolution_addresses,
        values.resolution_count,
        values.trust_bytes,
        values.trust_certificates
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  request · request_fields={} · request_field_name_bytes={} · request_field_value_bytes={} · request_fields_bytes={}",
        values.request_fields,
        values.request_field_name_bytes,
        values.request_field_value_bytes,
        values.request_fields_bytes
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  response · response_fields={} · response_field_name_bytes={} · response_field_value_bytes={} · response_fields_bytes={}",
        values.response_fields,
        values.response_field_name_bytes,
        values.response_field_value_bytes,
        values.response_fields_bytes
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  discovery · protocol_revisions={} · catalog_items={} · report_findings={} · report_bytes={}",
        values.protocol_revisions,
        values.catalog_items,
        values.report_findings,
        values.report_bytes
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  schema · schema_bytes={} · instance_bytes={} · schema_nodes={} · schema_depth={} · schema_ref_depth={} · schema_evaluation_steps={} · validation_errors={}",
        values.schema_bytes,
        values.instance_bytes,
        values.schema_nodes,
        values.schema_depth,
        values.schema_ref_depth,
        values.schema_evaluation_steps,
        values.validation_errors
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  generation · active_cases={} · generation_attempts={} · generation_candidates={} · generation_steps={}",
        values.active_cases,
        values.generation_attempts,
        values.generation_candidates,
        values.generation_steps
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "  activity · redirects={} · retries={} · concurrency={}",
        values.redirects, values.retries, values.concurrency
    )
    .expect("the bounded report writer records limit failures");
}

fn write_human_reproduction(
    output: &mut BoundedOutput,
    reproduction: Option<&GeneratedCaseReproduction>,
) {
    let Some(reproduction) = reproduction else {
        return;
    };
    let input = reproduction.input();
    if let Some(mutation_kind) = reproduction.mutation_kind() {
        writeln!(
            output,
            "      Reproduce: {} · seed={} · mutation={} · input={} bytes={} nodes={} depth={} · null={} boolean={} number={} string={} array={} array_items={} object={} object_members={}",
            reproduction.generator(),
            reproduction.seed(),
            mutation_kind,
            input.root().as_str(),
            input.byte_count(),
            input.node_count(),
            input.maximum_depth(),
            input.nulls(),
            input.booleans(),
            input.numbers(),
            input.strings(),
            input.arrays(),
            input.array_items(),
            input.objects(),
            input.object_members(),
        )
        .expect("the bounded report writer records limit failures");
        return;
    }
    writeln!(
        output,
        "      Reproduce: {} · seed={} · input={} bytes={} nodes={} depth={} · null={} boolean={} number={} string={} array={} array_items={} object={} object_members={}",
        reproduction.generator(),
        reproduction.seed(),
        input.root().as_str(),
        input.byte_count(),
        input.node_count(),
        input.maximum_depth(),
        input.nulls(),
        input.booleans(),
        input.numbers(),
        input.strings(),
        input.arrays(),
        input.array_items(),
        input.objects(),
        input.object_members(),
    )
    .expect("the bounded report writer records limit failures");
}

fn write_human_diagnosis(output: &mut BoundedOutput, report: &DiagnosticReport) {
    if let Some(diagnosis) = report.primary_diagnosis() {
        writeln!(output, "PRIMARY DIAGNOSIS · {}", diagnosis.check())
            .expect("the bounded report writer records limit failures");
        for finding in diagnosis.findings() {
            writeln!(
                output,
                "  {} · {}",
                finding.code().as_str(),
                finding.location()
            )
            .expect("the bounded report writer records limit failures");
        }
    } else {
        writeln!(output, "PRIMARY DIAGNOSIS · none")
            .expect("the bounded report writer records limit failures");
    }

    if !report.independent_findings().is_empty() {
        writeln!(
            output,
            "INDEPENDENT SAFETY FINDINGS · {}",
            report.independent_findings().len()
        )
        .expect("the bounded report writer records limit failures");
        for finding in report.independent_findings() {
            writeln!(
                output,
                "  {} · {} · {}",
                finding.code().as_str(),
                finding.check(),
                finding.location()
            )
            .expect("the bounded report writer records limit failures");
        }
    }
}

fn human_blocked_by(report: &DiagnosticReport, reason: super::model::SkipReason) -> String {
    if !reason.is_causal() {
        return String::new();
    }
    let diagnosis = report
        .primary_diagnosis()
        .expect("the report contract requires a diagnosis for a causal skip");
    let mut rendered = format!(" · blocked by {} (", diagnosis.check());
    for (index, finding) in diagnosis.findings().iter().enumerate() {
        if index > 0 {
            rendered.push_str(", ");
        }
        write!(
            rendered,
            "{} at {}",
            finding.code().as_str(),
            finding.location()
        )
        .expect("writing to a String cannot fail");
    }
    rendered.push(')');
    rendered
}

fn write_human_evidence(output: &mut BoundedOutput, finding: &Finding) {
    match finding.evidence() {
        FindingEvidence::None => {}
        FindingEvidence::RevisionAdvertisement(summary) => {
            writeln!(
                output,
                "      required {} · {} offered · {} recognized legacy · {} unknown date · {} opaque",
                finding.revision(),
                summary.offered(),
                summary.recognized_legacy(),
                summary.unknown_date(),
                summary.opaque()
            )
            .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::RedactedObservation(observation) => {
            writeln!(output, "      observed {observation}")
                .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::LimitViolation(violation) => {
            writeln!(
                output,
                "      {} observed {} {}; maximum {} {}",
                violation.kind().as_str(),
                violation.observed(),
                violation.kind().unit().as_str(),
                violation.maximum(),
                violation.kind().unit().as_str()
            )
            .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::SchemaValidationLimit { phase, violation } => {
            writeln!(
                output,
                "      phase {} · {} observed {} {}; maximum {} {}",
                phase.as_str(),
                violation.kind().as_str(),
                violation.observed(),
                violation.kind().unit().as_str(),
                violation.maximum(),
                violation.kind().unit().as_str()
            )
            .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::CredentialLiteral {
            keyword,
            literal_count,
        } => {
            writeln!(
                output,
                "      keyword {} · {literal_count} non-empty string literal(s)",
                keyword.as_str()
            )
            .expect("the bounded report writer records credential-literal evidence");
        }
        FindingEvidence::RuleViolation(violation) => {
            write_human_rule(output, *violation);
        }
        FindingEvidence::JsonRpcError(error) => {
            write!(output, "      json_rpc_error {}", error.as_str())
                .expect("the bounded report writer records limit failures");
            if let Some(code) = error.code() {
                write!(output, " · code {code}")
                    .expect("the bounded report writer records limit failures");
            }
            output.push('\n');
        }
    }
}

fn write_human_rule(output: &mut BoundedOutput, violation: RuleViolation) {
    write!(output, "      rule {}", violation.as_str())
        .expect("the bounded report writer records limit failures");
    if let Some(expected) = violation.expected_shape() {
        write!(output, " · expected {}", expected.as_str())
            .expect("the bounded report writer records limit failures");
    }
    if let Some(observed) = violation.observed() {
        write!(output, " · observed {}", observed.as_str())
            .expect("the bounded report writer records limit failures");
    }
    if let Some(error_count) = violation.error_count() {
        write!(output, " · {error_count} validation error(s)")
            .expect("the bounded report writer records limit failures");
    }
    if let Some(status) = violation.http_status() {
        write!(output, " · HTTP status {status}")
            .expect("the bounded report writer records limit failures");
    }
    if let Some(first_matching_tool_index) = violation.first_matching_tool_index() {
        write!(
            output,
            " · first_matching_tool_index {first_matching_tool_index}"
        )
        .expect("the bounded report writer records tool-description relationship evidence");
    }
    output.push('\n');
}

pub(super) struct BadgeReporter;

impl BadgeReporter {
    fn render(report: &DiagnosticReport) -> Result<String, ReportRenderError> {
        let badge = BadgeReport::from_outcome(report.outcome());
        let mut output = BoundedOutput::for_report(report);
        serde_json::to_writer_pretty(&mut output, &badge)
            .expect("the fixed badge report must serialize as JSON");
        output.push('\n');
        output.finish()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
struct BadgeReport {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    label: BadgeLabel,
    message: BadgeMessage,
    color: BadgeColor,
}

impl BadgeReport {
    const fn from_outcome(outcome: OverallOutcome) -> Self {
        let message = match outcome {
            OverallOutcome::Passed => BadgeMessage::Pass,
            OverallOutcome::Failed => BadgeMessage::Fail,
            OverallOutcome::Incomplete => BadgeMessage::Incomplete,
        };
        Self {
            schema_version: 1,
            label: BadgeLabel::McpDoctor,
            message,
            color: message.color(),
        }
    }
}

impl<'de> Deserialize<'de> for BadgeReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = BadgeReportWire::deserialize(deserializer)?;
        if wire.color != wire.message.color() {
            return Err(de::Error::custom(
                "badge message and color do not match the fixed contract",
            ));
        }
        Ok(Self {
            schema_version: wire.schema_version,
            label: wire.label,
            message: wire.message,
            color: wire.color,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BadgeReportWire {
    #[serde(
        rename = "schemaVersion",
        deserialize_with = "deserialize_badge_schema_version"
    )]
    schema_version: u8,
    label: BadgeLabel,
    message: BadgeMessage,
    color: BadgeColor,
}

fn deserialize_badge_schema_version<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let version = u8::deserialize(deserializer)?;
    if version == 1 {
        Ok(version)
    } else {
        Err(de::Error::custom("unsupported badge schema version"))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
enum BadgeLabel {
    #[serde(rename = "mcp-doctor")]
    McpDoctor,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum BadgeMessage {
    Pass,
    Fail,
    Incomplete,
}

impl BadgeMessage {
    const fn color(self) -> BadgeColor {
        match self {
            Self::Pass => BadgeColor::BrightGreen,
            Self::Fail => BadgeColor::Red,
            Self::Incomplete => BadgeColor::LightGrey,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
enum BadgeColor {
    #[serde(rename = "brightgreen")]
    BrightGreen,
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "lightgrey")]
    LightGrey,
}

pub(super) struct MarkdownReporter;

impl MarkdownReporter {
    fn render(report: &DiagnosticReport) -> Result<String, ReportRenderError> {
        let mut output = BoundedOutput::for_report(report);
        writeln!(output, "<!-- {MARKDOWN_REPORT_VERSION} -->")
            .expect("the bounded report writer records limit failures");
        writeln!(output, "# mcp-doctor diagnostic report")
            .expect("the bounded report writer records limit failures");
        output.push('\n');
        writeln!(output, "| Field | Value |")
            .expect("the bounded report writer records limit failures");
        writeln!(output, "| --- | --- |")
            .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "| Product | `mcp-doctor {}` |",
            env!("CARGO_PKG_VERSION")
        )
        .expect("the bounded report writer records limit failures");
        writeln!(output, "| Report contract | `{REPORT_SCHEMA_VERSION}` |")
            .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "| Markdown contract | `{MARKDOWN_REPORT_VERSION}` |"
        )
        .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "| Selected protocol revision | `{}` |",
            report.revision()
        )
        .expect("the bounded report writer records limit failures");
        match report.negotiated_revision() {
            Some(revision) => writeln!(
                output,
                "| Negotiated protocol revision | `{}` |",
                revision.as_str()
            ),
            None => writeln!(output, "| Negotiated protocol revision | Not present |"),
        }
        .expect("the bounded report writer records limit failures");
        writeln!(output, "| Outcome | `{}` |", report.outcome().as_str())
            .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "| Exit | `{}` (`{}`) |",
            report.exit_status().code(),
            report.exit_status().meaning()
        )
        .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "| Limit profile | `{}` |",
            report.limit_profile().as_str()
        )
        .expect("the bounded report writer records limit failures");

        output.push('\n');
        write_markdown_summary(&mut output, report.summary());
        output.push('\n');
        write_markdown_protocol_selection(&mut output, report);
        output.push('\n');
        write_markdown_diagnosis(&mut output, report);
        output.push('\n');
        write_markdown_causal_skips(&mut output, report);
        output.push('\n');
        write_markdown_limits(
            &mut output,
            report.limit_profile(),
            report.limits().values(),
        );
        output.push('\n');
        write_markdown_checks(&mut output, report);
        output.finish()
    }
}

fn write_markdown_summary(output: &mut BoundedOutput, summary: ReportSummary) {
    writeln!(output, "## Summary").expect("the bounded report writer records limit failures");
    output.push('\n');
    writeln!(output, "| Measure | Count |")
        .expect("the bounded report writer records limit failures");
    writeln!(output, "| --- | ---: |").expect("the bounded report writer records limit failures");
    for (name, count) in [
        ("Checks", summary.checks),
        ("Required", summary.required),
        ("Optional", summary.optional),
        ("Performed", summary.performed),
        ("Skipped", summary.skipped),
        ("Passed", summary.passed),
        ("Warned", summary.warned),
        ("Incomplete", summary.incomplete),
        ("Failed", summary.failed),
        ("Required skipped", summary.required_skipped),
        ("Info findings", summary.findings.info),
        ("Warning findings", summary.findings.warning),
        ("Error findings", summary.findings.error),
        ("Critical findings", summary.findings.critical),
    ] {
        writeln!(output, "| {name} | {count} |")
            .expect("the bounded report writer records limit failures");
    }
}

fn write_markdown_protocol_selection(output: &mut BoundedOutput, report: &DiagnosticReport) {
    writeln!(output, "## Protocol selection")
        .expect("the bounded report writer records limit failures");
    output.push('\n');
    let Some(selection) = report.protocol_selection() else {
        writeln!(output, "No passive selection evidence is present.")
            .expect("the bounded report writer records limit failures");
        return;
    };
    writeln!(output, "- Mode: `{}`", selection.mode().as_str())
        .expect("the bounded report writer records limit failures");
    writeln!(output, "- Path: `{}`", selection.path().as_str())
        .expect("the bounded report writer records limit failures");
    match selection.selected_revision() {
        Some(revision) => writeln!(output, "- Selected revision: `{revision}`"),
        None => writeln!(output, "- Selected revision: Not established"),
    }
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Bounded work: `process_launches={}`, `lifecycle_requests={}`, `lifecycle_notifications={}`, `fallbacks={}`",
        selection.process_launches(),
        selection.lifecycle_requests(),
        selection.lifecycle_notifications(),
        selection.fallbacks()
    )
    .expect("the bounded report writer records limit failures");
}

fn report_finding<'a>(report: &'a DiagnosticReport, reference: &FindingReference) -> &'a Finding {
    report
        .checks()
        .iter()
        .find(|check| check.id() == reference.check())
        .and_then(CheckResult::findings)
        .and_then(|findings| {
            findings.iter().find(|finding| {
                finding.code() == reference.code() && finding.location() == reference.location()
            })
        })
        .expect("a report finding reference must resolve within the same immutable report")
}

fn write_markdown_diagnosis(output: &mut BoundedOutput, report: &DiagnosticReport) {
    writeln!(output, "## Primary diagnosis")
        .expect("the bounded report writer records limit failures");
    output.push('\n');
    if let Some(diagnosis) = report.primary_diagnosis() {
        writeln!(output, "- Check: `{}`", diagnosis.check())
            .expect("the bounded report writer records limit failures");
        for reference in diagnosis.findings() {
            let finding = report_finding(report, reference);
            writeln!(
                output,
                "- `{}` at `{}`: {}",
                reference.code().as_str(),
                reference.location(),
                finding.remediation()
            )
            .expect("the bounded report writer records limit failures");
        }
    } else {
        writeln!(output, "None.").expect("the bounded report writer records limit failures");
    }

    output.push('\n');
    writeln!(output, "## Independent safety findings")
        .expect("the bounded report writer records limit failures");
    output.push('\n');
    if report.independent_findings().is_empty() {
        writeln!(output, "None.").expect("the bounded report writer records limit failures");
        return;
    }
    for reference in report.independent_findings() {
        let finding = report_finding(report, reference);
        writeln!(
            output,
            "- `{}` in `{}` at `{}`: {}",
            reference.code().as_str(),
            reference.check(),
            reference.location(),
            finding.remediation()
        )
        .expect("the bounded report writer records limit failures");
    }
}

fn write_markdown_causal_skips(output: &mut BoundedOutput, report: &DiagnosticReport) {
    writeln!(output, "## Causal skips").expect("the bounded report writer records limit failures");
    output.push('\n');
    let mut wrote_skip = false;
    for check in report.checks() {
        let Some(reason) = check.skip_reason().filter(|reason| reason.is_causal()) else {
            continue;
        };
        wrote_skip = true;
        write!(
            output,
            "- `{}` (`{}`) was skipped: {}. Blocked by",
            check.id(),
            check.requirement().as_str(),
            reason.description()
        )
        .expect("the bounded report writer records limit failures");
        let diagnosis = report
            .primary_diagnosis()
            .expect("the report contract requires a diagnosis for a causal skip");
        write!(output, " `{}` (", diagnosis.check())
            .expect("the bounded report writer records limit failures");
        for (index, finding) in diagnosis.findings().iter().enumerate() {
            if index > 0 {
                output.push_str(", ");
            }
            write!(
                output,
                "`{}` at `{}`",
                finding.code().as_str(),
                finding.location()
            )
            .expect("the bounded report writer records limit failures");
        }
        writeln!(output, ").").expect("the bounded report writer records limit failures");
    }
    if !wrote_skip {
        writeln!(output, "None.").expect("the bounded report writer records limit failures");
    }
}

fn write_markdown_limits(
    output: &mut BoundedOutput,
    profile: DiagnosticLimitProfile,
    values: LimitValues,
) {
    writeln!(output, "## Effective limits")
        .expect("the bounded report writer records limit failures");
    output.push('\n');
    writeln!(output, "- Profile: `{}`", profile.as_str())
        .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Time: `startup_ms={}`, `discovery_ms={}`, `request_ms={}`, `response_ms={}`, `shutdown_grace_ms={}`, `total_ms={}`",
        values.startup_ms,
        values.discovery_ms,
        values.request_ms,
        values.response_ms,
        values.shutdown_grace_ms,
        values.total_ms
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- I/O: `message_bytes={}`, `stdout_bytes={}`, `stderr_bytes={}`, `aggregate_output_bytes={}`, `message_count={}`",
        values.message_bytes,
        values.stdout_bytes,
        values.stderr_bytes,
        values.aggregate_output_bytes,
        values.message_count
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Network: `endpoint_bytes={}`, `resolution_addresses={}`, `resolution_count={}`, `trust_bytes={}`, `trust_certificates={}`",
        values.endpoint_bytes,
        values.resolution_addresses,
        values.resolution_count,
        values.trust_bytes,
        values.trust_certificates
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Request fields: `request_fields={}`, `request_field_name_bytes={}`, `request_field_value_bytes={}`, `request_fields_bytes={}`",
        values.request_fields,
        values.request_field_name_bytes,
        values.request_field_value_bytes,
        values.request_fields_bytes
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Response fields: `response_fields={}`, `response_field_name_bytes={}`, `response_field_value_bytes={}`, `response_fields_bytes={}`",
        values.response_fields,
        values.response_field_name_bytes,
        values.response_field_value_bytes,
        values.response_fields_bytes
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Discovery: `protocol_revisions={}`, `catalog_items={}`, `report_findings={}`, `report_bytes={}`",
        values.protocol_revisions,
        values.catalog_items,
        values.report_findings,
        values.report_bytes
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Schema: `schema_bytes={}`, `instance_bytes={}`, `schema_nodes={}`, `schema_depth={}`, `schema_ref_depth={}`, `schema_evaluation_steps={}`, `validation_errors={}`",
        values.schema_bytes,
        values.instance_bytes,
        values.schema_nodes,
        values.schema_depth,
        values.schema_ref_depth,
        values.schema_evaluation_steps,
        values.validation_errors
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Generation: `active_cases={}`, `generation_attempts={}`, `generation_candidates={}`, `generation_steps={}`",
        values.active_cases,
        values.generation_attempts,
        values.generation_candidates,
        values.generation_steps
    )
    .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Activity: `redirects={}`, `retries={}`, `concurrency={}`",
        values.redirects, values.retries, values.concurrency
    )
    .expect("the bounded report writer records limit failures");
}

fn write_markdown_checks(output: &mut BoundedOutput, report: &DiagnosticReport) {
    writeln!(output, "## Checks").expect("the bounded report writer records limit failures");
    for (check_index, check) in report.checks().iter().enumerate() {
        output.push('\n');
        writeln!(output, "### {}. `{}`", check_index + 1, check.id())
            .expect("the bounded report writer records limit failures");
        output.push('\n');
        writeln!(output, "- Requirement: `{}`", check.requirement().as_str())
            .expect("the bounded report writer records limit failures");
        if let Some(outcome) = check.outcome() {
            writeln!(output, "- State: `performed`")
                .expect("the bounded report writer records limit failures");
            writeln!(output, "- Outcome: `{}`", outcome.as_str())
                .expect("the bounded report writer records limit failures");
            write_markdown_reproduction(output, check.reproduction());
            let findings = check.findings().expect("performed check findings exist");
            if findings.is_empty() {
                writeln!(output, "- Findings: None.")
                    .expect("the bounded report writer records limit failures");
            } else {
                for (finding_index, finding) in findings.iter().enumerate() {
                    write_markdown_finding(output, report, check, finding, finding_index + 1);
                }
            }
        } else {
            let reason = check.skip_reason().expect("skipped check has a reason");
            writeln!(output, "- State: `skipped`")
                .expect("the bounded report writer records limit failures");
            writeln!(output, "- Skip reason: `{}`", reason.as_str())
                .expect("the bounded report writer records limit failures");
            writeln!(output, "- Explanation: {}", reason.description())
                .expect("the bounded report writer records limit failures");
            if reason.is_causal() {
                let diagnosis = report
                    .primary_diagnosis()
                    .expect("the report contract requires a diagnosis for a causal skip");
                writeln!(output, "- Blocked by check: `{}`", diagnosis.check())
                    .expect("the bounded report writer records limit failures");
            }
            write_markdown_reproduction(output, check.reproduction());
        }
    }
}

fn write_markdown_reproduction(
    output: &mut BoundedOutput,
    reproduction: Option<&GeneratedCaseReproduction>,
) {
    let Some(reproduction) = reproduction else {
        return;
    };
    let input = reproduction.input();
    write!(
        output,
        "- Reproduction: `generator={}`, `seed={}`",
        reproduction.generator(),
        reproduction.seed()
    )
    .expect("the bounded report writer records limit failures");
    if let Some(mutation_kind) = reproduction.mutation_kind() {
        write!(output, ", `mutation={mutation_kind}`")
            .expect("the bounded report writer records limit failures");
    }
    writeln!(
        output,
        ", `root={}`, `bytes={}`, `nodes={}`, `depth={}`, `null={}`, `boolean={}`, `number={}`, `string={}`, `array={}`, `array_items={}`, `object={}`, `object_members={}`",
        input.root().as_str(),
        input.byte_count(),
        input.node_count(),
        input.maximum_depth(),
        input.nulls(),
        input.booleans(),
        input.numbers(),
        input.strings(),
        input.arrays(),
        input.array_items(),
        input.objects(),
        input.object_members()
    )
    .expect("the bounded report writer records limit failures");
}

fn write_markdown_finding(
    output: &mut BoundedOutput,
    report: &DiagnosticReport,
    check: &CheckResult,
    finding: &Finding,
    finding_index: usize,
) {
    output.push('\n');
    writeln!(
        output,
        "#### Finding {finding_index}: `{}`",
        finding.code().as_str()
    )
    .expect("the bounded report writer records limit failures");
    output.push('\n');
    writeln!(output, "- Severity: `{}`", finding.severity().as_str())
        .expect("the bounded report writer records limit failures");
    writeln!(output, "- Protocol revision: `{}`", finding.revision())
        .expect("the bounded report writer records limit failures");
    writeln!(output, "- Location: `{}`", finding.location())
        .expect("the bounded report writer records limit failures");
    writeln!(output, "- What: {}", finding.code().title())
        .expect("the bounded report writer records limit failures");
    writeln!(output, "- Why: {}", finding.impact())
        .expect("the bounded report writer records limit failures");
    write_markdown_evidence(output, finding);
    writeln!(output, "- Expected: {}", finding.expectation())
        .expect("the bounded report writer records limit failures");
    writeln!(output, "- Corrective action: {}", finding.remediation())
        .expect("the bounded report writer records limit failures");
    writeln!(output, "- Reference: {}", finding.reference())
        .expect("the bounded report writer records limit failures");
    let primary = report.primary_diagnosis().is_some_and(|diagnosis| {
        diagnosis.check() == check.id()
            && diagnosis.findings().iter().any(|reference| {
                reference.code() == finding.code() && reference.location() == finding.location()
            })
    });
    writeln!(output, "- Primary diagnosis: `{primary}`")
        .expect("the bounded report writer records limit failures");
    writeln!(
        output,
        "- Independent safety finding: `{}`",
        finding.is_independent_safety()
    )
    .expect("the bounded report writer records limit failures");
}

fn write_markdown_evidence(output: &mut BoundedOutput, finding: &Finding) {
    match finding.evidence() {
        FindingEvidence::None => {
            writeln!(output, "- Evidence: None.")
                .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::RevisionAdvertisement(summary) => {
            writeln!(
                output,
                "- Evidence: `required_revision={}`, `offered={}`, `recognized_legacy={}`, `unknown_date={}`, `opaque={}`",
                finding.revision(),
                summary.offered(),
                summary.recognized_legacy(),
                summary.unknown_date(),
                summary.opaque()
            )
            .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::RedactedObservation(observation) => {
            writeln!(output, "- Evidence: observed {observation}")
                .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::LimitViolation(violation) => {
            writeln!(
                output,
                "- Evidence: `limit={}`, `unit={}`, `observed={}`, `maximum={}`",
                violation.kind().as_str(),
                violation.kind().unit().as_str(),
                violation.observed(),
                violation.maximum()
            )
            .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::SchemaValidationLimit { phase, violation } => {
            writeln!(
                output,
                "- Evidence: `phase={}`, `limit={}`, `unit={}`, `observed={}`, `maximum={}`",
                phase.as_str(),
                violation.kind().as_str(),
                violation.kind().unit().as_str(),
                violation.observed(),
                violation.maximum()
            )
            .expect("the bounded report writer records limit failures");
        }
        FindingEvidence::CredentialLiteral {
            keyword,
            literal_count,
        } => {
            writeln!(
                output,
                "- Evidence: `keyword={}`, `literal_count={literal_count}`",
                keyword.as_str()
            )
            .expect("the bounded report writer records credential-literal evidence");
        }
        FindingEvidence::RuleViolation(violation) => {
            write!(output, "- Evidence: `rule={}`", violation.as_str())
                .expect("the bounded report writer records limit failures");
            if let Some(expected) = violation.expected_shape() {
                write!(output, ", `expected={}`", expected.as_str())
                    .expect("the bounded report writer records limit failures");
            }
            if let Some(observed) = violation.observed() {
                write!(output, ", `observed={}`", observed.as_str())
                    .expect("the bounded report writer records limit failures");
            }
            if let Some(error_count) = violation.error_count() {
                write!(output, ", `validation_errors={error_count}`")
                    .expect("the bounded report writer records limit failures");
            }
            if let Some(status) = violation.http_status() {
                write!(output, ", `http_status={status}`")
                    .expect("the bounded report writer records limit failures");
            }
            if let Some(first_matching_tool_index) = violation.first_matching_tool_index() {
                write!(
                    output,
                    ", `first_matching_tool_index={first_matching_tool_index}`"
                )
                .expect("the bounded report writer records tool-description relationship evidence");
            }
            output.push('\n');
        }
        FindingEvidence::JsonRpcError(error) => {
            write!(output, "- Evidence: `json_rpc_error={}`", error.as_str())
                .expect("the bounded report writer records limit failures");
            if let Some(code) = error.code() {
                write!(output, ", `code={code}`")
                    .expect("the bounded report writer records limit failures");
            }
            output.push('\n');
        }
    }
}

pub(super) struct JunitReporter;

impl JunitReporter {
    pub(super) fn render(report: &DiagnosticReport) -> Result<String, ReportRenderError> {
        let mut output = BoundedOutput::for_report(report);
        let summary = report.summary();
        writeln!(output, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")
            .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "<testsuites name=\"mcp-doctor\" tests=\"{}\" failures=\"{}\" errors=\"0\" skipped=\"{}\" time=\"0\">",
            summary.checks,
            summary.failed,
            summary.skipped + summary.incomplete
        )
        .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "  <testsuite name=\"mcp-doctor\" tests=\"{}\" failures=\"{}\" errors=\"0\" skipped=\"{}\" time=\"0\">",
            summary.checks,
            summary.failed,
            summary.skipped + summary.incomplete
        )
        .expect("the bounded report writer records limit failures");

        for check in report.checks() {
            write_junit_testcase(&mut output, report, check);
        }

        writeln!(output, "  </testsuite>")
            .expect("the bounded report writer records limit failures");
        writeln!(output, "</testsuites>")
            .expect("the bounded report writer records limit failures");
        output.finish()
    }
}

fn write_junit_testcase(
    output: &mut BoundedOutput,
    report: &DiagnosticReport,
    check: &CheckResult,
) {
    write!(
        output,
        "    <testcase classname=\"mcp-doctor.diagnostic\" name=\""
    )
    .expect("the bounded report writer records limit failures");
    write_xml_escaped(output, &check.id().as_str());
    writeln!(output, "\" time=\"0\">").expect("the bounded report writer records limit failures");

    if let Some(reason) = check.skip_reason() {
        write!(output, "      <skipped message=\"")
            .expect("the bounded report writer records limit failures");
        write_xml_escaped(output, reason.as_str());
        output.push_str("\">");
        write_xml_line(output, "skip_reason", reason.as_str());
        write_xml_line(output, "skip_description", reason.description());
        if reason.is_causal() {
            let diagnosis = report
                .primary_diagnosis()
                .expect("the report contract requires a diagnosis for a causal skip");
            write_xml_line(output, "blocked_by.check_id", &diagnosis.check().as_str());
            for (index, finding) in diagnosis.findings().iter().enumerate() {
                write_indexed_xml_line(
                    output,
                    "blocked_by.finding",
                    index,
                    "code",
                    finding.code().as_str(),
                );
                write_indexed_xml_line(
                    output,
                    "blocked_by.finding",
                    index,
                    "location",
                    &finding.location().to_string(),
                );
            }
        }
        writeln!(output, "      </skipped>")
            .expect("the bounded report writer records limit failures");
    } else if check.outcome() == Some(CheckOutcome::Incomplete) {
        output.push_str("      <skipped message=\"incomplete\">");
        write_xml_line(
            output,
            "skip_description",
            "mcp-doctor could not complete this performed check within its fixed work bound",
        );
        writeln!(output, "      </skipped>")
            .expect("the bounded report writer records limit failures");
    } else if check.outcome() == Some(CheckOutcome::Failed) {
        let findings = check
            .findings()
            .expect("a performed failed check has findings");
        let first_failure = findings
            .iter()
            .find(|finding| finding.severity().is_failure() && !finding.is_incomplete())
            .expect("a failed check has one failing finding");
        write!(output, "      <failure message=\"")
            .expect("the bounded report writer records limit failures");
        write_xml_escaped(output, first_failure.code().title());
        output.push_str("\" type=\"");
        write_xml_escaped(output, first_failure.code().as_str());
        output.push_str("\">");
        write_junit_findings(output, report, check, findings);
        writeln!(output, "      </failure>")
            .expect("the bounded report writer records limit failures");
    }

    output.push_str("      <system-out>");
    write_junit_metadata(output, report, check);
    if check.outcome() != Some(CheckOutcome::Failed)
        && let Some(findings) = check.findings()
    {
        write_junit_findings(output, report, check, findings);
    }
    write_junit_reproduction(output, check.reproduction());
    writeln!(output, "      </system-out>")
        .expect("the bounded report writer records limit failures");
    writeln!(output, "    </testcase>").expect("the bounded report writer records limit failures");
}

fn write_junit_metadata(
    output: &mut BoundedOutput,
    report: &DiagnosticReport,
    check: &CheckResult,
) {
    write_xml_line(output, "schema_version", REPORT_SCHEMA_VERSION);
    write_xml_line(output, "schema_stability", "stable");
    write_xml_line(output, "protocol_revision", report.revision().as_str());
    if let Some(negotiated) = report.negotiated_revision() {
        write_xml_line(output, "negotiated_protocol_revision", negotiated.as_str());
    }
    if let Some(selection) = report.protocol_selection() {
        write_xml_line(output, "protocol_selection.mode", selection.mode().as_str());
        write_xml_line(output, "protocol_selection.path", selection.path().as_str());
        if let Some(selected) = selection.selected_revision() {
            write_xml_line(
                output,
                "protocol_selection.selected_revision",
                selected.as_str(),
            );
        }
        writeln!(
            output,
            "protocol_selection.process_launches={}",
            selection.process_launches()
        )
        .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "protocol_selection.lifecycle_requests={}",
            selection.lifecycle_requests()
        )
        .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "protocol_selection.lifecycle_notifications={}",
            selection.lifecycle_notifications()
        )
        .expect("the bounded report writer records limit failures");
        writeln!(
            output,
            "protocol_selection.fallbacks={}",
            selection.fallbacks()
        )
        .expect("the bounded report writer records limit failures");
    }
    write_xml_line(output, "report_outcome", report.outcome().as_str());
    writeln!(output, "exit_code={}", report.exit_status().code())
        .expect("the bounded report writer records limit failures");
    write_xml_line(output, "limits.profile", report.limit_profile().as_str());
    write_xml_line(output, "check_id", &check.id().as_str());
    write_xml_line(output, "requirement", check.requirement().as_str());
    if let Some(outcome) = check.outcome() {
        write_xml_line(output, "state", "performed");
        write_xml_line(output, "outcome", outcome.as_str());
    } else {
        write_xml_line(output, "state", "skipped");
        write_xml_line(output, "outcome", "skipped");
    }
    let primary = report
        .primary_diagnosis()
        .is_some_and(|diagnosis| diagnosis.check() == check.id());
    write_xml_line(
        output,
        "primary_diagnosis",
        if primary { "true" } else { "false" },
    );
}

fn write_junit_findings(
    output: &mut BoundedOutput,
    report: &DiagnosticReport,
    check: &CheckResult,
    findings: &[Finding],
) {
    for (index, finding) in findings.iter().enumerate() {
        write_indexed_xml_line(output, "finding", index, "code", finding.code().as_str());
        write_indexed_xml_line(
            output,
            "finding",
            index,
            "severity",
            finding.severity().as_str(),
        );
        write_indexed_xml_line(
            output,
            "finding",
            index,
            "protocol_revision",
            finding.revision().as_str(),
        );
        write_indexed_xml_line(
            output,
            "finding",
            index,
            "location",
            &finding.location().to_string(),
        );
        write_indexed_xml_line(output, "finding", index, "message", finding.code().title());
        write_indexed_xml_line(output, "finding", index, "impact", finding.impact());
        write_indexed_xml_line(
            output,
            "finding",
            index,
            "expectation",
            finding.expectation(),
        );
        write_indexed_xml_line(
            output,
            "finding",
            index,
            "remediation",
            finding.remediation(),
        );
        write_indexed_xml_line(output, "finding", index, "reference", finding.reference());
        write_indexed_xml_line(
            output,
            "finding",
            index,
            "primary",
            if finding_is_primary(report, check.id(), finding) {
                "true"
            } else {
                "false"
            },
        );
        write_indexed_xml_line(
            output,
            "finding",
            index,
            "independent_safety",
            if finding_is_independent(report, check.id(), finding) {
                "true"
            } else {
                "false"
            },
        );
        write_junit_evidence(output, index, finding);
    }
}

fn finding_is_primary(report: &DiagnosticReport, check: CheckId, finding: &Finding) -> bool {
    report.primary_diagnosis().is_some_and(|diagnosis| {
        diagnosis.check() == check
            && diagnosis.findings().iter().any(|reference| {
                reference.code() == finding.code() && reference.location() == finding.location()
            })
    })
}

fn finding_is_independent(report: &DiagnosticReport, check: CheckId, finding: &Finding) -> bool {
    report.independent_findings().iter().any(|reference| {
        reference.check() == check
            && reference.code() == finding.code()
            && reference.location() == finding.location()
    })
}

fn write_junit_evidence(output: &mut BoundedOutput, index: usize, finding: &Finding) {
    match finding.evidence() {
        FindingEvidence::None => {
            write_indexed_xml_line(output, "finding", index, "evidence.kind", "none");
        }
        FindingEvidence::RevisionAdvertisement(summary) => {
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.kind",
                "revision_advertisement",
            );
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.required",
                finding.revision().as_str(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.offered",
                summary.offered(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.recognized_legacy",
                summary.recognized_legacy(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.unknown_date",
                summary.unknown_date(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.opaque",
                summary.opaque(),
            );
        }
        FindingEvidence::RedactedObservation(observation) => {
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.kind",
                "redacted_observation",
            );
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.marker",
                REDACTION_MARKER,
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.byte_count",
                observation.byte_count(),
            );
        }
        FindingEvidence::LimitViolation(violation) => {
            write_indexed_xml_line(output, "finding", index, "evidence.kind", "limit_violation");
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.limit",
                violation.kind().as_str(),
            );
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.unit",
                violation.kind().unit().as_str(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.observed",
                violation.observed(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.maximum",
                violation.maximum(),
            );
        }
        FindingEvidence::SchemaValidationLimit { phase, violation } => {
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.kind",
                "schema_validation_limit",
            );
            write_indexed_xml_line(output, "finding", index, "evidence.phase", phase.as_str());
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.limit",
                violation.kind().as_str(),
            );
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.unit",
                violation.kind().unit().as_str(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.observed",
                violation.observed(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.maximum",
                violation.maximum(),
            );
        }
        FindingEvidence::CredentialLiteral {
            keyword,
            literal_count,
        } => {
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.kind",
                "credential_literal",
            );
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.keyword_class",
                keyword.as_str(),
            );
            write_indexed_number_line(
                output,
                "finding",
                index,
                "evidence.literal_count",
                *literal_count,
            );
        }
        FindingEvidence::RuleViolation(violation) => {
            write_indexed_xml_line(output, "finding", index, "evidence.kind", "rule_violation");
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.rule",
                violation.as_str(),
            );
            if let Some(expected) = violation.expected_shape() {
                write_indexed_xml_line(
                    output,
                    "finding",
                    index,
                    "evidence.expected",
                    expected.as_str(),
                );
            }
            if let Some(observed) = violation.observed() {
                write_indexed_xml_line(
                    output,
                    "finding",
                    index,
                    "evidence.observed",
                    observed.as_str(),
                );
            }
            if let Some(error_count) = violation.error_count() {
                write_indexed_number_line(
                    output,
                    "finding",
                    index,
                    "evidence.error_count",
                    error_count,
                );
            }
            if let Some(status) = violation.http_status() {
                write_indexed_number_line(output, "finding", index, "evidence.http_status", status);
            }
            if let Some(first_matching_tool_index) = violation.first_matching_tool_index() {
                write_indexed_number_line(
                    output,
                    "finding",
                    index,
                    "evidence.first_matching_tool_index",
                    first_matching_tool_index,
                );
            }
        }
        FindingEvidence::JsonRpcError(error) => {
            write_indexed_xml_line(output, "finding", index, "evidence.kind", "json_rpc_error");
            write_indexed_xml_line(
                output,
                "finding",
                index,
                "evidence.error_kind",
                error.as_str(),
            );
            if let Some(code) = error.code() {
                write_indexed_number_line(output, "finding", index, "evidence.code", code);
            }
        }
    }
}

fn write_junit_reproduction(
    output: &mut BoundedOutput,
    reproduction: Option<&GeneratedCaseReproduction>,
) {
    let Some(reproduction) = reproduction else {
        return;
    };
    let input = reproduction.input();
    write_xml_line(output, "reproduction.generator", reproduction.generator());
    writeln!(output, "reproduction.seed={}", reproduction.seed())
        .expect("the bounded report writer records limit failures");
    if let Some(mutation_kind) = reproduction.mutation_kind() {
        write_xml_line(output, "reproduction.mutation_kind", mutation_kind);
    }
    write_xml_line(output, "reproduction.input.root", input.root().as_str());
    for (name, value) in [
        ("byte_count", input.byte_count()),
        ("node_count", input.node_count()),
        ("maximum_depth", input.maximum_depth()),
        ("nulls", input.nulls()),
        ("booleans", input.booleans()),
        ("numbers", input.numbers()),
        ("strings", input.strings()),
        ("arrays", input.arrays()),
        ("array_items", input.array_items()),
        ("objects", input.objects()),
        ("object_members", input.object_members()),
    ] {
        writeln!(output, "reproduction.input.{name}={value}")
            .expect("the bounded report writer records limit failures");
    }
}

fn write_xml_line(output: &mut BoundedOutput, key: &str, value: &str) {
    write!(output, "{key}=").expect("the bounded report writer records limit failures");
    write_xml_escaped(output, value);
    output.push('\n');
}

fn write_indexed_xml_line(
    output: &mut BoundedOutput,
    prefix: &str,
    index: usize,
    key: &str,
    value: &str,
) {
    write!(output, "{prefix}[{index}].{key}=")
        .expect("the bounded report writer records limit failures");
    write_xml_escaped(output, value);
    output.push('\n');
}

fn write_indexed_number_line<T: fmt::Display>(
    output: &mut BoundedOutput,
    prefix: &str,
    index: usize,
    key: &str,
    value: T,
) {
    writeln!(output, "{prefix}[{index}].{key}={value}")
        .expect("the bounded report writer records limit failures");
}

fn write_xml_escaped(output: &mut BoundedOutput, value: &str) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '\"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            character if !is_xml_10_character(character) => output.push('\u{fffd}'),
            character => output.push(character),
        }
    }
}

fn is_xml_10_character(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{a}' | '\u{d}')
        || ('\u{20}'..='\u{d7ff}').contains(&character)
        || ('\u{e000}'..='\u{fffd}').contains(&character)
        || ('\u{10000}'..='\u{10ffff}').contains(&character)
}

pub(super) struct JsonReporter;

impl JsonReporter {
    pub(super) fn render(report: &DiagnosticReport) -> Result<String, ReportRenderError> {
        let envelope = JsonReport::from(report);
        let mut output = BoundedOutput::for_report(report);
        serde_json::to_writer_pretty(&mut output, &envelope)
            .expect("a typed diagnostic report must serialize as JSON");
        output.push('\n');
        output.finish()
    }
}

#[derive(Serialize)]
struct JsonReport {
    schema_version: &'static str,
    schema_stability: &'static str,
    protocol_revision: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    negotiated_protocol_revision: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_selection: Option<ProtocolSelectionEvidence>,
    primary_diagnosis: Option<JsonDiagnosis>,
    independent_findings: Vec<JsonIndependentFinding>,
    outcome: &'static str,
    exit_code: u8,
    limits: JsonLimits,
    summary: ReportSummary,
    checks: Vec<JsonCheck>,
}

impl From<&DiagnosticReport> for JsonReport {
    fn from(report: &DiagnosticReport) -> Self {
        Self {
            schema_version: REPORT_SCHEMA_VERSION,
            schema_stability: "stable",
            protocol_revision: report.revision().as_str(),
            negotiated_protocol_revision: report.negotiated_revision().map(KnownRevision::as_str),
            protocol_selection: report.protocol_selection(),
            primary_diagnosis: report.primary_diagnosis().map(JsonDiagnosis::from),
            independent_findings: report
                .independent_findings()
                .iter()
                .map(JsonIndependentFinding::from)
                .collect(),
            outcome: report.outcome().as_str(),
            exit_code: report.exit_status().code(),
            limits: JsonLimits::from_report(report),
            summary: report.summary(),
            checks: report
                .checks()
                .iter()
                .map(|check| JsonCheck::from_parts(check, report.primary_diagnosis()))
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct JsonDiagnosis {
    check_id: String,
    findings: Vec<JsonDiagnosisFinding>,
}

impl From<&Diagnosis> for JsonDiagnosis {
    fn from(diagnosis: &Diagnosis) -> Self {
        Self {
            check_id: diagnosis.check().as_str(),
            findings: diagnosis
                .findings()
                .iter()
                .map(JsonDiagnosisFinding::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct JsonDiagnosisFinding {
    code: &'static str,
    location: String,
}

impl From<&FindingReference> for JsonDiagnosisFinding {
    fn from(finding: &FindingReference) -> Self {
        Self {
            code: finding.code().as_str(),
            location: finding.location().to_string(),
        }
    }
}

#[derive(Serialize)]
struct JsonIndependentFinding {
    check_id: String,
    code: &'static str,
    location: String,
}

impl From<&FindingReference> for JsonIndependentFinding {
    fn from(finding: &FindingReference) -> Self {
        Self {
            check_id: finding.check().as_str(),
            code: finding.code().as_str(),
            location: finding.location().to_string(),
        }
    }
}

#[derive(Serialize)]
struct JsonLimits {
    profile: &'static str,
    startup_ms: u64,
    discovery_ms: u64,
    request_ms: u64,
    response_ms: u64,
    shutdown_grace_ms: u64,
    total_ms: u64,
    message_bytes: u64,
    stdout_bytes: u64,
    stderr_bytes: u64,
    aggregate_output_bytes: u64,
    endpoint_bytes: u64,
    resolution_addresses: u64,
    resolution_count: u64,
    trust_bytes: u64,
    trust_certificates: u64,
    request_fields: u64,
    request_field_name_bytes: u64,
    request_field_value_bytes: u64,
    request_fields_bytes: u64,
    response_fields: u64,
    response_field_name_bytes: u64,
    response_field_value_bytes: u64,
    response_fields_bytes: u64,
    message_count: u64,
    protocol_revisions: u64,
    catalog_items: u64,
    schema_bytes: u64,
    instance_bytes: u64,
    schema_nodes: u64,
    schema_depth: u64,
    schema_ref_depth: u64,
    schema_evaluation_steps: u64,
    validation_errors: u64,
    report_findings: u64,
    report_bytes: u64,
    active_cases: u64,
    generation_attempts: u64,
    generation_candidates: u64,
    generation_steps: u64,
    redirects: u64,
    retries: u64,
    concurrency: u64,
}

impl JsonLimits {
    fn from_report(report: &DiagnosticReport) -> Self {
        let values = report.limits().values();
        Self {
            profile: report.limit_profile().as_str(),
            startup_ms: values.startup_ms,
            discovery_ms: values.discovery_ms,
            request_ms: values.request_ms,
            response_ms: values.response_ms,
            shutdown_grace_ms: values.shutdown_grace_ms,
            total_ms: values.total_ms,
            message_bytes: values.message_bytes,
            stdout_bytes: values.stdout_bytes,
            stderr_bytes: values.stderr_bytes,
            aggregate_output_bytes: values.aggregate_output_bytes,
            endpoint_bytes: values.endpoint_bytes,
            resolution_addresses: values.resolution_addresses,
            resolution_count: values.resolution_count,
            trust_bytes: values.trust_bytes,
            trust_certificates: values.trust_certificates,
            request_fields: values.request_fields,
            request_field_name_bytes: values.request_field_name_bytes,
            request_field_value_bytes: values.request_field_value_bytes,
            request_fields_bytes: values.request_fields_bytes,
            response_fields: values.response_fields,
            response_field_name_bytes: values.response_field_name_bytes,
            response_field_value_bytes: values.response_field_value_bytes,
            response_fields_bytes: values.response_fields_bytes,
            message_count: values.message_count,
            protocol_revisions: values.protocol_revisions,
            catalog_items: values.catalog_items,
            schema_bytes: values.schema_bytes,
            instance_bytes: values.instance_bytes,
            schema_nodes: values.schema_nodes,
            schema_depth: values.schema_depth,
            schema_ref_depth: values.schema_ref_depth,
            schema_evaluation_steps: values.schema_evaluation_steps,
            validation_errors: values.validation_errors,
            report_findings: values.report_findings,
            report_bytes: values.report_bytes,
            active_cases: values.active_cases,
            generation_attempts: values.generation_attempts,
            generation_candidates: values.generation_candidates,
            generation_steps: values.generation_steps,
            redirects: values.redirects,
            retries: values.retries,
            concurrency: values.concurrency,
        }
    }
}

#[derive(Serialize)]
struct JsonCheck {
    id: String,
    requirement: &'static str,
    state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocked_by: Option<JsonDiagnosis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reproduction: Option<JsonReproduction>,
    findings: Vec<JsonFinding>,
}

impl JsonCheck {
    fn from_parts(check: &CheckResult, diagnosis: Option<&Diagnosis>) -> Self {
        let reason = check.skip_reason();
        Self {
            id: check.id().as_str(),
            requirement: check.requirement().as_str(),
            state: if check.findings().is_some() {
                "performed"
            } else {
                "skipped"
            },
            outcome: check.outcome().map(CheckOutcome::as_str),
            skip_reason: reason.map(|reason| reason.as_str()),
            blocked_by: reason
                .filter(|reason| reason.is_causal())
                .and(diagnosis)
                .map(JsonDiagnosis::from),
            reproduction: check.reproduction().map(JsonReproduction::from),
            findings: check
                .findings()
                .unwrap_or_default()
                .iter()
                .map(JsonFinding::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct JsonReproduction {
    generator: &'static str,
    seed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    mutation_kind: Option<&'static str>,
    input: JsonStructuralInput,
}

impl From<&GeneratedCaseReproduction> for JsonReproduction {
    fn from(reproduction: &GeneratedCaseReproduction) -> Self {
        Self {
            generator: reproduction.generator(),
            seed: reproduction.seed(),
            mutation_kind: reproduction.mutation_kind(),
            input: JsonStructuralInput::from(reproduction.input()),
        }
    }
}

#[derive(Serialize)]
struct JsonStructuralInput {
    root: &'static str,
    byte_count: u64,
    node_count: u64,
    maximum_depth: u64,
    nulls: u64,
    booleans: u64,
    numbers: u64,
    strings: u64,
    arrays: u64,
    array_items: u64,
    objects: u64,
    object_members: u64,
}

impl From<&StructuralInput> for JsonStructuralInput {
    fn from(input: &StructuralInput) -> Self {
        Self {
            root: input.root().as_str(),
            byte_count: input.byte_count(),
            node_count: input.node_count(),
            maximum_depth: input.maximum_depth(),
            nulls: input.nulls(),
            booleans: input.booleans(),
            numbers: input.numbers(),
            strings: input.strings(),
            arrays: input.arrays(),
            array_items: input.array_items(),
            objects: input.objects(),
            object_members: input.object_members(),
        }
    }
}

#[derive(Serialize)]
struct JsonFinding {
    code: &'static str,
    severity: &'static str,
    protocol_revision: &'static str,
    location: String,
    message: &'static str,
    impact: &'static str,
    expectation: &'static str,
    remediation: &'static str,
    reference: &'static str,
    evidence: JsonEvidence,
}

impl From<&Finding> for JsonFinding {
    fn from(finding: &Finding) -> Self {
        Self {
            code: finding.code().as_str(),
            severity: finding.severity().as_str(),
            protocol_revision: finding.revision().as_str(),
            location: finding.location().to_string(),
            message: finding.code().title(),
            impact: finding.impact(),
            expectation: finding.expectation(),
            remediation: finding.remediation(),
            reference: finding.reference(),
            evidence: JsonEvidence::from_parts(finding.evidence(), finding.revision()),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum JsonEvidence {
    None,
    RevisionAdvertisement {
        required: &'static str,
        offered: usize,
        recognized_legacy: usize,
        unknown_date: usize,
        opaque: usize,
    },
    RedactedObservation {
        marker: &'static str,
        byte_count: usize,
    },
    LimitViolation {
        limit: &'static str,
        unit: &'static str,
        observed: u64,
        maximum: u64,
    },
    SchemaValidationLimit {
        phase: &'static str,
        limit: &'static str,
        unit: &'static str,
        observed: u64,
        maximum: u64,
    },
    CredentialLiteral {
        keyword_class: &'static str,
        literal_count: u64,
    },
    RuleViolation {
        rule: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        observed: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_count: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http_status: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
        first_matching_tool_index: Option<u64>,
    },
    JsonRpcError {
        error_kind: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<i64>,
    },
}

impl JsonEvidence {
    fn from_parts(evidence: &FindingEvidence, revision: SupportedRevision) -> Self {
        match evidence {
            FindingEvidence::None => Self::None,
            FindingEvidence::RevisionAdvertisement(summary) => Self::RevisionAdvertisement {
                required: revision.as_str(),
                offered: summary.offered(),
                recognized_legacy: summary.recognized_legacy(),
                unknown_date: summary.unknown_date(),
                opaque: summary.opaque(),
            },
            FindingEvidence::RedactedObservation(observation) => Self::RedactedObservation {
                marker: REDACTION_MARKER,
                byte_count: observation.byte_count(),
            },
            FindingEvidence::LimitViolation(violation) => Self::LimitViolation {
                limit: violation.kind().as_str(),
                unit: violation.kind().unit().as_str(),
                observed: violation.observed(),
                maximum: violation.maximum(),
            },
            FindingEvidence::SchemaValidationLimit { phase, violation } => {
                Self::SchemaValidationLimit {
                    phase: phase.as_str(),
                    limit: violation.kind().as_str(),
                    unit: violation.kind().unit().as_str(),
                    observed: violation.observed(),
                    maximum: violation.maximum(),
                }
            }
            FindingEvidence::CredentialLiteral {
                keyword,
                literal_count,
            } => Self::CredentialLiteral {
                keyword_class: keyword.as_str(),
                literal_count: *literal_count,
            },
            FindingEvidence::RuleViolation(violation) => Self::RuleViolation {
                rule: violation.as_str(),
                expected: violation.expected_shape().map(|shape| shape.as_str()),
                observed: violation.observed().map(|kind| kind.as_str()),
                error_count: violation.error_count(),
                http_status: violation.http_status(),
                first_matching_tool_index: violation.first_matching_tool_index(),
            },
            FindingEvidence::JsonRpcError(error) => Self::JsonRpcError {
                error_kind: error.as_str(),
                code: error.code(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BadgeReport, BadgeReporter, BoundedOutput, DiagnosticReport, ExitStatus, HumanReporter,
        JsonReporter, JunitReporter, MarkdownReporter, OverallOutcome, ReportArtifactFormat,
        ReportContractError, ReportFormat, ReportRenderError, ReportRequest, render_reports,
        render_reports_with_limit, write_xml_escaped,
    };
    use crate::contract::limits::{
        DiagnosticLimitProfile, DiagnosticLimits, LimitKind, LimitValues, LimitViolation,
    };
    use crate::contract::model::{
        CheckId, CheckResult, ExpectedShape, Finding, FindingCode, JsonKind, Location,
        LocationField, Requirement, RuleViolation, SchemaValidationPhase, SkipReason,
    };
    use crate::contract::protocol::SupportedRevision;
    use crate::contract::redaction::Sensitive;
    use quick_xml::Reader;
    use quick_xml::events::Event;

    const STABLE_REPORT_SCHEMA: &str =
        include_str!("../../schemas/mcp-doctor.report.v1.schema.json");

    fn stable_report_validator() -> jsonschema::Validator {
        let schema: serde_json::Value = serde_json::from_str(STABLE_REPORT_SCHEMA)
            .expect("the committed report schema is JSON");
        jsonschema::draft202012::options()
            .build(&schema)
            .expect("the committed report schema follows Draft 2020-12")
    }

    fn assert_stable_report(value: &serde_json::Value) {
        let errors = stable_report_validator()
            .iter_errors(value)
            .map(|error| error.instance_path().to_string())
            .collect::<Vec<_>>();
        assert!(
            errors.is_empty(),
            "stable report schema rejected synthetic fields at {errors:?}"
        );
    }

    fn assert_common_junit_document(
        document: &str,
        expected_tests: usize,
        expected_failures: usize,
        expected_skips: usize,
    ) {
        let mut reader = Reader::from_str(document);
        let mut stack = Vec::<Vec<u8>>::new();
        let mut tests = 0;
        let mut failures = 0;
        let mut skips = 0;
        let mut system_outputs = 0;

        loop {
            match reader
                .read_event()
                .expect("quick-xml should accept the common JUnit document")
            {
                Event::Start(element) => {
                    for attribute in element.attributes() {
                        attribute.expect("every JUnit attribute should parse");
                    }
                    let name = element.name().as_ref().to_vec();
                    let parent = stack.last().map(Vec::as_slice);
                    match name.as_slice() {
                        b"testsuites" => assert!(parent.is_none()),
                        b"testsuite" => assert_eq!(parent, Some(b"testsuites".as_slice())),
                        b"testcase" => {
                            assert_eq!(parent, Some(b"testsuite".as_slice()));
                            tests += 1;
                        }
                        b"failure" => {
                            assert_eq!(parent, Some(b"testcase".as_slice()));
                            failures += 1;
                        }
                        b"skipped" => {
                            assert_eq!(parent, Some(b"testcase".as_slice()));
                            skips += 1;
                        }
                        b"system-out" => {
                            assert_eq!(parent, Some(b"testcase".as_slice()));
                            system_outputs += 1;
                        }
                        name => panic!("unexpected common JUnit element: {name:?}"),
                    }
                    stack.push(name);
                }
                Event::End(element) => {
                    let expected = stack.pop().expect("every JUnit end tag has a start tag");
                    assert_eq!(element.name().as_ref(), expected);
                }
                Event::Decl(_) | Event::Text(_) | Event::CData(_) | Event::Comment(_) => {}
                Event::Eof => break,
                event => panic!("unexpected XML event in common JUnit output: {event:?}"),
            }
        }

        assert!(stack.is_empty());
        assert_eq!(tests, expected_tests);
        assert_eq!(failures, expected_failures);
        assert_eq!(skips, expected_skips);
        assert_eq!(system_outputs, expected_tests);
    }

    fn passing_revision_check() -> CheckResult {
        CheckResult::performed(CheckId::ProtocolRevision, Requirement::Required, Vec::new())
    }

    fn synthetic_failed_report() -> DiagnosticReport {
        let violation = LimitViolation::new(LimitKind::MessageBytes, 1_048_577, 1_048_576)
            .expect("synthetic limit evidence should exceed its maximum");
        let limit_finding = Finding::limit_exceeded(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Tools)
                .index(3)
                .field(LocationField::InputSchema),
            violation,
        );

        DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![
                CheckResult::skipped(
                    CheckId::RuntimeTools,
                    Requirement::Optional,
                    SkipReason::NotAuthorized,
                ),
                CheckResult::performed(
                    CheckId::SchemaContracts,
                    Requirement::Required,
                    vec![limit_finding.clone(), limit_finding],
                ),
                passing_revision_check(),
            ],
        )
        .expect("synthetic report should satisfy the contract")
    }

    #[test]
    fn all_reporters_match_canonical_synthetic_fixtures() {
        let report = synthetic_failed_report();
        let human = HumanReporter::render(&report);
        let json = JsonReporter::render(&report).expect("typed report should serialize");
        let junit = JunitReporter::render(&report).expect("typed report should serialize as JUnit");
        let markdown =
            MarkdownReporter::render(&report).expect("typed report should serialize as Markdown");
        let badge = BadgeReporter::render(&report).expect("typed report should serialize as badge");
        let json_value: serde_json::Value =
            serde_json::from_str(&json).expect("the JSON reporter should emit one value");

        assert_eq!(
            human,
            include_str!("../../tests/fixtures/contracts/failed-report.txt")
        );
        assert_eq!(
            json,
            include_str!("../../tests/fixtures/contracts/failed-report.json")
        );
        assert_eq!(
            junit,
            include_str!("../../tests/fixtures/contracts/failed-report.junit.xml")
        );
        assert_eq!(
            markdown,
            include_str!("../../tests/fixtures/contracts/failed-report.md")
        );
        assert_eq!(
            badge,
            include_str!("../../tests/fixtures/contracts/failed-badge.json")
        );
        assert_eq!(report.outcome(), OverallOutcome::Failed);
        assert_eq!(report.exit_status(), ExitStatus::DiagnosticFailure);
        assert_stable_report(&json_value);
        assert_common_junit_document(&junit, 3, 1, 1);
    }

    #[test]
    fn selected_profile_is_deterministic_and_agrees_across_reporters() {
        let report =
            synthetic_failed_report().with_limit_profile(DiagnosticLimitProfile::SlowStart);
        let human = HumanReporter::render(&report);
        let json = JsonReporter::render(&report).expect("typed report should serialize");
        let junit = JunitReporter::render(&report).expect("typed report should serialize as JUnit");
        let markdown =
            MarkdownReporter::render(&report).expect("typed report should serialize as Markdown");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("the JSON reporter should emit one value");

        assert_eq!(report.limit_profile(), DiagnosticLimitProfile::SlowStart);
        assert!(human.contains("LIMITS · profile=slow-start"));
        assert!(human.contains("startup_ms=30000"));
        assert!(human.contains("total_ms=240000"));
        assert_eq!(value["limits"]["profile"], "slow-start");
        assert_eq!(value["limits"]["startup_ms"], 30_000);
        assert_eq!(value["limits"]["total_ms"], 240_000);
        assert!(junit.contains("limits.profile=slow-start"));
        assert!(markdown.contains("| Limit profile | `slow-start` |"));
        assert!(markdown.contains("`startup_ms=30000`"));
        assert!(markdown.contains("`total_ms=240000`"));
        assert_stable_report(&value);
        assert_common_junit_document(&junit, 3, 1, 1);
    }

    #[test]
    fn markdown_pass_is_byte_stable_and_contains_no_active_content() {
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![passing_revision_check()],
        )
        .expect("the passing Markdown fixture should satisfy the report contract");
        let first = MarkdownReporter::render(&report).expect("the Markdown report should fit");
        let second = MarkdownReporter::render(&report).expect("the Markdown report should repeat");

        assert_eq!(first, second);
        assert_eq!(
            first,
            include_str!("../../tests/fixtures/contracts/passed-report.md")
        );
        assert!(first.starts_with("<!-- mcp-doctor.markdown/v1 -->\n"));
        assert!(first.ends_with('\n'));
        assert!(!first.contains('\r'));
        assert!(!first.contains('\u{1b}'));
        assert!(!first.contains("!["));
        let body = first
            .strip_prefix("<!-- mcp-doctor.markdown/v1 -->\n")
            .expect("the version marker should be exact");
        assert!(!body.contains('<'));
        assert!(!body.contains('>'));
        assert!(first.contains("| Outcome | `passed` |"));
        assert!(first.contains("| Exit | `0` (`success`) |"));
        assert!(first.contains("| Checks | 1 |"));
        assert!(first.contains("- Findings: None."));
        assert!(first.contains("## Primary diagnosis\n\nNone."));
        assert!(first.contains("## Causal skips\n\nNone."));
    }

    #[test]
    fn badge_outcomes_are_byte_stable_fixed_and_strictly_deserializable() {
        let passed = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![passing_revision_check()],
        )
        .expect("the passing badge fixture should satisfy the report contract");
        let incomplete = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::skipped(
                CheckId::RuntimeTools,
                Requirement::Required,
                SkipReason::NotAuthorized,
            )],
        )
        .expect("the incomplete badge fixture should satisfy the report contract");
        let failed = synthetic_failed_report();

        for (report, fixture) in [
            (
                &passed,
                include_str!("../../tests/fixtures/contracts/passed-badge.json"),
            ),
            (
                &failed,
                include_str!("../../tests/fixtures/contracts/failed-badge.json"),
            ),
            (
                &incomplete,
                include_str!("../../tests/fixtures/contracts/incomplete-badge.json"),
            ),
        ] {
            let first = BadgeReporter::render(report).expect("the badge should fit");
            let second = BadgeReporter::render(report).expect("the badge should repeat");
            assert_eq!(first, second);
            assert_eq!(first, fixture);
            assert!(first.ends_with('\n'));
            assert!(!first.contains('\r'));
            serde_json::from_str::<BadgeReport>(&first)
                .expect("the fixed badge should satisfy its strict typed contract");
        }

        for invalid in [
            r#"{"schemaVersion":2,"label":"mcp-doctor","message":"pass","color":"brightgreen"}"#,
            r#"{"schemaVersion":1,"label":"dynamic","message":"pass","color":"brightgreen"}"#,
            r#"{"schemaVersion":1,"label":"mcp-doctor","message":"passed","color":"brightgreen"}"#,
            r#"{"schemaVersion":1,"label":"mcp-doctor","message":"pass","color":"green"}"#,
            r#"{"schemaVersion":1,"label":"mcp-doctor","message":"pass","color":"red"}"#,
            r#"{"schemaVersion":1,"label":"mcp-doctor","message":"pass","color":"brightgreen","score":100}"#,
        ] {
            assert!(
                serde_json::from_str::<BadgeReport>(invalid).is_err(),
                "the strict badge contract accepted {invalid}"
            );
        }
    }

    #[test]
    fn one_report_fans_out_in_fixed_order_under_one_aggregate_bound() {
        let report = synthetic_failed_report();
        let request = ReportRequest::new(ReportFormat::Human, true, true, true, true);
        let rendered = render_reports(&report, request)
            .expect("the synthetic report projections should fit their bounds");

        assert_eq!(
            rendered.stdout,
            include_str!("../../tests/fixtures/contracts/failed-report.txt")
        );
        assert_eq!(rendered.artifacts.len(), 4);
        assert_eq!(rendered.artifacts[0].format, ReportArtifactFormat::Json);
        assert_eq!(
            rendered.artifacts[0].output,
            include_str!("../../tests/fixtures/contracts/failed-report.json")
        );
        assert_eq!(rendered.artifacts[1].format, ReportArtifactFormat::Junit);
        assert_eq!(
            rendered.artifacts[1].output,
            include_str!("../../tests/fixtures/contracts/failed-report.junit.xml")
        );
        assert_eq!(rendered.artifacts[2].format, ReportArtifactFormat::Markdown);
        assert_eq!(
            rendered.artifacts[2].output,
            include_str!("../../tests/fixtures/contracts/failed-report.md")
        );
        assert_eq!(rendered.artifacts[3].format, ReportArtifactFormat::Badge);
        assert_eq!(
            rendered.artifacts[3].output,
            include_str!("../../tests/fixtures/contracts/failed-badge.json")
        );

        let aggregate_bytes = rendered
            .artifacts
            .iter()
            .map(|artifact| artifact.output.len())
            .sum::<usize>()
            .saturating_add(rendered.stdout.len());
        let maximum = u64::try_from(aggregate_bytes - 1).expect("fixture length should fit");
        assert_eq!(
            render_reports_with_limit(&report, request, maximum).err(),
            Some(ReportRenderError::AggregateSizeLimitExceeded { maximum })
        );
    }

    #[test]
    fn junit_escapes_xml_metacharacters_and_replaces_invalid_xml_characters() {
        let mut output = BoundedOutput {
            output: Vec::new(),
            maximum: 1_024,
            declared_maximum: 1_024,
            exceeded: false,
        };
        write_xml_escaped(&mut output, "<&>\"'\u{1}\t");

        assert_eq!(
            output.finish().expect("the escaped fixture should fit"),
            "&lt;&amp;&gt;&quot;&apos;�\t"
        );
    }

    #[test]
    fn every_reporter_enforces_the_declared_output_byte_limit() {
        let limits = DiagnosticLimits::try_from_values(LimitValues {
            report_bytes: 32,
            ..DiagnosticLimits::DEFAULTS.values()
        })
        .expect("the synthetic report byte limit should be valid");
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            limits,
            vec![passing_revision_check()],
        )
        .expect("the synthetic report should satisfy the contract");
        let expected = Err(ReportRenderError::SizeLimitExceeded { maximum: 32 });

        assert_eq!(HumanReporter::try_render(&report), expected);
        assert_eq!(JsonReporter::render(&report), expected);
        assert_eq!(JunitReporter::render(&report), expected);
        assert_eq!(MarkdownReporter::render(&report), expected);
        assert_eq!(BadgeReporter::render(&report), expected);
    }

    #[test]
    fn junit_and_markdown_preserve_primary_independent_and_causal_relationships() {
        let cleanup = Finding::cleanup_failed(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Process),
        );
        let catalog = Finding::catalog_contract_invalid(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Tools),
            RuleViolation::ExpectedShape {
                expected: ExpectedShape::Array,
                observed: JsonKind::String,
            },
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![
                CheckResult::performed(
                    CheckId::TransportStdio,
                    Requirement::Required,
                    vec![cleanup],
                ),
                CheckResult::performed(
                    CheckId::DiscoveryCatalogs,
                    Requirement::Required,
                    vec![catalog],
                ),
                CheckResult::skipped(
                    CheckId::RuntimeTools,
                    Requirement::Required,
                    SkipReason::PrerequisiteFailed,
                ),
            ],
        )
        .expect("the relationship fixture should satisfy the report contract");
        let junit = JunitReporter::render(&report).expect("the JUnit report should fit");
        let markdown = MarkdownReporter::render(&report).expect("the Markdown report should fit");
        assert_eq!(
            markdown,
            include_str!("../../tests/fixtures/contracts/relationship-report.md")
        );

        assert!(junit.contains("finding[0].independent_safety=true"));
        assert!(junit.contains("finding[0].primary=true"));
        assert!(junit.contains("blocked_by.check_id=discovery.catalogs"));
        assert!(junit.contains("blocked_by.finding[0].code=MCP-CATALOG-001"));
        assert!(junit.contains("report_outcome=failed\nexit_code=1"));
        assert_common_junit_document(&junit, 3, 2, 1);
        assert!(markdown.contains("## Primary diagnosis"));
        assert!(markdown.contains("- Check: `discovery.catalogs`"));
        assert!(markdown.contains("`MCP-CATALOG-001` at `tools`"));
        assert!(markdown.contains("## Independent safety findings"));
        assert!(markdown.contains("`MCP-SAFETY-001` in `transport.stdio` at `process`"));
        assert!(markdown.contains("## Causal skips"));
        assert!(markdown.contains("`runtime.tools` (`required`) was skipped"));
        assert!(markdown.contains("Blocked by `discovery.catalogs`"));
    }

    #[test]
    fn stable_schema_allows_compatible_optional_fields_and_new_finding_codes() {
        let rendered = JsonReporter::render(&synthetic_failed_report())
            .expect("typed report should serialize");
        let mut value: serde_json::Value =
            serde_json::from_str(&rendered).expect("the JSON reporter should emit one value");
        value["future_optional"] = serde_json::json!({"safe": true});
        value["checks"][1]["findings"][0]["code"] =
            serde_json::Value::String("MCP-FUTURE-999".to_owned());
        value["checks"][1]["findings"][0]["future_optional"] = serde_json::json!("safe metadata");

        assert_stable_report(&value);
    }

    #[test]
    fn stable_schema_rejects_a_removed_required_field() {
        let rendered = JsonReporter::render(&synthetic_failed_report())
            .expect("typed report should serialize");
        let mut value: serde_json::Value =
            serde_json::from_str(&rendered).expect("the JSON reporter should emit one value");
        value
            .as_object_mut()
            .expect("the report envelope is an object")
            .remove("exit_code");

        assert!(!stable_report_validator().is_valid(&value));
    }

    #[test]
    fn arbitrary_observations_are_redacted_in_every_reporter() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/contracts/redaction-case.json"
        ))
        .expect("synthetic redaction fixture should be valid JSON");
        let sentinel = fixture["sentinel"]
            .as_str()
            .expect("synthetic sentinel should be a string");
        let sensitive = Sensitive::new(sentinel.as_bytes());
        let finding = Finding::invalid_revision_value(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Request)
                .field(LocationField::Meta)
                .field(LocationField::ProtocolVersion),
            sensitive.redacted(),
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::ProtocolEnvelope,
                Requirement::Required,
                vec![finding],
            )],
        )
        .expect("redacted synthetic report should satisfy the contract");
        let rendered = format!(
            "{}\n{}\n{}\n{}\n{}",
            HumanReporter::render(&report),
            JsonReporter::render(&report).expect("typed report should serialize"),
            JunitReporter::render(&report).expect("typed report should serialize as JUnit"),
            MarkdownReporter::render(&report).expect("typed report should serialize as Markdown"),
            BadgeReporter::render(&report).expect("typed report should serialize as badge")
        );

        assert!(rendered.contains("[REDACTED]"));
        assert!(rendered.contains(&format!("{} bytes", sentinel.len())));
        assert!(
            !rendered.contains(sentinel),
            "reporters must not reveal a synthetic sentinel"
        );
    }

    #[test]
    fn rule_findings_give_humans_and_agents_the_same_safe_correction() {
        let finding = Finding::catalog_contract_invalid(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Prompts)
                .index(0)
                .field(LocationField::Arguments),
            RuleViolation::ExpectedShape {
                expected: ExpectedShape::Array,
                observed: JsonKind::String,
            },
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::DiscoveryCatalogs,
                Requirement::Required,
                vec![finding],
            )],
        )
        .expect("synthetic catalog report should satisfy the contract");
        let human = HumanReporter::render(&report);
        let json = JsonReporter::render(&report).expect("typed report should serialize");
        let junit = JunitReporter::render(&report).expect("typed report should serialize as JUnit");
        let markdown =
            MarkdownReporter::render(&report).expect("typed report should serialize as Markdown");
        for rendered in [&human, &json, &junit, &markdown] {
            assert!(rendered.contains("MCP-CATALOG-001"));
            assert!(rendered.contains("prompts[0].arguments"));
            assert!(rendered.contains("expected_shape"));
            assert!(rendered.contains("array"));
            assert!(rendered.contains("string"));
            assert!(rendered.contains("Correct the value"));
            assert!(rendered.contains("selected MCP revision catalog contracts"));
        }
    }

    #[test]
    fn reused_description_relationship_is_typed_and_consistent_across_reporters() {
        let finding = Finding::tool_description_reused_normalized(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Tools)
                .index(7)
                .field(LocationField::Description),
            2,
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::DiscoveryCatalogs,
                Requirement::Required,
                vec![finding],
            )],
        )
        .expect("synthetic quality report should satisfy the contract");

        let human = HumanReporter::render(&report);
        let json = JsonReporter::render(&report).expect("typed report should serialize");
        let junit = JunitReporter::render(&report).expect("typed report should serialize as JUnit");
        let markdown =
            MarkdownReporter::render(&report).expect("typed report should serialize as Markdown");
        let badge = BadgeReporter::render(&report).expect("typed report should serialize as badge");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("the JSON reporter should emit one value");
        let finding = &value["checks"][0]["findings"][0];

        assert_eq!(report.outcome(), OverallOutcome::Passed);
        assert_eq!(report.exit_status(), ExitStatus::Success);
        assert_eq!(finding["code"], "MCP-QUALITY-004");
        assert_eq!(finding["severity"], "warning");
        assert_eq!(finding["location"], "tools[7].description");
        assert_eq!(
            finding["evidence"],
            serde_json::json!({
                "kind": "rule_violation",
                "rule": "reused_normalized_tool_description",
                "first_matching_tool_index": 2
            })
        );
        for rendered in [&human, &junit, &markdown] {
            assert!(rendered.contains("MCP-QUALITY-004"));
            assert!(rendered.contains("tools[7].description"));
            assert!(rendered.contains("reused_normalized_tool_description"));
            assert!(rendered.contains("first_matching_tool_index"));
            assert!(rendered.contains('2'));
        }
        assert!(!badge.contains("MCP-QUALITY-004"));
        assert!(!badge.contains("first_matching_tool_index"));
        assert_stable_report(&value);
    }

    #[test]
    fn required_skips_are_incomplete_while_optional_skips_can_pass() {
        let required_skip = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![
                passing_revision_check(),
                CheckResult::skipped(
                    CheckId::DiscoveryCatalogs,
                    Requirement::Required,
                    SkipReason::NotAdvertised,
                ),
            ],
        )
        .expect("required skip report should be structurally valid");
        assert_eq!(required_skip.outcome(), OverallOutcome::Incomplete);
        assert_eq!(required_skip.exit_status(), ExitStatus::Incomplete);

        let optional_skip = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![
                passing_revision_check(),
                CheckResult::skipped(
                    CheckId::RuntimeTools,
                    Requirement::Optional,
                    SkipReason::NotAuthorized,
                ),
            ],
        )
        .expect("optional skip report should be structurally valid");
        assert_eq!(optional_skip.outcome(), OverallOutcome::Passed);
        assert_eq!(optional_skip.exit_status(), ExitStatus::Success);
    }

    #[test]
    fn diagnostic_failure_takes_precedence_without_hiding_required_skips() {
        let finding = Finding::schema_contract_invalid(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Tools)
                .index(0)
                .field(LocationField::InputSchema),
            RuleViolation::InvalidDraft202012 { error_count: 1 },
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![
                CheckResult::performed(
                    CheckId::SchemaContracts,
                    Requirement::Required,
                    vec![finding],
                ),
                CheckResult::skipped(
                    CheckId::RuntimeTools,
                    Requirement::Required,
                    SkipReason::PrerequisiteFailed,
                ),
            ],
        )
        .expect("failed report with a required skip should satisfy the contract");

        assert_eq!(report.outcome(), OverallOutcome::Failed);
        assert_eq!(report.exit_status(), ExitStatus::DiagnosticFailure);
        assert_eq!(report.summary().required_skipped, 1);
    }

    #[test]
    fn performed_schema_incomplete_is_primary_exit_three_and_reporter_safe() {
        let location = Location::root(LocationField::Tools)
            .index(0)
            .field(LocationField::InputSchema);
        let incomplete = Finding::schema_validation_incomplete(
            SupportedRevision::CURRENT,
            location.clone(),
            SchemaValidationPhase::CompileConstruction,
            LimitViolation::new(LimitKind::SchemaEvaluationSteps, 100_001, 100_000)
                .expect("the synthetic evidence should exceed the fixed bound"),
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::SchemaContracts,
                Requirement::Required,
                vec![incomplete.clone()],
            )],
        )
        .expect("performed incomplete evidence should satisfy the report contract");

        assert_eq!(report.outcome(), OverallOutcome::Incomplete);
        assert_eq!(report.exit_status(), ExitStatus::Incomplete);
        assert_eq!(report.summary().incomplete, 1);
        assert_eq!(
            report.primary_diagnosis().unwrap().findings()[0].code(),
            FindingCode::SchemaValidationIncomplete
        );

        let human = HumanReporter::render(&report);
        let json = JsonReporter::render(&report).expect("typed report should serialize");
        let junit = JunitReporter::render(&report).expect("typed report should serialize as JUnit");
        let markdown =
            MarkdownReporter::render(&report).expect("typed report should serialize as Markdown");
        assert_eq!(
            markdown,
            include_str!("../../tests/fixtures/contracts/incomplete-report.md")
        );
        for rendered in [&human, &json, &junit, &markdown] {
            assert!(rendered.contains("MCP-SCHEMA-005"));
            assert!(rendered.contains("compile_construction"));
            assert!(rendered.contains("schema_evaluation_steps"));
            assert!(rendered.contains("100001"));
            assert!(rendered.contains("100000"));
        }
        assert!(human.contains("INCOMPLETE schema.contracts"));
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_stable_report(&json_value);
        assert_eq!(json_value["checks"][0]["state"], "performed");
        assert_eq!(json_value["checks"][0]["outcome"], "incomplete");
        assert_common_junit_document(&junit, 1, 0, 1);
        assert!(junit.contains("<skipped message=\"incomplete\">"));

        let invalid = Finding::schema_contract_invalid(
            SupportedRevision::CURRENT,
            location,
            RuleViolation::InvalidDraft202012 { error_count: 1 },
        );
        let mixed = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::SchemaContracts,
                Requirement::Required,
                vec![incomplete, invalid],
            )],
        )
        .expect("mixed genuine and incomplete evidence should satisfy the report contract");
        assert_eq!(mixed.outcome(), OverallOutcome::Failed);
        assert_eq!(mixed.summary().failed, 1);
        assert_eq!(mixed.summary().incomplete, 0);
        assert_eq!(
            mixed.primary_diagnosis().unwrap().findings()[0].code(),
            FindingCode::SchemaContractInvalid
        );
        assert!(
            JsonReporter::render(&mixed)
                .unwrap()
                .contains("MCP-SCHEMA-005")
        );
    }

    #[test]
    fn unsupported_revision_evidence_is_actionable_without_echoing_advertisements() {
        use crate::contract::protocol::{RevisionSelection, select_server_revision};

        let sentinel = "synthetic-private-revision-never-report-7f2c";
        let RevisionSelection::Unsupported(advertisement) = select_server_revision(
            ["2025-11-25", "1900-01-01", sentinel],
            DiagnosticLimits::DEFAULTS.values().protocol_revisions,
        ) else {
            panic!("a legacy and unknown advertisement must not negotiate")
        };
        let finding = Finding::unsupported_revision(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Server).field(LocationField::SupportedVersions),
            advertisement,
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::ProtocolRevision,
                Requirement::Required,
                vec![finding],
            )],
        )
        .expect("unsupported revision report should satisfy the contract");
        let rendered = format!(
            "{}\n{}\n{}\n{}",
            HumanReporter::render(&report),
            JsonReporter::render(&report).expect("typed report should serialize"),
            JunitReporter::render(&report).expect("typed report should serialize as JUnit"),
            MarkdownReporter::render(&report).expect("typed report should serialize as Markdown")
        );

        assert_eq!(report.exit_status(), ExitStatus::DiagnosticFailure);
        assert!(rendered.contains("MCP-PROTOCOL-002"));
        assert!(rendered.contains("2026-07-28"));
        assert!(rendered.contains("recognized_legacy"));
        assert!(
            !rendered.contains(sentinel),
            "unsupported revision evidence must not echo opaque advertisements"
        );
    }

    #[test]
    fn a_report_with_no_performed_checks_is_incomplete() {
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::skipped(
                CheckId::RuntimeTools,
                Requirement::Optional,
                SkipReason::NotAuthorized,
            )],
        )
        .expect("all-skipped report should be structurally valid");

        assert_eq!(report.outcome(), OverallOutcome::Incomplete);
        assert_eq!(report.exit_status(), ExitStatus::Incomplete);
    }

    #[test]
    fn a_report_without_a_required_check_is_incomplete() {
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::RuntimeTools,
                Requirement::Optional,
                Vec::new(),
            )],
        )
        .expect("optional-only report should be structurally valid");

        assert_eq!(report.outcome(), OverallOutcome::Incomplete);
        assert_eq!(report.exit_status(), ExitStatus::Incomplete);
    }

    #[test]
    fn warnings_do_not_fail_but_critical_findings_do() {
        let warning = Finding::deprecated_protocol_feature(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Server),
        );
        let warning_report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::DiscoveryCatalogs,
                Requirement::Required,
                vec![warning],
            )],
        )
        .expect("warning report should satisfy the contract");
        assert_eq!(warning_report.outcome(), OverallOutcome::Passed);
        assert!(warning_report.primary_diagnosis().is_none());
        assert!(warning_report.independent_findings().is_empty());

        let critical = Finding::cleanup_failed(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Process),
        );
        let critical_report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![CheckResult::performed(
                CheckId::RuntimeTools,
                Requirement::Required,
                vec![critical],
            )],
        )
        .expect("critical report should satisfy the contract");
        assert_eq!(critical_report.outcome(), OverallOutcome::Failed);
        let diagnosis = critical_report
            .primary_diagnosis()
            .expect("an independent-only failure remains the primary fallback");
        assert_eq!(diagnosis.check(), CheckId::RuntimeTools);
        assert_eq!(diagnosis.findings()[0].code(), FindingCode::CleanupFailed);
        assert_eq!(critical_report.independent_findings().len(), 1);
    }

    #[test]
    fn independent_safety_failure_does_not_replace_a_later_actionable_cause() {
        let cleanup = Finding::cleanup_failed(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Process),
        );
        let catalog = Finding::catalog_contract_invalid(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Tools),
            RuleViolation::ExpectedShape {
                expected: ExpectedShape::Array,
                observed: JsonKind::String,
            },
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS,
            vec![
                CheckResult::performed(
                    CheckId::TransportStdio,
                    Requirement::Required,
                    vec![cleanup],
                ),
                CheckResult::performed(
                    CheckId::DiscoveryCatalogs,
                    Requirement::Required,
                    vec![catalog],
                ),
            ],
        )
        .expect("mixed safety and diagnostic failures should satisfy the contract");

        let diagnosis = report
            .primary_diagnosis()
            .expect("the actionable catalog failure should be primary");
        assert_eq!(diagnosis.check(), CheckId::DiscoveryCatalogs);
        assert_eq!(
            diagnosis.findings()[0].code(),
            FindingCode::CatalogContractInvalid
        );
        assert_eq!(report.independent_findings().len(), 1);
        assert_eq!(
            report.independent_findings()[0].code(),
            FindingCode::CleanupFailed
        );
    }

    #[test]
    fn causal_skips_require_an_earlier_primary_diagnosis() {
        assert_eq!(
            DiagnosticReport::new(
                SupportedRevision::CURRENT,
                DiagnosticLimits::DEFAULTS,
                vec![CheckResult::skipped(
                    CheckId::ProtocolEnvelope,
                    Requirement::Required,
                    SkipReason::PrerequisiteFailed,
                )],
            ),
            Err(ReportContractError::CausalSkipWithoutDiagnosis(
                CheckId::ProtocolEnvelope,
            ))
        );

        let revision_failure = Finding::invalid_revision_value(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Server).field(LocationField::SupportedVersions),
            Sensitive::new(b"synthetic-invalid-revision").redacted(),
        );
        assert_eq!(
            DiagnosticReport::new(
                SupportedRevision::CURRENT,
                DiagnosticLimits::DEFAULTS,
                vec![
                    CheckResult::skipped(
                        CheckId::ProtocolEnvelope,
                        Requirement::Required,
                        SkipReason::PrerequisiteFailed,
                    ),
                    CheckResult::performed(
                        CheckId::ProtocolRevision,
                        Requirement::Required,
                        vec![revision_failure],
                    ),
                ],
            ),
            Err(ReportContractError::CausalSkipPrecedesDiagnosis {
                check: CheckId::ProtocolEnvelope,
                diagnosis: CheckId::ProtocolRevision,
            })
        );
    }

    #[test]
    fn reports_reject_empty_or_duplicate_check_sets() {
        assert_eq!(
            DiagnosticReport::new(
                SupportedRevision::CURRENT,
                DiagnosticLimits::DEFAULTS,
                Vec::new(),
            ),
            Err(ReportContractError::NoChecks)
        );
        assert_eq!(
            DiagnosticReport::new(
                SupportedRevision::CURRENT,
                DiagnosticLimits::DEFAULTS,
                vec![passing_revision_check(), passing_revision_check()],
            ),
            Err(ReportContractError::DuplicateCheck(
                CheckId::ProtocolRevision
            ))
        );
    }

    #[test]
    fn reports_reject_more_findings_than_the_declared_limit() {
        let base = DiagnosticLimits::DEFAULTS.values();
        let limits = DiagnosticLimits::try_from_values(LimitValues {
            report_findings: 1,
            ..base
        })
        .expect("a one-finding synthetic report limit is valid");
        let findings = [0, 1]
            .into_iter()
            .map(|index| {
                Finding::schema_contract_invalid(
                    SupportedRevision::CURRENT,
                    Location::root(LocationField::Tools).index(index),
                    RuleViolation::InvalidDraft202012 { error_count: 1 },
                )
            })
            .collect();

        assert_eq!(
            DiagnosticReport::new(
                SupportedRevision::CURRENT,
                limits,
                vec![CheckResult::performed(
                    CheckId::SchemaContracts,
                    Requirement::Required,
                    findings,
                )],
            ),
            Err(ReportContractError::TooManyFindings {
                observed: 2,
                maximum: 1,
            })
        );
    }

    #[test]
    fn exit_code_assignments_are_stable_and_non_overlapping() {
        assert_eq!(ExitStatus::Success.code(), 0);
        assert_eq!(ExitStatus::DiagnosticFailure.code(), 1);
        assert_eq!(ExitStatus::InvocationError.code(), 2);
        assert_eq!(ExitStatus::Incomplete.code(), 3);
        assert_eq!(ExitStatus::InternalError.code(), 4);
    }
}
