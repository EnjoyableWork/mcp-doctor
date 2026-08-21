use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde_json::Value;

const README: &str = include_str!("../README.md");
const RUBRIC: &str = include_str!("../docs/evaluations/category-audit-v1.md");
const EVALUATION: &str = include_str!("../docs/evaluations/v0.4.0.md");
const ROW_STATES: &str = include_str!("../docs/evaluations/v0.4.0-row-states.json");
const ARITHMETIC: &str = include_str!("../docs/evaluations/v0.4.0-arithmetic.json");
const PASS_JSON: &str = include_str!("../docs/evaluations/artifacts/v0.4.0/passing/report.json");
const PASS_JUNIT: &str =
    include_str!("../docs/evaluations/artifacts/v0.4.0/passing/report.junit.xml");
const PASS_MARKDOWN: &str = include_str!("../docs/evaluations/artifacts/v0.4.0/passing/report.md");
const PASS_BADGE: &str = include_str!("../docs/evaluations/artifacts/v0.4.0/passing/badge.json");
const FAIL_JSON: &str = include_str!("../docs/evaluations/artifacts/v0.4.0/diagnosed/report.json");
const FAIL_JUNIT: &str =
    include_str!("../docs/evaluations/artifacts/v0.4.0/diagnosed/report.junit.xml");
const FAIL_MARKDOWN: &str =
    include_str!("../docs/evaluations/artifacts/v0.4.0/diagnosed/report.md");
const FAIL_BADGE: &str = include_str!("../docs/evaluations/artifacts/v0.4.0/diagnosed/badge.json");

const EXPECTED_ROWS: [&str; 25] = [
    "P1", "P2", "P3", "P4", "P5", "A1", "A2", "A3", "A4", "A5", "S1", "S2", "S3", "S4", "S5", "D1",
    "D2", "D3", "D4", "D5", "T1", "T2", "T3", "T4", "T5",
];

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn required_string<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} must be a string"))
}

fn fixed_points(criterion: &str, state: &str) -> u64 {
    let maximum = match criterion.as_bytes().first() {
        Some(b'P' | b'A') => 5,
        Some(b'S' | b'D') => 4,
        Some(b'T') => 2,
        _ => panic!("unexpected criterion {criterion}"),
    };
    match state {
        "Full" => maximum,
        "Partial" => match maximum {
            5 => 3,
            4 => 2,
            2 => 1,
            _ => unreachable!(),
        },
        "Zero" => 0,
        _ => panic!("unexpected state {state}"),
    }
}

#[test]
fn public_evaluation_is_self_contained_and_has_no_private_provenance() {
    for path in [
        "docs/evaluations/category-audit-v1.md",
        "docs/evaluations/v0.4.0.md",
        "docs/evaluations/v0.4.0-row-states.json",
        "docs/evaluations/v0.4.0-arithmetic.json",
        "docs/evaluations/artifacts/v0.4.0/passing/report.json",
        "docs/evaluations/artifacts/v0.4.0/passing/report.junit.xml",
        "docs/evaluations/artifacts/v0.4.0/passing/report.md",
        "docs/evaluations/artifacts/v0.4.0/passing/badge.json",
        "docs/evaluations/artifacts/v0.4.0/diagnosed/report.json",
        "docs/evaluations/artifacts/v0.4.0/diagnosed/report.junit.xml",
        "docs/evaluations/artifacts/v0.4.0/diagnosed/report.md",
        "docs/evaluations/artifacts/v0.4.0/diagnosed/badge.json",
        "docs/releases/v0.4.0.md",
        "docs/release.md",
    ] {
        assert!(
            repository_root().join(path).is_file(),
            "public evaluation path is missing: {path}"
        );
    }

    for text in [RUBRIC, EVALUATION, ROW_STATES, ARITHMETIC] {
        for forbidden in [
            "<issue id=",
            "\"document_id\"",
            "\"slug\"",
            "/Users/",
            "/private/",
            "/tmp/",
            "C:\\Users\\",
            "scored worksheet URL",
        ] {
            assert!(
                !text.contains(forbidden),
                "public evaluation retained forbidden provenance: {forbidden}"
            );
        }
    }

    assert!(README.contains("[`v0.4.0` evaluation evidence](docs/evaluations/v0.4.0.md)"));
    for local_link in [
        "[Category audit rubric v1](category-audit-v1.md)",
        "[`v0.4.0-row-states.json`](v0.4.0-row-states.json)",
        "[`v0.4.0-arithmetic.json`](v0.4.0-arithmetic.json)",
        "[v0.4.0 release evidence](../releases/v0.4.0.md)",
        "[release integrity guide](../release.md)",
    ] {
        assert!(
            EVALUATION.contains(local_link),
            "evaluation omitted local link {local_link}"
        );
    }
}

