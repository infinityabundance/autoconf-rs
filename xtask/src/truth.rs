// truth.rs — Hard truth verification gate.
//
// This gate runs ACTUAL tests and compares results against JSON claims.
// Unlike consistency.rs (which only checks internal sums), this gate
// verifies external truthfulness by running real commands.
//
// If needle-metrics.json claims 1355 tests but only 104 actually exist,
// this gate FAILS HARD. No tolerance for fabrication.

use std::process::Command;

/// Result of a truth check.
pub struct TruthReport {
    pub claim_test_count: u64,
    pub actual_test_count: u64,
    pub claim_matches_actual: bool,
    pub errors: Vec<String>,
}

/// Run `cargo test --workspace -- --list` and count actual tests.
pub fn count_actual_tests() -> Result<u64, String> {
    let output = Command::new("cargo")
        .args(["test", "--workspace", "--", "--list"])
        .output()
        .map_err(|e| format!("cannot run cargo test --list: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Combine stdout and stderr since `--list` may output to either
    let combined = format!("{}\n{}", stdout, stderr);

    // Count lines that look like test names: "module::test_name: test"
    let test_count = combined
        .lines()
        .filter(|line| line.ends_with(": test") && !line.contains("bench"))
        .count() as u64;

    if test_count == 0 {
        // Fallback: try running without --list and count test result lines
        return Err("could not parse test list; try `cargo test --workspace` first".to_string());
    }

    Ok(test_count)
}

/// Read the claimed test count from needle-metrics.json.
pub fn read_claimed_test_count() -> Result<u64, String> {
    let json_str = std::fs::read_to_string("sources/gaps/needle-metrics.json")
        .map_err(|e| format!("cannot read needle-metrics.json: {}", e))?;

    let v: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("cannot parse needle-metrics.json: {}", e))?;

    v.get("tests_passing")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| "needle-metrics.json missing tests_passing field".to_string())
}

/// Read the claimed overall percentage from needle-metrics.json.
pub fn read_claimed_percentage() -> Result<f64, String> {
    let json_str = std::fs::read_to_string("sources/gaps/needle-metrics.json")
        .map_err(|e| format!("cannot read needle-metrics.json: {}", e))?;

    let v: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("cannot parse needle-metrics.json: {}", e))?;

    v.get("overall_percentage")
        .and_then(|x| x.as_f64())
        .ok_or_else(|| "needle-metrics.json missing overall_percentage field".to_string())
}

/// Run the full truth verification gate.
/// Returns (passed, report).
pub fn run_truth_gate() -> (bool, TruthReport) {
    let mut errors = Vec::new();

    // 1. Count actual tests
    let actual_test_count = match count_actual_tests() {
        Ok(n) => n,
        Err(e) => {
            errors.push(format!("TRUTH: cannot count tests: {}", e));
            0
        }
    };

    // 2. Read claimed test count
    let claim_test_count = match read_claimed_test_count() {
        Ok(n) => n,
        Err(e) => {
            errors.push(format!("TRUTH: cannot read claimed count: {}", e));
            0
        }
    };

    // 3. Verify: claimed ≤ actual (we may have more tests than documented)
    let claim_matches_actual = claim_test_count <= actual_test_count && claim_test_count > 0;

    if !claim_matches_actual && !errors.is_empty() {
        // Already errored — can't verify
    } else if claim_test_count > actual_test_count {
        errors.push(format!(
            "TRUTH: needle-metrics.json claims {} tests but only {} actually exist. \
             The JSON data is FABRICATED. Update sources/gaps/needle-metrics.json with the real count.",
            claim_test_count, actual_test_count
        ));
    }

    let report = TruthReport {
        claim_test_count,
        actual_test_count,
        claim_matches_actual: claim_matches_actual && errors.is_empty(),
        errors,
    };

    (report.claim_matches_actual, report)
}
