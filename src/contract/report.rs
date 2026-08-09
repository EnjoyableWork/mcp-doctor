use std::fmt::{self, Write as _};
use std::process::ExitCode;

use serde::Serialize;

use super::limits::{DiagnosticLimits, LimitValues};
use super::model::{
    CheckId, CheckOutcome, CheckResult, Finding, FindingEvidence, Requirement, Severity,
};
use super::protocol::SupportedRevision;
use super::redaction::REDACTION_MARKER;

pub(super) const REPORT_SCHEMA_VERSION: &str = "mcp-doctor.report/v1alpha1";

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
pub(super) enum ExitStatus {
    Success = 0,
    DiagnosticFailure = 1,
    InvocationError = 2,
    Incomplete = 3,
    InternalError = 4,
}

impl ExitStatus {
    pub(super) const fn code(self) -> u8 {
        self as u8
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
    failed: usize,
    required_skipped: usize,
    findings: SeverityCounts,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct DiagnosticReport {
    revision: SupportedRevision,
    limits: DiagnosticLimits,
    checks: Vec<CheckResult>,
    summary: ReportSummary,
    outcome: OverallOutcome,
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
        } else if summary.required == 0 || summary.performed == 0 || summary.required_skipped > 0 {
            OverallOutcome::Incomplete
        } else {
            OverallOutcome::Passed
        };

        Ok(Self {
            revision,
            limits,
            checks,
            summary,
            outcome,
        })
    }

    pub(super) const fn revision(&self) -> SupportedRevision {
        self.revision
    }

    pub(super) const fn limits(&self) -> DiagnosticLimits {
        self.limits
    }

    pub(super) fn checks(&self) -> &[CheckResult] {
        &self.checks
    }

    pub(super) const fn summary(&self) -> ReportSummary {
        self.summary
    }

    pub(super) const fn outcome(&self) -> OverallOutcome {
        self.outcome
    }

    pub(super) const fn exit_status(&self) -> ExitStatus {
        self.outcome.exit_status()
    }
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
        }
    }
}

pub(super) struct HumanReporter;

impl HumanReporter {
    pub(super) fn render(report: &DiagnosticReport) -> String {
        let mut output = String::new();
        writeln!(
            output,
            "mcp-doctor report · MCP {} · {REPORT_SCHEMA_VERSION}",
            report.revision()
        )
        .expect("writing to a String cannot fail");
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
                .expect("writing to a String cannot fail");

                for finding in check.findings().expect("performed check findings exist") {
                    writeln!(
                        output,
                        "      {}  {:<8} {}",
                        finding.code().as_str(),
                        finding.severity().as_str(),
                        finding.location()
                    )
                    .expect("writing to a String cannot fail");
                    writeln!(output, "      {}", finding.code().title())
                        .expect("writing to a String cannot fail");
                    write_human_evidence(&mut output, finding);
                }
            } else {
                let reason = check.skip_reason().expect("skipped check has a reason");
                writeln!(
                    output,
                    "SKIP  {:<22} {} · {}",
                    check.id().as_str(),
                    check.requirement().as_str(),
                    reason.description()
                )
                .expect("writing to a String cannot fail");
            }
        }

        let summary = report.summary();
        output.push('\n');
        writeln!(
            output,
            "{} failed · {} warned · {} passed · {} skipped · outcome {} · exit {}",
            summary.failed,
            summary.warned,
            summary.passed,
            summary.skipped,
            report.outcome().as_str(),
            report.exit_status().code()
        )
        .expect("writing to a String cannot fail");
        output
    }
}

fn write_human_evidence(output: &mut String, finding: &Finding) {
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
            .expect("writing to a String cannot fail");
        }
        FindingEvidence::RedactedObservation(observation) => {
            writeln!(output, "      observed {observation}")
                .expect("writing to a String cannot fail");
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
            .expect("writing to a String cannot fail");
        }
    }
}

pub(super) struct JsonReporter;

impl JsonReporter {
    pub(super) fn render(report: &DiagnosticReport) -> Result<String, serde_json::Error> {
        let envelope = JsonReport::from(report);
        serde_json::to_string_pretty(&envelope).map(|mut output| {
            output.push('\n');
            output
        })
    }
}