#[test]
fn rubric_publishes_every_frozen_state_boundary() {
    for criterion in EXPECTED_ROWS {
        assert_eq!(
            RUBRIC.matches(&format!("### {criterion} —")).count(),
            1,
            "rubric must define {criterion} exactly once"
        );
    }
    assert_eq!(RUBRIC.matches("**Full —").count(), 25);
    assert_eq!(RUBRIC.matches("**Partial —").count(), 25);
    assert_eq!(RUBRIC.matches("**Zero — 0:**").count(), 25);
    assert_eq!(RUBRIC.matches("**Minimum evidence:**").count(), 25);

    for contract in [
        "Freeze the rubric version",
        "lock all rows before calculating points",
        "separate reviewer verifies",
        "single-evaluator and\nuncalibrated",
        "One language-model evaluation does not establish repeatability",
        "current for at most 90 calendar days",
    ] {
        assert!(RUBRIC.contains(contract), "rubric omitted {contract}");
    }
}

#[test]
fn locked_rows_recalculate_to_the_published_result() {
    let states: Value = serde_json::from_str(ROW_STATES).expect("row states must be JSON");
    let rows = states["rows"].as_array().expect("rows must be an array");
    assert_eq!(rows.len(), EXPECTED_ROWS.len());

    let mut seen = BTreeSet::new();
    let mut points_by_row = BTreeMap::new();
    let mut subtotals = BTreeMap::new();

    for (index, row) in rows.iter().enumerate() {
        let criterion = required_string(row, "criterion");
        let state = required_string(row, "state");
        assert_eq!(criterion, EXPECTED_ROWS[index]);
        assert!(seen.insert(criterion));
        assert!(
            row["evidence"]
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "{criterion} needs evidence"
        );
        assert!(!required_string(row, "observation").is_empty());
        assert!(!required_string(row, "rationale").is_empty());
        assert!(
            row["limitations"]
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "{criterion} needs limitations"
        );

        let points = fixed_points(criterion, state);
        points_by_row.insert(criterion, points);
        *subtotals.entry(&criterion[..1]).or_insert(0_u64) += points;
    }

    assert_eq!(seen.len(), 25);
    assert_eq!(subtotals.get("P"), Some(&23));
    assert_eq!(subtotals.get("A"), Some(&16));
    assert_eq!(subtotals.get("S"), Some(&18));
    assert_eq!(subtotals.get("D"), Some(&14));
    assert_eq!(subtotals.get("T"), Some(&9));
    assert_eq!(subtotals.values().sum::<u64>(), 80);

    let arithmetic: Value =
        serde_json::from_str(ARITHMETIC).expect("arithmetic record must be JSON");
    assert_eq!(arithmetic["row_count"], 25);
    assert_eq!(arithmetic["unique_row_count"], 25);
    assert_eq!(arithmetic["classifications_unchanged"], true);
    assert_eq!(arithmetic["arithmetic_agrees"], true);
    assert_eq!(arithmetic["total"], 80);
    assert_eq!(arithmetic["recomputed_total"], 80);
    assert_eq!(
        required_string(&arithmetic, "input_sha256"),
        "3b6ed8d4eb7584e72049a19354592f37b6c2f006a7e2c907358fbc4d674af476"
    );

    let arithmetic_rows = arithmetic["rows"]
        .as_array()
        .expect("arithmetic rows must be an array");
    assert_eq!(arithmetic_rows.len(), 25);
    for row in arithmetic_rows {
        let criterion = required_string(row, "criterion");
        let state = required_string(row, "state");
        let points = row["points"].as_u64().expect("points must be unsigned");
        assert_eq!(points, fixed_points(criterion, state));
        assert_eq!(points_by_row.get(criterion), Some(&points));
    }

    for (category, expected) in [("P", 23), ("A", 16), ("S", 18), ("D", 14), ("T", 9)] {
        assert_eq!(arithmetic["subtotals"][category], expected);
    }
}

