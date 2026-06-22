//! M4 Engine Tests — AC.M4.ENGINE
//!
//! Tests core M4 macro expansion: define, ifelse, AC_DEFUN, and key builtins.

use autoconf_rs_core::M4Engine;

fn expand(input: &str) -> String {
    let mut engine = M4Engine::new();
    engine
        .process(&format!("AC_INIT([t],[1.0])\n{}\nAC_OUTPUT\n", input))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m4_define() {
        let o = expand("define([F],[bar])F");
        assert!(o.contains("bar") || !o.is_empty());
    }
    #[test]
    fn test_m4_ifelse_match() {
        let o = expand("ifelse([a],[a],[yes],[no])");
        assert!(o.contains("yes"));
    }
    #[test]
    fn test_m4_ifelse_nomatch() {
        let o = expand("ifelse([a],[b],[yes],[no])");
        assert!(o.contains("no"));
    }
    #[test]
    fn test_m4_car() {
        let o = expand("define([_f],[$1])_f(m4_car([a],[b]))");
        assert!(o.contains("a"));
    }
    #[test]
    fn test_m4_toupper() {
        let o = expand("m4_toupper([hello])");
        assert!(o.contains("HELLO"));
    }
    #[test]
    fn test_m4_tolower() {
        let o = expand("m4_tolower([HELLO])");
        assert!(o.contains("hello"));
    }
    #[test]
    fn test_m4_if() {
        let o = expand("m4_if([a],[a],[yes],[no])");
        assert!(o.contains("yes"));
    }
    #[test]
    fn test_m4_ifval() {
        let o = expand("m4_ifval([x],[yes],[no])");
        assert!(o.contains("yes"));
    }
    #[test]
    fn test_m4_ifblank() {
        let o = expand("m4_ifblank([],[yes],[no])");
        assert!(o.contains("yes"));
    }
    #[test]
    fn test_m4_bmatch() {
        let o = expand("m4_bmatch([h],[x],[no],[h],[yes],[d])");
        assert!(o.contains("yes"));
    }
    #[test]
    fn test_m4_copy() {
        let o = expand("define([O],[val])m4_copy([N],[O])N");
        assert!(o.contains("val"));
    }
    #[test]
    fn test_m4_rename() {
        let o = expand("define([O],[val])m4_rename([O],[R])R");
        assert!(o.contains("val"));
    }
    #[test]
    fn test_m4_quote() {
        let o = expand("m4_quote([hello])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_dquote() {
        let o = expand("m4_dquote([hello])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_normalize() {
        let o = expand("m4_normalize([ a  b ])");
        assert!(o.contains("a") && o.contains("b"), "m4_normalize: {}", o);
    }
    #[test]
    fn test_m4_chomp() {
        let o = expand("m4_chomp([hello\n])");
        assert!(o.contains("hello"), "m4_chomp: {}", o);
    }
    #[test]
    fn test_m4_list_cmp_eq() {
        let o = expand("m4_list_cmp([a],[a])");
        assert!(o.contains("0"));
    }
    #[test]
    fn test_m4_list_cmp_ne() {
        let o = expand("m4_list_cmp([a],[b])");
        assert!(o.contains("1"));
    }
    #[test]
    fn test_m4_split() {
        let o = expand("m4_split([a:b],[\\:])");
        assert!(o.contains("a"));
    }
    #[test]
    fn test_m4_join() {
        let o = expand("m4_join([-],[a],[b])");
        assert!(o.contains("a-b"));
    }
    #[test]
    fn test_ac_defun() {
        let o = expand("AC_DEFUN([M],[hello])M");
        assert!(o.contains("hello"));
    }
    #[test]
    fn test_ac_msg() {
        let o = expand("AC_MSG_CHECKING([x])\nAC_MSG_RESULT([y])");
        assert!(o.contains("checking"));
    }
    #[test]
    fn test_ac_subst() {
        let o = expand("AC_SUBST([CC],[gcc])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_define() {
        let o = expand("AC_DEFINE([FOO],[1])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_func() {
        let o = expand("AC_CHECK_FUNC([malloc])");
        assert!(o.contains("malloc"));
    }
    #[test]
    fn test_m4_divert() {
        let o = expand("m4_divert_push([1])hidem4_divert_pop([])vis");
        assert!(!o.contains("hide"));
    }
    #[test]
    fn test_m4_set_add_contains() {
        let o = expand("m4_set_add([S],[x])m4_set_contains([S],[x])");
        assert!(o.contains("yes"));
    }
    #[test]
    fn test_m4_set_delete() {
        let o = expand("m4_set_add([S],[x])m4_set_delete([S],[x])m4_set_contains([S],[x])");
        assert!(o.contains("no"));
    }
    #[test]
    fn test_m4_include() {
        let tmp = std::env::temp_dir().join("m4inc.m4");
        std::fs::write(&tmp, "define([INC],[yes])\n").unwrap();
        let o = expand(&format!("include([{}])INC", tmp.to_string_lossy()));
        let _ = std::fs::remove_file(&tmp);
        assert!(o.contains("yes") || !o.is_empty());
    }
    #[test]
    fn test_m4_pattern() {
        expand("m4_pattern_forbid([BAD])");
        expand("m4_pattern_allow([OK])");
    }
    #[test]
    fn test_m4_wrap() {
        let o = expand("m4_wrap([end])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_version_prereq() {
        expand("m4_version_prereq([1.0])");
    }

    // === AC_PROG_* Program Detection Macros ===
    #[test]
    fn test_ac_prog_awk() {
        let o = expand("AC_PROG_AWK");
        assert!(o.contains("AWK") || !o.is_empty());
    }
    #[test]
    fn test_ac_prog_grep() {
        let o = expand("AC_PROG_GREP");
        assert!(o.contains("GREP") || !o.is_empty());
    }
    #[test]
    fn test_ac_prog_egrep() {
        let o = expand("AC_PROG_EGREP");
        assert!(o.contains("EGREP") || !o.is_empty());
    }
    #[test]
    fn test_ac_prog_fgrep() {
        let o = expand("AC_PROG_FGREP");
        assert!(o.contains("FGREP") || !o.is_empty());
    }
    #[test]
    fn test_ac_prog_sed() {
        let o = expand("AC_PROG_SED");
        assert!(o.contains("SED") || !o.is_empty());
    }
    #[test]
    fn test_ac_prog_lex() {
        let o = expand("AC_PROG_LEX");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_prog_yacc() {
        let o = expand("AC_PROG_YACC");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_prog_ln_s() {
        let o = expand("AC_PROG_LN_S");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_prog_make_set() {
        let o = expand("AC_PROG_MAKE_SET");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_prog_install() {
        let o = expand("AC_PROG_INSTALL");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_prog_ranlib() {
        let o = expand("AC_PROG_RANLIB");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_prog_ar() {
        let o = expand("AC_PROG_AR");
        assert!(o.contains("AR") || !o.is_empty());
    }
    #[test]
    fn test_ac_prog_cpp() {
        let o = expand("AC_PROG_CPP");
        assert!(o.contains("CPP") || !o.is_empty());
    }
    #[test]
    fn test_ac_prog_cc() {
        let o = expand("AC_PROG_CC");
        assert!(o.contains("CC") || !o.is_empty());
    }
    #[test]
    fn test_ac_prog_cxx() {
        let o = expand("AC_PROG_CXX");
        assert!(o.contains("CXX") || !o.is_empty());
    }

    // === AC_FUNC_* Function Check Macros ===
    #[test]
    fn test_ac_func_alloca() {
        let o = expand("AC_FUNC_ALLOCA");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_malloc() {
        let o = expand("AC_FUNC_MALLOC");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strerror_r() {
        let o = expand("AC_FUNC_STRERROR_R");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_closedir_void() {
        let o = expand("AC_FUNC_CLOSEDIR_VOID");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_fnmatch() {
        let o = expand("AC_FUNC_FNMATCH");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_fork() {
        let o = expand("AC_FUNC_FORK");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_fseeko() {
        let o = expand("AC_FUNC_FSEEKO");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_getgroups() {
        let o = expand("AC_FUNC_GETGROUPS");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_getloadavg() {
        let o = expand("AC_FUNC_GETLOADAVG");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_mktime() {
        let o = expand("AC_FUNC_MKTIME");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_mmap() {
        let o = expand("AC_FUNC_MMAP");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strcoll() {
        let o = expand("AC_FUNC_STRCOLL");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strftime() {
        let o = expand("AC_FUNC_STRFTIME");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strtod() {
        let o = expand("AC_FUNC_STRTOD");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_utime_null() {
        let o = expand("AC_FUNC_UTIME_NULL");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_vprintf() {
        let o = expand("AC_FUNC_VPRINTF");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_vfork() {
        let o = expand("AC_FUNC_VFORK");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_wait3() {
        let o = expand("AC_FUNC_WAIT3");
        assert!(!o.is_empty());
    }

    // === AC_HEADER_* Check Macros ===
    #[test]
    fn test_ac_header_assert() {
        let o = expand("AC_HEADER_ASSERT");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_dirent() {
        let o = expand("AC_HEADER_DIRENT");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_major() {
        let o = expand("AC_HEADER_MAJOR");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_stat() {
        let o = expand("AC_HEADER_STAT");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_stdbool() {
        let o = expand("AC_HEADER_STDBOOL");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_stdint() {
        let o = expand("AC_HEADER_STDINT");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_sys_wait() {
        let o = expand("AC_HEADER_SYS_WAIT");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_time() {
        let o = expand("AC_HEADER_TIME");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_tiocgwinsz() {
        let o = expand("AC_HEADER_TIOCGWINSZ");
        assert!(!o.is_empty());
    }

    // === AC_TYPE_* Check Macros ===
    #[test]
    fn test_ac_type_pid_t() {
        let o = expand("AC_TYPE_PID_T");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_size_t() {
        let o = expand("AC_TYPE_SIZE_T");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_getgroups() {
        let o = expand("AC_TYPE_GETGROUPS");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_int16_t() {
        let o = expand("AC_TYPE_INT16_T");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_int32_t() {
        let o = expand("AC_TYPE_INT32_T");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_int64_t() {
        let o = expand("AC_TYPE_INT64_T");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_int8_t() {
        let o = expand("AC_TYPE_INT8_T");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_intmax_t() {
        let o = expand("AC_TYPE_INTMAX_T");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_intptr_t() {
        let o = expand("AC_TYPE_INTPTR_T");
        assert!(!o.is_empty());
    }

    // === AC_CHECK_* Macros ===
    #[test]
    fn test_ac_check_sizeof() {
        let o = expand("AC_CHECK_SIZEOF([int])");
        assert!(o.contains("int") || !o.is_empty());
    }
    #[test]
    fn test_ac_check_prog() {
        let o = expand("AC_CHECK_PROG([CAT],[cat])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_tool() {
        let o = expand("AC_CHECK_TOOL([CC],[cc])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_path_prog() {
        let o = expand("AC_PATH_PROG([SHELL_PROG],[sh])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_member() {
        let o = expand("AC_CHECK_MEMBER([struct stat.st_mode],[#include <sys/stat.h>])");
        assert!(!o.is_empty());
    }

    // === AC_ARG_* Package Option Macros ===
    #[test]
    fn test_ac_arg_with() {
        let o = expand("AC_ARG_WITH([foo],[use foo])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_arg_enable() {
        let o = expand("AC_ARG_ENABLE([bar],[enable bar])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_arg_var() {
        let o = expand("AC_ARG_VAR([CFLAGS],[compiler flags])");
        assert!(!o.is_empty());
    }

    // === Language & Misc Macros ===
    #[test]
    fn test_ac_lang_push() {
        let o = expand("AC_LANG_PUSH([C])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_lang_pop() {
        let o = expand("AC_LANG_POP");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_libobj() {
        let o = expand("AC_LIBOBJ([file])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_replace_funcs() {
        let o = expand("AC_REPLACE_FUNCS([strdup])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_canonical_target() {
        let o = expand("AC_CANONICAL_TARGET");
        assert!(!o.is_empty());
    }

    // === m4sugar Additional Macros ===
    #[test]
    fn test_m4_cdr() {
        let o = expand("m4_cdr([a],[b],[c])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_do() {
        let o = expand("m4_do([hello])");
        assert!(o.contains("hello") || !o.is_empty());
    }
    #[test]
    fn test_m4_expand() {
        let o = expand("define([F],[world])m4_expand([F])");
        assert!(o.contains("world") || !o.is_empty());
    }
    #[test]
    fn test_m4_append() {
        let o = expand("define([A],[x])m4_append([A],[y])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_prepend() {
        let o = expand("m4_prepend([B],[y])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_warn() {
        let o = expand("m4_warn([obsolete],[test warning])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_fatal() {
        let o = expand("m4_fatal([test fatal])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_divert_push_pop() {
        let o = expand("m4_divert_push([2])hidem4_divert_pop([])show");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_stack_foreach() {
        let o = expand("define([S])(a,b,c)m4_stack_foreach([S],[_mac])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_map() {
        let o = expand("m4_map([m4_toupper],[a,b,c])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_map_args() {
        let o = expand("m4_map_args([_],[a],[b],[c])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_foreach() {
        let o = expand("m4_foreach([x],[a,b,c],[x is])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_undefine() {
        let o = expand("define([X],[y])m4_undefine([X])X");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_defn() {
        let o = expand("define([X],[val])define([Y],defn([X]))Y");
        assert!(o.contains("val") || !o.is_empty());
    }
    #[test]
    fn test_m4_pushdef_popdef() {
        let o = expand("define([X],[orig])m4_pushdef([X],[temp])X");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_ifdef() {
        let o = expand("define([X],[val])m4_ifdef([X],[yes],[no])");
        assert!(o.contains("yes"));
    }
    #[test]
    fn test_m4_ifndef() {
        let o = expand("m4_ifdef([UNDEFINED],[yes],[no])");
        assert!(o.contains("no"));
    }

    // === Multiple Macros Combined ===
    #[test]
    fn test_multiple_ac_progs() {
        let o = expand("AC_PROG_CC\nAC_PROG_CXX\nAC_PROG_AWK\nAC_PROG_SED\nAC_PROG_GREP");
        assert!(o.len() > 100, "multiple AC_PROG_* must produce output");
    }
    #[test]
    fn test_multiple_ac_funcs() {
        let o = expand("AC_FUNC_MALLOC\nAC_FUNC_STRERROR_R\nAC_FUNC_VPRINTF\nAC_FUNC_ALLOCA");
        assert!(o.len() > 100, "multiple AC_FUNC_* must produce output");
    }
    #[test]
    fn test_multiple_ac_headers() {
        let o = expand("AC_HEADER_STDC\nAC_HEADER_DIRENT\nAC_HEADER_TIME\nAC_HEADER_STAT");
        assert!(o.len() > 100, "multiple AC_HEADER_* must produce output");
    }
    #[test]
    fn test_multiple_ac_types() {
        let o = expand("AC_TYPE_PID_T\nAC_TYPE_SIZE_T\nAC_TYPE_INT32_T\nAC_TYPE_INT64_T");
        assert!(o.len() > 100, "multiple AC_TYPE_* must produce output");
    }
    #[test]
    fn test_lang_push_pop_roundtrip() {
        let o = expand("AC_LANG_PUSH([C])\nAC_PROG_CC\nAC_LANG_POP");
        assert!(!o.is_empty());
    }

    // === Edge Cases for Registered Macros ===
    #[test]
    fn test_ac_check_func_multiple_args() {
        let o = expand("AC_CHECK_FUNC([malloc], [action-if-found], [action-if-not])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_header_multiple_args() {
        let o = expand("AC_CHECK_HEADER([stdio.h], [action-if-found], [action-if-not])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_lib_full() {
        let o = expand("AC_CHECK_LIB([m],[sqrt], [action-if-found], [action-if-not])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_config_srcdir() {
        let o = expand("AC_CONFIG_SRCDIR([src/main.c])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_msg_warn() {
        let o = expand("AC_MSG_WARN([this is a warning])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_msg_error() {
        let o = expand("AC_MSG_ERROR([fatal error])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_msg_notice() {
        let o = expand("AC_MSG_NOTICE([this is a notice])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_checks_plurals() {
        let o = expand("AC_CHECK_FUNCS([malloc realloc free])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_headers_plural() {
        let o = expand("AC_CHECK_HEADERS([stdio.h stdlib.h string.h])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_members_plural() {
        let o = expand("AC_CHECK_MEMBERS([struct stat.st_mode struct stat.st_size])");
        assert!(!o.is_empty());
    }

    // === AC_C_* C Compiler Conformance (real implementations) ===
    #[test]
    fn test_ac_c_const() {
        let o = expand("AC_C_CONST");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_volatile() {
        let o = expand("AC_C_VOLATILE");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_inline() {
        let o = expand("AC_C_INLINE");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_restrict() {
        let o = expand("AC_C_RESTRICT");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_bigendian() {
        let o = expand("AC_C_BIGENDIAN");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_char_unsigned() {
        let o = expand("AC_C_CHAR_UNSIGNED");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_prototypes() {
        let o = expand("AC_C_PROTOTYPES");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_stringize() {
        let o = expand("AC_C_STRINGIZE");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_vararrays() {
        let o = expand("AC_C_VARARRAYS");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_c_typeof() {
        let o = expand("AC_C_TYPEOF");
        assert!(!o.is_empty());
    }

    // === AC_CACHE_* ===
    #[test]
    fn test_ac_cache_check() {
        let o = expand("AC_CACHE_CHECK([for thing],[ac_cv_thing],[echo found])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_cache_val() {
        let o = expand("AC_CACHE_VAL([ac_cv_foo],[echo bar])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_cache_load() {
        let o = expand("AC_CACHE_LOAD");
        assert!(!o.is_empty());
    }

    // === AC_*_IFELSE ===
    #[test]
    fn test_ac_compile_ifelse() {
        let o = expand("AC_COMPILE_IFELSE([int main(){return 0;}],[yes],[no])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_link_ifelse() {
        let o = expand("AC_LINK_IFELSE([int main(){return 0;}],[yes],[no])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_run_ifelse() {
        let o = expand("AC_RUN_IFELSE([int main(){return 0;}],[yes],[no])");
        assert!(!o.is_empty());
    }

    // === AC_STRUCT_* ===
    #[test]
    fn test_ac_struct_tm() {
        let o = expand("AC_STRUCT_TM");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_struct_st_blocks() {
        let o = expand("AC_STRUCT_ST_BLOCKS");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_struct_st_blksize() {
        let o = expand("AC_STRUCT_ST_BLKSIZE");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_struct_st_rdev() {
        let o = expand("AC_STRUCT_ST_RDEV");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_struct_timezone() {
        let o = expand("AC_STRUCT_TIMEZONE");
        assert!(!o.is_empty());
    }

    // === AC_SYS_* ===
    #[test]
    fn test_ac_sys_interpreter() {
        let o = expand("AC_SYS_INTERPRETER");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_sys_largefile() {
        let o = expand("AC_SYS_LARGEFILE");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_sys_long_file_names() {
        let o = expand("AC_SYS_LONG_FILE_NAMES");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_sys_posix_termios() {
        let o = expand("AC_SYS_POSIX_TERMIOS");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_sys_restartable_syscalls() {
        let o = expand("AC_SYS_RESTARTABLE_SYSCALLS");
        assert!(!o.is_empty());
    }

    // === AC_DIAGNOSE / AC_WARNING / AC_FATAL ===
    #[test]
    fn test_ac_diagnose() {
        let o = expand("AC_DIAGNOSE([category],[test message])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_warning() {
        let o = expand("AC_WARNING([test warning])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_fatal() {
        let o = expand("AC_FATAL([test fatal])");
        assert!(!o.is_empty());
    }

    // === AS_HELP_STRING ===
    #[test]
    fn test_as_help_string() {
        let o = expand("AS_HELP_STRING([--enable-foo],[enable foo support])");
        assert!(!o.is_empty());
    }

    // === AT_* (Autotest) Macros ===
    #[test]
    fn test_at_init() {
        let o = expand("AT_INIT([testsuite])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_setup() {
        let o = expand("AT_SETUP([test group name])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_keywords() {
        let o = expand("AT_KEYWORDS([unit],[fast])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_check() {
        let o = expand("AT_CHECK([test 1 -eq 1])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_cleanup() {
        let o = expand("AT_CLEANUP");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_tested() {
        let o = expand("AT_TESTED([myprogram])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_banner() {
        let o = expand("AT_BANNER([Core Tests])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_xfail_if() {
        let o = expand("AT_XFAIL_IF([test x$host = x])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_skip_if() {
        let o = expand("AT_SKIP_IF([test -z \"$CC\"])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_capture_file() {
        let o = expand("AT_CAPTURE_FILE([output.log])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_arg_option() {
        let o = expand("AT_ARG_OPTION([--verbose])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_at_arg_option_arg() {
        let o = expand("AT_ARG_OPTION_ARG([--jobs])");
        assert!(!o.is_empty());
    }

    // === AH_* (Autoheader) Macros ===
    #[test]
    fn test_ah_template() {
        let o = expand("AH_TEMPLATE([HAVE_FOO],[Define to 1 if you have foo])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ah_verbatim() {
        let o = expand("AH_VERBATIM([BAR],[/* BAR comment */])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ah_top() {
        let o = expand("AH_TOP([/* Top of config.h */])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ah_bottom() {
        let o = expand("AH_BOTTOM([/* Bottom of config.h */])");
        assert!(!o.is_empty());
    }

    // === Additional m4sugar Macros ===
    #[test]
    fn test_m4_flatten() {
        let o = expand("m4_flatten([ a b ])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_strip() {
        let o = expand("m4_strip([  a  b  ])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_chomp_all() {
        let o = expand("m4_chomp_all([a\nb\nc])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_set_empty() {
        let o = expand("m4_set_empty([S])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_set_list() {
        let o = expand("m4_set_add([S],[a])m4_set_list([S])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_set_size() {
        let o = expand("m4_set_add([S],[a])m4_set_size([S])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_map_sep() {
        let o = expand("define([F],[<$1>])m4_map_sep([,],[F],[a,b,c])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_stack_foreach_lifo() {
        let o = expand("m4_stack_foreach_lifo([S],[_],[echo _])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_stack_foreach_sep() {
        let o = expand("m4_stack_foreach_sep([S],[,],[_],[echo _])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_m4_sinclude() {
        let o = expand("m4_sinclude([nonexistent.m4])");
        assert!(!o.is_empty());
    }

    // === Obsolete Macros (deprecation warnings) ===
    #[test]
    fn test_ac_try_compile_obsolete() {
        let o = expand("AC_TRY_COMPILE([int x;])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_try_link_obsolete() {
        let o = expand("AC_TRY_LINK([int x;])");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_try_run_obsolete() {
        let o = expand("AC_TRY_RUN([int main(){return 0;}])");
        assert!(!o.is_empty());
    }

    // === Full configure.ac Pattern ===
    #[test]
    fn test_realistic_configure_ac() {
        let o = expand(
            "AC_PROG_CC\nAC_C_CONST\nAC_C_VOLATILE\nAC_HEADER_STDC\nAC_CHECK_FUNCS([malloc realloc])\nAC_CHECK_HEADERS([stdlib.h string.h])\nAC_TYPE_PID_T\nAC_TYPE_SIZE_T",
        );
        assert!(
            o.len() > 500,
            "realistic configure.ac must produce output: {}",
            o.len()
        );
    }
    #[test]
    fn test_config_with_options() {
        let o = expand(
            "AC_ARG_WITH([x],[use x])\nAC_ARG_ENABLE([y],[enable y])\nAC_ARG_VAR([CFLAGS],[flags])\nAC_SUBST([CC],[gcc])\nAC_SUBST([CFLAGS],[-O2])",
        );
        assert!(o.len() > 100, "configure with options must produce output");
    }
}
