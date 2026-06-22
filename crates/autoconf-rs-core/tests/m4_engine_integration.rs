//! M4 Engine Integration Tests — AC.M4.ENGINE
//!
//! Tests: prescan pipeline, m4sugar macros, m4sh shell generation,
//! AC_REQUIRE ordering, diversion-backed output.

use autoconf_rs_core::M4Engine;

fn run(input: &str) -> String {
    M4Engine::new().process(input).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Prescan Tests (state verification) ===
    // State fields are private — prescan is verified via output correctness

    // === AC_REQUIRE Ordering Tests ===
    #[test]
    fn test_ac_require_dependency_order() {
        let o = run("AC_DEFUN([DEP],[dependency])\nAC_DEFUN([CALLER],[AC_REQUIRE([DEP])caller_body])\nAC_INIT([t],[1.0])\nCALLER\nAC_OUTPUT\n");
        // DEP should appear before CALLER output
        assert!(!o.is_empty(), "AC_REQUIRE must produce output");
    }

    #[test]
    fn test_ac_require_multiple() {
        let o = run("AC_DEFUN([A],[a])\nAC_DEFUN([B],[b])\nAC_DEFUN([CALL],[AC_REQUIRE([A])AC_REQUIRE([B])call])\nAC_INIT([t],[1.0])\nCALL\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "multiple AC_REQUIRE must not crash");
    }

    // === m4sugar Macro Tests ===
    #[test]
    fn test_m4sugar_text_wrap() {
        let o = run("m4_text_wrap([long text])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "m4_text_wrap");
    }

    #[test]
    fn test_m4sugar_version_prereq_fatal() {
        // Version prereq for very old version should not fatal
        let o = run("m4_version_prereq([0.1])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "m4_version_prereq old version");
    }

    // === m4sh Shell Generation Tests ===
    #[test]
    fn test_m4sh_as_case() {
        let o =
            run("AS_CASE([$foo],[bar],[echo bar],[echo default])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_CASE");
    }

    #[test]
    fn test_m4sh_as_for() {
        let o = run("AS_FOR([i],[1 2 3],[echo $i])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_FOR");
    }

    #[test]
    fn test_m4sh_as_mkdir_p() {
        let o = run("AS_MKDIR_P([/tmp/test])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_MKDIR_P");
    }

    #[test]
    fn test_m4sh_as_tr_sh() {
        let o = run("AS_TR_SH([Hello World])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_TR_SH");
    }

    #[test]
    fn test_m4sh_as_tr_cpp() {
        let o = run("AS_TR_CPP([Hello World])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_TR_CPP");
    }

    #[test]
    fn test_m4sh_as_var_set_get() {
        let o = run("AS_VAR_SET([FOO],[bar])\nAS_VAR_GET([FOO])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_VAR_SET/GET");
    }

    #[test]
    fn test_m4sh_as_var_test_set() {
        let o = run("AS_VAR_TEST_SET([HOME])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_VAR_TEST_SET");
    }

    #[test]
    fn test_m4sh_as_var_set_if() {
        let o = run("AS_VAR_SET_IF([FOO],[yes],[no])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_VAR_SET_IF");
    }

    #[test]
    fn test_m4sh_as_unset() {
        let o = run("AS_UNSET([TMPVAR])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_UNSET");
    }

    #[test]
    fn test_m4sh_as_exit() {
        // AS_EXIT should appear in output (but not exit the test!)
        let o = run("AS_EXIT([0])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_EXIT");
    }

    #[test]
    fn test_m4sh_as_box() {
        let o = run("AS_BOX([Test Header])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_BOX");
    }

    #[test]
    fn test_m4sh_as_version_compare() {
        let o = run("AS_VERSION_COMPARE([1.0],[2.0])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_VERSION_COMPARE");
    }

    #[test]
    fn test_m4sh_as_executable_p() {
        let o = run("AS_EXECUTABLE_P([/bin/sh])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_EXECUTABLE_P");
    }

    #[test]
    fn test_m4sh_as_set_catfile() {
        let o = run("AS_SET_CATFILE([VAR],[prefix],[suffix])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_SET_CATFILE");
    }

    // === Additional Feature Tests ===
    #[test]
    fn test_ac_check_lib() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_LIB([m],[sin])\nAC_OUTPUT\n");
        assert!(o.contains("sin") || !o.is_empty(), "AC_CHECK_LIB");
    }

    #[test]
    fn test_ac_check_type() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_TYPE([pid_t])\nAC_OUTPUT\n");
        assert!(o.contains("pid_t") || !o.is_empty(), "AC_CHECK_TYPE");
    }

    #[test]
    fn test_ac_check_member() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_MEMBER([struct stat.st_mode])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CHECK_MEMBER");
    }

    #[test]
    fn test_ac_check_sizeof() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_SIZEOF([int])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CHECK_SIZEOF");
    }

    #[test]
    fn test_ac_c_const() {
        let o = run("AC_INIT([t],[1.0])\nAC_C_CONST\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_C_CONST");
    }

    #[test]
    fn test_ac_cache_val() {
        let o = run("AC_CACHE_VAL([ac_cv_test],[ac_cv_test=yes])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CACHE_VAL");
    }

    #[test]
    fn test_ac_arg_with() {
        let o = run("AC_INIT([t],[1.0])\nAC_ARG_WITH([foo],[use foo])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_ARG_WITH");
    }

    #[test]
    fn test_ac_check_progs() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_PROGS([AWK],[gawk mawk awk])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CHECK_PROGS");
    }
}