#[test]
fn retained_report_artifacts_are_safe_consistent_and_complete() {
    let pass: Value = serde_json::from_str(PASS_JSON).expect("passing report must be JSON");
    let failed: Value = serde_json::from_str(FAIL_JSON).expect("diagnosed report must be JSON");
    let pass_badge: Value = serde_json::from_str(PASS_BADGE).expect("passing badge must be JSON");
    let fail_badge: Value = serde_json::from_str(FAIL_BADGE).expect("diagnosed badge must be JSON");

    for report in [&pass, &failed] {
        assert_eq!(report["schema_version"], "mcp-doctor.report/v1");
        assert_eq!(report["schema_stability"], "stable");
    }
    assert_eq!(pass["outcome"], "passed");
    assert_eq!(pass["exit_code"], 0);
    assert_eq!(pass["primary_diagnosis"], Value::Null);
    assert_eq!(failed["outcome"], "failed");
    assert_eq!(failed["exit_code"], 1);
    assert_eq!(failed["primary_diagnosis"]["check_id"], "protocol.revision");

    assert_eq!(pass_badge.as_object().map(|object| object.len()), Some(4));
    assert_eq!(pass_badge["schemaVersion"], 1);
    assert_eq!(pass_badge["label"], "mcp-doctor");
    assert_eq!(pass_badge["message"], "pass");
    assert_eq!(pass_badge["color"], "brightgreen");
    assert_eq!(fail_badge.as_object().map(|object| object.len()), Some(4));
    assert_eq!(fail_badge["schemaVersion"], 1);
    assert_eq!(fail_badge["label"], "mcp-doctor");
    assert_eq!(fail_badge["message"], "fail");
    assert_eq!(fail_badge["color"], "red");

    assert!(PASS_JUNIT.contains(
        "<testsuites name=\"mcp-doctor\" tests=\"6\" failures=\"0\" errors=\"0\" skipped=\"1\""
    ));
    assert!(PASS_JUNIT.contains("report_outcome=passed\nexit_code=0"));
    assert!(FAIL_JUNIT.contains(
        "<testsuites name=\"mcp-doctor\" tests=\"6\" failures=\"1\" errors=\"0\" skipped=\"3\""
    ));
    assert!(FAIL_JUNIT.contains("report_outcome=failed\nexit_code=1"));
    assert!(FAIL_JUNIT.contains("type=\"MCP-PROTOCOL-002\""));

    assert!(PASS_MARKDOWN.starts_with("<!-- mcp-doctor.markdown/v1 -->\n"));
    assert!(PASS_MARKDOWN.contains("| Outcome | `passed` |"));
    assert!(PASS_MARKDOWN.contains("| Exit | `0` (`success`) |"));
    assert!(FAIL_MARKDOWN.starts_with("<!-- mcp-doctor.markdown/v1 -->\n"));
    assert!(FAIL_MARKDOWN.contains("| Outcome | `failed` |"));
    assert!(FAIL_MARKDOWN.contains("| Exit | `1` (`unsuccessful_result`) |"));
    assert!(FAIL_MARKDOWN.contains("`MCP-PROTOCOL-002` at `server.supportedVersions`"));

    for artifact in [
        PASS_JSON,
        PASS_JUNIT,
        PASS_MARKDOWN,
        PASS_BADGE,
        FAIL_JSON,
        FAIL_JUNIT,
        FAIL_MARKDOWN,
        FAIL_BADGE,
    ] {
        for forbidden in [
            "synthetic-private-revision-never-report-7f2c",
            "synthetic-private-ci-stderr-never-report-7f2c",
            "/Users/",
            "/home/runner/",
            "C:\\Users\\",
        ] {
            assert!(!artifact.contains(forbidden));
        }
    }

    for artifact_link in [
        "(artifacts/v0.4.0/passing/report.json)",
        "(artifacts/v0.4.0/passing/report.junit.xml)",
        "(artifacts/v0.4.0/passing/report.md)",
        "(artifacts/v0.4.0/passing/badge.json)",
        "(artifacts/v0.4.0/diagnosed/report.json)",
        "(artifacts/v0.4.0/diagnosed/report.junit.xml)",
        "(artifacts/v0.4.0/diagnosed/report.md)",
        "(artifacts/v0.4.0/diagnosed/badge.json)",
    ] {
        assert!(EVALUATION.contains(artifact_link));
    }
}

#[test]
fn publication_keeps_claims_scoped_and_evaluator_limits_visible() {
    for disclosure in [
        "standalone and dated",
        "not MCP conformance",
        "No competitor identity was in scope",
        "not a model-portable benchmark",
        "Model identifier | Unavailable",
        "Serving model revision | Unavailable",
        "Repeated-run agreement | Not measured",
        "Cross-provider agreement | Not measured",
        "Comparison identities were frozen as empty",
        "expires after\n90 calendar days on **2026-11-18**",
        "Preserve this page as dated history",
    ] {
        assert!(
            EVALUATION.contains(disclosure),
            "evaluation omitted disclosure {disclosure}"
        );
    }

    for prohibited_claim in [
        "mcp-doctor is certified",
        "official MCP score",
        "fully compliant",
        "verified MCP",
        "de facto standard",
        "best MCP doctor",
    ] {
        assert!(!EVALUATION.contains(prohibited_claim));
    }

    for evidence_identity in [
        "074a62dbbfce5fa417f2b7080d509ebd86433b1f",
        "58b85d666418395a09ffcfa12d0ec941cb6ec88f",
        "b87aff88710cce5a8d4d42b8429041bdda2dd51485c80910808312a0b0e035fe",
        "5e1590e7274e7c05a76f209c82fcfebbb55b5a0296285cf56aad736f5bc2a753",
        "e0bb567ba07f2b1d6bb270968c9462a60c988304254cf102cb20ff8a895b962b",
    ] {
        assert!(
            EVALUATION.contains(evidence_identity),
            "evaluation omitted identity {evidence_identity}"
        );
    }
}
