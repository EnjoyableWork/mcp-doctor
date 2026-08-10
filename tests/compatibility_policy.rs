use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

const MATRIX: &str = include_str!("compatibility/matrix.json");
const DART_LOCK: &str = include_str!("compatibility/locks/mcp_dart-v2.4.0.pubspec.lock");
const PHP_LOCK: &str = include_str!("compatibility/locks/mcp-sdk-php-v2.0.0.composer.lock");

fn object<'a>(value: &'a Value, context: &str) -> &'a serde_json::Map<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("{context} must be an object"))
}

fn string<'a>(object: &'a serde_json::Map<String, Value>, key: &str) -> &'a str {
    object
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a string"))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[test]
fn compatibility_matrix_is_pinned_scoped_and_evidence_backed() {
    let matrix: Value = serde_json::from_str(MATRIX).expect("matrix must be valid JSON");
    let matrix = object(&matrix, "matrix");

    assert_eq!(
        string(matrix, "schema_version"),
        "mcp-doctor.compatibility/v1"
    );
    assert_eq!(string(matrix, "protocol_revision"), "2026-07-28");
    assert_eq!(string(matrix, "tool_execution"), "forbidden");
    assert_eq!(string(matrix, "transport"), "stdio");
    assert_eq!(string(matrix, "release_position"), "broad-current-revision");

    let runtimes = object(
        matrix.get("runtimes").expect("runtimes must exist"),
        "runtimes",
    );
    assert_eq!(runtimes.len(), 4);
    for (runtime_id, runtime) in runtimes {
        let runtime = object(runtime, runtime_id);
        let image = string(runtime, "image");
        let (_, digest) = image
            .split_once("@sha256:")
            .unwrap_or_else(|| panic!("{runtime_id} image must use a SHA-256 digest"));
        assert!(
            is_lower_hex(digest, 64),
            "{runtime_id} image digest is invalid"
        );
        assert!(!string(runtime, "version").is_empty());
    }

    let cases = matrix
        .get("cases")
        .and_then(Value::as_array)
        .expect("cases must be an array");
    assert_eq!(cases.len(), 4);

    let mut ids = BTreeSet::new();
    let mut languages = BTreeSet::new();
    let mut provenance_counts = BTreeMap::new();
    let mut reviewed_locks = BTreeSet::new();

    for case in cases {
        let case = object(case, "case");
        let id = string(case, "id");
        assert!(ids.insert(id), "case IDs must be unique");
        languages.insert(string(case, "language"));
        *provenance_counts
            .entry(string(case, "provenance"))
            .or_insert(0_usize) += 1;

        assert!(string(case, "repository").starts_with("https://github.com/"));
        assert!(!string(case, "release").is_empty());
        assert!(is_lower_hex(string(case, "commit"), 40));
        assert!(!string(case, "server").is_empty());
        assert_eq!(string(case, "expected_outcome"), "passed");

        let lock = object(
            case.get("dependency_lock")
                .expect("dependency_lock must exist"),
            "dependency_lock",
        );
        assert!(is_lower_hex(string(lock, "sha256"), 64));
        if string(lock, "source") == "mcp-doctor-reviewed" {
            let path = string(lock, "path");
            assert!(reviewed_locks.insert(path));
            assert!(
                Path::new(env!("CARGO_MANIFEST_DIR")).join(path).is_file(),
                "reviewed lock must be checked in: {path}"
            );
        } else {
            assert_eq!(string(lock, "source"), "upstream");
        }
    }

    assert_eq!(languages.len(), 4);
    assert_eq!(provenance_counts.get("official"), Some(&2));
    assert_eq!(provenance_counts.get("independent"), Some(&2));
    assert_eq!(reviewed_locks.len(), 2);
    assert!(DART_LOCK.contains("sdks:\n  dart:"));
    assert!(PHP_LOCK.contains("\"name\": \"psr/log\""));

    let last_verified = object(
        matrix
            .get("last_verified")
            .expect("last_verified must exist"),
        "last_verified",
    );
    assert_eq!(string(last_verified, "date"), "2026-08-10");
    assert_eq!(string(last_verified, "result"), "4/4 passed");
}
