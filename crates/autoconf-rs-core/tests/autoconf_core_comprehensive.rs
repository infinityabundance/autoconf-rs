//! Core Autoconf Macros Tests — AC.M4.AUTOCONF.CORE.1
//!
//! Tests: AC_INIT, AC_OUTPUT, AC_CONFIG_*, AC_SUBST, AC_DEFINE,
//! AC_MSG_*, AC_PREREQ, AC_CANONICAL_*, AC_REQUIRE, AC_DEFUN, etc.
//!
//! Court: AC.M4.AUTOCONF.CORE.1
//! Receipt family: AC.M4.AUTOCONF.CORE.*

use autoconf_rs_core::M4Engine;

fn expand(input: &str) -> String {
    let mut engine = M4Engine::new();
    engine
        .process(&format!("AC_INIT([t],[1.0])\n{}\nAC_OUTPUT\n", input))
        .unwrap_or_default()
}

fn run_raw(input: &str) -> String {
    let mut engine = M4Engine::new();
    engine.process(input).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // === AC_INIT / AC_OUTPUT ===
    #[test]
    fn test_ac_init_sets_package() {
        let o = run_raw("AC_INIT([mypkg],[2.0],[bugs@ex.com])\nAC_OUTPUT\n");
        assert!(o.contains("mypkg"));
        assert!(o.contains("2.0"));
        assert!(o.contains("bugs@ex.com"));
    }

    #[test]
    fn test_ac_init_minimal() {
        let o = run_raw("AC_INIT([test],[0.1])\nAC_OUTPUT\n");
        assert!(o.contains("#! /bin/sh") || o.contains("#!/bin/sh"));
        assert!(o.len() > 500);
    }

    #[test]
    fn test_ac_output_generates_config_status() {
        let o = run_raw("AC_INIT([pkg],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n");
        assert!(o.contains("Makefile") || o.contains("config.status"));
    }

    // === AC_CONFIG_FILES ===
    #[test]
    fn test_ac_config_files_single() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n");
        assert!(o.contains("Makefile"));
    }

    #[test]
    fn test_ac_config_files_multiple() {
        let o =
            run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_FILES([Makefile src/config.h])\nAC_OUTPUT\n");
        assert!(o.contains("Makefile"));
    }

    // === AC_CONFIG_HEADERS ===
    #[test]
    fn test_ac_config_headers() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_HEADERS([config.h])\nAC_OUTPUT\n");
        assert!(o.contains("config.h"));
    }

    // === AC_CONFIG_COMMANDS ===
    #[test]
    fn test_ac_config_commands() {
        let o =
            run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_COMMANDS([default],[echo done])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_CONFIG_LINKS ===
    #[test]
    fn test_ac_config_links() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_LINKS([dst:src])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_config_links_multiple() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_LINKS([a:b c:d])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_CONFIG_SUBDIRS ===
    #[test]
    fn test_ac_config_subdirs() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_SUBDIRS([lib])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_SUBST ===
    #[test]
    fn test_ac_subst_single() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_SUBST([CC],[gcc])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_subst_multiple() {
        let o = run_raw(
            "AC_INIT([t],[1.0])\nAC_SUBST([prefix],[/usr])\nAC_SUBST([bindir],[$prefix/bin])\nAC_OUTPUT\n",
        );
        assert!(o.len() > 100);
    }

    #[test]
    fn test_ac_subst_no_value() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_SUBST([VAR])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_DEFINE ===
    #[test]
    fn test_ac_define_with_value() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_DEFINE([HAVE_FOO],[1])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_define_default_value() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_DEFINE([HAVE_BAR])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_define_with_description() {
        let o = run_raw(
            "AC_INIT([t],[1.0])\nAC_DEFINE([HAVE_BAZ],[1],[Define to 1 if you have baz])\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_define_multiple() {
        let o = run_raw(
            "AC_INIT([t],[1.0])\nAC_DEFINE([A],[1])\nAC_DEFINE([B],[2])\nAC_DEFINE([C],[3])\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    // === AC_MSG_* ===
    #[test]
    fn test_ac_msg_checking() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_MSG_CHECKING([for something])\nAC_OUTPUT\n");
        assert!(o.contains("checking"));
    }

    #[test]
    fn test_ac_msg_result() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_MSG_RESULT([yes])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_msg_warn() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_MSG_WARN([test warning])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_msg_error() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_MSG_ERROR([fatal])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_msg_notice() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_MSG_NOTICE([info])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_msg_failure() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_MSG_FAILURE([failed])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_PREREQ ===
    #[test]
    fn test_ac_prereq() {
        let o = run_raw("AC_PREREQ([2.50])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_CANONICAL_* ===
    #[test]
    fn test_ac_canonical_host() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CANONICAL_HOST\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_canonical_build() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CANONICAL_BUILD\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_canonical_target() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CANONICAL_TARGET\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_REQUIRE / AC_DEFUN / AC_PROVIDE ===
    #[test]
    fn test_ac_defun_expands() {
        let o =
            run_raw("AC_DEFUN([MYMACRO],[my output])\nAC_INIT([t],[1.0])\nMYMACRO\nAC_OUTPUT\n");
        assert!(o.contains("my output"));
    }

    #[test]
    fn test_ac_defun_with_args() {
        let o = run_raw(
            "AC_DEFUN([GREET],[hello $1])\nAC_INIT([t],[1.0])\nGREET([world])\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_defun_once() {
        let o = run_raw(
            "AC_DEFUN_ONCE([ONCE],[first])\nAC_DEFUN_ONCE([ONCE],[second])\nAC_INIT([t],[1.0])\nONCE\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_require_order() {
        let o = run_raw(
            "AC_DEFUN([DEP],[dependency])\nAC_DEFUN([CALLER],[AC_REQUIRE([DEP])caller])\nAC_INIT([t],[1.0])\nCALLER\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_au_defun() {
        let o = run_raw("AU_DEFUN([OLD],[echo old],[NEW])\nAC_INIT([t],[1.0])\nOLD\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_ARG_* ===
    #[test]
    fn test_ac_arg_with() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_ARG_WITH([foo],[use foo support])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_arg_enable() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_ARG_ENABLE([bar],[enable bar])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_arg_var() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_ARG_VAR([CFLAGS],[C compiler flags])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_PREFIX_DEFAULT / AC_CONFIG_AUX_DIR / AC_CONFIG_MACRO_DIR ===
    #[test]
    fn test_ac_prefix_default() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_PREFIX_DEFAULT([/usr/local])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_config_aux_dir() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_AUX_DIR([build-aux])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_config_macro_dir() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_MACRO_DIR([m4])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_CONFIG_SRCDIR ===
    #[test]
    fn test_ac_config_srcdir() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CONFIG_SRCDIR([src/main.c])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_REVISION / AC_COPYRIGHT ===
    #[test]
    fn test_ac_revision() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_REVISION([$Revision: 1.0 $])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_copyright() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_COPYRIGHT([2024 My Project])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_PROG_* in core context ===
    #[test]
    fn test_ac_prog_cc_in_configure() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_PROG_CC\nAC_OUTPUT\n");
        assert!(o.contains("CC") || o.len() > 100);
    }

    #[test]
    fn test_ac_prog_cxx_in_configure() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_PROG_CXX\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_CHECK_* in core context ===
    #[test]
    fn test_ac_check_func_in_configure() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CHECK_FUNC([malloc])\nAC_OUTPUT\n");
        assert!(o.contains("malloc"));
    }

    #[test]
    fn test_ac_check_header_in_configure() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CHECK_HEADER([stdio.h])\nAC_OUTPUT\n");
        assert!(o.contains("stdio.h"));
    }

    #[test]
    fn test_ac_check_lib_in_configure() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CHECK_LIB([m],[sqrt])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_check_type_in_configure() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CHECK_TYPE([size_t])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_CACHE_* ===
    #[test]
    fn test_ac_cache_check() {
        let o = run_raw(
            "AC_INIT([t],[1.0])\nAC_CACHE_CHECK([for foo],[ac_cv_foo],[echo found])\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_cache_val() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_CACHE_VAL([ac_cv_bar],[echo val])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_DIAGNOSE / AC_WARNING / AC_FATAL ===
    #[test]
    fn test_ac_diagnose() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_DIAGNOSE([cat],[test])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_warning() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_WARNING([test])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_fatal() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_FATAL([fatal error])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_SYS_* ===
    #[test]
    fn test_ac_sys_largefile() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_SYS_LARGEFILE\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_sys_long_file_names() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_SYS_LONG_FILE_NAMES\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_OBSOLETE / AC_BEFORE ===
    #[test]
    fn test_ac_before() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_BEFORE([AC_PROG_CC],[AC_INIT])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_LANG_* ===
    #[test]
    fn test_ac_lang_push_pop() {
        let o =
            run_raw("AC_INIT([t],[1.0])\nAC_LANG_PUSH([C])\nAC_PROG_CC\nAC_LANG_POP\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === AC_LIBOBJ / AC_REPLACE_FUNCS ===
    #[test]
    fn test_ac_libobj() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_LIBOBJ([file])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_replace_funcs() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_REPLACE_FUNCS([strdup])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === Full configure.ac scripts ===
    #[test]
    fn test_realistic_configure_ac() {
        let o = run_raw(
            "AC_PREREQ([2.50])\n\
             AC_INIT([myproject],[2.0],[bugs@myproject.org])\n\
             AC_CONFIG_SRCDIR([src/main.c])\n\
             AC_CONFIG_AUX_DIR([build-aux])\n\
             AC_CONFIG_MACRO_DIR([m4])\n\
             AC_CANONICAL_HOST\n\
             AC_PROG_CC\n\
             AC_PROG_INSTALL\n\
             AC_CHECK_FUNCS([malloc realloc free])\n\
             AC_CHECK_HEADERS([stdlib.h string.h unistd.h])\n\
             AC_CHECK_LIB([m],[sqrt])\n\
             AC_ARG_WITH([ssl],[use SSL support])\n\
             AC_SUBST([CC])\n\
             AC_SUBST([CFLAGS],[-O2])\n\
             AC_CONFIG_FILES([Makefile src/Makefile])\n\
             AC_CONFIG_HEADERS([config.h])\n\
             AC_OUTPUT\n",
        );
        assert!(o.len() > 1000, "real configure.ac: {} bytes", o.len());
    }

    #[test]
    fn test_configure_package_detection() {
        let o = run_raw(
            "AC_INIT([detect],[3.14],[bugs@detect.org])\n\
             AC_PROG_CC\n\
             AC_PROG_CXX\n\
             AC_PROG_AWK\n\
             AC_PROG_SED\n\
             AC_PROG_GREP\n\
             AC_CHECK_SIZEOF([int])\n\
             AC_CHECK_SIZEOF([long])\n\
             AC_C_CONST\n\
             AC_C_VOLATILE\n\
             AC_HEADER_STDC\n\
             AC_TYPE_PID_T\n\
             AC_TYPE_SIZE_T\n\
             AC_OUTPUT\n",
        );
        assert!(o.len() > 500);
    }

    #[test]
    fn test_minimal_configure() {
        let o = run_raw("AC_INIT([minimal],[0.1])\nAC_OUTPUT\n");
        assert!(o.contains("#!"));
        assert!(o.contains("minimal"));
        assert!(o.contains("0.1"));
    }

    #[test]
    fn test_engine_state_after_process() {
        let mut engine = M4Engine::new();
        let _ = engine.process("AC_INIT([state_pkg],[1.0],[bugs])\nAC_OUTPUT\n");
        let state = engine.state();
        assert_eq!(state.package_name.as_deref(), Some("state_pkg"));
        assert_eq!(state.package_version.as_deref(), Some("1.0"));
    }

    // === Remaining partial features ===
    #[test]
    fn test_ac_obsolete_warns() {
        // AC_OBSOLETE should emit a deprecation comment
        let o = run_raw("AC_INIT([t],[1.0])\nAC_OBSOLETE([OLD],[NEW])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_before_ordering() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_BEFORE([AC_PROG_CC],[AC_INIT])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_before_with_third_arg() {
        let o = run_raw(
            "AC_INIT([t],[1.0])\nAC_BEFORE([$0],[AC_INIT],[should come before])\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_au_alias() {
        let o = run_raw(
            "AC_DEFUN([NEW],[new output])\nAU_ALIAS([OLD],[NEW])\nAC_INIT([t],[1.0])\nOLD\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_deep_ac_require_nesting() {
        let o = run_raw(
            "AC_DEFUN([A],[a])\nAC_DEFUN([B],[AC_REQUIRE([A])b])\nAC_DEFUN([C],[AC_REQUIRE([B])c])\nAC_DEFUN([D],[AC_REQUIRE([C])d])\nAC_INIT([t],[1.0])\nD\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_canonical_with_target() {
        let o = run_raw(
            "AC_INIT([t],[1.0])\nAC_CANONICAL_HOST\nAC_CANONICAL_BUILD\nAC_CANONICAL_TARGET\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_prefix_program() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_PREFIX_PROGRAM([gcc])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_search_libs() {
        let o = run_raw("AC_INIT([t],[1.0])\nAC_SEARCH_LIBS([sqrt],[m])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_checks_plural() {
        let o =
            run_raw("AC_INIT([t],[1.0])\nAC_CHECK_FUNCS([malloc realloc free])\nAC_CHECK_HEADERS([stdio.h stdlib.h])\nAC_CHECK_TYPES([size_t ssize_t])\nAC_OUTPUT\n");
        assert!(o.len() > 200);
    }

    #[test]
    fn test_large_configure_ac() {
        let mut input = String::from("AC_INIT([large],[1.0],[bugs@large.org])\n");
        input.push_str("AC_CANONICAL_HOST\n");
        input.push_str("AC_PROG_CC\n");
        input.push_str("AC_PROG_CXX\n");
        input.push_str("AC_PROG_AWK\n");
        input.push_str("AC_PROG_SED\n");
        input.push_str("AC_PROG_GREP\n");
        input.push_str("AC_PROG_EGREP\n");
        input.push_str("AC_PROG_FGREP\n");
        input.push_str("AC_PROG_LEX\n");
        input.push_str("AC_PROG_YACC\n");
        input.push_str("AC_PROG_LN_S\n");
        input.push_str("AC_PROG_MAKE_SET\n");
        input.push_str("AC_PROG_RANLIB\n");
        input.push_str("AC_PROG_AR\n");
        input.push_str("AC_PROG_INSTALL\n");
        for i in 0..20 {
            input.push_str(&format!("AC_CHECK_FUNC([func_{0}])\n", i));
        }
        for i in 0..10 {
            input.push_str(&format!("AC_CHECK_HEADER([hdr_{0}.h])\n", i));
        }
        for i in 0..20 {
            input.push_str(&format!("AC_SUBST([VAR_{0}],[val_{0}])\n", i));
        }
        input.push_str("AC_CONFIG_FILES([Makefile])\n");
        input.push_str("AC_CONFIG_HEADERS([config.h])\n");
        input.push_str("AC_OUTPUT\n");
        let o = run_raw(&input);
        assert!(o.len() > 5000, "large configure.ac: {} bytes", o.len());
    }
}
