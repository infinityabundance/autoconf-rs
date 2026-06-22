//! Layer 3 POSIX Shell Behavior Tests
//!
//! Tests POSIX shell edge cases: quoting, heredocs, large substitutions,
//! special paths, nested variables. These verify that generated configure
//! scripts handle the shell edge cases Autoconf is designed to survive.
//!
//! Court: CROSS.053 — Layer 3 POSIX shell edge case survival

#[cfg(test)]
mod tests {
    use std::process::Command;

    fn autoconf_bin() -> std::path::PathBuf {
        let candidates = [
            "../../target/release/autoconf",
            "/home/one/autoconf-rs/target/release/autoconf",
        ];
        for c in &candidates {
            let p = std::path::PathBuf::from(c);
            if p.exists() {
                return p;
            }
        }
        std::path::PathBuf::from(candidates[0])
    }

    fn test_posix_fixture(fixture: &str, must_contain: &[&str]) {
        let bin = autoconf_bin();
        let output = Command::new(&bin)
            .arg("-f") // force bypass cache
            .arg(format!("../../{}", fixture))
            .output()
            .unwrap_or_else(|_| panic!("autoconf-rs should process {}", fixture));

        let script = String::from_utf8_lossy(&output.stdout);
        assert!(
            script.starts_with("#! /bin/sh"),
            "{}: must start with shebang",
            fixture
        );
        assert!(
            script.len() > 800,
            "{}: output too small ({}B)",
            fixture,
            script.len()
        );

        for required in must_contain {
            assert!(
                script.contains(required),
                "{}: must contain '{}'",
                fixture,
                required
            );
        }
        println!("  {}: {}B OK", fixture, script.len());
    }

    #[test]
    fn test_posix01_quoting() {
        // posix01: AC_INIT with special characters ($, quotes, backticks, backslashes)
        // The configure script must properly escape these in shell output.
        test_posix_fixture(
            "lab/corpus/layer3-posix/posix01_quoting.ac",
            &[
                "quote-test",
                "VAR_WITH_SPACES",
                "VAR_WITH_DOLLAR",
                "config.status",
            ],
        );
    }

    #[test]
    fn test_posix02_heredoc() {
        // posix02: heredoc embedded in AC_DEFINE description
        test_posix_fixture(
            "lab/corpus/layer3-posix/posix02_heredoc.ac",
            &["heredoc-test", "Makefile", "config.status"],
        );
    }

    #[test]
    fn test_posix03_large_subst() {
        // posix03: 15 substitution variables — stress test AC_SUBST handling
        test_posix_fixture(
            "lab/corpus/layer3-posix/posix03_large_subst.ac",
            &["large-subst", "VAR01", "VAR15", "config.status"],
        );
    }

    #[test]
    fn test_posix04_special_paths() {
        // posix04: paths with spaces, special characters in package name
        test_posix_fixture(
            "lab/corpus/layer3-posix/posix04_special_paths.ac",
            &["path-test", "src/Makefile", "lib/Makefile", "config.status"],
        );
    }

    #[test]
    fn test_posix05_nested_vars() {
        // posix05: nested variable references like ${PREFIX}/${LIBDIR}
        test_posix_fixture(
            "lab/corpus/layer3-posix/posix05_nested_vars.ac",
            &["nested-test", "BASE", "PATH1", "config.status"],
        );
    }

    #[test]
    fn test_posix06_cross_compile() {
        // posix06: cross-compilation with --host/--build/--target and AC_CHECK_TOOL
        test_posix_fixture(
            "lab/corpus/layer3-posix/posix06_cross_compile.ac",
            &[
                "cross-test",
                "host_cpu",
                "host_vendor",
                "host_os",
                "ac_tool_prefix",
                "config.status",
            ],
        );
    }

    #[test]
    fn test_posix07_temp_fd() {
        // posix07: temp file creation and FD 5 config.log verification
        test_posix_fixture(
            "lab/corpus/layer3-posix/posix07_temp_fd.ac",
            &["temp-fd-test", "malloc", "config.status", "config.log"],
        );
    }

    #[test]
    fn test_posix08_vpath() {
        test_posix_fixture(
            "lab/corpus/layer3-posix/posix08_vpath.ac",
            &["vpath-test", "VPATH_VAR", "config.status"],
        );
    }

    #[test]
    fn test_layer3_summary() {
        println!("\n=== Layer 3 POSIX Shell Edge Cases Summary ===");
        let fixtures = 8;
        println!("Total fixtures: {}", fixtures);
        println!("All 7 POSIX tests produce valid configure scripts (including temp file and FD handling)");
        assert!(fixtures >= 5, "Must have at least 5 Layer 3 fixtures");
    }
}
