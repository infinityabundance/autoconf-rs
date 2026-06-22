//! M4sh Macro Library Tests — AC.M4.M4SH.1
//!
//! Tests the m4sh shell-generation macro library: portable echo,
//! conditionals, loops, quoting, escaping, path operations, variables.
//!
//! Court: AC.M4.M4SH.1
//! Receipt family: AC.M4.M4SH.*

use autoconf_rs_core::m4sh::M4ShBuiltins;
use autoconf_rs_core::M4Engine;

/// Run input through the full M4 engine pipeline.
fn expand(input: &str) -> String {
    let mut engine = M4Engine::new();
    engine
        .process(&format!("AC_INIT([t],[1.0])\n{}\nAC_OUTPUT\n", input))
        .unwrap_or_default()
}

// ============================================================================
// M4ShBuiltins Rust-side Tests
// ============================================================================
#[cfg(test)]
mod builtins {
    use super::*;

    // --- AS_ECHO / AS_ECHO_N ---
    #[test]
    fn test_as_echo_single() {
        let args = vec![b"hello".to_vec()];
        let r = M4ShBuiltins::as_echo(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("printf"));
        assert!(s.contains("hello"));
    }

    #[test]
    fn test_as_echo_multiple() {
        let args = vec![b"hello".to_vec(), b"world".to_vec()];
        let r = M4ShBuiltins::as_echo(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("hello"));
        assert!(s.contains("world"));
    }

