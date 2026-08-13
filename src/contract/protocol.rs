use std::fmt;

use serde::Serialize;

use super::limits::{LimitKind, LimitViolation};
use super::redaction::RedactedValue;

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ProtocolEra {
    Legacy,
    Modern,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RevisionSupport {
    Supported,
    RecognizedUnsupported,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum NegotiationStyle {
    InitializeHandshake,
    PerRequestMetadata,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum KnownRevision {
    V2024_11_05,
    V2025_03_26,
    V2025_06_18,
    V2025_11_25,
    V2026_07_28,
}

impl KnownRevision {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::V2024_11_05 => "2024-11-05",
            Self::V2025_03_26 => "2025-03-26",
            Self::V2025_06_18 => "2025-06-18",
            Self::V2025_11_25 => "2025-11-25",
            Self::V2026_07_28 => "2026-07-28",
        }
    }

    pub(super) const fn parse(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"2024-11-05" => Some(Self::V2024_11_05),
            b"2025-03-26" => Some(Self::V2025_03_26),
            b"2025-06-18" => Some(Self::V2025_06_18),
            b"2025-11-25" => Some(Self::V2025_11_25),
            b"2026-07-28" => Some(Self::V2026_07_28),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SupportedRevision {
    V2025_06_18,
    V2025_11_25,
    V2026_07_28,
}

impl SupportedRevision {
    pub(crate) const CURRENT: Self = Self::V2026_07_28;

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::V2025_06_18 => "2025-06-18",
            Self::V2025_11_25 => "2025-11-25",
            Self::V2026_07_28 => "2026-07-28",
        }
    }

    pub(crate) const fn uses_initialize(self) -> bool {
        !matches!(self, Self::V2026_07_28)
    }

    pub(super) const fn known(self) -> KnownRevision {
        match self {
            Self::V2025_06_18 => KnownRevision::V2025_06_18,
            Self::V2025_11_25 => KnownRevision::V2025_11_25,
            Self::V2026_07_28 => KnownRevision::V2026_07_28,
        }
    }
}

impl fmt::Display for SupportedRevision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for SupportedRevision {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct RevisionMatrixEntry {
    pub(super) revision: KnownRevision,
    pub(super) era: ProtocolEra,
    pub(super) support: RevisionSupport,
    pub(super) negotiation: NegotiationStyle,
}

pub(super) const REVISION_MATRIX: [RevisionMatrixEntry; 5] = [
    RevisionMatrixEntry {
        revision: KnownRevision::V2026_07_28,
        era: ProtocolEra::Modern,
        support: RevisionSupport::Supported,
        negotiation: NegotiationStyle::PerRequestMetadata,
    },
    RevisionMatrixEntry {
        revision: KnownRevision::V2025_11_25,
        era: ProtocolEra::Legacy,
        support: RevisionSupport::Supported,
        negotiation: NegotiationStyle::InitializeHandshake,
    },
    RevisionMatrixEntry {
        revision: KnownRevision::V2025_06_18,
        era: ProtocolEra::Legacy,
        support: RevisionSupport::Supported,
        negotiation: NegotiationStyle::InitializeHandshake,
    },
    RevisionMatrixEntry {
        revision: KnownRevision::V2025_03_26,
        era: ProtocolEra::Legacy,
        support: RevisionSupport::RecognizedUnsupported,
        negotiation: NegotiationStyle::InitializeHandshake,
    },
    RevisionMatrixEntry {
        revision: KnownRevision::V2024_11_05,
        era: ProtocolEra::Legacy,
        support: RevisionSupport::RecognizedUnsupported,
        negotiation: NegotiationStyle::InitializeHandshake,
    },
];

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct RevisionDate {
    year: u16,
    month: u8,
    day: u8,
}

impl fmt::Display for RevisionDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum RevisionClaim {
    Supported(SupportedRevision),
    KnownUnsupported(KnownRevision),
    UnknownDate(RevisionDate),
    Opaque(RedactedValue),
}

impl RevisionClaim {
    #[cfg(test)]
    fn classification(self) -> &'static str {
        match self {
            Self::Supported(_) => "supported_modern",
            Self::KnownUnsupported(_) => "recognized_legacy",
            Self::UnknownDate(_) => "unknown_date",
            Self::Opaque(_) => "opaque",
        }
    }
}

pub(super) fn classify_revision(value: &str) -> RevisionClaim {
    match value {
        "2026-07-28" => RevisionClaim::Supported(SupportedRevision::V2026_07_28),
        "2025-11-25" => RevisionClaim::KnownUnsupported(KnownRevision::V2025_11_25),
        "2025-06-18" => RevisionClaim::KnownUnsupported(KnownRevision::V2025_06_18),
        "2025-03-26" => RevisionClaim::KnownUnsupported(KnownRevision::V2025_03_26),
        "2024-11-05" => RevisionClaim::KnownUnsupported(KnownRevision::V2024_11_05),
        _ => parse_date_revision(value)
            .map(RevisionClaim::UnknownDate)
            .unwrap_or_else(|| RevisionClaim::Opaque(RedactedValue::new(value.len()))),
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct RevisionAdvertisementSummary {
    offered: usize,
    recognized_legacy: usize,
    unknown_date: usize,
    opaque: usize,
}

impl RevisionAdvertisementSummary {
    pub(super) const fn offered(self) -> usize {
        self.offered
    }

    pub(super) const fn recognized_legacy(self) -> usize {
        self.recognized_legacy
    }

    pub(super) const fn unknown_date(self) -> usize {
        self.unknown_date
    }

    pub(super) const fn opaque(self) -> usize {
        self.opaque
    }

    fn observe(&mut self, claim: RevisionClaim) {
        self.offered += 1;
        match claim {
            RevisionClaim::Supported(_) => {}
            RevisionClaim::KnownUnsupported(_) => self.recognized_legacy += 1,
            RevisionClaim::UnknownDate(_) => self.unknown_date += 1,
            RevisionClaim::Opaque(_) => self.opaque += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum RevisionSelection {
    Selected(SupportedRevision),
    Unsupported(RevisionAdvertisementSummary),
    LimitExceeded(LimitViolation),
}

pub(super) fn select_server_revision<'a, I>(
    advertised: I,
    maximum_revisions: u64,
) -> RevisionSelection
where
    I: IntoIterator<Item = &'a str>,
{
    let mut summary = RevisionAdvertisementSummary::default();

    for (index, value) in advertised.into_iter().enumerate() {
        let observed = u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1);
        if observed > maximum_revisions {
            let violation =
                LimitViolation::new(LimitKind::ProtocolRevisions, observed, maximum_revisions)
                    .expect("the observed revision count is above the checked maximum");
            return RevisionSelection::LimitExceeded(violation);
        }
        let claim = classify_revision(value);
        if let RevisionClaim::Supported(revision) = claim {
            return RevisionSelection::Selected(revision);
        }
        summary.observe(claim);
    }

    RevisionSelection::Unsupported(summary)
}

fn parse_date_revision(value: &str) -> Option<RevisionDate> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }

    let year = parse_digits(&bytes[0..4])? as u16;
    let month = parse_digits(&bytes[5..7])? as u8;
    let day = parse_digits(&bytes[8..10])? as u8;
    let maximum_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return None,
    };

    if year == 0 || day == 0 || day > maximum_day {
        return None;
    }

    Some(RevisionDate { year, month, day })
}

fn parse_digits(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0_u32, |value, byte| {
        byte.is_ascii_digit()
            .then(|| value * 10 + u32::from(byte - b'0'))
    })
}

const fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::{
        KnownRevision, NegotiationStyle, ProtocolEra, REVISION_MATRIX, RevisionSelection,
        RevisionSupport, SupportedRevision, classify_revision, select_server_revision,
    };
    use crate::contract::limits::{DiagnosticLimits, LimitKind};

    fn revision_limit() -> u64 {
        DiagnosticLimits::M1_DEFAULTS.values().protocol_revisions
    }

    #[derive(Debug, Deserialize)]
    struct RevisionCase {
        input: String,
        expected: String,
    }

    #[test]
    fn official_revision_matrix_supports_explicit_selected_legacy_revisions() {
        let revisions: Vec<_> = REVISION_MATRIX
            .iter()
            .map(|entry| entry.revision.as_str())
            .collect();

        assert_eq!(
            revisions,
            [
                "2026-07-28",
                "2025-11-25",
                "2025-06-18",
                "2025-03-26",
                "2024-11-05",
            ]
        );
        assert_eq!(REVISION_MATRIX[0].era, ProtocolEra::Modern);
        assert_eq!(REVISION_MATRIX[0].support, RevisionSupport::Supported);
        assert_eq!(
            REVISION_MATRIX[0].negotiation,
            NegotiationStyle::PerRequestMetadata
        );

        for entry in &REVISION_MATRIX[1..3] {
            assert_eq!(entry.era, ProtocolEra::Legacy);
            assert_eq!(entry.support, RevisionSupport::Supported);
            assert_eq!(entry.negotiation, NegotiationStyle::InitializeHandshake);
            assert_ne!(entry.revision, KnownRevision::V2026_07_28);
        }
        for entry in &REVISION_MATRIX[3..] {
            assert_eq!(entry.era, ProtocolEra::Legacy);
            assert_eq!(entry.support, RevisionSupport::RecognizedUnsupported);
            assert_eq!(entry.negotiation, NegotiationStyle::InitializeHandshake);
        }
    }

    #[test]
    fn synthetic_revision_fixture_has_safe_explicit_classifications() {
        let cases: Vec<RevisionCase> = serde_json::from_str(include_str!(
            "../../tests/fixtures/contracts/revision-cases.json"
        ))
        .expect("synthetic revision cases should be valid JSON");

        for (index, case) in cases.iter().enumerate() {
            assert_eq!(
                classify_revision(&case.input).classification(),
                case.expected,
                "synthetic revision case {index} should retain its classification"
            );
        }
    }

    #[test]
    fn selection_accepts_the_current_revision_without_legacy_fallback() {
        assert_eq!(
            select_server_revision(["2025-11-25", "2026-07-28"], revision_limit()),
            RevisionSelection::Selected(SupportedRevision::CURRENT)
        );

        let RevisionSelection::Unsupported(summary) = select_server_revision(
            ["2025-11-25", "2025-06-18", "1900-01-01", "draft"],
            revision_limit(),
        ) else {
            panic!("legacy-only advertisement must not negotiate a fallback")
        };

        assert_eq!(summary.offered(), 4);
        assert_eq!(summary.recognized_legacy(), 2);
        assert_eq!(summary.unknown_date(), 1);
        assert_eq!(summary.opaque(), 1);
    }

    #[test]
    fn an_empty_advertisement_is_unsupported_not_successful() {
        assert_eq!(
            select_server_revision(std::iter::empty(), revision_limit()),
            RevisionSelection::Unsupported(Default::default())
        );
    }

    #[test]
    fn revision_selection_stops_at_its_limit_even_for_an_infinite_source() {
        let maximum = revision_limit();
        let RevisionSelection::LimitExceeded(violation) =
            select_server_revision(std::iter::repeat("2025-11-25"), maximum)
        else {
            panic!("an infinite advertisement must stop at the revision limit")
        };

        assert_eq!(violation.kind(), LimitKind::ProtocolRevisions);
        assert_eq!(violation.maximum(), maximum);
        assert_eq!(violation.observed(), maximum + 1);
    }
}
