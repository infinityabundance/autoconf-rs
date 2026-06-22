//! M4 Engine Deep Tests — AC.M4.ENGINE
//!
//! Tests: DiversionManager edge cases, TraceLog operations,
//! m4-rs builtin macros, shell generation edge cases.

use autoconf_rs_core::diversion::DiversionManager;
use autoconf_rs_core::trace::{AutoconfEvent, Span, TraceLog};
use autoconf_rs_core::M4Engine;

fn run(input: &str) -> String {
    M4Engine::new().process(input).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // === DiversionManager Deep Tests ===
    #[test]
    fn test_diversion_clear() {
        let mut dm = DiversionManager::new();
        dm.write(b"data");
        dm.clear();
        let out = dm.collect_all();
        assert!(out.is_empty(), "clear should empty all diversions");
    }

    #[test]
    fn test_diversion_multiple_writes() {
        let mut dm = DiversionManager::new();
        dm.write(b"a");
        dm.write(b"b");
        dm.divert(1);
        dm.write(b"c");
        dm.divert(0);
        dm.write(b"d");
        let out = dm.collect_all();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("ab"), "div0: {}", s);
        assert!(s.contains("c"), "div1: {}", s);
        assert!(s.contains("d"), "div0 after: {}", s);
    }

    #[test]
    fn test_diversion_large_number() {
        let mut dm = DiversionManager::new();
        dm.divert(999);
        dm.write(b"far");
        dm.divert(0);
        dm.write(b"near");
        let out = dm.collect_all();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("near"), "div0: {}", s);
        assert!(s.contains("far"), "div999: {}", s);
    }

    #[test]
    fn test_diversion_negative() {
        let mut dm = DiversionManager::new();
        dm.divert(-1);
        dm.write(b"discarded");
        dm.divert(0);
        dm.write(b"kept");
        let out = dm.collect_all();
        let s = String::from_utf8_lossy(&out);
        assert!(!s.contains("discarded"), "div-1 should discard: {}", s);
        assert!(s.contains("kept"), "div0: {}", s);
    }

    #[test]
    fn test_diversion_stats() {
        let mut dm = DiversionManager::new();
        dm.write(b"x");
        dm.divert(1);
        dm.write(b"y");
        let stats = dm.stats();
        assert!(stats.0 >= 1, "stats should have buffers: {:?}", stats);
        assert!(stats.1 > 0, "stats should track total_written: {:?}", stats);
    }

    // === TraceLog Deep Tests ===
    #[test]
    fn test_tracelog_new_empty() {
        let tl = TraceLog::new();
        assert!(tl.events.is_empty());
        assert_eq!(tl.emit_autom4te_traces().len(), 0);
    }

    #[test]
    fn test_tracelog_push_multiple() {
        let mut tl = TraceLog::new();
        tl.push(AutoconfEvent::Init {
            package: "t".into(),
            version: "1".into(),
            bug_report: None,
            tarname: None,
            origin: Span::new("f", 1, 1),
        });
        tl.push(AutoconfEvent::Init {
            package: "t2".into(),
            version: "2".into(),
            bug_report: None,
            tarname: None,
            origin: Span::new("f", 2, 1),
        });
        assert_eq!(tl.events.len(), 2);
    }

    #[test]
    fn test_tracelog_structured_traces() {
        let mut tl = TraceLog::new();
        tl.push(AutoconfEvent::Define {
            name: "FOO".into(),
            value: Some("1".into()),
            description: Some("desc".into()),
            origin: Span::new("f", 1, 1),
        });
        let structured = tl.structured_traces();
        assert!(!structured.is_empty());
        assert_eq!(structured[0].2, "AC_DEFINE");
    }

    #[test]
    fn test_tracelog_format_trace() {
        let formatted = TraceLog::format_trace(
            "configure.ac",
            42,
            "AC_DEFINE",
            &["FOO".into(), "1".into()],
            "$f:$l:$n:$1",
        );
        assert!(formatted.contains("configure.ac"));
        assert!(formatted.contains("42"));
        assert!(formatted.contains("AC_DEFINE"));
        assert!(formatted.contains("FOO"));
    }

    // === Builtin M4 Macro Tests (via m4-rs-core) ===
    #[test]
    fn test_builtin_dnl() {
        let o = run("dnl this is a comment\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(
            o.contains("#!"),
            "dnl should not affect output: {}",
            &o[..100]
        );
    }

    #[test]
    fn test_builtin_eval() {
        let o = run("AC_INIT([t],eval(1+2))\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "eval must not crash");
    }

    #[test]
    fn test_builtin_len() {
        let o = run("define([L],len([hello]))L\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "len must not crash");
    }

    #[test]
    fn test_builtin_substr() {
        let o = run("define([S],substr([hello],1,3))S\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "substr must not crash");
    }

    #[test]
    fn test_builtin_index() {
        let o = run("define([I],index([hello],[l]))I\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "index must not crash");
    }

    #[test]
    fn test_builtin_regexp() {
        let o = run("define([R],regexp([hello123],[0-9]+))R\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "regexp must not crash");
    }

    #[test]
    fn test_builtin_patsubst() {
        let o =
            run("define([P],patsubst([hello world],[o],[X]))P\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "patsubst must not crash");
    }

    #[test]
    fn test_builtin_translit() {
        let o = run("define([T],translit([hello],[a-z],[A-Z]))T\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "translit must not crash");
    }

    #[test]
    fn test_builtin_incr_decr() {
        let o = run("define([I],incr(5))I\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "incr must not crash");
    }

    #[test]
    fn test_builtin_divnum() {
        let o = run("define([D],divnum)D\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "divnum must not crash");
    }

    #[test]
    fn test_builtin_errprint() {
        let o = run("errprint([test warning])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "errprint must not crash");
    }

    #[test]
    fn test_builtin_shift() {
        let o = run("define([S],$2)define([_],[a],[b],[c])S(_,_)\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "shift must not crash");
    }

    // === Additional Edge Cases ===
    #[test]
    fn test_pipeline_with_comments() {
        let o = run("# This is a shell comment\nAC_INIT([t],[1.0])\n# Another comment\ndnl M4 comment\nAC_OUTPUT\n");
        assert!(o.contains("#!"), "comments must not break output");
    }

    #[test]
    fn test_pipeline_with_newlines() {
        let o = run("\n\n\nAC_INIT([t],[1.0])\n\n\nAC_OUTPUT\n\n\n");
        assert!(!o.is_empty(), "extra newlines must not crash");
    }

    #[test]
    fn test_pipeline_with_tabs() {
        let o = run("\tAC_INIT([t],[1.0])\n\tAC_OUTPUT\n");
        assert!(!o.is_empty(), "tabs must not crash");
    }

    #[test]
    fn test_pipeline_config_links() {
        let o = run("AC_INIT([t],[1.0])\nAC_CONFIG_LINKS([dst:src])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CONFIG_LINKS");
    }

    #[test]
    fn test_pipeline_config_commands() {
        let o = run("AC_INIT([t],[1.0])\nAC_CONFIG_COMMANDS([default])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CONFIG_COMMANDS");
    }

    #[test]
    fn test_pipeline_config_subdirs() {
        let o = run("AC_INIT([t],[1.0])\nAC_CONFIG_SUBDIRS([lib])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CONFIG_SUBDIRS");
    }

    #[test]
    fn test_pipeline_prefix_default() {
        let o = run("AC_INIT([t],[1.0])\nAC_PREFIX_DEFAULT([/usr/local])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_PREFIX_DEFAULT");
    }

    #[test]
    fn test_pipeline_revision() {
        let o = run("AC_INIT([t],[1.0])\nAC_REVISION([$Revision: 1.0 $])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_REVISION");
    }

    #[test]
    fn test_pipeline_copyright() {
        let o = run("AC_INIT([t],[1.0])\nAC_COPYRIGHT([2024])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_COPYRIGHT");
    }

    #[test]
    fn test_pipeline_prereq() {
        let o = run("AC_PREREQ([2.50])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_PREREQ");
    }

    #[test]
    fn test_pipeline_aux_dir() {
        let o = run("AC_INIT([t],[1.0])\nAC_CONFIG_AUX_DIR([build-aux])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CONFIG_AUX_DIR");
    }

    #[test]
    fn test_pipeline_macro_dir() {
        let o = run("AC_INIT([t],[1.0])\nAC_CONFIG_MACRO_DIR([m4])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CONFIG_MACRO_DIR");
    }

    // === Recursive & Nested Macro Expansion ===
    #[test]
    fn test_recursive_define() {
        let o = run("define([RECURSE],[RECURSE])RECURSE\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "recursive define must not infinite-loop");
    }

    #[test]
    fn test_deeply_nested_macros_10_levels() {
        let input = "define([L0],[0])define([L1],[L0])define([L2],[L1])define([L3],[L2])define([L4],[L3])define([L5],[L4])define([L6],[L5])define([L7],[L6])define([L8],[L7])define([L9],[L8])\nAC_INIT([t],[1.0])\nL9\nAC_OUTPUT\n";
        let o = run(input);
        assert!(!o.is_empty(), "10-level nested macros must expand");
    }

    #[test]
    fn test_self_referential_args() {
        let o = run("define([SELF],[$1])SELF([hello])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "self-referential args must not crash");
    }

    #[test]
    fn test_macro_with_empty_args() {
        let o = run("define([EMPTY],[])EMPTY\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "empty macro definition must not crash");
    }

    #[test]
    fn test_macro_with_many_args_8() {
        let o = run("define([MANY],[$1$2$3$4$5$6$7$8])MANY([a],[b],[c],[d],[e],[f],[g],[h])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "8-arg macro must expand");
    }

    #[test]
    fn test_shift_many_args() {
        let o = run("define([SHIFTED],[$2$3])define([_],[a],[b],[c])SHIFTED(shift(_))\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "shift with many args must work");
    }

    // === Combined Builtin Tests ===
    #[test]
    fn test_combined_eval_len_substr() {
        let o =
            run("define([VAL],[substr([hello],0,eval(1+2))])VAL\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "combined eval+len+substr must not crash");
    }

    #[test]
    fn test_combined_patsubst_translit() {
        let o = run("define([X],[translit(patsubst([hello world],[o],[X]),[a-z],[A-Z])])X\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "combined patsubst+translit must not crash");
    }

    #[test]
    fn test_combined_regexp_index() {
        let o = run("define([IDX],[index([hello123],regexp([hello123],[0-9]+]))])IDX\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "combined regexp+index must not crash");
    }

    #[test]
    fn test_incr_decr_chain() {
        let o = run("define([C],[decr(incr(incr(2)))])\nAC_INIT([t],C)\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "incr/decr chain must not crash");
    }

    // === Diversion & Output Ordering ===
    #[test]
    fn test_divert_five_levels() {
        let mut dm = DiversionManager::new();
        dm.divert(1);
        dm.write(b"first");
        dm.divert(3);
        dm.write(b"third");
        dm.divert(2);
        dm.write(b"second");
        dm.divert(0);
        dm.write(b"zero");
        dm.divert(5);
        dm.write(b"fifth");
        let out = dm.collect_all();
        let s = String::from_utf8_lossy(&out);
        // zero appears first, then 1,2,3,5
        let p0 = s.find("zero").unwrap();
        let p1 = s.find("first").unwrap();
        let p2 = s.find("second").unwrap();
        let p3 = s.find("third").unwrap();
        let p5 = s.find("fifth").unwrap();
        assert!(p0 < p1, "div0 before div1");
        assert!(p1 < p2, "div1 before div2");
        assert!(p2 < p3, "div2 before div3");
        assert!(p3 < p5, "div3 before div5");
    }

    #[test]
    fn test_undivert_then_write() {
        let mut dm = DiversionManager::new();
        dm.divert(2);
        dm.write(b"stored");
        dm.divert(0);
        dm.undivert(2);
        dm.write(b" after");
        let out = dm.collect_all();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("stored"), "undivert must flush: {}", s);
        assert!(s.contains("after"), "write after undivert: {}", s);
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut dm = DiversionManager::new();
        dm.divert(7);
        dm.write(b"peekaboo");
        let peek1 = dm.peek(7);
        let peek2 = dm.peek(7);
        assert!(peek1.is_some() && peek2.is_some());
        assert_eq!(peek1.unwrap(), peek2.unwrap());
    }

    #[test]
    fn test_divnum_after_clear() {
        let mut dm = DiversionManager::new();
        dm.divert(42);
        dm.clear();
        assert_eq!(dm.divnum(), 0);
    }

    // === Trace Log Completeness ===
    #[test]
    fn test_tracelog_multiple_event_types() {
        let mut engine = M4Engine::new();
        engine.process("AC_INIT([pkg],[2.0])\nAC_SUBST([VAR],[val])\nAC_DEFINE([HAVE_FOO],[1])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n").ok();
        assert!(
            engine.trace_log.events.len() >= 4,
            "should have >= 4 events: {}",
            engine.trace_log.events.len()
        );
    }

    #[test]
    fn test_tracelog_check_func_event() {
        let mut engine = M4Engine::new();
        engine
            .process("AC_INIT([t],[1.0])\nAC_CHECK_FUNC([malloc])\nAC_OUTPUT\n")
            .ok();
        let has_check = engine
            .trace_log
            .events
            .iter()
            .any(|e| matches!(e, autoconf_rs_core::trace::AutoconfEvent::CheckFunc { .. }));
        assert!(has_check, "should have CheckFunc event");
    }

    #[test]
    fn test_tracelog_check_header_event() {
        let mut engine = M4Engine::new();
        engine
            .process("AC_INIT([t],[1.0])\nAC_CHECK_HEADER([stdio.h])\nAC_OUTPUT\n")
            .ok();
        let has_check = engine.trace_log.events.iter().any(|e| {
            matches!(
                e,
                autoconf_rs_core::trace::AutoconfEvent::CheckHeader { .. }
            )
        });
        assert!(has_check, "should have CheckHeader event");
    }

    #[test]
    fn test_tracelog_config_file_event() {
        let mut engine = M4Engine::new();
        engine
            .process("AC_INIT([t],[1.0])\nAC_CONFIG_FILES([Makefile src/config.h])\nAC_OUTPUT\n")
            .ok();
        let config_file_count = engine
            .trace_log
            .events
            .iter()
            .filter(|e| matches!(e, autoconf_rs_core::trace::AutoconfEvent::ConfigFile { .. }))
            .count();
        assert!(
            config_file_count >= 1,
            "should have ConfigFile events: {}",
            config_file_count
        );
    }

    // === Engine State Verification (via public APIs) ===
    #[test]
    fn test_engine_state_package_name() {
        let engine = M4Engine::new();
        let state = engine.state();
        assert!(state.package_name.is_none(), "fresh engine has no package");
    }

    #[test]
    fn test_engine_diversion_output_default() {
        let engine = M4Engine::new();
        let out = engine.diversion_output();
        assert!(out.is_empty(), "fresh engine has empty diversion output");
    }

    #[test]
    fn test_engine_diversion_stats_default() {
        let engine = M4Engine::new();
        let (count, written, discarded) = engine.diversion_stats();
        assert_eq!(count, 0, "fresh engine has zero diversion buffers");
        assert_eq!(written, 0);
        assert_eq!(discarded, 0);
    }

    // === Output Verification ===
    #[test]
    fn test_output_has_shebang() {
        let o = run("AC_INIT([testpkg],[3.14])\nAC_OUTPUT\n");
        assert!(
            o.contains("#! /bin/sh") || o.contains("#!/bin/sh"),
            "output must have shebang"
        );
    }

    #[test]
    fn test_output_contains_package_name() {
        let o = run("AC_INIT([testpkg],[3.14])\nAC_OUTPUT\n");
        assert!(
            o.contains("testpkg"),
            "output must contain package name: {}",
            &o[..200]
        );
    }

    #[test]
    fn test_output_contains_version() {
        let o = run("AC_INIT([testpkg],[3.14])\nAC_OUTPUT\n");
        assert!(o.contains("3.14"), "output must contain version");
    }

    // === Special Characters ===
    #[test]
    fn test_backtick_in_input() {
        let o = run("AC_INIT([t],[1.0])\nAC_DEFINE([FOO],[backtick\x60quoted\x60])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "backticks must not break output");
    }

    #[test]
    fn test_dollar_sign_in_input() {
        let o = run("AC_INIT([t],[$version])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "dollar signs must not break output");
    }

    #[test]
    fn test_comma_in_package_name() {
        let o = run("AC_INIT([name,with,commas],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "commas in package name must not crash");
    }

    #[test]
    fn test_unicode_in_string() {
        let o = run("AC_INIT([t],[1.0])\nAC_DEFINE([DESC],[Café üñîçø∂é])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "unicode must not crash");
    }

    // === Engine Pipeline Integration ===
    #[test]
    fn test_full_pipeline_with_substitutions() {
        let o = run("AC_INIT([full],[2.0],[bugs@ex.com])\nAC_SUBST([prefix],[/usr])\nAC_SUBST([bindir],[$prefix/bin])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n");
        assert!(o.contains("bugs@ex.com"), "bug report should appear");
        assert!(o.contains("Makefile"), "config files should appear");
    }

    #[test]
    fn test_full_pipeline_with_libs() {
        let o = run("AC_INIT([libtest],[1.0])\nAC_CHECK_LIB([m],[sqrt])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CHECK_LIB must not crash");
    }

    #[test]
    fn test_full_pipeline_with_search_libs() {
        let o = run("AC_INIT([srch],[1.0])\nAC_SEARCH_LIBS([sqrt],[m])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_SEARCH_LIBS must not crash");
    }

    #[test]
    fn test_pipeline_with_canonical_host() {
        let o = run("AC_INIT([canon],[1.0])\nAC_CANONICAL_HOST\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CANONICAL_HOST must not crash");
    }

    #[test]
    fn test_pipeline_with_canonical_build() {
        let o = run("AC_INIT([canon],[1.0])\nAC_CANONICAL_BUILD\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CANONICAL_BUILD must not crash");
    }

    #[test]
    fn test_pipeline_with_m4_pattern_forbid() {
        let o = run("m4_pattern_forbid([^_AC_])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "m4_pattern_forbid must not crash");
    }

    #[test]
    fn test_pipeline_with_m4_pattern_allow() {
        let o = run("m4_pattern_allow([^AC_])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "m4_pattern_allow must not crash");
    }

    // === Large Input ===
    #[test]
    fn test_large_input_200_defines() {
        let mut input = String::new();
        for i in 0..200 {
            input.push_str(&format!("AC_DEFINE([VAR_{0}],[{0}])\n", i));
        }
        input.push_str("AC_INIT([big],[1.0])\nAC_OUTPUT\n");
        let o = run(&input);
        assert!(
            o.len() > 1000,
            "large input must produce output, got {} bytes",
            o.len()
        );
    }

    #[test]
    fn test_large_substitutions_100() {
        let mut input = String::from("AC_INIT([big],[1.0])\n");
        for i in 0..100 {
            input.push_str(&format!("AC_SUBST([VAR_{0}],[val_{0}])\n", i));
        }
        input.push_str("AC_OUTPUT\n");
        let o = run(&input);
        assert!(
            o.len() > 1000,
            "100 substitutions must produce output: {}",
            o.len()
        );
    }

    #[test]
    fn test_output_not_empty_for_smoke() {
        let o = run("AC_INIT([smoke],[0.1])\nAC_OUTPUT\n");
        assert!(
            o.len() > 100,
            "smoke output must be substantial: {}",
            o.len()
        );
    }

    // === Config Links/Commands ===
    #[test]
    fn test_pipeline_config_links_multiple() {
        let o = run("AC_INIT([t],[1.0])\nAC_CONFIG_LINKS([a:b c:d])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "AC_CONFIG_LINKS multiple");
    }

    #[test]
    fn test_pipeline_config_commands_with_cmds() {
        let o = run(
            "AC_INIT([t],[1.0])\nAC_CONFIG_COMMANDS([default],[echo ok],[echo help])\nAC_OUTPUT\n",
        );
        assert!(!o.is_empty(), "AC_CONFIG_COMMANDS with init/cmds");
    }

    // === ifelse/conditional Macros ===
    #[test]
    fn test_ifelse_three_way() {
        let o = run("define([X],2)define([CMP],[ifelse(X,1,[one],X,2,[two],[other])])CMP\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "ifelse three-way must not crash");
    }

    #[test]
    fn test_ifelse_with_quotes() {
        let o = run("define([F],[ifelse([$1],[hello],[hi],[goodbye])])F([hello])\nAC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(!o.is_empty(), "ifelse with quoted args must not crash");
    }
}
