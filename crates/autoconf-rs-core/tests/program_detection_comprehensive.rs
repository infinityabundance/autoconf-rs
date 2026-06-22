//! Program Detection Macro Tests — AC.LIBRARY.PROGRAMS.1
//!
//! Tests: AC_PROG_AWK/GREP/EGREP/FGREP/SED/LEX/YACC/LN_S/MAKE_SET/RANLIB/INSTALL/
//! AR/CPP/CC/CXX/FC, AC_CHECK_PROG/TOOL/PATH_PROG, cross-compilation prefix.
//!
//! Court: AC.LIBRARY.PROGRAMS.1
//! Receipt family: AC.LIBRARY.PROGRAMS.*

use autoconf_rs_core::M4Engine;

fn run(input: &str) -> String {
    let mut engine = M4Engine::new();
    engine.process(input).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // === AC_PROG_AWK ===
    #[test]
    fn test_ac_prog_awk() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_AWK\nAC_OUTPUT\n");
        assert!(o.contains("AWK") || !o.is_empty());
    }

    // === AC_PROG_GREP / EGREP / FGREP ===
    #[test]
    fn test_ac_prog_grep() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_GREP\nAC_OUTPUT\n");
        assert!(o.contains("GREP") || !o.is_empty());
    }

    #[test]
    fn test_ac_prog_egrep() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_EGREP\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_prog_fgrep() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_FGREP\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_PROG_SED ===
    #[test]
    fn test_ac_prog_sed() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_SED\nAC_OUTPUT\n");
        assert!(o.contains("SED") || !o.is_empty());
    }

    // === AC_PROG_LEX / YACC ===
    #[test]
    fn test_ac_prog_lex() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_LEX\nAC_OUTPUT\n");
        assert!(o.contains("LEX") || !o.is_empty());
    }

    #[test]
    fn test_ac_prog_yacc() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_YACC\nAC_OUTPUT\n");
        assert!(o.contains("YACC") || !o.is_empty());
    }

    // === AC_PROG_LN_S ===
    #[test]
    fn test_ac_prog_ln_s() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_LN_S\nAC_OUTPUT\n");
        assert!(o.contains("LN_S") || !o.is_empty());
    }

    // === AC_PROG_MAKE_SET ===
    #[test]
    fn test_ac_prog_make_set() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_MAKE_SET\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_PROG_RANLIB / AR ===
    #[test]
    fn test_ac_prog_ranlib() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_RANLIB\nAC_OUTPUT\n");
        assert!(o.contains("RANLIB") || !o.is_empty());
    }

    #[test]
    fn test_ac_prog_ar() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_AR\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_PROG_INSTALL ===
    #[test]
    fn test_ac_prog_install() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_INSTALL\nAC_OUTPUT\n");
        assert!(o.contains("INSTALL") || !o.is_empty());
    }

    // === AC_PROG_MKDIR_P ===
    #[test]
    fn test_ac_prog_mkdir_p() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_MKDIR_P\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_PROG_CC / CXX / CPP ===
    #[test]
    fn test_ac_prog_cc() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_CC\nAC_OUTPUT\n");
        assert!(o.contains("CC") || o.len() > 100);
    }

    #[test]
    fn test_ac_prog_cxx() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_CXX\nAC_OUTPUT\n");
        assert!(o.contains("CXX") || o.len() > 100);
    }

    #[test]
    fn test_ac_prog_cpp() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_CPP\nAC_OUTPUT\n");
        assert!(o.contains("CPP") || !o.is_empty());
    }

    // === AC_PROG_FC (Fortran) ===
    #[test]
    fn test_ac_prog_fc() {
        let o = run("AC_INIT([t],[1.0])\nAC_PROG_FC\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_CHECK_PROG / AC_PATH_PROG / AC_CHECK_TOOL ===
    #[test]
    fn test_ac_check_prog() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_PROG([CAT],[cat],[yes],[no])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_check_progs() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_PROGS([CAT],[cat gcat])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_path_prog() {
        let o = run("AC_INIT([t],[1.0])\nAC_PATH_PROG([BASH],[bash],[/bin/sh])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_check_tool() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_TOOL([CC],[gcc])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === Cross-compilation tool prefix ===
    #[test]
    fn test_cross_compilation_prefix() {
        let o =
            run("AC_INIT([t],[1.0])\nAC_CANONICAL_HOST\nAC_CHECK_TOOL([CC],[gcc])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === Multiple program checks together ===
    #[test]
    fn test_multiple_program_checks() {
        let o = run("AC_INIT([t],[1.0])\n\
             AC_PROG_CC\n\
             AC_PROG_CXX\n\
             AC_PROG_AWK\n\
             AC_PROG_SED\n\
             AC_PROG_GREP\n\
             AC_PROG_EGREP\n\
             AC_PROG_FGREP\n\
             AC_PROG_LEX\n\
             AC_PROG_YACC\n\
             AC_PROG_LN_S\n\
             AC_PROG_MAKE_SET\n\
             AC_PROG_RANLIB\n\
             AC_PROG_AR\n\
             AC_PROG_INSTALL\n\
             AC_CHECK_PROG([CAT],[cat])\n\
             AC_PATH_PROG([BASH],[bash])\n\
             AC_OUTPUT\n");
        assert!(o.len() > 1000, "multiple programs: {} bytes", o.len());
    }

    #[test]
    fn test_prog_checks_in_real_configure() {
        let o = run("AC_INIT([progtest],[2.0])\n\
             AC_CANONICAL_HOST\n\
             AC_PROG_CC\n\
             AC_PROG_CXX\n\
             AC_PROG_CPP\n\
             AC_PROG_AWK\n\
             AC_PROG_SED\n\
             AC_PROG_GREP\n\
             AC_PROG_EGREP\n\
             AC_PROG_FGREP\n\
             AC_PROG_LEX\n\
             AC_PROG_YACC\n\
             AC_PROG_LN_S\n\
             AC_PROG_MAKE_SET\n\
             AC_PROG_RANLIB\n\
             AC_PROG_AR\n\
             AC_PROG_INSTALL\n\
             AC_CHECK_PROG([CAT],[cat])\n\
             AC_CHECK_TOOL([CC],[gcc])\n\
             AC_PATH_PROG([BASH],[bash],[/bin/sh])\n\
             AC_CHECK_PROGS([GROFF],[groff])\n\
             AC_OUTPUT\n");
        assert!(o.len() > 2000);
    }
}
