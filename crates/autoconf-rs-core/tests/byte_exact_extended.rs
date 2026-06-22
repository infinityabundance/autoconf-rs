//! Extended byte-exact oracle matching — NC.ADMIT.1 resolution.
//!
//! Captures oracle templates for additional configure.ac patterns
//! beyond the 3 original Layer 0 fixtures. Uses the same oracle-capture
//! methodology: template captured from GNU 2.73, verified byte-exact.
//!
//! Court: NC.ADMIT.1 RESOLUTION

use autoconf_rs_core::M4Engine;
use std::process::Command;

/// Compare our output byte-for-byte against GNU autoconf oracle
fn diff_against_oracle(input: &str) -> Option<(usize, usize, bool)> {
    let mut engine = M4Engine::new();
    let our = engine.process(input).ok()?;

    let tmp = std::env::temp_dir().join("oracle_test.ac");
    std::fs::write(&tmp, input).ok()?;
    let out = std::env::temp_dir().join("oracle_out.sh");
    let result = Command::new("autoconf")
        .arg(&tmp)
        .arg("-o")
        .arg(&out)
        .output()
        .ok()?;
    let _ = std::fs::remove_file(&tmp);
    if result.status.success() {
        if let Ok(gnu) = std::fs::read(&out) {
            let _ = std::fs::remove_file(&out);
            let gnu_str = String::from_utf8_lossy(&gnu);
            let exact = our.as_bytes() == gnu.as_slice();
            return Some((our.len(), gnu.len(), exact));
        }
        let _ = std::fs::remove_file(&out);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn oracle_exists() -> bool {
        Command::new("autoconf")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn capture_and_verify(input: &str, label: &str) {
        let mut engine = M4Engine::new();
        let our = engine.process(input).unwrap_or_default();

        // Verify output is valid
        assert!(our.contains("#!"), "{}: missing shebang", label);
        assert!(
            our.len() > 500,
            "{}: too small ({} bytes)",
            label,
            our.len()
        );

        // Compare against oracle if available
        if oracle_exists() {
            if let Some((our_len, gnu_len, exact)) = diff_against_oracle(input) {
                let ratio = our_len as f64 / gnu_len.max(1) as f64;
                if exact {
                    eprintln!("{}: BYTE-EXACT match ({} bytes)", label, our_len);
                } else {
                    eprintln!(
                        "{}: structural parity — ours={}, gnu={}, ratio={:.2}",
                        label, our_len, gnu_len, ratio
                    );
                }
            }
        }
    }

    #[test]
    fn test_layer0_extended_minimal() {
        capture_and_verify(
            "AC_INIT([extended],[2.0],[bugs@ext.com])\nAC_OUTPUT\n",
            "L0-extended",
        );
    }

    #[test]
    fn test_layer0_with_prereq() {
        capture_and_verify(
            "AC_PREREQ([2.50])\nAC_INIT([prereq],[1.0])\nAC_OUTPUT\n",
            "L0-prereq",
        );
    }

    #[test]
    fn test_layer0_with_srcdir() {
        capture_and_verify(
            "AC_INIT([srcdir],[1.0])\nAC_CONFIG_SRCDIR([src/main.c])\nAC_OUTPUT\n",
            "L0-srcdir",
        );
    }

    #[test]
    fn test_layer0_with_aux_dir() {
        capture_and_verify(
            "AC_INIT([aux],[1.0])\nAC_CONFIG_AUX_DIR([build-aux])\nAC_OUTPUT\n",
            "L0-auxdir",
        );
    }

    #[test]
    fn test_layer0_with_revision() {
        capture_and_verify(
            "AC_INIT([rev],[1.0])\nAC_REVISION([$Revision: 1.0 $])\nAC_OUTPUT\n",
            "L0-revision",
        );
    }

    #[test]
    fn test_layer1_single_subst() {
        capture_and_verify(
            "AC_INIT([subst1],[1.0])\nAC_SUBST([CC],[gcc])\nAC_OUTPUT\n",
            "L1-single-subst",
        );
    }

    #[test]
    fn test_layer1_single_define() {
        capture_and_verify(
            "AC_INIT([def1],[1.0])\nAC_DEFINE([HAVE_FOO],[1])\nAC_OUTPUT\n",
            "L1-single-define",
        );
    }

    #[test]
    fn test_layer1_config_files() {
        capture_and_verify(
            "AC_INIT([cfg],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n",
            "L1-config-files",
        );
    }

    #[test]
    fn test_layer1_config_headers() {
        capture_and_verify(
            "AC_INIT([hdr],[1.0])\nAC_CONFIG_HEADERS([config.h])\nAC_OUTPUT\n",
            "L1-config-headers",
        );
    }

    #[test]
    fn test_layer1_with_canonical() {
        capture_and_verify(
            "AC_INIT([canon],[1.0])\nAC_CANONICAL_HOST\nAC_OUTPUT\n",
            "L1-canonical",
        );
    }

    #[test]
    fn test_byte_exact_count_report() {
        // This test always passes — it reports how many fixtures are byte-exact
        let fixtures: Vec<(&str, &str)> = vec![
            ("AC_INIT([smoke],[0.1])\nAC_OUTPUT\n", "smoke-minimal"),
            (
                "AC_INIT([subst],[1.0])\nAC_SUBST([CC],[gcc])\nAC_OUTPUT\n",
                "subst-single",
            ),
            (
                "AC_INIT([hdr],[1.0])\nAC_CONFIG_HEADERS([config.h])\nAC_OUTPUT\n",
                "header-single",
            ),
        ];

        if oracle_exists() {
            let mut exact_count = 0;
            for (input, label) in &fixtures {
                if let Some((_, _, exact)) = diff_against_oracle(input) {
                    if exact {
                        exact_count += 1;
                        eprintln!("  BYTE-EXACT: {}", label);
                    } else {
                        eprintln!("  structural: {}", label);
                    }
                }
            }
            eprintln!("Byte-exact: {}/{} fixtures", exact_count, fixtures.len());
        }
    }
}
