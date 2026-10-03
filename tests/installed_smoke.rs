#![cfg(all(unix, feature = "internal-test-fixtures"))]

mod support;

use std::path::Path;

use serde_json::Value;
use support::{TestEnvironment, parse_and_validate_capabilities, parse_and_validate_report};

const SMOKE_SCRIPT: &str = include_str!("../scripts/smoke-installed.sh");

fn expected_object(predicate: &str) -> Value {
    let (_, expected) = SMOKE_SCRIPT
        .split_once(predicate)
        .expect("installed smoke should assert the complete compiled cleanup contract");
    serde_json::Deserializer::from_str(expected)
        .into_iter::<Value>()
        .next()
        .expect("the smoke predicate should contain an exact JSON object")
        .expect("the smoke predicate object should be JSON")
}

#[test]
fn installed_smoke_expectations_match_the_compiled_scope_and_observed_reap() {
    let environment = TestEnvironment::new();
    let output = environment
        .command()
        .args(["capabilities", "--format", "json"])
        .output()
        .expect("compiled capabilities should return");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let capabilities = parse_and_validate_capabilities(&output.stdout);
    assert_eq!(capabilities["platform"], expected_object(".platform == "));

    let fixture = Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"));
    let output = environment
        .command()
        .args(["inspect", "--format", "json", "--"])
        .arg(fixture)
        .arg("catalog-valid")
        .output()
        .expect("the benign installed-smoke fixture should return");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["outcome"], "passed");
    assert_eq!(
        report["process_cleanup"],
        expected_object(".process_cleanup == ")
    );
}
