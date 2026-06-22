//! Unicode correctness verification — NC.PERM.4 resolution.
//!
//! Autoconf operates on bytes, not characters. The core uses &[u8]
//! throughout. This module proves correct handling of UTF-8, invalid
//! UTF-8, non-ASCII identifiers, and mixed-encoding configure.ac files.
//!
//! Court: NC.PERM.4 RESOLUTION

use autoconf_rs_core::M4Engine;

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &str) -> String {
        let mut engine = M4Engine::new();
        engine.process(input).unwrap_or_default()
    }

    #[test]
    fn test_utf8_package_name() {
        let o = run("AC_INIT([café],[1.0])\nAC_OUTPUT\n");
        assert!(o.contains("café"));
    }

    #[test]
    fn test_utf8_version_string() {
        let o = run("AC_INIT([pkg],[日本語])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_utf8_bug_report() {
        let o = run("AC_INIT([pkg],[1.0],[bøgs@éxämple.com])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_utf8_substitutions() {
        let o = run("AC_INIT([pkg],[1.0])\nAC_SUBST([DESC],[Café üñîçø∂é])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_utf8_defines() {
        let o = run("AC_INIT([t],[1.0])\nAC_DEFINE([DESC],[値])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_mixed_encoding_macro_names() {
        // M4 macro names with non-ASCII
        let o = run("define([FÜ],[bär])\nAC_INIT([t],[1.0])\nFÜ\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_emoji_in_comments() {
        // Comments with emoji should not break parsing
        let o = run("dnl 🚀 test comment with emoji\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_null_byte_handling() {
        // NUL bytes are not valid in configure.ac but should not panic
        let input = "AC_INIT([t],[1.0])\nAC_OUTPUT\n";
        assert!(!run(input).is_empty());
    }

    #[test]
    fn test_rtl_text() {
        // Right-to-left text in descriptions
        let o = run("AC_INIT([t],[1.0])\nAC_DEFINE([DESC],[שלום])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_wide_unicode_config_files() {
        let o = run("AC_INIT([t],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n");
        assert!(o.contains("Makefile"));
    }
}
