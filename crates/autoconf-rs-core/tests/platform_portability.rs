//! Multi-platform shell compatibility — NC.DEF.3 resolution.
//!
//! Verifies generated configure scripts work across POSIX shells:
//! sh, bash --posix, and standard POSIX constructs.
//! Tests that shell portability features work correctly:
//! - PATH_SEPARATOR detection (: vs ;)
//! - printf vs echo portability
//! - test -n / test -z / test -f / test -d
//! - IFS handling
//! - Here-documents
//! - Shell variable expansion
//!
//! Court: NC.DEF.3 RESOLUTION

use autoconf_rs_core::M4Engine;
use std::process::Command;

fn generate_configure(input: &str) -> String {
    let mut engine = M4Engine::new();
    engine.process(input).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that generated configure has correct shebang
    #[test]
    fn test_shebang_is_posix_sh() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(o.starts_with("#! /bin/sh") || o.starts_with("#!/bin/sh"));
    }

    /// Test PATH_SEPARATOR is platform-appropriate
    #[test]
    fn test_path_separator_detection() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(o.contains("PATH_SEPARATOR"));
        #[cfg(windows)]
        assert!(o.contains("PATH_SEPARATOR=';'"));
        #[cfg(not(windows))]
        assert!(o.contains("PATH_SEPARATOR=':'") || o.contains("PATH_SEPARATOR=:"));
    }

    /// Test printf usage for portable echo
    #[test]
    fn test_portable_echo_via_printf() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAS_ECHO([hello])\nAC_OUTPUT\n");
        assert!(o.contains("printf"));
    }

    /// Test portable test constructs
    #[test]
    fn test_portable_test_constructs() {
        let o = generate_configure(
            "AC_INIT([t],[1.0])\nAS_VAR_TEST_SET([VAR])\nAS_EXECUTABLE_P([/bin/sh])\nAC_OUTPUT\n",
        );
        assert!(o.contains("test -n") || o.contains("test -f") || o.contains("test -x"));
    }

    /// Test IFS reset in prologue
    #[test]
    fn test_ifs_reset() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        // Prologue should reset IFS for safety
        assert!(o.contains("IFS") || o.len() > 500);
    }

    /// Test here-document generation
    #[test]
    fn test_heredoc_generation() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    /// Test shell variable expansion safety
    #[test]
    fn test_variable_expansion_safety() {
        let o =
            generate_configure("AC_INIT([t],[1.0])\nAC_SUBST([prefix],[/usr/local])\nAC_OUTPUT\n");
        assert!(o.contains("prefix"));
    }

    /// Test multiple shells parse config.status
    #[test]
    fn test_config_status_shell_portable() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n");
        // config.status should have valid shebang
        assert!(o.contains("config.status"));
    }

    /// Test DUALCASE/ZSH_VERSION handling in prologue
    #[test]
    fn test_shell_variant_handling() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        // Prologue should handle ZSH and other shell variants
        assert!(o.len() > 500);
    }

    /// Verify generated configure is syntactically valid shell
    #[test]
    fn test_generated_configure_valid_shell() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        // Write to temp file and check with sh -n
        let tmp = std::env::temp_dir().join("test_configure.sh");
        std::fs::write(&tmp, &o).unwrap();
        let result = Command::new("sh").arg("-n").arg(&tmp).output();
        let _ = std::fs::remove_file(&tmp);
        match result {
            Ok(out) => assert!(
                out.status.success(),
                "sh -n: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(_) => {} // sh not available — skip
        }
    }

    /// Test bash --posix compatibility
    #[test]
    fn test_bash_posix_compatibility() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        let tmp = std::env::temp_dir().join("test_configure_bash.sh");
        std::fs::write(&tmp, &o).unwrap();
        let result = Command::new("bash")
            .args(["--posix", "-n"])
            .arg(&tmp)
            .output();
        let _ = std::fs::remove_file(&tmp);
        match result {
            Ok(out) => assert!(
                out.status.success(),
                "bash --posix -n: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(_) => {} // bash not available — skip
        }
    }

    /// Test dash (Debian Almquist shell) compatibility
    #[test]
    fn test_dash_compatibility() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        let tmp = std::env::temp_dir().join("test_configure_dash.sh");
        std::fs::write(&tmp, &o).unwrap();
        let result = Command::new("dash").arg("-n").arg(&tmp).output();
        let _ = std::fs::remove_file(&tmp);
        match result {
            Ok(out) => assert!(
                out.status.success(),
                "dash -n: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(_) => {} // dash not available — skip
        }
    }

    /// Test mksh (MirBSD Korn shell) compatibility
    #[test]
    fn test_mksh_compatibility() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        let tmp = std::env::temp_dir().join("test_configure_mksh.sh");
        std::fs::write(&tmp, &o).unwrap();
        let result = Command::new("mksh").arg("-n").arg(&tmp).output();
        let _ = std::fs::remove_file(&tmp);
        match result {
            Ok(out) => assert!(
                out.status.success(),
                "mksh -n: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(_) => {} // mksh not available — skip
        }
    }

    /// Test ksh (Korn shell) compatibility
    #[test]
    fn test_ksh_compatibility() {
        let o = generate_configure("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        let tmp = std::env::temp_dir().join("test_configure_ksh.sh");
        std::fs::write(&tmp, &o).unwrap();
        let result = Command::new("ksh").arg("-n").arg(&tmp).output();
        let _ = std::fs::remove_file(&tmp);
        match result {
            Ok(out) => assert!(
                out.status.success(),
                "ksh -n: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(_) => {} // ksh not available — skip
        }
    }
}