#[derive(Serialize)]
struct JsonReport {
    schema_version: &'static str,
    protocol_revision: &'static str,
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
            protocol_revision: report.revision().as_str(),
            outcome: report.outcome().as_str(),
            exit_code: report.exit_status().code(),
            limits: JsonLimits::from(report.limits().values()),
            summary: report.summary(),
            checks: report.checks().iter().map(JsonCheck::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct JsonLimits {
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
    active_cases: u64,
    redirects: u64,
    retries: u64,
    concurrency: u64,
}

impl From<LimitValues> for JsonLimits {
    fn from(values: LimitValues) -> Self {
        Self {
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
            active_cases: values.active_cases,
            redirects: values.redirects,
            retries: values.retries,
            concurrency: values.concurrency,
        }
    }
}

#[derive(Serialize)]
struct JsonCheck {
    id: &'static str,
    requirement: &'static str,
    state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_reason: Option<&'static str>,
    findings: Vec<JsonFinding>,
}

impl From<&CheckResult> for JsonCheck {
    fn from(check: &CheckResult) -> Self {
        Self {
            id: check.id().as_str(),
            requirement: check.requirement().as_str(),
            state: if check.findings().is_some() {
                "performed"
            } else {
                "skipped"
            },
            outcome: check.outcome().map(CheckOutcome::as_str),
            skip_reason: check.skip_reason().map(|reason| reason.as_str()),
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
struct JsonFinding {
    code: &'static str,
    severity: &'static str,
    protocol_revision: &'static str,
    location: String,
    message: &'static str,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiagnosticReport, ExitStatus, HumanReporter, JsonReporter, OverallOutcome,
        ReportContractError,
    };
    use crate::contract::limits::{DiagnosticLimits, LimitKind, LimitValues, LimitViolation};
    use crate::contract::model::{
        CheckId, CheckResult, Finding, Location, LocationField, Requirement, SkipReason,
    };
    use crate::contract::protocol::SupportedRevision;
    use crate::contract::redaction::Sensitive;

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
            DiagnosticLimits::M1_DEFAULTS,
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
    fn human_and_json_reports_match_canonical_synthetic_fixtures() {
        let report = synthetic_failed_report();
        let human = HumanReporter::render(&report);
        let json = JsonReporter::render(&report).expect("typed report should serialize");

        assert_eq!(
            human,
            include_str!("../../tests/fixtures/contracts/failed-report.txt")
        );
        assert_eq!(
            json,
            include_str!("../../tests/fixtures/contracts/failed-report.json")
        );
        assert_eq!(report.outcome(), OverallOutcome::Failed);
        assert_eq!(report.exit_status(), ExitStatus::DiagnosticFailure);
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
            DiagnosticLimits::M1_DEFAULTS,
            vec![CheckResult::performed(
                CheckId::ProtocolEnvelope,
                Requirement::Required,
                vec![finding],
            )],
        )
        .expect("redacted synthetic report should satisfy the contract");
        let rendered = format!(
            "{}\n{}",
            HumanReporter::render(&report),
            JsonReporter::render(&report).expect("typed report should serialize")
        );

        assert!(rendered.contains("[REDACTED]"));
        assert!(rendered.contains(&format!("{} bytes", sentinel.len())));
        assert!(
            !rendered.contains(sentinel),
            "reporters must not reveal a synthetic sentinel"
        );
    }

    #[test]
    fn required_skips_are_incomplete_while_optional_skips_can_pass() {
        let required_skip = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::M1_DEFAULTS,
            vec![
                passing_revision_check(),
                CheckResult::skipped(
                    CheckId::DiscoveryCatalogs,
                    Requirement::Required,
                    SkipReason::PrerequisiteFailed,
                ),
            ],
        )
        .expect("required skip report should be structurally valid");
        assert_eq!(required_skip.outcome(), OverallOutcome::Incomplete);
        assert_eq!(required_skip.exit_status(), ExitStatus::Incomplete);

        let optional_skip = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::M1_DEFAULTS,
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
        );
        let report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::M1_DEFAULTS,
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
    fn unsupported_revision_evidence_is_actionable_without_echoing_advertisements() {
        use crate::contract::protocol::{RevisionSelection, select_server_revision};

        let sentinel = "synthetic-private-revision-never-report-7f2c";
        let RevisionSelection::Unsupported(advertisement) = select_server_revision(
            ["2025-11-25", "1900-01-01", sentinel],
            DiagnosticLimits::M1_DEFAULTS.values().protocol_revisions,
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
            DiagnosticLimits::M1_DEFAULTS,
            vec![CheckResult::performed(
                CheckId::ProtocolRevision,
                Requirement::Required,
                vec![finding],
            )],
        )
        .expect("unsupported revision report should satisfy the contract");
        let rendered = format!(
            "{}\n{}",
            HumanReporter::render(&report),
            JsonReporter::render(&report).expect("typed report should serialize")
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
            DiagnosticLimits::M1_DEFAULTS,
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
            DiagnosticLimits::M1_DEFAULTS,
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
            DiagnosticLimits::M1_DEFAULTS,
            vec![CheckResult::performed(
                CheckId::DiscoveryCatalogs,
                Requirement::Required,
                vec![warning],
            )],
        )
        .expect("warning report should satisfy the contract");
        assert_eq!(warning_report.outcome(), OverallOutcome::Passed);

        let critical = Finding::cleanup_failed(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Process),
        );
        let critical_report = DiagnosticReport::new(
            SupportedRevision::CURRENT,
            DiagnosticLimits::M1_DEFAULTS,
            vec![CheckResult::performed(
                CheckId::RuntimeTools,
                Requirement::Required,
                vec![critical],
            )],
        )
        .expect("critical report should satisfy the contract");
        assert_eq!(critical_report.outcome(), OverallOutcome::Failed);
    }

    #[test]
    fn reports_reject_empty_or_duplicate_check_sets() {
        assert_eq!(
            DiagnosticReport::new(
                SupportedRevision::CURRENT,
                DiagnosticLimits::M1_DEFAULTS,
                Vec::new(),
            ),
            Err(ReportContractError::NoChecks)
        );
        assert_eq!(
            DiagnosticReport::new(
                SupportedRevision::CURRENT,
                DiagnosticLimits::M1_DEFAULTS,
                vec![passing_revision_check(), passing_revision_check()],
            ),
            Err(ReportContractError::DuplicateCheck(
                CheckId::ProtocolRevision
            ))
        );
    }

    #[test]
    fn reports_reject_more_findings_than_the_declared_limit() {
        let base = DiagnosticLimits::M1_DEFAULTS.values();
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
