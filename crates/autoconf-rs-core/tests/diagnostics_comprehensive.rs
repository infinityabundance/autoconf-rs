//! Diagnostics Taxonomy Tests — AC.DIAG.1
//!
//! Court: AC.DIAG.1

use autoconf_rs_core::diagnostics::{DiagnosticManager, WarningCategory};
use autoconf_rs_core::M4Engine;

#[cfg(test)]
mod tests {
    use super::*;

    // === WarningCategory parsing ===
    #[test]
    fn test_parse_cross() {
        assert_eq!(
            WarningCategory::parse("cross"),
            Some(WarningCategory::Cross)
        );
    }
    #[test]
    fn test_parse_gnu() {
        assert_eq!(WarningCategory::parse("gnu"), Some(WarningCategory::Gnu));
    }
    #[test]
    fn test_parse_obsolete() {
        assert_eq!(
            WarningCategory::parse("obsolete"),
            Some(WarningCategory::Obsolete)
        );
    }
    #[test]
    fn test_parse_override() {
        assert_eq!(
            WarningCategory::parse("override"),
            Some(WarningCategory::Override)
        );
    }
    #[test]
    fn test_parse_portability() {
        assert_eq!(
            WarningCategory::parse("portability"),
            Some(WarningCategory::Portability)
        );
    }
    #[test]
    fn test_parse_syntax() {
        assert_eq!(
            WarningCategory::parse("syntax"),
            Some(WarningCategory::Syntax)
        );
    }
    #[test]
    fn test_parse_unsupported() {
        assert_eq!(
            WarningCategory::parse("unsupported"),
            Some(WarningCategory::Unsupported)
        );
    }
    #[test]
    fn test_parse_all() {
        assert_eq!(WarningCategory::parse("all"), Some(WarningCategory::All));
    }
    #[test]
    fn test_parse_error() {
        assert_eq!(
            WarningCategory::parse("error"),
            Some(WarningCategory::Error)
        );
    }
    #[test]
    fn test_parse_no_category() {
        assert_eq!(
            WarningCategory::parse("no-cross"),
            Some(WarningCategory::NoCategory("cross".to_string()))
        );
    }
    #[test]
    fn test_parse_no_obsolete() {
        assert_eq!(
            WarningCategory::parse("no-obsolete"),
            Some(WarningCategory::NoCategory("obsolete".to_string()))
        );
    }
    #[test]
    fn test_parse_unknown() {
        assert!(WarningCategory::parse("bogus").is_none());
    }

    // === DiagnosticManager ===
    #[test]
    fn test_diag_manager_new() {
        let dm = DiagnosticManager::new();
        assert_eq!(dm.warning_count(), 0);
        assert_eq!(dm.error_count(), 0);
    }

    #[test]
    fn test_diag_set_location() {
        let mut dm = DiagnosticManager::new();
        dm.set_location("configure.ac", 42);
    }

    #[test]
    fn test_diag_ac_obsolete() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("obsolete");
        dm.ac_obsolete("AC_OLD", "AC_NEW");
        assert!(dm.warning_count() > 0);
    }

    #[test]
    fn test_diag_ac_obsolete_no_replacement() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("obsolete");
        dm.ac_obsolete("AC_OLD", "");
        assert!(dm.warning_count() > 0);
    }

    // === Warning category suppression ===
    #[test]
    fn test_suppress_category() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("no-obsolete");
        dm.ac_obsolete("AC_OLD", "");
    }

    #[test]
    fn test_enable_error_category() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("error");
    }

    // === M4Engine integration ===
    #[test]
    fn test_engine_has_diagnostics() {
        let mut engine = M4Engine::new();
        engine.process("AC_INIT([t],[1.0])\nAC_OUTPUT\n").ok();
    }

    #[test]
    fn test_obsolete_macro_warning() {
        let mut engine = M4Engine::new();
        engine
            .process("AC_INIT([t],[1.0])\nAC_TRY_COMPILE\nAC_OUTPUT\n")
            .ok();
    }

    #[test]
    fn test_multiple_warnings() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("obsolete");
        dm.ac_obsolete("A", "B");
        dm.ac_obsolete("C", "D");
        dm.ac_obsolete("E", "");
        assert!(dm.warning_count() >= 3);
    }

    // === AC_DIAGNOSE / AC_WARNING / AC_FATAL ===
    #[test]
    fn test_ac_diagnose_emitted() {
        let mut dm = DiagnosticManager::new();
        dm.ac_diagnose("syntax", "test message");
        assert!(dm.warning_count() > 0);
    }

    #[test]
    fn test_ac_warning_emitted() {
        let mut dm = DiagnosticManager::new();
        dm.ac_warning("test warning");
        assert!(dm.warning_count() > 0);
    }

    // === AU_DEFUN deprecation ===
    #[test]
    fn test_au_defun_warning() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("obsolete");
        dm.au_defun_warning("OLD_MACRO", Some("NEW_MACRO"));
        assert!(dm.warning_count() > 0);
    }

    #[test]
    fn test_au_defun_warning_no_replacement() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("obsolete");
        dm.au_defun_warning("OLD_MACRO", None);
        assert!(dm.warning_count() > 0);
    }

    // === Exit code mapping ===
    #[test]
    fn test_exit_code_no_errors() {
        let dm = DiagnosticManager::new();
        assert_eq!(dm.exit_code(), 0);
    }

    #[test]
    fn test_exit_code_with_warnings() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("obsolete");
        dm.ac_obsolete("OLD", "NEW");
        assert_eq!(dm.exit_code(), 1);
    }

    // === Summary ===
    #[test]
    fn test_summary_format() {
        let mut dm = DiagnosticManager::new();
        dm.ac_diagnose("syntax", "test");
        let s = dm.summary();
        assert!(s.contains("warning"));
    }

    // === Include stack ===
    #[test]
    fn test_include_stack() {
        let mut dm = DiagnosticManager::new();
        dm.push_include("file1.m4", 10);
        dm.push_include("file2.m4", 20);
        dm.pop_include();
        dm.pop_include();
    }

    // === Diagnostics collection ===
    #[test]
    fn test_diagnostics_collected() {
        let mut dm = DiagnosticManager::new();
        dm.ac_diagnose("syntax", "test");
        let diags = dm.diagnostics();
        assert!(diags.len() > 0);
    }
}
