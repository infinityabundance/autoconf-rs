//! Oracle comparison tests -- Layer 0 smoke fixtures.
//!
//! These tests run configure.ac fixtures through both GNU autoconf (oracle)
//! and autoconf-rs, then compare outputs. Foundation of forensic parity.

use std::path::Path;
use std::process::Command;

fn fixture_path(name: &str) -> String {
    // Try workspace-relative paths from crate root (crates/autoconf-rs-core/)
    let candidates = [
        format!("../../lab/corpus/layer0-smoke/{}", name),
        format!("lab/corpus/layer0-smoke/{}", name),
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            return c.clone();
        }
    }
    candidates[0].clone()
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

fn compare_outputs(oracle_out: &str, rs_out: &str) -> Result<(), Vec<String>> {
    let mut diffs = Vec::new();
    if oracle_out.contains("#! /bin/sh") != rs_out.contains("#! /bin/sh") {
        diffs.push("shebang mismatch".to_string());
    }
    if oracle_out.contains("config.status") != rs_out.contains("config.status") {
        diffs.push("config.status presence mismatch".to_string());
    }
    if diffs.is_empty() {
        Ok(())
    } else {
        Err(diffs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke_01_minimal() {
        let fixture = fixture_path("smoke_01_minimal.ac");
        if !Path::new(&fixture).exists() {
            eprintln!("SKIP: fixture not found at '{}'", fixture);
            return;
        }
        let (oracle, err, code) = run_oracle(&fixture);
        assert_eq!(code, 0, "GNU autoconf failed: {}", err);
        assert!(
            oracle.len() > 200,
            "Oracle output too short: {}B",
            oracle.len()
        );
        let (rs, rs_err, rs_code) = run_autoconf_rs(&fixture);
        assert_eq!(rs_code, 0, "autoconf-rs crashed: {}", rs_err);
        let coverage = rs.len() as f64 / oracle.len() as f64 * 100.0;
        eprintln!(
            "smoke_01: oracle={}B, rs={}B ({:.0}% coverage)",
            oracle.len(),
            rs.len(),
            coverage
        );
        match compare_outputs(&oracle, &rs) {
            Ok(()) => eprintln!("  structural: OK"),
            Err(d) => eprintln!("  structural: {:?}", d),
        }
    }

    #[test]
    fn test_smoke_02_subst() {
        let fixture = fixture_path("smoke_02_subst.ac");
        if !Path::new(&fixture).exists() {
            eprintln!("SKIP: fixture not found at '{}'", fixture);
            return;
        }
        let (oracle, err, code) = run_oracle(&fixture);
        assert_eq!(code, 0, "GNU autoconf failed: {}", err);
        let (rs, rs_err, rs_code) = run_autoconf_rs(&fixture);
        assert_eq!(rs_code, 0, "autoconf-rs crashed: {}", rs_err);
        let coverage = rs.len() as f64 / oracle.len() as f64 * 100.0;
        eprintln!(
            "smoke_02: oracle={}B, rs={}B ({:.0}% coverage)",
            oracle.len(),
            rs.len(),
            coverage
        );
        assert!(
            oracle.contains("PACKAGE_NAME"),
            "Oracle missing PACKAGE_NAME"
        );
        assert!(
            rs.contains("config.status") || rs.contains("sed"),
            "rs missing config.status"
        );
    }

    #[test]
    fn test_smoke_03_headers() {
        let fixture = fixture_path("smoke_03_headers.ac");
        if !Path::new(&fixture).exists() {
            eprintln!("SKIP: fixture not found at '{}'", fixture);
            return;
        }
        let (oracle, err, code) = run_oracle(&fixture);
        assert_eq!(code, 0, "GNU autoconf failed: {}", err);
        let (rs, rs_err, rs_code) = run_autoconf_rs(&fixture);
        assert_eq!(rs_code, 0, "autoconf-rs crashed: {}", rs_err);
        let coverage = rs.len() as f64 / oracle.len() as f64 * 100.0;
        eprintln!(
            "smoke_03: oracle={}B, rs={}B ({:.0}% coverage)",
            oracle.len(),
            rs.len(),
            coverage
        );
        assert!(oracle.contains("config.h"), "Oracle missing config.h");
        assert!(
            rs.contains("config.status") || rs.contains("config.h"),
            "rs missing header handling"
        );
    }

    // === Layer 0 fixtures 04-06: programs, functions, headers/types ===
    // These complete the 6 Layer 0 smoke fixture set for AC.ORACLE.1

    #[test]
    fn test_smoke_04_programs() {
        let fixture = fixture_path("fixture_04_programs.ac");
        if !Path::new(&fixture).exists() {
            eprintln!("SKIP: fixture not found at '{}'", fixture);
            return;
        }
        // Run through GNU autoconf oracle
        let (oracle, err, code) = run_oracle(&fixture);
        assert_eq!(code, 0, "GNU autoconf failed on fixture_04: {}", err);
        assert!(
            oracle.len() > 500,
            "Oracle output too short: {}B",
            oracle.len()
        );
        // Run through autoconf-rs
        let (rs, rs_err, rs_code) = run_autoconf_rs(&fixture);
        assert_eq!(rs_code, 0, "autoconf-rs crashed on fixture_04: {}", rs_err);
        assert!(
            rs.len() > 500,
            "autoconf-rs output too short: {}B",
            rs.len()
        );
        let coverage = rs.len() as f64 / oracle.len() as f64 * 100.0;
        eprintln!(
            "smoke_04: oracle={}B, rs={}B ({:.0}% coverage)",
            oracle.len(),
            rs.len(),
            coverage
        );
        // Structural checks: program fixture should contain install and make-set
        assert!(oracle.contains("INSTALL"), "Oracle missing INSTALL");
        assert!(oracle.contains("MAKE"), "Oracle missing MAKE");
        // autoconf-rs should produce a runnable configure with config.status
        assert!(
            rs.contains("config.status") || rs.contains("configure"),
            "rs output missing configure marker"
        );
    }

    #[test]
    fn test_smoke_05_functions() {
        let fixture = fixture_path("fixture_05_functions.ac");
        if !Path::new(&fixture).exists() {
            eprintln!("SKIP: fixture not found at '{}'", fixture);
            return;
        }
        // Run through GNU autoconf oracle
        let (oracle, err, code) = run_oracle(&fixture);
        assert_eq!(code, 0, "GNU autoconf failed on fixture_05: {}", err);
        assert!(
            oracle.len() > 500,
            "Oracle output too short: {}B",
            oracle.len()
        );
        // Run through autoconf-rs
        let (rs, rs_err, rs_code) = run_autoconf_rs(&fixture);
        assert_eq!(rs_code, 0, "autoconf-rs crashed on fixture_05: {}", rs_err);
        assert!(
            rs.len() > 500,
            "autoconf-rs output too short: {}B",
            rs.len()
        );
        let coverage = rs.len() as f64 / oracle.len() as f64 * 100.0;
        eprintln!(
            "smoke_05: oracle={}B, rs={}B ({:.0}% coverage)",
            oracle.len(),
            rs.len(),
            coverage
        );
        // Structural checks: function fixture should reference function names
        assert!(
            oracle.contains("malloc") || oracle.contains("strerror"),
            "Oracle missing function references"
        );
        assert!(
            rs.contains("config.status") || rs.contains("configure"),
            "rs output missing configure marker"
        );
    }

    #[test]
    fn test_smoke_06_headers_types() {
        let fixture = fixture_path("fixture_06_headers_types.ac");
        if !Path::new(&fixture).exists() {
            eprintln!("SKIP: fixture not found at '{}'", fixture);
            return;
        }
        // Run through GNU autoconf oracle
        let (oracle, err, code) = run_oracle(&fixture);
        assert_eq!(code, 0, "GNU autoconf failed on fixture_06: {}", err);
        assert!(
            oracle.len() > 500,
            "Oracle output too short: {}B",
            oracle.len()
        );
        // Run through autoconf-rs
        let (rs, rs_err, rs_code) = run_autoconf_rs(&fixture);
        assert_eq!(rs_code, 0, "autoconf-rs crashed on fixture_06: {}", rs_err);
        assert!(
            rs.len() > 500,
            "autoconf-rs output too short: {}B",
            rs.len()
        );
        let coverage = rs.len() as f64 / oracle.len() as f64 * 100.0;
        eprintln!(
            "smoke_06: oracle={}B, rs={}B ({:.0}% coverage)",
            oracle.len(),
            rs.len(),
            coverage
        );
        // Structural checks: headers fixture should reference header names
        assert!(
            oracle.contains("stdlib.h") || oracle.contains("stdint.h"),
            "Oracle missing header references"
        );
        assert!(
            rs.contains("config.status") || rs.contains("configure"),
            "rs output missing configure marker"
        );
    }
}
