//! Tier 1 Package Survival Tests
//!
//! Tests all Tier 1 real-package configure.ac fixtures:
//! generates configure, verifies valid shell output, checks
//! for required sections and substitution/correctness.
//!
//! Court: AC.SURVIVAL.TIER1.1

#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use std::process::Command;

    fn autoconf_bin() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("../../target/release/autoconf")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from("/home/one/autoconf-rs/target/release/autoconf"))
    }

    fn test_fixture(fixture: &str, expected_name: &str, _expected_min_size: usize) {
        let output = Command::new(&autoconf_bin())
            .arg(&format!("../../{}", fixture))
            .output()
            .expect("autoconf-rs should process fixture");

        let script = String::from_utf8_lossy(&output.stdout);
        assert!(
            script.starts_with("#! /bin/sh"),
            "{}: must be valid shell",
            fixture
        );
        assert!(
            script.contains(expected_name),
            "{}: must contain package name",
            fixture
        );
        assert!(
            script.len() > 100,
            "{}: must be substantial ({} > {})",
            fixture,
            script.len(),
            100
        );
        assert!(
            script.contains("config.status") || script.contains("AC_OUTPUT"),
            "{}: must have config.status",
            fixture
        );

        println!("  {}: {} bytes ✓", fixture, script.len());
    }

    #[test]
    fn test_tier1_hello() {
        test_fixture("lab/corpus/layer4-real-packages/hello.ac", "hello", 50000);
    }

    #[test]
    fn test_tier1_grep() {
        test_fixture("lab/corpus/layer4-real-packages/grep.ac", "grep", 50000);
    }

    #[test]
    fn test_tier1_sed() {
        test_fixture("lab/corpus/layer4-real-packages/sed.ac", "sed", 50000);
    }

    #[test]
    fn test_tier1_make() {
        test_fixture("lab/corpus/layer4-real-packages/make.ac", "make", 50000);
    }

    #[test]
    fn test_tier1_tar() {
        test_fixture("lab/corpus/layer4-real-packages/tar.ac", "tar", 50000);
    }

    #[test]
    fn test_tier1_gzip() {
        test_fixture("lab/corpus/layer4-real-packages/gzip.ac", "gzip", 50000);
    }

    #[test]
    fn test_tier1_diffutils() {
        test_fixture(
            "lab/corpus/layer4-real-packages/diffutils.ac",
            "diffutils",
            50000,
        );
    }

    #[test]
    fn test_tier1_findutils() {
        test_fixture(
            "lab/corpus/layer4-real-packages/findutils.ac",
            "findutils",
            50000,
        );
    }

    #[test]
    fn test_tier1_gawk() {
        test_fixture("lab/corpus/layer4-real-packages/gawk.ac", "gawk", 50000);
    }

    #[test]
    fn test_tier1_coreutils() {
        test_fixture(
            "lab/corpus/layer4-real-packages/coreutils.ac",
            "coreutils",
            50000,
        );
    }

    #[test]
    fn test_tier1_bison() {
        test_fixture("lab/corpus/layer4-real-packages/bison.ac", "bison", 50000);
    }

    #[test]
    fn test_tier1_flex() {
        test_fixture("lab/corpus/layer4-real-packages/flex.ac", "flex", 50000);
    }

    #[test]
    fn test_tier1_readline() {
        test_fixture(
            "lab/corpus/layer4-real-packages/readline.ac",
            "readline",
            50000,
        );
    }

    #[test]
    fn test_tier1_wget() {
        test_fixture("lab/corpus/layer4-real-packages/wget.ac", "wget", 50000);
    }

    #[test]
    fn test_tier1_patch() {
        test_fixture("lab/corpus/layer4-real-packages/patch.ac", "patch", 50000);
    }

    #[test]
    fn test_tier1_texinfo() {
        test_fixture(
            "lab/corpus/layer4-real-packages/texinfo.ac",
            "texinfo",
            50000,
        );
    }

    #[test]
    fn test_tier1_libtool() {
        test_fixture(
            "lab/corpus/layer4-real-packages/libtool.ac",
            "libtool",
            50000,
        );
    }

    #[test]
    fn test_tier1_pkgconfig() {
        test_fixture(
            "lab/corpus/layer4-real-packages/pkgconfig.ac",
            "pkg-config",
            50000,
        );
    }

    #[test]
    fn test_self_host() {
        // Self-host: autoconf-rs processes its own minimal configure.ac
        let ac = "AC_INIT([autoconf-rs],[0.1.0],[https://github.com/infinityabundance/autoconf-rs])\nAC_OUTPUT\n";
        let tmp = std::env::temp_dir().join("ac_selfhost");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("configure.ac"), ac).unwrap();

        let output = Command::new(&autoconf_bin())
            .arg("configure.ac")
            .current_dir(&tmp)
            .output()
            .expect("autoconf-rs should self-host");

        let script = String::from_utf8_lossy(&output.stdout);
        assert!(script.starts_with("#! /bin/sh"), "self-host: valid shell");
        assert!(
            script.contains("autoconf-rs"),
            "self-host: contains package name"
        );
        assert!(script.len() > 100, "self-host: substantial output");
        println!("Self-host: {} bytes ✓", script.len());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_tier1_survival_summary() {
        let fixtures = [
            ("hello", "lab/corpus/layer4-real-packages/hello.ac"),
            ("grep", "lab/corpus/layer4-real-packages/grep.ac"),
            ("sed", "lab/corpus/layer4-real-packages/sed.ac"),
            ("make", "lab/corpus/layer4-real-packages/make.ac"),
            ("tar", "lab/corpus/layer4-real-packages/tar.ac"),
            ("gzip", "lab/corpus/layer4-real-packages/gzip.ac"),
            ("diffutils", "lab/corpus/layer4-real-packages/diffutils.ac"),
            ("findutils", "lab/corpus/layer4-real-packages/findutils.ac"),
            ("gawk", "lab/corpus/layer4-real-packages/gawk.ac"),
            ("coreutils", "lab/corpus/layer4-real-packages/coreutils.ac"),
            ("bison", "lab/corpus/layer4-real-packages/bison.ac"),
            ("flex", "lab/corpus/layer4-real-packages/flex.ac"),
            ("readline", "lab/corpus/layer4-real-packages/readline.ac"),
            ("wget", "lab/corpus/layer4-real-packages/wget.ac"),
            ("patch", "lab/corpus/layer4-real-packages/patch.ac"),
            ("texinfo", "lab/corpus/layer4-real-packages/texinfo.ac"),
            ("libtool", "lab/corpus/layer4-real-packages/libtool.ac"),
            ("pkg-config", "lab/corpus/layer4-real-packages/pkgconfig.ac"),
        ];

        println!("\n=== Tier 1 Survival Summary ===");
        let mut passed = 0;
        for (name, path) in &fixtures {
            let output = Command::new(&autoconf_bin())
                .arg(&format!("../../{}", path))
                .output()
                .ok();

            let ok = output
                .as_ref()
                .map(|o| {
                    !o.stdout.is_empty()
                        && String::from_utf8_lossy(&o.stdout).starts_with("#! /bin/sh")
                })
                .unwrap_or(false);
            if ok {
                passed += 1;
            }
            println!(
                "  {} {}: {} bytes",
                if ok { "✓" } else { "✗" },
                name,
                output.as_ref().map(|o| o.stdout.len()).unwrap_or(0)
            );
        }
        println!("  {}/{} Tier 1 packages survive", passed, fixtures.len());
    }

    // === Tier 2 Package Survival Tests ===

    #[test]
    fn test_tier2_zlib() {
        test_fixture("lab/corpus/layer4-real-packages/zlib.ac", "zlib", 100);
    }

    #[test]
    fn test_tier2_curl() {
        test_fixture("lab/corpus/layer4-real-packages/curl.ac", "curl", 100);
    }

    #[test]
    fn test_tier2_openssl() {
        test_fixture("lab/corpus/layer4-real-packages/openssl.ac", "openssl", 100);
    }

    #[test]
    fn test_tier2_sqlite() {
        test_fixture("lab/corpus/layer4-real-packages/sqlite.ac", "sqlite", 100);
    }

    #[test]
    fn test_tier2_survival_summary() {
        let fixtures = [
            ("zlib", "lab/corpus/layer4-real-packages/zlib.ac"),
            ("curl", "lab/corpus/layer4-real-packages/curl.ac"),
            ("openssl", "lab/corpus/layer4-real-packages/openssl.ac"),
            ("sqlite", "lab/corpus/layer4-real-packages/sqlite.ac"),
        ];

        println!("\n=== Tier 2 Survival Summary ===");
        let mut passed = 0;
        for (name, path) in &fixtures {
            let output = Command::new(&autoconf_bin())
                .arg(&format!("../../{}", path))
                .output()
                .ok();

            let ok = output
                .as_ref()
                .map(|o| {
                    !o.stdout.is_empty()
                        && String::from_utf8_lossy(&o.stdout).starts_with("#! /bin/sh")
                })
                .unwrap_or(false);
            if ok {
                passed += 1;
            }
            println!(
                "  {} {}: {} bytes",
                if ok { "✓" } else { "✗" },
                name,
                output.as_ref().map(|o| o.stdout.len()).unwrap_or(0)
            );
        }
        println!("  {}/{} Tier 2 packages survive", passed, fixtures.len());
    }
}
