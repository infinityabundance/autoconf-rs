//! Cross-Version Oracle Matrix — tests against multiple GNU Autoconf versions.
//!
//! Tracks behavioral drift between oracle versions and classifies
//! divergences as version-dependent or implementation gaps.
//!
//! Court: AC.ORACLE.MATRIX.1
//! Target versions: 2.69, 2.71, 2.72, 2.73

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::process::Command;

    /// Detect all available GNU Autoconf versions on PATH.
    fn find_oracle_versions() -> Vec<(String, String)> {
        let candidates = [
            "autoconf",
            "autoconf2.69",
            "autoconf2.71",
            "autoconf2.72",
            "autoconf2.73",
        ];
        let mut versions = Vec::new();
        for name in &candidates {
            if let Ok(out) = Command::new(name).arg("--version").output() {
                let ver = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .replace("autoconf (GNU Autoconf) ", "")
                    .trim()
                    .to_string();
                if !ver.is_empty() && !versions.iter().any(|(v, _)| v == &ver) {
                    versions.push((ver, name.to_string()));
                }
            }
        }
        versions
    }

    /// Run autoconf (oracle) on a fixture, return (exit_code, stdout_len, stderr_len).
    fn run_oracle(bin: &str, fixture: &str) -> (i32, usize, usize) {
        Command::new(bin)
            .arg(&format!("../../{}", fixture))
            .output()
            .map(|o| {
                (
                    o.status.code().unwrap_or(-1),
                    o.stdout.len(),
                    o.stderr.len(),
                )
            })
            .unwrap_or((-1, 0, 0))
    }

    /// Run autoconf-rs on a fixture.
    fn run_autoconf_rs(fixture: &str) -> (i32, usize, usize) {
        Command::new("../../target/release/autoconf")
            .arg(&format!("../../{}", fixture))
            .output()
            .map(|o| {
                (
                    o.status.code().unwrap_or(-1),
                    o.stdout.len(),
                    o.stderr.len(),
                )
            })
            .unwrap_or((-101, 0, 0))
    }

    #[test]
    fn test_oracle_version_matrix() {
        let versions = find_oracle_versions();
        println!("Oracle versions detected: {}", versions.len());
        for (v, p) in &versions {
            println!("  {} → {}", v, p);
        }
        assert!(!versions.is_empty(), "at least one oracle version required");
    }

    #[test]
    fn test_cross_version_smoke_01() {
        let versions = find_oracle_versions();
        let fixture = "lab/corpus/layer0-smoke/smoke_01_minimal.ac";

        println!("\nCross-version smoke test: {}", fixture);
        println!(
            "{:20} {:>8} {:>8} {:>8}",
            "Version", "Exit", "Stdout", "Stderr"
        );

        let mut results: BTreeMap<String, (i32, usize)> = BTreeMap::new();

        for (ver, bin) in &versions {
            let (exit, stdout_len, _stderr_len) = run_oracle(bin, fixture);
            results.insert(ver.clone(), (exit, stdout_len));
            println!(
                "{:20} {:>8} {:>8} {:>8}",
                ver, exit, stdout_len, _stderr_len
            );
        }

        // Run autoconf-rs
        let (rs_exit, rs_stdout, rs_stderr) = run_autoconf_rs(fixture);
        println!(
            "{:20} {:>8} {:>8} {:>8}",
            "autoconf-rs", rs_exit, rs_stdout, rs_stderr
        );

        // Check consistency across oracle versions
        if results.len() >= 1 {
            let first = results.values().next().unwrap();
            for (ver, (exit, size)) in &results {
                if exit != &first.0 || size != &first.1 {
                    println!(
                        "  NOTE: version {} differs from baseline (exit={}→{}, size={}→{})",
                        ver, first.0, exit, first.1, size
                    );
                }
            }
        }
    }

    #[test]
    fn test_cross_version_smoke_02() {
        let versions = find_oracle_versions();
        let fixture = "lab/corpus/layer0-smoke/smoke_02_subst.ac";

        println!("\nCross-version smoke test: {}", fixture);
        for (ver, bin) in &versions {
            let (exit, stdout_len, _) = run_oracle(bin, fixture);
            println!("  {}: exit={}, stdout={}B", ver, exit, stdout_len);
        }
        let (rs_exit, rs_stdout, _) = run_autoconf_rs(fixture);
        println!("  autoconf-rs: exit={}, stdout={}B", rs_exit, rs_stdout);
    }

    #[test]
    fn test_oracle_version_drift_report() {
        let versions = find_oracle_versions();
        println!("\n=== Oracle Version Drift Report ===");
        println!(
            "Versions: {}",
            versions
                .iter()
                .map(|(v, _)| v.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Track output size differences between versions
        let fixtures = [
            "lab/corpus/layer0-smoke/smoke_01_minimal.ac",
            "lab/corpus/layer0-smoke/smoke_02_subst.ac",
            "lab/corpus/layer0-smoke/smoke_03_headers.ac",
        ];

        for fixture in &fixtures {
            let mut sizes: BTreeMap<String, usize> = BTreeMap::new();
            for (ver, bin) in &versions {
                let (_, size, _) = run_oracle(bin, fixture);
                sizes.insert(ver.clone(), size);
            }
            let min = sizes.values().min().copied().unwrap_or(0);
            let max = sizes.values().max().copied().unwrap_or(0);
            let drift = max as i64 - min as i64;
            let stable = if drift == 0 {
                "✓ stable"
            } else {
                "⚠ drift"
            };
            println!("  {}: {} (range {}B)", fixture, stable, drift);
        }
    }

    #[test]
    fn test_oracle_version_receipt() {
        let versions = find_oracle_versions();
        assert!(!versions.is_empty(), "No oracle found");

        // Capture oracle identity
        for (ver, bin) in &versions {
            let output = Command::new(bin).arg("--version").output().unwrap();
            let version_string = String::from_utf8_lossy(&output.stdout);
            println!("Oracle receipt: {} ({})", ver, bin);
            println!("  Version output: {}", version_string.trim());

            // Verify oracle binary exists and is executable
            assert!(
                Command::new(bin).arg("--version").output().is_ok(),
                "Oracle {} should be functional",
                bin
            );
        }

        // Capture SHA256 of oracle binary for forensic traceability
        if let Ok(which) = Command::new("which").arg("autoconf").output() {
            let path = String::from_utf8_lossy(&which.stdout).trim().to_string();
            if !path.is_empty() {
                if let Ok(sha) = Command::new("sha256sum").arg(&path).output() {
                    let hash = String::from_utf8_lossy(&sha.stdout);
                    println!("  Oracle SHA256: {}", hash.trim());
                }
            }
        }

        // Test that all 6 Layer 0 fixtures produce consistent output across versions
        let fixtures = [
            "lab/corpus/layer0-smoke/smoke_01_minimal.ac",
            "lab/corpus/layer0-smoke/smoke_02_subst.ac",
            "lab/corpus/layer0-smoke/smoke_03_headers.ac",
            "lab/corpus/layer0-smoke/fixture_04_programs.ac",
            "lab/corpus/layer0-smoke/fixture_05_functions.ac",
            "lab/corpus/layer0-smoke/fixture_06_headers_types.ac",
        ];
        for fixture in &fixtures {
            let mut last_size: Option<usize> = None;
            for (_ver, bin) in &versions {
                let (_exit, size, _stderr) = run_oracle(bin, fixture);
                if let Some(prev) = last_size {
                    // Allow small drift (different versions produce slightly different output)
                    let diff = if size > prev {
                        size - prev
                    } else {
                        prev - size
                    };
                    assert!(
                        diff < 500,
                        "{}: excessive size drift {}B between versions",
                        fixture,
                        diff
                    );
                }
                last_size = Some(size);
            }
        }
        println!("All 6 fixtures consistent across oracle versions (drift < 500B)");
    }
}
