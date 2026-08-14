use std::fmt;

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum LimitKind {
    StartupTime,
    DiscoveryTime,
    RequestTime,
    ResponseTime,
    ShutdownGrace,
    TotalTime,
    MessageBytes,
    StdoutBytes,
    StderrBytes,
    AggregateOutputBytes,
    ScenarioBytes,
    ActiveInputBytes,
    EnvironmentBytes,
    EndpointBytes,
    ResolutionAddresses,
    ResolutionCount,
    TrustBytes,
    TrustCertificates,
    RequestFields,
    RequestFieldNameBytes,
    RequestFieldValueBytes,
    RequestFieldsBytes,
    ResponseFields,
    ResponseFieldNameBytes,
    ResponseFieldValueBytes,
    ResponseFieldsBytes,
    MessageCount,
    ProtocolRevisions,
    CatalogItems,
    SchemaBytes,
    InstanceBytes,
    SchemaNodes,
    SchemaDepth,
    SchemaRefDepth,
    SchemaEvaluationSteps,
    ValidationErrors,
    ReportFindings,
    ReportBytes,
    ActiveCases,
    GenerationAttempts,
    GenerationCandidates,
    GenerationSteps,
    Redirects,
    Retries,
    Concurrency,
}

impl LimitKind {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::StartupTime => "startup_time",
            Self::DiscoveryTime => "discovery_time",
            Self::RequestTime => "request_time",
            Self::ResponseTime => "response_time",
            Self::ShutdownGrace => "shutdown_grace",
            Self::TotalTime => "total_time",
            Self::MessageBytes => "message_bytes",
            Self::StdoutBytes => "stdout_bytes",
            Self::StderrBytes => "stderr_bytes",
            Self::AggregateOutputBytes => "aggregate_output_bytes",
            Self::ScenarioBytes => "scenario_bytes",
            Self::ActiveInputBytes => "active_input_bytes",
            Self::EnvironmentBytes => "environment_bytes",
            Self::EndpointBytes => "endpoint_bytes",
            Self::ResolutionAddresses => "resolution_addresses",
            Self::ResolutionCount => "resolution_count",
            Self::TrustBytes => "trust_bytes",
            Self::TrustCertificates => "trust_certificates",
            Self::RequestFields => "request_fields",
            Self::RequestFieldNameBytes => "request_field_name_bytes",
            Self::RequestFieldValueBytes => "request_field_value_bytes",
            Self::RequestFieldsBytes => "request_fields_bytes",
            Self::ResponseFields => "response_fields",
            Self::ResponseFieldNameBytes => "response_field_name_bytes",
            Self::ResponseFieldValueBytes => "response_field_value_bytes",
            Self::ResponseFieldsBytes => "response_fields_bytes",
            Self::MessageCount => "message_count",
            Self::ProtocolRevisions => "protocol_revisions",
            Self::CatalogItems => "catalog_items",
            Self::SchemaBytes => "schema_bytes",
            Self::InstanceBytes => "instance_bytes",
            Self::SchemaNodes => "schema_nodes",
            Self::SchemaDepth => "schema_depth",
            Self::SchemaRefDepth => "schema_ref_depth",
            Self::SchemaEvaluationSteps => "schema_evaluation_steps",
            Self::ValidationErrors => "validation_errors",
            Self::ReportFindings => "report_findings",
            Self::ReportBytes => "report_bytes",
            Self::ActiveCases => "active_cases",
            Self::GenerationAttempts => "generation_attempts",
            Self::GenerationCandidates => "generation_candidates",
            Self::GenerationSteps => "generation_steps",
            Self::Redirects => "redirects",
            Self::Retries => "retries",
            Self::Concurrency => "concurrency",
        }
    }

    pub(super) const fn unit(self) -> LimitUnit {
        match self {
            Self::StartupTime
            | Self::DiscoveryTime
            | Self::RequestTime
            | Self::ResponseTime
            | Self::ShutdownGrace
            | Self::TotalTime => LimitUnit::Milliseconds,
            Self::MessageBytes
            | Self::StdoutBytes
            | Self::StderrBytes
            | Self::AggregateOutputBytes
            | Self::ScenarioBytes
            | Self::ActiveInputBytes
            | Self::EnvironmentBytes
            | Self::EndpointBytes
            | Self::TrustBytes
            | Self::RequestFieldNameBytes
            | Self::RequestFieldValueBytes
            | Self::RequestFieldsBytes
            | Self::ResponseFieldNameBytes
            | Self::ResponseFieldValueBytes
            | Self::ResponseFieldsBytes
            | Self::SchemaBytes
            | Self::InstanceBytes
            | Self::ReportBytes => LimitUnit::Bytes,
            Self::MessageCount
            | Self::ResolutionAddresses
            | Self::ResolutionCount
            | Self::TrustCertificates
            | Self::RequestFields
            | Self::ResponseFields
            | Self::ProtocolRevisions
            | Self::CatalogItems
            | Self::SchemaNodes
            | Self::SchemaDepth
            | Self::SchemaRefDepth
            | Self::SchemaEvaluationSteps
            | Self::ValidationErrors
            | Self::ReportFindings
            | Self::ActiveCases
            | Self::GenerationAttempts
            | Self::GenerationCandidates
            | Self::GenerationSteps
            | Self::Redirects
            | Self::Retries
            | Self::Concurrency => LimitUnit::Count,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum LimitUnit {
    Milliseconds,
    Bytes,
    Count,
}

impl LimitUnit {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Milliseconds => "milliseconds",
            Self::Bytes => "bytes",
            Self::Count => "count",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct LimitValues {
    pub(super) startup_ms: u64,
    pub(super) discovery_ms: u64,
    pub(super) request_ms: u64,
    pub(super) response_ms: u64,
    pub(super) shutdown_grace_ms: u64,
    pub(super) total_ms: u64,
    pub(super) message_bytes: u64,
    pub(super) stdout_bytes: u64,
    pub(super) stderr_bytes: u64,
    pub(super) aggregate_output_bytes: u64,
    pub(super) endpoint_bytes: u64,
    pub(super) resolution_addresses: u64,
    pub(super) resolution_count: u64,
    pub(super) trust_bytes: u64,
    pub(super) trust_certificates: u64,
    pub(super) request_fields: u64,
    pub(super) request_field_name_bytes: u64,
    pub(super) request_field_value_bytes: u64,
    pub(super) request_fields_bytes: u64,
    pub(super) response_fields: u64,
    pub(super) response_field_name_bytes: u64,
    pub(super) response_field_value_bytes: u64,
    pub(super) response_fields_bytes: u64,
    pub(super) message_count: u64,
    pub(super) protocol_revisions: u64,
    pub(super) catalog_items: u64,
    pub(super) schema_bytes: u64,
    pub(super) instance_bytes: u64,
    pub(super) schema_nodes: u64,
    pub(super) schema_depth: u64,
    pub(super) schema_ref_depth: u64,
    pub(super) schema_evaluation_steps: u64,
    pub(super) validation_errors: u64,
    pub(super) report_findings: u64,
    pub(super) report_bytes: u64,
    pub(super) active_cases: u64,
    pub(super) generation_attempts: u64,
    pub(super) generation_candidates: u64,
    pub(super) generation_steps: u64,
    pub(super) redirects: u64,
    pub(super) retries: u64,
    pub(super) concurrency: u64,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum DiagnosticLimitProfile {
    #[default]
    Default,
    SlowStart,
}

impl DiagnosticLimitProfile {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::SlowStart => "slow-start",
        }
    }

    pub(super) const fn limits(self) -> DiagnosticLimits {
        match self {
            Self::Default => DiagnosticLimits::M1_DEFAULTS,
            Self::SlowStart => DiagnosticLimits::SLOW_START,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct DiagnosticLimits(LimitValues);

impl DiagnosticLimits {
    pub(super) const M1_DEFAULTS: Self = Self(LimitValues {
        startup_ms: 10_000,
        discovery_ms: 10_000,
        request_ms: 30_000,
        response_ms: 30_000,
        shutdown_grace_ms: 2_000,
        total_ms: 120_000,
        message_bytes: 1_048_576,
        stdout_bytes: 8_388_608,
        stderr_bytes: 1_048_576,
        aggregate_output_bytes: 8_388_608,
        endpoint_bytes: 8_192,
        resolution_addresses: 16,
        resolution_count: 1,
        trust_bytes: 1_048_576,
        trust_certificates: 32,
        request_fields: 64,
        request_field_name_bytes: 256,
        request_field_value_bytes: 8_192,
        request_fields_bytes: 32_768,
        response_fields: 96,
        response_field_name_bytes: 256,
        response_field_value_bytes: 16_384,
        response_fields_bytes: 65_536,
        message_count: 1_024,
        protocol_revisions: 32,
        catalog_items: 10_000,
        schema_bytes: 1_048_576,
        instance_bytes: 1_048_576,
        schema_nodes: 100_000,
        schema_depth: 64,
        schema_ref_depth: 32,
        schema_evaluation_steps: 100_000,
        validation_errors: 100,
        report_findings: 256,
        report_bytes: 4_194_304,
        active_cases: 100,
        generation_attempts: 256,
        generation_candidates: 64,
        generation_steps: 100_000,
        redirects: 0,
        retries: 0,
        concurrency: 1,
    });

    pub(super) const SLOW_START: Self = {
        let mut values = Self::M1_DEFAULTS.0;
        values.startup_ms = 30_000;
        values.discovery_ms = 30_000;
        values.request_ms = 60_000;
        values.response_ms = 60_000;
        values.total_ms = 240_000;
        Self(values)
    };

    pub(super) fn try_from_values(values: LimitValues) -> Result<Self, LimitContractError> {
        for (kind, value) in [
            (LimitKind::StartupTime, values.startup_ms),
            (LimitKind::DiscoveryTime, values.discovery_ms),
            (LimitKind::RequestTime, values.request_ms),
            (LimitKind::ResponseTime, values.response_ms),
            (LimitKind::ShutdownGrace, values.shutdown_grace_ms),
            (LimitKind::TotalTime, values.total_ms),
            (LimitKind::MessageBytes, values.message_bytes),
            (LimitKind::StdoutBytes, values.stdout_bytes),
            (LimitKind::StderrBytes, values.stderr_bytes),
            (
                LimitKind::AggregateOutputBytes,
                values.aggregate_output_bytes,
            ),
            (LimitKind::EndpointBytes, values.endpoint_bytes),
            (LimitKind::ResolutionAddresses, values.resolution_addresses),
            (LimitKind::ResolutionCount, values.resolution_count),
            (LimitKind::TrustBytes, values.trust_bytes),
            (LimitKind::TrustCertificates, values.trust_certificates),
            (LimitKind::RequestFields, values.request_fields),
            (
                LimitKind::RequestFieldNameBytes,
                values.request_field_name_bytes,
            ),
            (
                LimitKind::RequestFieldValueBytes,
                values.request_field_value_bytes,
            ),
            (LimitKind::RequestFieldsBytes, values.request_fields_bytes),
            (LimitKind::ResponseFields, values.response_fields),
            (
                LimitKind::ResponseFieldNameBytes,
                values.response_field_name_bytes,
            ),
            (
                LimitKind::ResponseFieldValueBytes,
                values.response_field_value_bytes,
            ),
            (LimitKind::ResponseFieldsBytes, values.response_fields_bytes),
            (LimitKind::MessageCount, values.message_count),
            (LimitKind::ProtocolRevisions, values.protocol_revisions),
            (LimitKind::CatalogItems, values.catalog_items),
            (LimitKind::SchemaBytes, values.schema_bytes),
            (LimitKind::InstanceBytes, values.instance_bytes),
            (LimitKind::SchemaNodes, values.schema_nodes),
            (LimitKind::SchemaDepth, values.schema_depth),
            (LimitKind::SchemaRefDepth, values.schema_ref_depth),
            (
                LimitKind::SchemaEvaluationSteps,
                values.schema_evaluation_steps,
            ),
            (LimitKind::ValidationErrors, values.validation_errors),
            (LimitKind::ReportFindings, values.report_findings),
            (LimitKind::ReportBytes, values.report_bytes),
            (LimitKind::ActiveCases, values.active_cases),
            (LimitKind::GenerationAttempts, values.generation_attempts),
            (
                LimitKind::GenerationCandidates,
                values.generation_candidates,
            ),
            (LimitKind::GenerationSteps, values.generation_steps),
            (LimitKind::Concurrency, values.concurrency),
        ] {
            if value == 0 {
                return Err(LimitContractError::Zero(kind));
            }
        }

        for (kind, value) in [
            (LimitKind::StartupTime, values.startup_ms),
            (LimitKind::DiscoveryTime, values.discovery_ms),
            (LimitKind::RequestTime, values.request_ms),
            (LimitKind::ResponseTime, values.response_ms),
            (LimitKind::ShutdownGrace, values.shutdown_grace_ms),
        ] {
            if value > values.total_ms {
                return Err(LimitContractError::StageExceedsTotal(kind));
            }
        }

        if values.message_bytes > values.stdout_bytes {
            return Err(LimitContractError::MessageExceedsStdout);
        }
        for (kind, value) in [
            (LimitKind::StdoutBytes, values.stdout_bytes),
            (LimitKind::StderrBytes, values.stderr_bytes),
        ] {
            if value > values.aggregate_output_bytes {
                return Err(LimitContractError::StreamExceedsAggregate(kind));
            }
        }
        for (kind, value) in [
            (LimitKind::SchemaBytes, values.schema_bytes),
            (LimitKind::InstanceBytes, values.instance_bytes),
        ] {
            if value > values.message_bytes {
                return Err(LimitContractError::PayloadExceedsMessage(kind));
            }
        }
        if values.schema_ref_depth > values.schema_depth {
            return Err(LimitContractError::RefDepthExceedsSchemaDepth);
        }
        if values.generation_candidates > values.generation_attempts {
            return Err(LimitContractError::CandidatesExceedAttempts);
        }
        if values.concurrency > values.active_cases {
            return Err(LimitContractError::ConcurrencyExceedsCases);
        }

        Ok(Self(values))
    }

    pub(super) const fn values(self) -> LimitValues {
        self.0
    }
}

impl Default for DiagnosticLimits {
    fn default() -> Self {
        Self::M1_DEFAULTS
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum LimitContractError {
    Zero(LimitKind),
    StageExceedsTotal(LimitKind),
    MessageExceedsStdout,
    StreamExceedsAggregate(LimitKind),
    PayloadExceedsMessage(LimitKind),
    RefDepthExceedsSchemaDepth,
    CandidatesExceedAttempts,
    ConcurrencyExceedsCases,
}

impl fmt::Display for LimitContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero(kind) => write!(formatter, "{} must be greater than zero", kind.as_str()),
            Self::StageExceedsTotal(kind) => {
                write!(formatter, "{} cannot exceed total_time", kind.as_str())
            }
            Self::MessageExceedsStdout => {
                formatter.write_str("message_bytes cannot exceed stdout_bytes")
            }
            Self::StreamExceedsAggregate(kind) => write!(
                formatter,
                "{} cannot exceed aggregate_output_bytes",
                kind.as_str()
            ),
            Self::PayloadExceedsMessage(kind) => {
                write!(formatter, "{} cannot exceed message_bytes", kind.as_str())
            }
            Self::RefDepthExceedsSchemaDepth => {
                formatter.write_str("schema_ref_depth cannot exceed schema_depth")
            }
            Self::CandidatesExceedAttempts => {
                formatter.write_str("generation_candidates cannot exceed generation_attempts")
            }
            Self::ConcurrencyExceedsCases => {
                formatter.write_str("concurrency cannot exceed active_cases")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct LimitViolation {
    kind: LimitKind,
    observed: u64,
    maximum: u64,
}

impl LimitViolation {
    pub(super) fn new(
        kind: LimitKind,
        observed: u64,
        maximum: u64,
    ) -> Result<Self, LimitViolationError> {
        if observed <= maximum {
            return Err(LimitViolationError::NotExceeded);
        }

        Ok(Self {
            kind,
            observed,
            maximum,
        })
    }

    pub(super) const fn kind(self) -> LimitKind {
        self.kind
    }

    pub(super) const fn observed(self) -> u64 {
        self.observed
    }

    pub(super) const fn maximum(self) -> u64 {
        self.maximum
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum LimitViolationError {
    NotExceeded,
}

#[cfg(test)]
mod tests {
    use super::{
        DiagnosticLimitProfile, DiagnosticLimits, LimitContractError, LimitKind, LimitValues,
        LimitViolation, LimitViolationError,
    };

    #[test]
    fn m1_defaults_are_finite_and_internally_consistent() {
        let values = DiagnosticLimits::M1_DEFAULTS.values();

        assert_eq!(
            DiagnosticLimits::try_from_values(values),
            Ok(DiagnosticLimits::M1_DEFAULTS)
        );
        assert_eq!(values.startup_ms, 10_000);
        assert_eq!(values.total_ms, 120_000);
        assert_eq!(values.message_bytes, 1_048_576);
        assert_eq!(values.stdout_bytes, 8_388_608);
        assert_eq!(values.aggregate_output_bytes, 8_388_608);
        assert_eq!(values.schema_bytes, 1_048_576);
        assert_eq!(values.instance_bytes, 1_048_576);
        assert_eq!(values.schema_nodes, 100_000);
        assert_eq!(values.schema_depth, 64);
        assert_eq!(values.schema_ref_depth, 32);
        assert_eq!(values.schema_evaluation_steps, 100_000);
        assert_eq!(values.protocol_revisions, 32);
        assert_eq!(values.report_findings, 256);
        assert_eq!(values.active_cases, 100);
        assert_eq!(values.generation_attempts, 256);
        assert_eq!(values.generation_candidates, 64);
        assert_eq!(values.generation_steps, 100_000);
        assert_eq!(values.redirects, 0);
        assert_eq!(values.retries, 0);
        assert_eq!(values.concurrency, 1);
    }

    #[test]
    fn named_profiles_are_finite_and_slow_start_changes_only_phase_patience() {
        let default = DiagnosticLimitProfile::Default.limits().values();
        let slow_start = DiagnosticLimitProfile::SlowStart.limits().values();

        assert_eq!(DiagnosticLimitProfile::Default.as_str(), "default");
        assert_eq!(DiagnosticLimitProfile::SlowStart.as_str(), "slow-start");
        assert_eq!(
            DiagnosticLimits::try_from_values(slow_start),
            Ok(DiagnosticLimits::SLOW_START)
        );
        assert_eq!(slow_start.startup_ms, 30_000);
        assert_eq!(slow_start.discovery_ms, 30_000);
        assert_eq!(slow_start.request_ms, 60_000);
        assert_eq!(slow_start.response_ms, 60_000);
        assert_eq!(slow_start.shutdown_grace_ms, default.shutdown_grace_ms);
        assert_eq!(slow_start.total_ms, 240_000);
        let synthetic_cold_start_ms = 20_000;
        let synthetic_slow_response_ms = 45_000;
        assert!(synthetic_cold_start_ms > default.startup_ms);
        assert!(synthetic_cold_start_ms <= slow_start.startup_ms);
        assert!(synthetic_slow_response_ms > default.response_ms);
        assert!(synthetic_slow_response_ms <= slow_start.response_ms);
        assert_eq!(
            LimitValues {
                startup_ms: default.startup_ms,
                discovery_ms: default.discovery_ms,
                request_ms: default.request_ms,
                response_ms: default.response_ms,
                total_ms: default.total_ms,
                ..slow_start
            },
            default,
            "a named profile must not increase capacity, cleanup, retry, or concurrency authority"
        );
    }

    #[test]
    fn invalid_limit_relationships_are_rejected() {
        let base = DiagnosticLimits::M1_DEFAULTS.values();

        assert_eq!(
            DiagnosticLimits::try_from_values(LimitValues {
                request_ms: base.total_ms + 1,
                ..base
            }),
            Err(LimitContractError::StageExceedsTotal(
                LimitKind::RequestTime
            ))
        );
        assert_eq!(
            DiagnosticLimits::try_from_values(LimitValues {
                message_bytes: base.stdout_bytes + 1,
                ..base
            }),
            Err(LimitContractError::MessageExceedsStdout)
        );
        assert_eq!(
            DiagnosticLimits::try_from_values(LimitValues {
                stderr_bytes: base.aggregate_output_bytes + 1,
                ..base
            }),
            Err(LimitContractError::StreamExceedsAggregate(
                LimitKind::StderrBytes
            ))
        );
        assert_eq!(
            DiagnosticLimits::try_from_values(LimitValues {
                schema_bytes: base.message_bytes + 1,
                ..base
            }),
            Err(LimitContractError::PayloadExceedsMessage(
                LimitKind::SchemaBytes
            ))
        );
        assert_eq!(
            DiagnosticLimits::try_from_values(LimitValues {
                schema_ref_depth: base.schema_depth + 1,
                ..base
            }),
            Err(LimitContractError::RefDepthExceedsSchemaDepth)
        );
        assert_eq!(
            DiagnosticLimits::try_from_values(LimitValues {
                generation_candidates: base.generation_attempts + 1,
                ..base
            }),
            Err(LimitContractError::CandidatesExceedAttempts)
        );
        assert_eq!(
            DiagnosticLimits::try_from_values(LimitValues {
                concurrency: base.active_cases + 1,
                ..base
            }),
            Err(LimitContractError::ConcurrencyExceedsCases)
        );
        assert_eq!(
            DiagnosticLimits::try_from_values(LimitValues {
                validation_errors: 0,
                ..base
            }),
            Err(LimitContractError::Zero(LimitKind::ValidationErrors))
        );
    }

    #[test]
    fn a_limit_finding_requires_an_actual_excess() {
        assert_eq!(
            LimitViolation::new(LimitKind::MessageBytes, 1_048_576, 1_048_576),
            Err(LimitViolationError::NotExceeded)
        );

        let violation = LimitViolation::new(LimitKind::MessageBytes, 1_048_577, 1_048_576)
            .expect("an observed value above the maximum is a violation");
        assert_eq!(violation.kind(), LimitKind::MessageBytes);
        assert_eq!(violation.observed(), 1_048_577);
        assert_eq!(violation.maximum(), 1_048_576);
    }
}
