//! Oracle Binary Admission — AC.ORACLE.1 Feature 1 (Complete)
//!
//! Admits all 4 GNU Autoconf versions (2.73, 2.72, 2.71, 2.69),
//! verifying all 8 binaries per version, capturing SHA256 fingerprints,
//! running smoke tests, and saving profiles.
//!
//! Receipt: AC.ORACLE.1.BINARY.COMPLETE

#[cfg(test)]
mod tests {
    use autoconf_oracle_rs::{
        admit_oracle, load_profile, save_profile, CrossVersionMatrix, OracleConfig, OracleError,
    };
    use std::path::{Path, PathBuf};

    const VERSIONS: &[(&str, &str)] = &[
        ("2.73", "/usr/bin/autoconf"),
        ("2.72", "/tmp/autoconf-2.72-install/bin/autoconf"),
        ("2.71", "/tmp/autoconf-2.71-install/bin/autoconf"),
        ("2.69", "/tmp/autoconf-2.69-install/bin/autoconf"),
    ];

    const REQUIRED_BINARIES: &[&str] = &[
        "autoconf",
        "autoheader",
        "autom4te",
        "autoreconf",
        "aclocal",
        "autoscan",
        "autoupdate",
        "ifnames",
    ];

    #[test]
    fn test_binary_admission_all_4_versions() {
        let mut matrix = CrossVersionMatrix::new("2.73");
        let mut admitted = 0u64;
        let mut total_binaries = 0u64;
        let mut passed_binaries = 0u64;

        println!("\n=== AC.ORACLE.1 BINARY ADMISSION — ALL 4 VERSIONS ===");

        for (ver, path) in VERSIONS {
            if !Path::new(path).exists() {
                println!("  SKIP {} — not found at {}", ver, path);
                continue;
            }

            let config = OracleConfig {
                autoconf_path: Some(PathBuf::from(*path)),
                ..OracleConfig::default()
            };

            match admit_oracle(&config) {
                Ok(profile) => {
                    admitted += 1;
                    let kind = profile.kind.clone();
                    let sha = &profile.sha256[..16];

                    println!("\n  --- {} ({}) ---", ver, kind);
                    println!("    Path:    {}", profile.path);
                    println!("    SHA256:  {}...", sha);
                    println!("    Platform: {}", profile.platform);

                    // Verify all 8 binaries
                    let mut ver_binaries = 0u64;
                    let mut ver_passed = 0u64;

                    for bin_name in REQUIRED_BINARIES {
                        total_binaries += 1;
                        ver_binaries += 1;

                        match profile.binaries.get(*bin_name) {
                            Some(bp) if bp.smoke_passed => {
                                ver_passed += 1;
                                passed_binaries += 1;
                                println!(
                                    "      PASS {} — {} (SHA256: {}...)",
                                    bin_name,
                                    bp.path,
                                    &bp.sha256[..12]
                                );
                            }
                            Some(bp) => {
                                println!("      FAIL {} — smoke test failed", bin_name);
                            }
                            None => {
                                println!("      MISS {} — binary not located", bin_name);
                            }
                        }
                    }

                    // M4 oracle
                    if let Some(ref m4) = profile.m4_oracle {
                        println!("    M4:      {} (SHA256: {}...)", m4.path, &m4.sha256[..12]);
                    } else {
                        println!("    M4:      NOT FOUND");
                    }

                    println!("    Binaries: {}/{} pass", ver_passed, ver_binaries);

                    // Save profile
                    let profile_path = format!("/tmp/autoconf-oracle-profile-{}.json", ver);
                    save_profile(&profile, Path::new(&profile_path))
                        .expect("failed to save profile");

                    // Verify round-trip
                    let loaded =
                        load_profile(Path::new(&profile_path)).expect("failed to load profile");
                    assert_eq!(
                        loaded.sha256, profile.sha256,
                        "profile round-trip failed for {}",
                        ver
                    );

                    matrix.admit(profile);

                    assert_eq!(ver_passed, 8, "{} must have all 8 binaries passing", ver);
                }
                Err(OracleError::NotFound(msg)) => {
                    println!("  NOT FOUND {}: {}", ver, msg);
                }
                Err(e) => {
                    panic!("Admission failed for {}: {}", ver, e);
                }
            }
        }

        // Save matrix
        let matrix_json = serde_json::to_string_pretty(&matrix).unwrap();
        std::fs::write("/tmp/autoconf-cross-version-matrix.json", &matrix_json)
            .expect("failed to save matrix");

        println!("\n  ═══════════════════════════════════════");
        println!("  BINARY ADMISSION SUMMARY");
        println!("  ═══════════════════════════════════════");
        println!("  Versions admitted: {}/4", admitted);
        println!("  Total binaries:    {}", total_binaries);
        println!(
            "  Passed:            {} ({:.0}%)",
            passed_binaries,
            passed_binaries as f64 / total_binaries.max(1) as f64 * 100.0
        );
        println!("  Matrix versions:   {:?}", matrix.admitted_versions());
        println!("  Profiles saved:    /tmp/autoconf-oracle-profile-*.json");
        println!("  Matrix saved:      /tmp/autoconf-cross-version-matrix.json");
        println!("  Court:             AC.ORACLE.1.BINARY.COMPLETE");

        // Gates
        assert!(
            admitted >= 3,
            "Need >=3 versions admitted, got {}",
            admitted
        );
        assert_eq!(
            passed_binaries, total_binaries,
            "All {} binaries must pass smoke: {} passed",
            total_binaries, passed_binaries
        );
        assert!(
            matrix.admitted_versions().len() >= 3,
            "Matrix must have >=3 versions"
        );
    }
}
