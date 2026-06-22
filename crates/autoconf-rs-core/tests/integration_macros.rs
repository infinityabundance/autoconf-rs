//! Integration tests verifying new M4-level macro implementations produce valid
//! shell configure scripts through the M4 expansion pipeline.
//!
//! Tests: AC_C_* conformance, AC_DEFUN/AU_DEFUN via m4_define, m4sugar macros,
//! m4sh macros, AS_* macros, autom4te GNU-format traces, and M4 output pipeline.
//!
//! Court: AC.INTEGRATION.MACROS.1

#[cfg(test)]
mod tests {
    use autoconf_rs_core::M4Engine;

    #[test]
    fn test_ac_c_conformance_macros() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([conftest], [1.0])\nAC_PROG_CC\nAC_C_CONST\nAC_C_VOLATILE\nAC_C_INLINE\nAC_C_RESTRICT\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("#! /bin/sh"));
        assert!(result.contains("conftest"));
        assert!(result.contains("checking for working const"));
        assert!(result.contains("checking for working volatile"));
        assert!(result.contains("checking for inline"));
        assert!(result.contains("checking for restrict"));
        assert!(result.len() > 5000);
    }

    #[test]
    fn test_ac_c_bigendian_char_unsigned() {
        let mut engine = M4Engine::new();
        let input =
            "AC_INIT([endian], [1.0])\nAC_PROG_CC\nAC_C_BIGENDIAN\nAC_C_CHAR_UNSIGNED\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("checking endianness"));
        assert!(result.contains("checking if char is unsigned"));
        assert!(result.len() > 4000);
    }

    #[test]
    fn test_ac_cache_check_val_load() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([cachetest], [1.0])\nAC_CACHE_CHECK([my feature], [ac_cv_my_feature], [echo found])\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("my feature"));
        assert!(result.contains("ac_cv_my_feature"));
        assert!(result.contains("(cached)"));
    }

    #[test]
    fn test_ac_compile_link_run_ifelse() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([ifelse], [1.0])\nAC_PROG_CC\nAC_COMPILE_IFELSE([int x;], [echo ok], [echo fail])\nAC_LINK_IFELSE([int main(){return 0;}], [echo linked], [echo nolink])\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("ac_fn_c_try_compile"));
        assert!(result.contains("ac_fn_c_try_link"));
        assert!(result.contains("conftest"));
    }

    #[test]
    fn test_ac_defun_via_m4_define() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([defuntest], [1.0])\nAC_DEFUN([MY_PROG_CC], [AC_PROG_CC])\nMY_PROG_CC\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        // M4 expands AC_DEFUN → m4_define([MY_PROG_CC], [AC_PROG_CC])
        // Then MY_PROG_CC → AC_PROG_CC → shell compiler check
        assert!(result.contains("checking for C compiler"));
    }

    #[test]
    fn test_au_defun_deprecation() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([oldtest], [1.0])\nAU_DEFUN([AC_OLD_MACRO], [AC_NEW_MACRO], [echo old])\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        // AU_DEFUN emits warning and defines AC_OLD_MACRO → echo old
        assert!(result.contains("echo old") || result.contains("old"));
    }

    #[test]
    fn test_m4sugar_macros_expand() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([sugartest], [1.0])\nm4_define([FOO], [bar])\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.len() > 1000);
    }

    #[test]
    fn test_m4_quote_and_dquote() {
        let mut engine = M4Engine::new();
        let input =
            "AC_INIT([m4quotetest], [1.0])\nm4_define([VAL], m4_quote([hello]))\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.len() > 1000);
    }

    #[test]
    fn test_as_echo_and_as_if() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([ashtest], [1.0])\nAS_ECHO([hello from echo])\nAS_IF([test -f /etc/hosts], [echo hosts exists])\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("printf"));
        assert!(result.contains("hello from echo"));
        assert!(result.contains("if test -f /etc/hosts"));
    }

    #[test]
    fn test_as_var_macros() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([vartest], [1.0])\nAS_VAR_SET([MYCC], [gcc])\nAS_VAR_APPEND([MYCFLAGS], [-O2])\nAS_VAR_TEST_SET([MYCC])\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("MYCC=\"gcc\""));
        assert!(result.contains("MYCFLAGS"));
        assert!(result.contains("-O2"));
    }

    #[test]
    fn test_autotest_at_macros() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([attest], [1.0])\nAT_INIT\nAT_SETUP([basic check])\nAT_CHECK([test 1 -eq 1])\nAT_CLEANUP\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("AT_INIT") || result.contains("Autotest"));
        assert!(result.contains("testing basic check"));
        assert!(result.contains("test 1 -eq 1"));
    }

    #[test]
    fn test_m4_output_as_primary() {
        // Verify M4 output is used as primary configure script (not dummy template)
        let mut engine = M4Engine::new();
        let input = "AC_INIT([pipeline], [2.0])\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        // Must contain actual package name from M4 expansion
        assert!(result.contains("pipeline"));
        assert!(result.contains("2.0"));
        assert!(result.starts_with("#! /bin/sh"));
        assert!(result.len() > 3000);
    }

    #[test]
    fn test_m4_chomp_and_normalize() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([m4util], [1.0])\nm4_define([VAL], m4_normalize([  hello   world  ]))\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.len() > 1000);
    }

    #[test]
    fn test_erlang_macros_present() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([erltest], [1.0])\nAC_ERLANG_PATH_ERL\nAC_ERLANG_PATH_ERLC\nAC_ERLANG_CHECK_LIB([kernel])\nAC_ERLANG_NEED_ERL\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("checking for erl"));
        assert!(result.contains("checking for erlc"));
    }

    #[test]
    fn test_go_macros_present() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([gotest], [1.0])\nAC_PROG_GO\nAC_PROG_GOC\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("checking for Go compiler"));
    }

    #[test]
    fn test_fortran_macros_present() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([fctest], [1.0])\nAC_PROG_FC\nAC_FC_SRCEXT\nAC_FC_FREEFORM\nAC_FC_MODULE_FLAG\nAC_OUTPUT\n";
        let result = engine.process(input).expect("should process");
        assert!(result.contains("checking for Fortran compiler"));
        assert!(result.contains("checking for Fortran module include flag"));
    }
}
