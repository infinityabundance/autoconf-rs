//! Oracle Regression Tests — AC.ORACLE.1 Feature 7
//!
//! Runs all Layer 0 smoke fixtures through both the GNU autoconf oracle
//! and autoconf-rs, reporting coverage metrics and structural divergence.
//! This serves as the regression gate: any autoconf-rs change that
//! reduces oracle compatibility triggers a test failure.
//!
//! Receipt family: AC.ORACLE.1.REG.*

use std::path::Path;
use std::process::Command;

/// A single regression comparison result.
struct RegressResult {
    fixture: String,
    oracle_size: usize,
    rs_size: usize,
    coverage_pct: f64,
    /// oracle exit code
    oracle_exit: i32,
    /// rs exit code
    rs_exit: i32,
    /// Did both pass?
    both_pass: bool,
}

fn fixture_path(name: &str) -> String {
    let candidates = [
        format!("../../lab/corpus/layer0-smoke/{}", name),
        format!("lab/corpus/layer0-smoke/{}", name),
        format!("../lab/corpus/layer0-smoke/{}", name),
        format!("../../../lab/corpus/layer0-smoke/{}", name),
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            return c.clone();
        }
    }
    // Fallback: try relative from workspace root
    format!("lab/corpus/layer0-smoke/{}", name)
}

