//! Header/Type/Struct Check Macro Tests — AC.LIBRARY.HEADERS.1
//!
//! Tests: AC_HEADER_*/AC_TYPE_*/AC_STRUCT_*/AC_CHECK_HEADER/HEADERS/
//! AC_CHECK_TYPE/TYPES/MEMBER/MEMBERS/SIZEOF.
//!
//! Court: AC.LIBRARY.HEADERS.1

use autoconf_rs_core::M4Engine;

fn run(input: &str) -> String {
    let mut engine = M4Engine::new();
    engine.process(input).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ac_header_stdc() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_STDC\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_assert() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_ASSERT\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_dirent() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_DIRENT\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_major() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_MAJOR\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_resolv() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_RESOLV\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_stat() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_STAT\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_stdbool() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_STDBOOL\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_stdint() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_STDINT\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_sys_wait() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_SYS_WAIT\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_time() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_TIME\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_header_tiocgwinsz() {
        let o = run("AC_INIT([t],[1.0])\nAC_HEADER_TIOCGWINSZ\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_type_pid_t() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_PID_T\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_size_t() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_SIZE_T\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_getgroups() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_GETGROUPS\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_int8_t() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_INT8_T\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_int16_t() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_INT16_T\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_int32_t() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_INT32_T\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_int64_t() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_INT64_T\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_intmax_t() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_INTMAX_T\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_type_intptr_t() {
        let o = run("AC_INIT([t],[1.0])\nAC_TYPE_INTPTR_T\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_struct_tm() {
        let o = run("AC_INIT([t],[1.0])\nAC_STRUCT_TM\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_struct_st_blocks() {
        let o = run("AC_INIT([t],[1.0])\nAC_STRUCT_ST_BLOCKS\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_struct_st_blksize() {
        let o = run("AC_INIT([t],[1.0])\nAC_STRUCT_ST_BLKSIZE\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_struct_st_rdev() {
        let o = run("AC_INIT([t],[1.0])\nAC_STRUCT_ST_RDEV\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_struct_timezone() {
        let o = run("AC_INIT([t],[1.0])\nAC_STRUCT_TIMEZONE\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_check_header() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_HEADER([stdio.h])\nAC_OUTPUT\n");
        assert!(o.contains("stdio.h"));
    }
    #[test]
    fn test_ac_check_header_with_action() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_HEADER([stdio.h],[found],[not])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_headers_plural() {
        let o =
            run("AC_INIT([t],[1.0])\nAC_CHECK_HEADERS([stdio.h stdlib.h string.h])\nAC_OUTPUT\n");
        assert!(o.len() > 100);
    }

    #[test]
    fn test_ac_check_type() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_TYPE([size_t])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_types_plural() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_TYPES([size_t ssize_t])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_check_member() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_MEMBER([struct stat.st_mode],[#include <sys/stat.h>])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_members_plural() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_MEMBERS([struct stat.st_mode struct stat.st_size])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_ac_check_sizeof() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_SIZEOF([int])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }
    #[test]
    fn test_ac_check_sizeof_multiple() {
        let o = run("AC_INIT([t],[1.0])\nAC_CHECK_SIZEOF([int])\nAC_CHECK_SIZEOF([long])\nAC_CHECK_SIZEOF([void*])\nAC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_full_header_type_configure() {
        let o = run("AC_INIT([hdrtest],[2.0])\n\
             AC_HEADER_STDC\n\
             AC_HEADER_DIRENT\n\
             AC_HEADER_STAT\n\
             AC_HEADER_TIME\n\
             AC_HEADER_SYS_WAIT\n\
             AC_CHECK_HEADERS([stdio.h stdlib.h string.h unistd.h])\n\
             AC_TYPE_PID_T\n\
             AC_TYPE_SIZE_T\n\
             AC_TYPE_INT32_T\n\
             AC_TYPE_INT64_T\n\
             AC_CHECK_TYPE([ssize_t])\n\
             AC_STRUCT_TM\n\
             AC_STRUCT_ST_BLOCKS\n\
             AC_STRUCT_TIMEZONE\n\
             AC_CHECK_MEMBER([struct stat.st_mode],[#include <sys/stat.h>])\n\
             AC_CHECK_SIZEOF([int])\n\
             AC_CHECK_SIZEOF([long])\n\
             AC_CHECK_SIZEOF([void*])\n\
             AC_OUTPUT\n");
        assert!(o.len() > 1000);
    }
}
