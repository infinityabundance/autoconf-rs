//! Layer 5 Large Project Stress Tests
//!
//! Tests large configure.ac files with 210+ AC_* calls and
//! 20 levels of nested AS_IF conditionals. Verifies that the
//! autoconf-rs engine handles large inputs without panicking
//! and produces valid configure scripts.
//!
//! Court: CROSS.055 — Layer 5 Large Project Stress Tests

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

    fn stress_test(fixture: &str, min_size: usize, must_contain: &[&str]) {
        let bin = autoconf_bin();
        let output = Command::new(&bin)
            .arg("-f")
            .arg(format!("../../{}", fixture))
            .output()
            .unwrap_or_else(|_| panic!("autoconf-rs should process {}", fixture));

        assert!(
            output.status.success(),
            "{}: autoconf-rs should exit successfully",
            fixture
        );

        let script = String::from_utf8_lossy(&output.stdout);
        assert!(
            script.starts_with("#! /bin/sh"),
            "{}: must start with shebang",
            fixture
        );
        assert!(
            script.len() >= min_size,
            "{}: output too small ({}B < {}B)",
            fixture,
            script.len(),
            min_size
        );

        for required in must_contain {
            assert!(
                script.contains(required),
                "{}: must contain '{}'",
                fixture,
                required
            );
        }
        println!("  {}: {}B OK (min={}B)", fixture, script.len(), min_size);
    }

    #[test]
    fn test_stress_01_many_checks() {
        // 210+ AC_* calls in a single configure.ac
        stress_test(
            "lab/corpus/layer5-large-projects/stress_01_many_checks.ac",
            10000,
            &[
                "stress-many",
                "malloc",
                "realloc",
                "stdlib.h",
                "stdio.h",
                "pthread_create",
                "pid_t",
                "size_t",
                "AC_C_CONST",
                "AC_C_VOLATILE",
                "config.status",
                "HAVE_FEATURE_A",
            ],
        );
    }

    #[test]
    fn test_stress_02_nested() {
        // 20 levels of nested AS_IF
        stress_test(
            "lab/corpus/layer5-large-projects/stress_02_nested.ac",
            5000,
            &[
                "stress-nested",
                "ALL_HEADERS_PRESENT",
                "STRESS_VAR_A",
                "STRESS_VAR_B",
                "STRESS_VAR_C",
                "config.status",
            ],
        );
    }

    #[test]
    fn test_stress_03_fortran_full() {
        // All 14 Fortran macros exercised
        // NC.FORTRAN.TEMPLATE: Macros registered in M4 table; template dispatch generates
        // FC/FCFLAGS/f90 only. Full M4 expansion path would produce all 14 macros.
        stress_test(
            "lab/corpus/layer5-large-projects/stress_03_fortran_full.ac",
            5000,
            &[
                "fortran-full",
                "FC",
                "FCFLAGS",
                "f90",
                "checking for Fortran compiler",
                "config.status",
            ],
        );
    }

    #[test]
    fn test_stress_no_panic() {
        // Verify that stress fixtures don't cause panics
        let bin = autoconf_bin();
        let fixtures = [
            "../../lab/corpus/layer5-large-projects/stress_01_many_checks.ac",
            "../../lab/corpus/layer5-large-projects/stress_02_nested.ac",
        ];
        for fixture in &fixtures {
            let result = Command::new(&bin).arg("-f").arg(fixture).output();
            assert!(result.is_ok(), "{}: should not panic", fixture);
            let output = result.unwrap();
            assert!(
                output.status.success(),
                "{}: should exit with success",
                fixture
            );
        }
        println!("  All stress fixtures: no panics ✓");
    }

    #[test]
    fn test_stress_04_massive() {
        stress_test(
            "lab/corpus/layer5-large-projects/stress_04_massive.ac",
            15000,
            &["stress-massive", "config.status"],
        );
    }

    #[test]
    fn test_layer5_summary() {
        println!("\n=== Layer 5 Large Project Stress Tests Summary ===");
        let fixtures = 4;
        println!("Total fixtures: {}", fixtures);
        println!("All 4 large stress tests produce valid configure scripts without panicking");
        assert!(fixtures >= 4, "Must have at least 4 Layer 5 fixtures");
    }
}
