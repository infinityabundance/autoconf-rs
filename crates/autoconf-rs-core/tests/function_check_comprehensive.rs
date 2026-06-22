//! Function Check Macro Tests — AC.LIBRARY.FUNCTIONS.1
//!
//! Tests: AC_CHECK_FUNC/FUNCS, AC_REPLACE_FUNCS, AC_LIBOBJ,
//! AC_FUNC_ALLOCA/MALLOC/STRERROR_R/CLOSEDIR_VOID/FNMATCH/FORK/
//! FSEEKO/GETGROUPS/GETLOADAVG/GETMNTENT/MKTIME/MMAP/STRCOLL/
//! STRFTIME/STRTOD/UTIME_NULL/VPRINTF/VFORK/WAIT3 plus many more.
//!
//! Court: AC.LIBRARY.FUNCTIONS.1

use autoconf_rs_core::M4Engine;

fn run(input: &str) -> String {
    let mut engine = M4Engine::new();
    engine.process(input).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ac_check_func() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_FUNC([malloc])\nAC_OUTPUT\n");
        assert!(o.contains("malloc"));
    }
    #[test]
    fn test_ac_check_func_with_action() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_FUNC([malloc],[found],[not])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_funcs_plural() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_FUNCS([malloc realloc free])\nAC_OUTPUT\n");
        assert!(o.len() > 100);
    }
    #[test]
    fn test_ac_replace_funcs() {
        let o = run("AC_INIT([t],[1.0])\nAC_REPLACE_FUNCS([strdup])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_libobj() {
        let o = run("AC_INIT([t],[1.0])\nAC_LIBOBJ([file])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_func_alloca() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_ALLOCA\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_malloc() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_MALLOC\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strerror_r() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_STRERROR_R\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_closedir_void() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_CLOSEDIR_VOID\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_fnmatch() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_FNMATCH\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_fnmatch_gnu() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_FNMATCH_GNU\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_fork() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_FORK\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_fseeko() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_FSEEKO\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_fstatfs() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_FSTATFS\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_ftruncate() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_FTRUNCATE\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_getgroups() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_GETGROUPS\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_gethostbyname_r() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_GETHOSTBYNAME_R\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_getloadavg() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_GETLOADAVG\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_getmntent() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_GETMNTENT\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_getpgrp() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_GETPGRP\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_lstat() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_LSTAT_FOLLOWS_SLASHED_SYMLINK\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_malloc_0_nonnull() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_MALLOC_0_NONNULL\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_mbrtowc() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_MBRTOWC\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_memcmp() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_MEMCMP\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_mktime() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_MKTIME\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_mmap() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_MMAP\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_obstack() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_OBSTACK\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_printf_posix() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_PRINTF_POSIX\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_realloc() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_REALLOC\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_realloc_0_nonnull() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_REALLOC_0_NONNULL\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_select() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_SELECT_ARGTYPES\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_setpgrp() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_SETPGRP\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_stat() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_STAT\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strcoll() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_STRCOLL\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strerror() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_STRERROR\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strftime() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_STRFTIME\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strnlen() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_STRNLEN\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strtod() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_STRTOD\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_strtold() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_STRTOLD\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_utime_null() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_UTIME_NULL\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_vfork() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_VFORK\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_vprintf() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_VPRINTF\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_func_wait3() {
        let o = run("AC_INIT([t],[1.0])\nAC_FUNC_WAIT3\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_multiple_func_checks() {
        let o = run(
            "AC_INIT([t],[1.0])\nAC_CHECK_FUNCS([malloc realloc free strdup strndup])\nAC_OUTPUT\n",
        );
        assert!(o.len() > 100);
    }

    #[test]
    fn test_full_function_configure() {
        let o = run("AC_INIT([functest],[2.0])\n\
             AC_CHECK_FUNC([malloc],[AC_DEFINE([HAVE_MALLOC],[1])])\n\
             AC_CHECK_FUNCS([realloc free strdup])\n\
             AC_FUNC_ALLOCA\n\
             AC_FUNC_MALLOC\n\
             AC_FUNC_STRERROR_R\n\
             AC_FUNC_FNMATCH\n\
             AC_FUNC_FORK\n\
             AC_FUNC_GETGROUPS\n\
             AC_FUNC_MKTIME\n\
             AC_FUNC_STRFTIME\n\
             AC_FUNC_VPRINTF\n\
             AC_FUNC_UTIME_NULL\n\
             AC_FUNC_WAIT3\n\
             AC_REPLACE_FUNCS([strdup])\n\
             AC_LIBOBJ([malloc])\n\
             AC_OUTPUT\n");
        assert!(o.len() > 1000);
    }
}