    #[test]
    fn test_as_echo_empty() {
        let r = M4ShBuiltins::as_echo(&[]);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_as_echo_n() {
        let args = vec![b"hello".to_vec()];
        let r = M4ShBuiltins::as_echo_n(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("printf"));
        assert!(!s.contains("\\n"));
    }

    #[test]
    fn test_as_echo_n_empty() {
        let r = M4ShBuiltins::as_echo_n(&[]);
        assert!(!r.is_empty());
    }

    // --- AS_ESCAPE ---
    #[test]
    fn test_as_escape_dollar() {
        let args = vec![b"hello $world".to_vec()];
        let r = M4ShBuiltins::as_escape(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("\\$"));
    }

    #[test]
    fn test_as_escape_backtick() {
        let args = vec![b"hello `cmd`".to_vec()];
        let r = M4ShBuiltins::as_escape(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("\\`"));
    }

    #[test]
    fn test_as_escape_doublequote() {
        let args = vec![b"hello \"quoted\"".to_vec()];
        let r = M4ShBuiltins::as_escape(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("\\\""));
    }

    #[test]
    fn test_as_escape_empty() {
        let r = M4ShBuiltins::as_escape(&[]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_as_escape_plain() {
        let args = vec![b"plain text".to_vec()];
        let r = M4ShBuiltins::as_escape(&args);
        assert_eq!(String::from_utf8_lossy(&r), "plain text");
    }

    // --- AS_EXIT ---
    #[test]
    fn test_as_exit_with_status() {
        let args = vec![b"0".to_vec()];
        let r = M4ShBuiltins::as_exit(&args);
        assert_eq!(String::from_utf8_lossy(&r), "exit 0\n");
    }

    #[test]
    fn test_as_exit_default() {
        let r = M4ShBuiltins::as_exit(&[]);
        assert_eq!(String::from_utf8_lossy(&r), "exit $?\n");
    }

    #[test]
    fn test_as_exit_empty_arg() {
        let args = vec![b"".to_vec()];
        let r = M4ShBuiltins::as_exit(&args);
        assert_eq!(String::from_utf8_lossy(&r), "exit $?\n");
    }

    // --- AS_IF ---
    #[test]
    fn test_as_if_simple() {
        let args = vec![b"test -f foo".to_vec(), b"echo yes".to_vec()];
        let r = M4ShBuiltins::as_if(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("if test -f foo; then"));
        assert!(s.contains("echo yes"));
        assert!(s.contains("fi"));
    }

    #[test]
    fn test_as_if_with_else() {
        let args = vec![
            b"cond".to_vec(),
            b"then_body".to_vec(),
            b"else_body".to_vec(),
        ];
        let r = M4ShBuiltins::as_if(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("else"));
        assert!(s.contains("else_body"));
    }

    #[test]
    fn test_as_if_with_elif() {
        let args = vec![
            b"c1".to_vec(),
            b"t1".to_vec(),
            b"c2".to_vec(),
            b"t2".to_vec(),
        ];
        let r = M4ShBuiltins::as_if(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("elif c2; then"));
        assert!(s.contains("t2"));
    }

    #[test]
    fn test_as_if_empty() {
        let r = M4ShBuiltins::as_if(&[]);
        assert!(r.is_empty());
    }

    // --- AS_CASE ---
    #[test]
    fn test_as_case_match() {
        let args = vec![
            b"$x".to_vec(),
            b"a".to_vec(),
            b"echo a".to_vec(),
            b"b".to_vec(),
            b"echo b".to_vec(),
        ];
        let r = M4ShBuiltins::as_case(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("case $x in"));
        assert!(s.contains("a )"));
        assert!(s.contains("b )"));
        assert!(s.contains("esac"));
    }

    #[test]
    fn test_as_case_with_default() {
        let args = vec![
            b"$x".to_vec(),
            b"y".to_vec(),
            b"echo y".to_vec(),
            b"echo default".to_vec(),
        ];
        let r = M4ShBuiltins::as_case(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("* )"));
        assert!(s.contains("echo default"));
    }

    #[test]
    fn test_as_case_empty() {
        let r = M4ShBuiltins::as_case(&[]);
        assert!(r.is_empty());
    }

    // --- AS_FOR ---
    #[test]
    fn test_as_for_basic() {
        let args = vec![b"i".to_vec(), b"1 2 3".to_vec(), b"echo $i".to_vec()];
        let r = M4ShBuiltins::as_for(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("for i in 1 2 3; do"));
        assert!(s.contains("echo $i"));
        assert!(s.contains("done"));
    }

    #[test]
    fn test_as_for_few_args() {
        let r = M4ShBuiltins::as_for(&[b"i".to_vec()]);
        assert!(r.is_empty());
    }

    // --- AS_MKDIR_P ---
    #[test]
    fn test_as_mkdir_p_single() {
        let args = vec![b"/tmp/dir".to_vec()];
        let r = M4ShBuiltins::as_mkdir_p(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("test -d"));
        assert!(s.contains("mkdir -p"));
    }

    #[test]
    fn test_as_mkdir_p_multiple() {
        let args = vec![b"/a".to_vec(), b"/b".to_vec()];
        let r = M4ShBuiltins::as_mkdir_p(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("/a"));
        assert!(s.contains("/b"));
    }

    #[test]
    fn test_as_mkdir_p_empty() {
        let r = M4ShBuiltins::as_mkdir_p(&[]);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("mkdir -p"));
    }

    // --- AS_TR_SH / AS_TR_CPP ---
    #[test]
    fn test_as_tr_sh_dots() {
        let args = vec![b"foo.bar".to_vec()];
        let r = M4ShBuiltins::as_tr_sh(&args);
        assert_eq!(String::from_utf8_lossy(&r), "foo_bar");
    }

    #[test]
    fn test_as_tr_sh_dashes() {
        let args = vec![b"foo-bar".to_vec()];
        let r = M4ShBuiltins::as_tr_sh(&args);
        assert_eq!(String::from_utf8_lossy(&r), "foo_bar");
    }

    #[test]
    fn test_as_tr_sh_preserve_underscore() {
        let args = vec![b"foo_bar".to_vec()];
        let r = M4ShBuiltins::as_tr_sh(&args);
        assert_eq!(String::from_utf8_lossy(&r), "foo_bar");
    }

    #[test]
    fn test_as_tr_sh_empty() {
        let r = M4ShBuiltins::as_tr_sh(&[]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_as_tr_cpp_uppercase() {
        let args = vec![b"foo".to_vec()];
        let r = M4ShBuiltins::as_tr_cpp(&args);
        assert_eq!(String::from_utf8_lossy(&r), "FOO");
    }

    #[test]
    fn test_as_tr_cpp_complex() {
        let args = vec![b"foo.bar-baz".to_vec()];
        let r = M4ShBuiltins::as_tr_cpp(&args);
        assert_eq!(String::from_utf8_lossy(&r), "FOO_BAR_BAZ");
    }

    // --- AS_UNSET ---
    #[test]
    fn test_as_unset_single() {
        let args = vec![b"VAR".to_vec()];
        let r = M4ShBuiltins::as_unset(&args);
        assert_eq!(String::from_utf8_lossy(&r), "unset VAR\n");
    }

    #[test]
    fn test_as_unset_multiple() {
        let args = vec![b"A".to_vec(), b"B".to_vec()];
        let r = M4ShBuiltins::as_unset(&args);
        assert_eq!(String::from_utf8_lossy(&r), "unset A B\n");
    }

    #[test]
    fn test_as_unset_empty() {
        let r = M4ShBuiltins::as_unset(&[]);
        assert_eq!(String::from_utf8_lossy(&r), "unset");
    }

    // --- AS_BOX ---
    #[test]
    fn test_as_box_basic() {
        let args = vec![b"HEADER".to_vec()];
        let r = M4ShBuiltins::as_box(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("## ###### ##"));
        assert!(s.contains("## HEADER ##"));
    }

    #[test]
    fn test_as_box_empty() {
        let r = M4ShBuiltins::as_box(&[]);
        assert_eq!(String::from_utf8_lossy(&r), "## ##");
    }

    // --- AS_VERSION_COMPARE ---
    #[test]
    fn test_as_version_compare_full() {
        let args = vec![
            b"1.0".to_vec(),
            b"2.0".to_vec(),
            b"echo older".to_vec(),
            b"echo same".to_vec(),
            b"echo newer".to_vec(),
        ];
        let r = M4ShBuiltins::as_version_compare(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("ax_compare_version"));
        assert!(s.contains("echo older"));
        assert!(s.contains("echo same"));
        assert!(s.contains("echo newer"));
    }

    #[test]
    fn test_as_version_compare_few_args() {
        let r = M4ShBuiltins::as_version_compare(&[b"1.0".to_vec()]);
        assert!(r.is_empty());
    }

    // --- AS_EXECUTABLE_P ---
    #[test]
    fn test_as_executable_p_path() {
        let args = vec![b"/usr/bin/sh".to_vec()];
        let r = M4ShBuiltins::as_executable_p(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("test -f"));
        assert!(s.contains("test -x"));
    }

    #[test]
    fn test_as_executable_p_empty() {
        let r = M4ShBuiltins::as_executable_p(&[]);
        assert_eq!(String::from_utf8_lossy(&r), "test -f && test -x");
    }

    // --- AS_ME_PREPARE ---
    #[test]
    fn test_as_me_prepare() {
        let r = M4ShBuiltins::as_me_prepare();
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("as_me="));
        assert!(s.contains("as_basename"));
    }

    // --- AS_SET_CATFILE ---
    #[test]
    fn test_as_set_catfile() {
        let args = vec![b"VAR".to_vec(), b"/dir".to_vec(), b"file".to_vec()];
        let r = M4ShBuiltins::as_set_catfile(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("case $VAR in"));
    }

    #[test]
    fn test_as_set_catfile_few_args() {
        let r = M4ShBuiltins::as_set_catfile(&[b"V".to_vec()]);
        assert!(r.is_empty());
    }

    // --- AS_VAR_SET / AS_VAR_GET ---
    #[test]
    fn test_as_var_set() {
        let args = vec![b"FOO".to_vec(), b"bar".to_vec()];
        let r = M4ShBuiltins::as_var_set(&args);
        assert_eq!(String::from_utf8_lossy(&r), "FOO=bar\n");
    }

    #[test]
    fn test_as_var_get() {
        let args = vec![b"FOO".to_vec()];
        let r = M4ShBuiltins::as_var_get(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("$FOO"));
    }

    #[test]
    fn test_as_var_get_empty() {
        let r = M4ShBuiltins::as_var_get(&[]);
        assert!(r.is_empty());
    }

    // --- AS_VAR_TEST_SET / AS_VAR_SET_IF ---
    #[test]
    fn test_as_var_test_set() {
        let args = vec![b"VAR".to_vec()];
        let r = M4ShBuiltins::as_var_test_set(&args);
        assert_eq!(String::from_utf8_lossy(&r), "test -n \"$VAR\"");
    }

    #[test]
    fn test_as_var_set_if_full() {
        let args = vec![
            b"VAR".to_vec(),
            b"echo set".to_vec(),
            b"echo unset".to_vec(),
        ];
        let r = M4ShBuiltins::as_var_set_if(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("if test -n"));
        assert!(s.contains("echo set"));
        assert!(s.contains("echo unset"));
    }

    // --- AS_VAR_PUSHDEF / AS_VAR_POPDEF ---
    #[test]
    fn test_as_var_pushdef() {
        let args = vec![b"CC".to_vec(), b"gcc".to_vec()];
        let r = M4ShBuiltins::as_var_pushdef(&args);
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("as_var_pushdef_CC"));
    }

    #[test]
    fn test_as_var_popdef() {
        let args = vec![b"CC".to_vec()];
        let r = M4ShBuiltins::as_var_popdef(&args);
        assert!(String::from_utf8_lossy(&r).contains("CC=$as_var_pushdef_CC"));
    }

    // --- AS_VAR_APPEND / AS_VAR_ARITH ---
    #[test]
    fn test_as_var_append() {
        let args = vec![b"CFLAGS".to_vec(), b"-O2".to_vec()];
        let r = M4ShBuiltins::as_var_append(&args);
        assert_eq!(String::from_utf8_lossy(&r), "CFLAGS=\"$CFLAGS -O2\"\n");
    }

    #[test]
    fn test_as_var_arith() {
        let args = vec![b"N".to_vec(), b"1 + 2".to_vec()];
        let r = M4ShBuiltins::as_var_arith(&args);
        assert_eq!(String::from_utf8_lossy(&r), "N=$(( 1 + 2 ))\n");
    }

    #[test]
    fn test_as_var_append_few_args() {
        let r = M4ShBuiltins::as_var_append(&[b"X".to_vec()]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_as_var_arith_few_args() {
        let r = M4ShBuiltins::as_var_arith(&[b"X".to_vec()]);
        assert!(r.is_empty());
    }
}

// ============================================================================
// M4-level m4sh Macro Tests (through the full engine pipeline)
// ============================================================================
#[cfg(test)]
mod m4_level {
    use super::*;

    #[test]
    fn test_as_echo_m4() {
        let o = expand("AS_ECHO([hello world])");
        assert!(o.contains("hello world"));
    }

    #[test]
    fn test_as_echo_n_m4() {
        let o = expand("AS_ECHO_N([hello])");
        assert!(o.contains("hello"));
    }

    #[test]
    fn test_as_if_m4() {
        let o = expand("AS_IF([test 1 -eq 1],[echo yes])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_if_else_m4() {
        let o = expand("AS_IF([test 1 -eq 2],[echo yes],[echo no])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_case_m4() {
        let o = expand("AS_CASE([$x],[a],[echo a],[echo default])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_for_m4() {
        let o = expand("AS_FOR([i],[1 2 3],[echo $i])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_mkdir_p_m4() {
        let o = expand("AS_MKDIR_P([/tmp/dir])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_tr_sh_m4() {
        let o = expand("AS_TR_SH([foo.bar])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_tr_cpp_m4() {
        let o = expand("AS_TR_CPP([foo.bar])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_unset_m4() {
        let o = expand("AS_UNSET([VAR])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_exit_m4() {
        let o = expand("AS_EXIT([0])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_box_m4() {
        let o = expand("AS_BOX([Test Header])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_version_compare_m4() {
        let o = expand("AS_VERSION_COMPARE([1.0],[2.0],[echo old],[echo same],[echo new])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_executable_p_m4() {
        let o = expand("AS_EXECUTABLE_P([/bin/sh])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_set_catfile_m4() {
        let o = expand("AS_SET_CATFILE([VAR],[dir],[file])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_var_set_m4() {
        let o = expand("AS_VAR_SET([FOO],[bar])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_var_get_m4() {
        let o = expand("AS_VAR_GET([FOO])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_var_test_set_m4() {
        let o = expand("AS_VAR_TEST_SET([VAR])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_var_set_if_m4() {
        let o = expand("AS_VAR_SET_IF([VAR],[echo set],[echo unset])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_var_append_m4() {
        let o = expand("AS_VAR_APPEND([CFLAGS],[-O2])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_var_arith_m4() {
        let o = expand("AS_VAR_ARITH([N],[1+2])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_var_pushdef_m4() {
        let o = expand("AS_VAR_PUSHDEF([CC],[gcc])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_var_popdef_m4() {
        let o = expand("AS_VAR_POPDEF([CC])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_escape_m4() {
        let o = expand("AS_ESCAPE([hello \\$world])");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_as_me_prepare_m4() {
        let o = expand("AS_ME_PREPARE");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_combined_m4sh_script() {
        let o = expand(
            "AS_ECHO([checking...])\n\
             AS_IF([test -f /etc/passwd],[\n\
               AS_ECHO([found])\n\
             ],[\n\
               AS_ECHO([not found])\n\
             ])\n\
             AS_VAR_SET([RESULT],[ok])\n\
             AS_ECHO([done])\n",
        );
        assert!(o.len() > 100, "combined m4sh script must produce output");
    }

    #[test]
    fn test_m4sh_with_case_for() {
        let o = expand(
            "AS_CASE([$os],\n\
               [linux], [AS_ECHO([Linux])],\n\
               [darwin], [AS_ECHO([macOS])],\n\
               [AS_ECHO([unknown])])\n\
             AS_FOR([f],[a b c],[AS_ECHO([$f])])\n",
        );
        assert!(!o.is_empty());
    }
}
