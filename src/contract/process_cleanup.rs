use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CleanupMechanism {
    ProcessGroup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CleanupScope {
    DirectChildAndOriginalProcessGroup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DetachedDescendants {
    Unverified,
}

/// Observed direct-child cleanup with an explicit process-control boundary.
/// Reaping a direct child never establishes detached-descendant termination.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(crate) struct ProcessCleanupEvidence {
    mechanism: CleanupMechanism,
    scope: CleanupScope,
    process_launches: u64,
    direct_children_reaped: u64,
    descendant_containment: bool,
    detached_descendants: DetachedDescendants,
}

impl ProcessCleanupEvidence {
    pub(crate) fn process_group(process_launches: u64, direct_children_reaped: u64) -> Self {
        assert!(
            process_launches <= 2,
            "STDIO permits at most two finite launches"
        );
        assert!(
            direct_children_reaped <= process_launches,
            "only owned children can be reaped"
        );
        Self {
            mechanism: CleanupMechanism::ProcessGroup,
            scope: CleanupScope::DirectChildAndOriginalProcessGroup,
            process_launches,
            direct_children_reaped,
            descendant_containment: false,
            detached_descendants: DetachedDescendants::Unverified,
        }
    }

    pub(crate) const fn process_launches(self) -> u64 {
        self.process_launches
    }

    pub(crate) const fn direct_children_reaped(self) -> u64 {
        self.direct_children_reaped
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessCleanupEvidence;

    #[test]
    fn observed_reaps_never_imply_descendant_containment() {
        for (launches, reaped) in [(0, 0), (1, 0), (1, 1), (2, 1), (2, 2)] {
            let evidence = ProcessCleanupEvidence::process_group(launches, reaped);
            let json = serde_json::to_value(evidence).expect("typed evidence should serialize");
            assert_eq!(json["process_launches"], launches);
            assert_eq!(json["direct_children_reaped"], reaped);
            assert_eq!(json["descendant_containment"], false);
            assert_eq!(json["detached_descendants"], "unverified");
        }
    }
}
