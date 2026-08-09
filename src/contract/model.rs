use std::fmt;

use super::limits::LimitViolation;
use super::protocol::{RevisionAdvertisementSummary, SupportedRevision};
use super::redaction::RedactedValue;

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum CheckId {
    ProtocolRevision,
    ProtocolEnvelope,
    DiscoveryCatalogs,
    SchemaContracts,
    RuntimeTools,
}

impl CheckId {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ProtocolRevision => "protocol.revision",
            Self::ProtocolEnvelope => "protocol.envelope",
            Self::DiscoveryCatalogs => "discovery.catalogs",
            Self::SchemaContracts => "schema.contracts",
            Self::RuntimeTools => "runtime.tools",
        }
    }
}

impl fmt::Display for CheckId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Requirement {
    Required,
    Optional,
}

impl Requirement {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum SkipReason {
    NotAuthorized,
    NotAdvertised,
    UnsupportedRevision,
    PrerequisiteFailed,
    LimitReached,
    NotApplicable,
}

impl SkipReason {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::NotAuthorized => "not_authorized",
            Self::NotAdvertised => "not_advertised",
            Self::UnsupportedRevision => "unsupported_revision",
            Self::PrerequisiteFailed => "prerequisite_failed",
            Self::LimitReached => "limit_reached",
            Self::NotApplicable => "not_applicable",
        }
    }

    pub(super) const fn description(self) -> &'static str {
        match self {
            Self::NotAuthorized => "active behavior was not authorized",
            Self::NotAdvertised => "the capability was not advertised",
            Self::UnsupportedRevision => "the protocol revision is unsupported",
            Self::PrerequisiteFailed => "a required prerequisite did not pass",
            Self::LimitReached => "a safety limit prevented the check",
            Self::NotApplicable => "the check does not apply to this target",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Severity {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }

    pub(super) const fn is_failure(self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FindingCode {
    ProtocolRevisionConfirmed,
    UnsupportedProtocolRevision,
    InvalidProtocolRevisionValue,
    DeprecatedProtocolFeature,
    LimitExceeded,
    CleanupFailed,
    SchemaContractInvalid,
}

impl FindingCode {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ProtocolRevisionConfirmed => "MCP-PROTOCOL-001",
            Self::UnsupportedProtocolRevision => "MCP-PROTOCOL-002",
            Self::InvalidProtocolRevisionValue => "MCP-PROTOCOL-003",
            Self::DeprecatedProtocolFeature => "MCP-PROTOCOL-004",
            Self::LimitExceeded => "MCP-LIMIT-001",
            Self::CleanupFailed => "MCP-SAFETY-001",
            Self::SchemaContractInvalid => "MCP-SCHEMA-001",
        }
    }

    pub(super) const fn severity(self) -> Severity {
        match self {
            Self::ProtocolRevisionConfirmed => Severity::Info,
            Self::DeprecatedProtocolFeature => Severity::Warning,
            Self::UnsupportedProtocolRevision
            | Self::InvalidProtocolRevisionValue
            | Self::LimitExceeded
            | Self::SchemaContractInvalid => Severity::Error,
            Self::CleanupFailed => Severity::Critical,
        }
    }

    pub(super) const fn title(self) -> &'static str {
        match self {
            Self::ProtocolRevisionConfirmed => "The requested protocol revision is supported.",
            Self::UnsupportedProtocolRevision => {
                "The server does not advertise the required protocol revision."
            }
            Self::InvalidProtocolRevisionValue => {
                "The protocol revision value is missing or has the wrong JSON type."
            }
            Self::DeprecatedProtocolFeature => {
                "The server advertises a feature deprecated by this protocol revision."
            }
            Self::LimitExceeded => "A configured diagnostic safety limit was exceeded.",
            Self::CleanupFailed => "The managed target could not be fully cleaned up.",
            Self::SchemaContractInvalid => "An advertised JSON Schema contract is invalid.",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum LocationField {
    Request,
    Meta,
    ProtocolVersion,
    Server,
    SupportedVersions,
    Tools,
    InputSchema,
    Required,
    Process,
}

impl LocationField {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Meta => "_meta",
            Self::ProtocolVersion => "io.modelcontextprotocol/protocolVersion",
            Self::Server => "server",
            Self::SupportedVersions => "supportedVersions",
            Self::Tools => "tools",
            Self::InputSchema => "inputSchema",
            Self::Required => "required",
            Self::Process => "process",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum LocationSegment {
    Field(LocationField),
    Index(usize),
}

/// A location made only from trusted field identifiers and numeric indices.
/// It cannot contain paths, payloads, server-provided names, or credentials.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Location {
    segments: Vec<LocationSegment>,
}

impl Location {
    pub(super) fn root(field: LocationField) -> Self {
        Self {
            segments: vec![LocationSegment::Field(field)],
        }
    }

    pub(super) fn field(mut self, field: LocationField) -> Self {
        self.segments.push(LocationSegment::Field(field));
        self
    }

    pub(super) fn index(mut self, index: usize) -> Self {
        self.segments.push(LocationSegment::Index(index));
        self
    }
}

impl fmt::Display for Location {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, segment) in self.segments.iter().enumerate() {
            match segment {
                LocationSegment::Field(field) => {
                    if index > 0 {
                        formatter.write_str(".")?;
                    }
                    formatter.write_str(field.as_str())?;
                }
                LocationSegment::Index(value) => write!(formatter, "[{value}]")?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FindingEvidence {
    None,
    RevisionAdvertisement(RevisionAdvertisementSummary),
    RedactedObservation(RedactedValue),
    LimitViolation(LimitViolation),
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Finding {
    code: FindingCode,
    revision: SupportedRevision,
    location: Location,
    evidence: FindingEvidence,
}

impl Finding {
    pub(super) fn revision_confirmed(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::ProtocolRevisionConfirmed,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn unsupported_revision(
        revision: SupportedRevision,
        location: Location,
        advertisement: RevisionAdvertisementSummary,
    ) -> Self {
        Self::new(
            FindingCode::UnsupportedProtocolRevision,
            revision,
            location,
            FindingEvidence::RevisionAdvertisement(advertisement),
        )
    }

    pub(super) fn invalid_revision_value(
        revision: SupportedRevision,
        location: Location,
        observation: RedactedValue,
    ) -> Self {
        Self::new(
            FindingCode::InvalidProtocolRevisionValue,
            revision,
            location,
            FindingEvidence::RedactedObservation(observation),
        )
    }

    pub(super) fn deprecated_protocol_feature(
        revision: SupportedRevision,
        location: Location,
    ) -> Self {
        Self::new(
            FindingCode::DeprecatedProtocolFeature,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn limit_exceeded(
        revision: SupportedRevision,
        location: Location,
        violation: LimitViolation,
    ) -> Self {
        Self::new(
            FindingCode::LimitExceeded,
            revision,
            location,
            FindingEvidence::LimitViolation(violation),
        )
    }

    pub(super) fn cleanup_failed(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::CleanupFailed,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn schema_contract_invalid(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::SchemaContractInvalid,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    fn new(
        code: FindingCode,
        revision: SupportedRevision,
        location: Location,
        evidence: FindingEvidence,
    ) -> Self {
        Self {
            code,
            revision,
            location,
            evidence,
        }
    }

    pub(super) const fn code(&self) -> FindingCode {
        self.code
    }

    pub(super) const fn severity(&self) -> Severity {
        self.code.severity()
    }

    pub(super) const fn revision(&self) -> SupportedRevision {
        self.revision
    }

    pub(super) fn location(&self) -> &Location {
        &self.location
    }

    pub(super) fn evidence(&self) -> &FindingEvidence {
        &self.evidence
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum CheckOutcome {
    Passed,
    Warning,
    Failed,
}

impl CheckOutcome {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }

    pub(super) const fn human_label(self) -> &'static str {
        match self {
            Self::Passed => "PASS",
            Self::Warning => "WARN",
            Self::Failed => "FAIL",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum CheckState {
    Performed { findings: Vec<Finding> },
    Skipped { reason: SkipReason },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct CheckResult {
    id: CheckId,
    requirement: Requirement,
    state: CheckState,
}

impl CheckResult {
    pub(super) fn performed(
        id: CheckId,
        requirement: Requirement,
        mut findings: Vec<Finding>,
    ) -> Self {
        findings.sort();
        findings.dedup();
        Self {
            id,
            requirement,
            state: CheckState::Performed { findings },
        }
    }

    pub(super) const fn skipped(id: CheckId, requirement: Requirement, reason: SkipReason) -> Self {
        Self {
            id,
            requirement,
            state: CheckState::Skipped { reason },
        }
    }

    pub(super) const fn id(&self) -> CheckId {
        self.id
    }

    pub(super) const fn requirement(&self) -> Requirement {
        self.requirement
    }

    pub(super) fn findings(&self) -> Option<&[Finding]> {
        match &self.state {
            CheckState::Performed { findings } => Some(findings),
            CheckState::Skipped { .. } => None,
        }
    }

    pub(super) const fn skip_reason(&self) -> Option<SkipReason> {
        match self.state {
            CheckState::Performed { .. } => None,
            CheckState::Skipped { reason } => Some(reason),
        }
    }

    pub(super) fn outcome(&self) -> Option<CheckOutcome> {
        let findings = self.findings()?;
        if findings
            .iter()
            .any(|finding| finding.severity().is_failure())
        {
            Some(CheckOutcome::Failed)
        } else if findings
            .iter()
            .any(|finding| finding.severity() == Severity::Warning)
        {
            Some(CheckOutcome::Warning)
        } else {
            Some(CheckOutcome::Passed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CheckId, CheckOutcome, CheckResult, Finding, FindingCode, Location, LocationField,
        Requirement, Severity, SkipReason,
    };
    use crate::contract::protocol::SupportedRevision;

    #[test]
    fn finding_codes_own_their_stable_severity_policy() {
        let cases = [
            (
                FindingCode::ProtocolRevisionConfirmed,
                "MCP-PROTOCOL-001",
                Severity::Info,
            ),
            (
                FindingCode::UnsupportedProtocolRevision,
                "MCP-PROTOCOL-002",
                Severity::Error,
            ),
            (
                FindingCode::InvalidProtocolRevisionValue,
                "MCP-PROTOCOL-003",
                Severity::Error,
            ),
            (
                FindingCode::DeprecatedProtocolFeature,
                "MCP-PROTOCOL-004",
                Severity::Warning,
            ),
            (FindingCode::LimitExceeded, "MCP-LIMIT-001", Severity::Error),
            (
                FindingCode::CleanupFailed,
                "MCP-SAFETY-001",
                Severity::Critical,
            ),
            (
                FindingCode::SchemaContractInvalid,
                "MCP-SCHEMA-001",
                Severity::Error,
            ),
        ];

        for (code, stable_code, severity) in cases {
            assert_eq!(code.as_str(), stable_code);
            assert_eq!(code.severity(), severity);
            assert!(!code.title().is_empty());
        }
    }

    #[test]
    fn check_ids_and_skip_reasons_have_stable_report_values() {
        let check_ids = [
            (CheckId::ProtocolRevision, "protocol.revision"),
            (CheckId::ProtocolEnvelope, "protocol.envelope"),
            (CheckId::DiscoveryCatalogs, "discovery.catalogs"),
            (CheckId::SchemaContracts, "schema.contracts"),
            (CheckId::RuntimeTools, "runtime.tools"),
        ];
        let skip_reasons = [
            (SkipReason::NotAuthorized, "not_authorized"),
            (SkipReason::NotAdvertised, "not_advertised"),
            (SkipReason::UnsupportedRevision, "unsupported_revision"),
            (SkipReason::PrerequisiteFailed, "prerequisite_failed"),
            (SkipReason::LimitReached, "limit_reached"),
            (SkipReason::NotApplicable, "not_applicable"),
        ];

        for (check, expected) in check_ids {
            assert_eq!(check.as_str(), expected);
        }
        for (reason, expected) in skip_reasons {
            assert_eq!(reason.as_str(), expected);
            assert!(!reason.description().is_empty());
        }
    }

    #[test]
    fn locations_contain_only_trusted_structure() {
        let location = Location::root(LocationField::Tools)
            .index(3)
            .field(LocationField::InputSchema)
            .field(LocationField::Required);

        assert_eq!(location.to_string(), "tools[3].inputSchema.required");
    }

    #[test]
    fn performed_warning_and_skipped_checks_remain_distinct() {
        let warning = Finding::deprecated_protocol_feature(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Server),
        );
        let performed = CheckResult::performed(
            CheckId::DiscoveryCatalogs,
            Requirement::Required,
            vec![warning],
        );
        let skipped = CheckResult::skipped(
            CheckId::RuntimeTools,
            Requirement::Optional,
            SkipReason::NotAuthorized,
        );

        assert_eq!(performed.outcome(), Some(CheckOutcome::Warning));
        assert!(performed.skip_reason().is_none());
        assert!(skipped.outcome().is_none());
        assert_eq!(skipped.skip_reason(), Some(SkipReason::NotAuthorized));
    }
}
