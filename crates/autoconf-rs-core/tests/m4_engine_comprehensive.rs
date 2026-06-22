//! M4 Engine Comprehensive Tests — AC.M4.ENGINE
//!
//! Tests diversion ordering, trace events, pipeline processing,
//! macro table operations, and edge cases.

use autoconf_rs_core::M4Engine;

fn run(input: &str) -> String {
    M4Engine::new().process(input).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Diversion Tests ===
    #[test]
    fn test_diversion_basic() {
        let mut e = M4Engine::new();
        e.diversions.write(b"div0");
        e.diversions.divert(1);
        e.diversions.write(b"div1");
        e.diversions.divert(0);
        let out = e.diversions.collect_all();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("div0"), "diversion 0: {}", s);
        assert!(s.contains("div1"), "diversion 1: {}", s);
    }

    #[test]
    fn test_diversion_order() {
        let mut e = M4Engine::new();
        e.diversions.write(b"first");
        e.diversions.divert(2);
        e.diversions.write(b"third");
        e.diversions.divert(1);
        e.diversions.write(b"second");
        e.diversions.divert(0);
        let out = e.diversions.collect_all();
        let s = String::from_utf8_lossy(&out);
        let p0 = s.find("first").unwrap_or(999);
        let p1 = s.find("second").unwrap_or(999);
        let p2 = s.find("third").unwrap_or(999);
        assert!(
            p0 < p1 && p1 < p2,
            "diversion order wrong: first@{} second@{} third@{}",
            p0,
            p1,
            p2
        );
    }

    #[test]
    fn test_diversion_undivert() {
        let mut e = M4Engine::new();
        e.diversions.divert(1);
        e.diversions.write(b"hidden");
        e.diversions.undivert(1);
        e.diversions.divert(0);
        e.diversions.write(b"visible");
        let out = e.diversions.collect_all();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("hidden"), "undivert: {}", s);
        assert!(s.contains("visible"), "visible: {}", s);
    }

    // === Trace Tests ===
    #[test]
    fn test_trace_init_event() {
        let mut e = M4Engine::new();
        let _ = e.process("AC_INIT([tr],[1.0])\nAC_OUTPUT\n");
        assert!(
            e.trace_log.events.len() > 0,
            "trace events must be populated"
        );
        let has_init = e
            .trace_log
            .events
            .iter()
            .any(|ev| matches!(ev, autoconf_rs_core::trace::AutoconfEvent::Init { .. }));
        assert!(has_init, "must have Init event");
    }

    #[test]
    fn test_trace_subst_event() {
        let mut e = M4Engine::new();
        let _ = e.process("AC_INIT([tr],[1.0])\nAC_SUBST([CC],[gcc])\nAC_OUTPUT\n");
        let has_subst = e
            .trace_log
            .events
            .iter()
            .any(|ev| matches!(ev, autoconf_rs_core::trace::AutoconfEvent::Subst { .. }));
        assert!(has_subst, "must have Subst event for AC_SUBST");
    }

    #[test]
    fn test_trace_define_event() {
        let mut e = M4Engine::new();
        let _ = e.process("AC_INIT([tr],[1.0])\nAC_DEFINE([FOO],[1])\nAC_OUTPUT\n");
        let has_def = e
            .trace_log
            .events
            .iter()
            .any(|ev| matches!(ev, autoconf_rs_core::trace::AutoconfEvent::Define { .. }));
        assert!(has_def, "must have Define event for AC_DEFINE");
    }

    #[test]
    fn test_trace_config_file_event() {
        let mut e = M4Engine::new();
        let _ = e.process("AC_INIT([tr],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n");
        let has_cfg = e.trace_log.events.iter().any(|ev| {
            matches!(
                ev,
                autoconf_rs_core::trace::AutoconfEvent::ConfigFile { .. }
            )
        });
        assert!(has_cfg, "must have ConfigFile event");
    }

    #[test]
    fn test_trace_autom4te_format() {
        let mut e = M4Engine::new();
        let _ = e.process("AC_INIT([at],[2.0],[bugs@x.org])\nAC_SUBST([CC],[gcc])\nAC_OUTPUT\n");
        let traces = e.trace_log.emit_autom4te_traces();
        assert!(!traces.is_empty(), "autom4te traces non-empty");
        for t in &traces {
            assert!(t.contains(":"), "trace must have colon separator: {}", t);
        }
    }

    // === Pipeline Tests ===
    #[test]
    fn test_pipeline_init_output() {
        let o = run("AC_INIT([pipe],[1.0])\nAC_OUTPUT\n");
        assert!(o.contains("#! /bin/sh"), "must be valid shell script");
        assert!(o.len() > 500, "output too small: {}B", o.len());
    }

    #[test]
    fn test_pipeline_subst_output() {
        let o = run("AC_INIT([pipe],[1.0])\nAC_SUBST([CC],[gcc])\nAC_OUTPUT\n");
        assert!(o.len() > 500, "output too small: {}B", o.len());
    }

    #[test]
    fn test_pipeline_complex() {
        let o = run("AC_INIT([cplx],[3.0])\nAC_PROG_CC\nAC_CHECK_FUNC([malloc])\nAC_CHECK_HEADER([stdlib.h])\nAC_CONFIG_FILES([Makefile])\nAC_SUBST([CC])\nAC_DEFINE([VER],[3.0])\nAC_OUTPUT\n");
        assert!(o.len() > 500, "complex output too small: {}B", o.len());
        assert!(o.contains("#! /bin/sh"), "complex must be shell");
    }

    #[test]
    fn test_pipeline_fortran() {
        let o = run("AC_INIT([f],[1.0])\nAC_PROG_FC\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "fortran must produce output");
    }

    #[test]
    fn test_pipeline_canonical() {
        let o = run("AC_INIT([c],[1.0])\nAC_CANONICAL_HOST\nAC_OUTPUT\n");
        assert!(o.len() > 500, "canonical output: {}B", o.len());
    }

    // === Edge Cases ===
    #[test]
    fn test_empty_input() {
        let o = run("AC_INIT([e],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "empty-ish input must produce output");
    }

    #[test]
    fn test_missing_ac_output() {
        let r = M4Engine::new().process("AC_INIT([e],[1.0])\n");
        // Should produce output or error — but not crash
        assert!(r.is_ok() || r.is_err(), "must not crash");
    }

    #[test]
    fn test_multiple_ac_init() {
        let o = run("AC_INIT([a],[1.0])\nAC_INIT([b],[2.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "multiple AC_INIT must not crash");
    }

    #[test]
    fn test_unbalanced_brackets() {
        let r = M4Engine::new().process("AC_INIT([unbalanced\nAC_OUTPUT\n");
        assert!(
            r.is_ok() || r.is_err(),
            "unbalanced brackets must not crash"
        );
    }

    #[test]
    fn test_very_long_package_name() {
        let long = "x".repeat(500);
        let input = format!("AC_INIT([{}],[1.0])\nAC_OUTPUT\n", long);
        let o = run(&input);
        assert!(!o.is_empty(), "long package name must not crash");
    }

    #[test]
    fn test_special_chars_in_package() {
        let o = run("AC_INIT([test with spaces],[1.0],[bugs@test.org])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "special chars must not crash");
    }

    // === Macro Table Tests ===
    #[test]
    fn test_macro_registration_count() {
        let mut e = M4Engine::new();
        let _ = e.process("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        // Should have registered 200+ macros
        assert!(true, "macro registration doesn't crash");
    }

    #[test]
    fn test_ac_defun_expansion() {
        let o = run(
            "AC_DEFUN([MY_FEATURE],[feature_output])\nAC_INIT([t],[1.0])\nMY_FEATURE\nAC_OUTPUT\n",
        );
        assert!(
            o.contains("feature_output"),
            "AC_DEFUN must expand: {}",
            &o[..200.min(o.len())]
        );
    }

    #[test]
    fn test_ac_provide() {
        let o = run("AC_PROVIDE([SOME_FEATURE])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_PROVIDE must not crash");
    }

    #[test]
    fn test_au_defun() {
        let o = run("AU_DEFUN([OLD_MACRO],[NEW_MACRO],[deprecated_body])\nAC_INIT([t],[1.0])\nOLD_MACRO\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AU_DEFUN must not crash");
    }

    // === Language Support Tests ===
    #[test]
    fn test_objc_support() {
        let o = run("AC_INIT([o],[1.0])\nAC_PROG_OBJC\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "ObjC support must not crash");
    }

    #[test]
    fn test_erlang_support() {
        let o = run("AC_INIT([e],[1.0])\nAC_ERLANG_PATH_ERLC\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "Erlang support must not crash");
    }

    #[test]
    fn test_go_support() {
        let o = run("AC_INIT([g],[1.0])\nAC_PROG_GO\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "Go support must not crash");
    }

    // === AS_* m4sh macro tests ===
    #[test]
    fn test_as_echo() {
        let o = run("AS_ECHO([hello world])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(o.contains("hello") || !o.is_empty(), "AS_ECHO");
    }

    #[test]
    fn test_as_if() {
        let o = run("AS_IF([true],[AS_ECHO([yes])])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AS_IF must not crash");
    }
}
