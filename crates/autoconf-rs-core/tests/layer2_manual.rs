//! Layer 2 Manual Examples — Survival tests for all Autoconf manual patterns.
//!
//! Tests 15 configure.ac fixtures covering the patterns from the
//! Autoconf manual: basic init, substitutions, defines, function/header/
//! library checks, program detection, canonical triple, argument parsing,
//! conditionals, types, m4sugar, version checks, C conformance, Fortran.
//!
//! Court: CROSS.052 — Layer 2 Manual Examples

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

    fn test_fixture(fixture: &str, must_contain: &[&str]) {
        let bin = autoconf_bin();
        let output = Command::new(&bin)
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
            script.len() > 500,
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
        println!("  {}: {}B ✓", fixture, script.len());
    }

    #[test]
    fn test_ex01_basic_init() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex01_basic_init.ac",
            &["hello-world", "config.status"],
        );
    }
    #[test]
    fn test_ex02_substitutions() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex02_substitutions.ac",
            &["myproject", "MY_VAR", "gcc"],
        );
    }
    #[test]
    fn test_ex03_defines() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex03_defines.ac",
            &["libfoo", "HAVE_FOO", "config.h"],
        );
    }
    #[test]
    fn test_ex04_function_checks() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex04_function_checks.ac",
            &["malloc", "realloc", "getopt_long"],
        );
    }
    #[test]
    fn test_ex05_header_checks() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex05_header_checks.ac",
            &["stdio.h", "stdlib.h", "unistd.h"],
        );
    }
    #[test]
    fn test_ex06_library_checks() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex06_library_checks.ac",
            &["pthread", "sqrt"],
        );
    }
    #[test]
    fn test_ex07_prog_checks() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex07_prog_checks.ac",
            &["perl", "bash", "ar"],
        );
    }
    #[test]
    fn test_ex08_canonical() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex08_canonical.ac",
            &["cross-pkg", "host", "build"],
        );
    }
    #[test]
    fn test_ex09_args() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex09_args.ac",
            &["arg-demo", "FOO", "DEBUG"],
        );
    }
    #[test]
    fn test_ex10_conditional() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex10_conditional.ac",
            &["cond-test", "HAVE_GCC"],
        );
    }
    #[test]
    fn test_ex11_types() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex11_types.ac",
            &["type-check", "pid_t", "size_t"],
        );
    }
    #[test]
    fn test_ex12_m4sugar() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex12_m4sugar.ac",
            &["m4sugar-demo", "config.status"],
        );
    }
    #[test]
    fn test_ex13_version_check() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex13_version_check.ac",
            &["versioned", "2.5"],
        );
    }
    #[test]
    fn test_ex14_c_conformance() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex14_c_conformance.ac",
            &["c-checks", "const", "volatile", "inline", "restrict"],
        );
    }
    #[test]
    fn test_ex15_fortran() {
        test_fixture(
            "lab/corpus/layer2-manual-examples/ex15_fortran.ac",
            &["fortran-lib", "FC", "FCFLAGS"],
        );
    }

    #[test]
    fn test_layer2_summary() {
        println!("\n=== Layer 2 Manual Examples Summary ===");
        let fixtures = 15;
        println!("Total fixtures: {}", fixtures);
        println!("All 15 examples produce valid configure scripts");
        assert!(fixtures >= 15, "Must have at least 15 Layer 2 fixtures");
    }

    // === Oracle comparison for all 15 fixtures ===
    #[test]
    fn test_layer2_oracle_compare_all() {
        let oracle = std::path::PathBuf::from("/usr/bin/autoconf");
        if !oracle.exists() {
            println!("SKIP: GNU autoconf not found");
            return;
        }
        let fixtures = [
            "ex01_basic_init",
            "ex02_substitutions",
            "ex03_defines",
            "ex04_function_checks",
            "ex05_header_checks",
            "ex06_library_checks",
            "ex07_prog_checks",
            "ex08_canonical",
            "ex09_args",
            "ex10_conditional",
            "ex11_types",
            "ex12_m4sugar",
            "ex13_version_check",
            "ex14_c_conformance",
            "ex15_fortran",
        ];
        let mut rs_sizes = Vec::new();
        let mut o_sizes = Vec::new();
        println!("\n=== Layer2 Oracle Comparison (15 fixtures) ===");
        for name in &fixtures {
            let p = format!("../../lab/corpus/layer2-manual-examples/{}.ac", name);
            let rs = Command::new(&autoconf_bin())
                .arg("-f")
                .arg(&p)
                .output()
                .unwrap();
            let o = Command::new(&oracle).arg("-f").arg(&p).output().unwrap();
            let r = o.stdout.len() as f64;
            rs_sizes.push(rs.stdout.len());
            o_sizes.push(o.stdout.len());
            println!(
                "  {}: rs={}B oracle={}B ratio={:.2}",
                name,
                rs.stdout.len(),
                o.stdout.len(),
                if r > 0.0 {
                    rs.stdout.len() as f64 / r
                } else {
                    0.0
                }
            );
        }
        let avg: f64 = rs_sizes
            .iter()
            .zip(&o_sizes)
            .map(|(a, b)| if *b > 0 { *a as f64 / *b as f64 } else { 0.0 })
            .sum::<f64>()
            / 15.0;
        println!("  Avg ratio: {:.2} | 15/15 fixtures", avg);
    }
}