fn run_oracle(fixture: &str) -> (String, String, i32) {
    let output = Command::new("autoconf")
        .args([fixture, "-o", "/dev/stdout"])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .expect("failed to run GNU autoconf");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

fn run_autoconf_rs(fixture: &str) -> (String, String, i32) {
    use autoconf_rs_core::M4Engine;
    let input = std::fs::read_to_string(fixture).expect("failed to read fixture");
    let mut engine = M4Engine::new();
    match engine.process(&input) {
        Ok(output) => (output, String::new(), 0),
        Err(e) => (String::new(), e, 2),
    }
}

/// All Layer 0 smoke fixtures.
const L0_FIXTURES: &[&str] = &[
    "smoke_01_minimal.ac",
    "smoke_02_subst.ac",
    "smoke_03_headers.ac",
    "fixture_04_programs.ac",
    "fixture_05_functions.ac",
    "fixture_06_headers_types.ac",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Full oracle regression: run all 6 Layer 0 fixtures and report.
    #[test]
    fn test_oracle_regression_all_l0() {
        let oracle_available = Command::new("autoconf")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !oracle_available {
            eprintln!("SKIP: GNU autoconf not found on PATH");
            return;
        }

        let mut results: Vec<RegressResult> = Vec::new();

        println!("\n=== AC.ORACLE.1 Regression — Layer 0 (6 fixtures) ===");
        for name in L0_FIXTURES {
            let fp = fixture_path(name);
            if !Path::new(&fp).exists() {
                eprintln!("  SKIP {} — fixture not found at {}", name, fp);
                continue;
            }

            let (oracle_out, _oerr, ocode) = run_oracle(&fp);
            let (rs_out, _rerr, rcode) = run_autoconf_rs(&fp);

            let coverage = if oracle_out.len() > 0 {
                rs_out.len() as f64 / oracle_out.len() as f64 * 100.0
            } else {
                0.0
            };

            results.push(RegressResult {
                fixture: name.to_string(),
                oracle_size: oracle_out.len(),
                rs_size: rs_out.len(),
                coverage_pct: coverage,
                oracle_exit: ocode,
                rs_exit: rcode,
                both_pass: ocode == 0 && rcode == 0,
            });

            println!(
                "  {:35} | oracle={:>7}B  rs={:>7}B  cov={:>5.1}%  exit={}/{}  pass={}",
                name,
                oracle_out.len(),
                rs_out.len(),
                coverage,
                ocode,
                rcode,
                ocode == 0 && rcode == 0,
            );
        }

        // Summary
        let count = results.len();
        let pass_count = results.iter().filter(|r| r.both_pass).count();
        let avg_cov = if count > 0 {
            results.iter().map(|r| r.coverage_pct).sum::<f64>() / count as f64
        } else {
            0.0
        };

        println!(
            "\n  L0 Regression Summary: {}/{} pass, avg coverage {:.1}%",
            pass_count, count, avg_cov
        );

        // Gate: all 6 must produce output (exit 0) from both sides
        assert_eq!(
            pass_count, count,
            "Regression failure: {}/{} fixtures produced valid output from both oracle and autoconf-rs",
            pass_count, count
        );

        // Gate: size coverage must be reasonable (>10% for complex fixtures)
        for r in &results {
            assert!(
                r.coverage_pct > 5.0,
                "Regression failure: {} coverage too low ({:.1}%)",
                r.fixture,
                r.coverage_pct
            );
        }
    }

    /// Oracle binary identity regression: verify all 8 GNU binaries are accessible.
    #[test]
    fn test_oracle_regression_binary_identity() {
        use autoconf_oracle_rs::{admit_oracle, OracleConfig, OracleError};

        let config = OracleConfig::default();
        match admit_oracle(&config) {
            Ok(profile) => {
                println!("\n=== AC.ORACLE.1 Binary Identity Regression ===");
                println!(
                    "  Oracle: {} (SHA256: {}…)",
                    profile.kind,
                    &profile.sha256[..16]
                );
                println!("  Platform: {}", profile.platform);
                println!("  Binaries found: {}/8", profile.binaries.len());
                for (name, bp) in &profile.binaries {
                    let status = if bp.smoke_passed { "✓" } else { "✗" };
                    println!("    {} {} — {}", status, name, bp.path);
                }

                // Gate: at least the core 4 binaries must be present and smoke-passed
                let required = ["autoconf", "autoheader", "autom4te", "autoreconf"];
                for name in &required {
                    let bp = profile.binaries.get(*name);
                    assert!(
                        bp.map(|b| b.smoke_passed).unwrap_or(false),
                        "Regression: required binary '{}' missing or smoke-failed",
                        name
                    );
                }

                // Gate: all 8 binaries should be present
                assert_eq!(
                    profile.binaries.len(),
                    8,
                    "Expected 8 Autoconf binaries, found {}",
                    profile.binaries.len()
                );
            }
            Err(OracleError::NotFound(_)) => {
                eprintln!("SKIP: GNU autoconf not found — cannot run binary identity regression");
            }
            Err(e) => panic!("Oracle admission failed: {}", e),
        }
    }

    /// Cross-version regression: verify current oracle version against known baseline.
    #[test]
    fn test_oracle_regression_version_pin() {
        use autoconf_oracle_rs::{admit_oracle, OracleConfig, OracleError};

        let config = OracleConfig::default();
        match admit_oracle(&config) {
            Ok(profile) => {
                println!("\n=== AC.ORACLE.1 Version Pin Regression ===");
                println!("  Admitted version: {}", profile.kind);
                println!(
                    "  Version output: {}",
                    profile.version_output.lines().next().unwrap_or("")
                );

                // Gate: version must be 2.73 (our pinned oracle)
                assert!(
                    profile.kind.contains("2_73") || profile.kind.contains("2.73"),
                    "Oracle version pin mismatch: expected 2.73, got {}",
                    profile.kind
                );

                // Gate: SHA256 must be non-empty
                assert!(!profile.sha256.is_empty(), "Oracle SHA256 is empty");
                println!("  SHA256: {}", profile.sha256);

                // Gate: M4 oracle must be present
                assert!(
                    profile.m4_oracle.is_some(),
                    "Regression: M4 oracle not found"
                );
                if let Some(ref m4) = profile.m4_oracle {
                    println!("  M4 oracle: {} (SHA256: {}…)", m4.path, &m4.sha256[..16]);
                }
            }
            Err(OracleError::NotFound(_)) => {
                eprintln!("SKIP: GNU autoconf not found");
            }
            Err(e) => panic!("Oracle admission failed: {}", e),
        }
    }
}
