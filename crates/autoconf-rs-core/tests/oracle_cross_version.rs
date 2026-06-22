//! Cross-Version Oracle Comparison Tests — AC.ORACLE.1 Feature 6
//!
//! Compares GNU Autoconf 2.73 (system) against 2.72 and 2.71 (local builds)
//! on Layer 0 smoke fixtures. Reports byte-exact match rate and structural diffs.
//!
//! Receipt: AC.ORACLE.1.CROSSV

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

fn fixture_path(name: &str) -> String {
    let candidates = [
        format!("../../lab/corpus/layer0-smoke/{}", name),
        format!("lab/corpus/layer0-smoke/{}", name),
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            return c.clone();
        }
    }
    format!("../../lab/corpus/layer0-smoke/{}", name)
}

fn run_version(binary: &str, fixture: &str) -> (i32, usize, String) {
    match Command::new(binary)
        .args(["-f", fixture])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(o) => {
            let code = o.status.code().unwrap_or(-1);
            let len = o.stdout.len();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            (code, len, stderr)
        }
        Err(e) => (-1, 0, format!("exec error: {}", e)),
    }
}

const L0_FIXTURES: &[(&str, &str)] = &[
    ("smoke_01_minimal.ac", "minimal"),
    ("smoke_02_subst.ac", "subst"),
    ("smoke_03_headers.ac", "headers"),
    ("fixture_04_programs.ac", "programs"),
    ("fixture_05_functions.ac", "functions"),
    ("fixture_06_headers_types.ac", "headers+types"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_version_273_vs_272() {
        let v273 = "/usr/bin/autoconf";
        let v272 = "/tmp/autoconf-2.72-install/bin/autoconf";

        if !Path::new(v272).exists() {
            eprintln!("SKIP: autoconf 2.72 not found");
            return;
        }

        let (c273, _, _) = run_version(v273, &fixture_path("smoke_01_minimal.ac"));
        let (c272, _, _) = run_version(v272, &fixture_path("smoke_01_minimal.ac"));
        assert_eq!(c273, 0, "2.73 smoke failed");
        assert_eq!(c272, 0, "2.72 smoke failed");

        println!("\n=== CROSS-VERSION 2.73 vs 2.72 ===");
        println!(
            "  {:35} | {:>8} {:>8} {:>8} {:>6}",
            "Fixture", "2.73", "2.72", "match?", "ratio"
        );

        let mut byte_exact = 0u64;
        let mut total = 0u64;
        let mut ratios: Vec<f64> = Vec::new();

        for (name, _desc) in L0_FIXTURES {
            let fp = fixture_path(name);
            if !Path::new(&fp).exists() {
                continue;
            }
            total += 1;
            let out_a = Command::new(v273)
                .args(["-f", &fp])
                .output()
                .map(|o| o.stdout)
                .unwrap_or_default();
            let out_b = Command::new(v272)
                .args(["-f", &fp])
                .output()
                .map(|o| o.stdout)
                .unwrap_or_default();
            let match_str = if out_a == out_b { "YES" } else { "no" };
            if out_a == out_b {
                byte_exact += 1;
            }
            let ratio = if out_b.len() > 0 {
                out_a.len() as f64 / out_b.len() as f64
            } else {
                0.0
            };
            ratios.push(ratio);
            println!(
                "  {:35} | {:>8} {:>8} {:>8} {:>5.2}x",
                name,
                out_a.len(),
                out_b.len(),
                match_str,
                ratio
            );
        }

        let avg_ratio = if ratios.is_empty() {
            0.0
        } else {
            ratios.iter().sum::<f64>() / ratios.len() as f64
        };
        println!(
            "\n  Summary: {}/{} byte-exact ({:.0}%), avg ratio {:.2}x",
            byte_exact,
            total,
            byte_exact as f64 / total.max(1) as f64 * 100.0,
            avg_ratio
        );
        println!("  Court: AC.CROSS_VERSION.273_VS_272");
        assert!(
            byte_exact > 0 || avg_ratio > 0.9,
            "divergence too high: {} exact, {:.2}x ratio",
            byte_exact,
            avg_ratio
        );
    }

    #[test]
    fn test_admit_versions_to_matrix() {
        use autoconf_oracle_rs::{admit_oracle, CrossVersionMatrix, OracleConfig};

        let versions: Vec<(&str, &str)> = vec![
            ("2.73", "/usr/bin/autoconf"),
            ("2.72", "/tmp/autoconf-2.72-install/bin/autoconf"),
            ("2.71", "/tmp/autoconf-2.71-install/bin/autoconf"),
            ("2.69", "/tmp/autoconf-2.69-install/bin/autoconf"),
        ];

        let mut matrix = CrossVersionMatrix::new("2.73");

        for (ver, path) in &versions {
            if !Path::new(path).exists() {
                eprintln!("SKIP: {} not found", ver);
                continue;
            }
            let config = OracleConfig {
                autoconf_path: Some(std::path::PathBuf::from(*path)),
                ..OracleConfig::default()
            };
            let profile = admit_oracle(&config).expect("admission failed");
            matrix.admit(profile);
            eprintln!("  Admitted {}", ver);
        }

        println!(
            "\n=== CROSS-VERSION MATRIX ({} versions) ===",
            matrix.admitted_versions().len()
        );
        println!("  Primary: {}", matrix.primary_version);
        println!("  Admitted: {:?}", matrix.admitted_versions());

        assert!(
            matrix.admitted_versions().len() >= 4,
            "Expected >=4 versions, got {}",
            matrix.admitted_versions().len()
        );

        let fixture = "AC_INIT([test], [1.0])\nAC_OUTPUT\n";
        let env = HashMap::new();
        let admitted: Vec<String> = matrix
            .admitted_versions()
            .iter()
            .map(|s| s.to_string())
            .collect();

        for i in 0..admitted.len() {
            for j in i + 1..admitted.len() {
                let a = &admitted[i];
                let b = &admitted[j];
                let cmp = matrix
                    .compare_versions(a, b, "minimal.ac", fixture, &env)
                    .expect("comparison failed");
                println!(
                    "  {} vs {}: exact={} exit_match={} sizes={}/{}",
                    a, b, cmp.byte_exact, cmp.exit_match, cmp.size_a, cmp.size_b
                );
                assert!(cmp.exit_match, "exit mismatch {} vs {}", a, b);
            }
        }

        println!("  Comparisons: {}", matrix.comparisons.len());
        println!("  Court: AC.CROSS_VERSION.MATRIX");
    }

    #[test]
    fn test_cross_version_3way_l0() {
        let versions: Vec<(&str, &str)> = vec![
            ("2.73", "/usr/bin/autoconf"),
            ("2.72", "/tmp/autoconf-2.72-install/bin/autoconf"),
            ("2.71", "/tmp/autoconf-2.71-install/bin/autoconf"),
            ("2.69", "/tmp/autoconf-2.69-install/bin/autoconf"),
        ];

        let mut available: Vec<(&str, &str)> = Vec::new();
        for (ver, path) in &versions {
            if Path::new(path).exists() {
                available.push((*ver, *path));
            }
        }

        if available.len() < 2 {
            eprintln!("SKIP: need >=2 versions, found {}", available.len());
            return;
        }

        println!(
            "\n=== 3-WAY CROSS-VERSION L0 ({} versions, 6 fixtures) ===",
            available.len()
        );

        let mut exit_mismatches = 0u64;
        let mut byte_exact_pairs = 0u64;
        let mut comparisons = 0u64;
        let mut total_fixtures = 0u64;

        for (name, _desc) in L0_FIXTURES {
            let fp = fixture_path(name);
            if !Path::new(&fp).exists() {
                continue;
            }
            total_fixtures += 1;

            let mut outputs: Vec<(&str, usize, Vec<u8>)> = Vec::new();
            for (ver, path) in &available {
                match Command::new(path).args(["-f", &fp]).output() {
                    Ok(o) => {
                        let code = o.status.code().unwrap_or(-1);
                        if code != 0 {
                            exit_mismatches += 1;
                        }
                        outputs.push((ver, o.stdout.len(), o.stdout));
                    }
                    Err(_) => {
                        exit_mismatches += 1;
                    }
                }
            }

            for i in 0..outputs.len() {
                for j in i + 1..outputs.len() {
                    comparisons += 1;
                    if outputs[i].2 == outputs[j].2 {
                        byte_exact_pairs += 1;
                    }
                }
            }

            let sizes: Vec<String> = outputs
                .iter()
                .map(|(v, s, _)| format!("{}={}B", v, s))
                .collect();
            println!("  {}: {}", name, sizes.join(" "));
        }

        let match_pct = if comparisons > 0 {
            byte_exact_pairs as f64 / comparisons as f64 * 100.0
        } else {
            0.0
        };

        println!(
            "\n  Summary: {} fixtures, {} comps, {} exact ({:.0}%), {} mismatches",
            total_fixtures, comparisons, byte_exact_pairs, match_pct, exit_mismatches
        );
        println!("  Court: AC.CROSS_VERSION.3WAY");

        assert!(
            exit_mismatches <= 2,
            "Too many exit mismatches: {} (allowed <=2 for cross-version edge cases)",
            exit_mismatches
        );
    }
}
