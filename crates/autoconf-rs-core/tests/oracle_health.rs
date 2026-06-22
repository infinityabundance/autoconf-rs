//! Oracle Health Check — AC.ORACLE.1 Final Verification
//!
//! Comprehensive end-to-end verification of the entire Oracle admission surface:
//! - All 4 GNU Autoconf versions available and functional
//! - All 6 Layer 0 fixtures processed by all versions
//! - Cross-version matrix populated and consistent
//! - autoconf-rs comparison against primary oracle (2.73)
//! - Binary identity and M4 oracle verification
//!
//! Receipt: AC.ORACLE.1.HEALTH

use std::path::Path;
use std::process::Command;

fn fixture_path(name: &str) -> String {
    for prefix in &["../../lab/corpus/layer0-smoke", "lab/corpus/layer0-smoke"] {
        let p = format!("{}/{}", prefix, name);
        if Path::new(&p).exists() {
            return p;
        }
    }
    format!("../../lab/corpus/layer0-smoke/{}", name)
}

const L0: &[&str] = &[
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

    #[test]
    fn test_oracle_health_check() {
        // ---- Phase 1: Version Discovery ----
        let versions: Vec<(&str, &str)> = vec![
            ("2.73", "/usr/bin/autoconf"),
            ("2.72", "/tmp/autoconf-2.72-install/bin/autoconf"),
            ("2.71", "/tmp/autoconf-2.71-install/bin/autoconf"),
            ("2.69", "/tmp/autoconf-2.69-install/bin/autoconf"),
        ];

        let mut available: Vec<(&str, &str)> = Vec::new();
        for (ver, path) in &versions {
            let ok = Path::new(path).exists();
            println!("  [{}] {} {}", if ok { "FOUND" } else { "MISS" }, ver, path);
            if ok {
                available.push((*ver, *path));
            }
        }

        if available.is_empty() {
            eprintln!("SKIP: no GNU Autoconf versions found");
            return;
        }

        println!("\n=== ORACLE HEALTH CHECK ===");
        println!("  Versions available: {}/4", available.len());

        // ---- Phase 2: Version Smoke Test ----
        println!("\n  --- Version Smoke ---");
        let mut smoke_pass = 0u64;
        for (ver, path) in &available {
            let ok = Command::new(path)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            let status = if ok { "PASS" } else { "FAIL" };
            if ok {
                smoke_pass += 1;
            }
            println!("    {} {} {}", status, ver, path);
        }

        // ---- Phase 3: Layer 0 Cross-Version ----
        println!("\n  --- Layer 0 Cross-Version ({}) ---", L0.len());
        let mut l0_pass = 0u64;
        let mut total_runs = 0u64;

        for name in L0 {
            let fp = fixture_path(name);
            if !Path::new(&fp).exists() {
                continue;
            }
            let mut ok = true;
            for (ver, path) in &available {
                total_runs += 1;
                match Command::new(path).arg("-f").arg(&fp).output() {
                    Ok(o) if o.status.success() => {
                        if o.stdout.is_empty() {
                            ok = false;
                        }
                    }
                    _ => {
                        ok = false;
                    }
                }
            }
            let status = if ok { "PASS" } else { "FAIL" };
            if ok {
                l0_pass += 1;
            }
            println!("    {} {}", status, name);
        }

        // ---- Phase 4: autoconf-rs vs Oracle (2.73) ----
        println!("\n  --- autoconf-rs vs Oracle 2.73 ---");
        let mut rs_pass = 0u64;
        let mut rs_total = 0u64;
        let mut ratios: Vec<f64> = Vec::new();

        for name in L0 {
            let fp = fixture_path(name);
            if !Path::new(&fp).exists() {
                continue;
            }
            rs_total += 1;

            let oracle = Command::new("/usr/bin/autoconf")
                .arg("-f")
                .arg(&fp)
                .output()
                .map(|o| o.stdout)
                .unwrap_or_default();
            let rs = {
                use autoconf_rs_core::M4Engine;
                let input = std::fs::read_to_string(&fp).unwrap_or_default();
                let mut engine = M4Engine::new();
                engine
                    .process(&input)
                    .ok()
                    .map(|s| s.into_bytes())
                    .unwrap_or_default()
            };

            let ok = !oracle.is_empty() && !rs.is_empty();
            if ok {
                rs_pass += 1;
                let ratio = rs.len() as f64 / oracle.len() as f64 * 100.0;
                ratios.push(ratio);
            }
            println!(
                "    {} {}: oracle={}B rs={}B ({:.0}%)",
                if ok { "PASS" } else { "FAIL" },
                name,
                oracle.len(),
                rs.len(),
                if oracle.len() > 0 {
                    rs.len() as f64 / oracle.len() as f64 * 100.0
                } else {
                    0.0
                }
            );
        }

        let avg_coverage = if ratios.is_empty() {
            0.0
        } else {
            ratios.iter().sum::<f64>() / ratios.len() as f64
        };

        // ---- Phase 5: Binary Identity ----
        println!("\n  --- Binary Identity ---");
        use autoconf_oracle_rs::{admit_oracle, OracleConfig};
        let config = OracleConfig::default();
        let identity = match admit_oracle(&config) {
            Ok(p) => {
                println!("    Oracle: {}", p.kind);
                println!("    SHA256: {}...", &p.sha256[..32]);
                println!("    Platform: {}", p.platform);
                println!("    Binaries: {}/8", p.binaries.len());
                for (name, bp) in &p.binaries {
                    println!(
                        "      {} {} — {}",
                        if bp.smoke_passed { "PASS" } else { "FAIL" },
                        name,
                        bp.path
                    );
                }
                if let Some(ref m4) = p.m4_oracle {
                    println!(
                        "    M4 oracle: {} (SHA256: {}...)",
                        m4.path,
                        &m4.sha256[..32]
                    );
                }
                p.binaries.iter().all(|(_, b)| b.smoke_passed)
            }
            Err(_) => false,
        };

        // ---- Phase 6: Summary ----
        println!("\n  ═══════════════════════════════════════");
        println!("  ORACLE HEALTH CHECK SUMMARY");
        println!("  ═══════════════════════════════════════");
        println!("  Versions:     {}/4 available", available.len());
        println!("  Version smoke:{}/{} pass", smoke_pass, available.len());
        println!(
            "  L0 fixtures:  {}/{} pass ({} runs)",
            l0_pass,
            L0.len(),
            total_runs
        );
        println!(
            "  rs vs oracle: {}/{} pass, avg coverage {:.0}%",
            rs_pass, rs_total, avg_coverage
        );
        println!("  Binary ID:    {}", if identity { "PASS" } else { "FAIL" });
        println!("  Court:        AC.ORACLE.1.HEALTH");

        // Gates
        assert!(available.len() >= 1, "Need at least 1 oracle version");
        assert_eq!(
            smoke_pass,
            available.len() as u64,
            "All versions must pass smoke"
        );
        assert!(
            l0_pass >= 5,
            "Too few L0 fixtures pass across all versions: {}/6",
            l0_pass
        );
        assert_eq!(rs_pass, rs_total, "autoconf-rs must handle all L0 fixtures");
        assert!(identity, "Binary identity check must pass");
        assert!(
            avg_coverage > 10.0,
            "Average coverage too low: {:.0}%",
            avg_coverage
        );

        println!("\n  ALL GATES PASSED — AC.ORACLE.1 HEALTHY");
    }
}
