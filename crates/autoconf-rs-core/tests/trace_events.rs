//! Trace event integration tests.
//!
//! Proves that M4 macro expansion populates the TraceLog with structured
//! events — the panel-mandated architectural spine.
//!
//! Court: AC.TRACE.1

#[cfg(test)]
mod tests {
    use autoconf_rs_core::m4_engine::M4Engine;
    use autoconf_rs_core::trace::{AutoconfEvent, TraceLog};

    #[test]
    fn test_trace_events_from_m4_expansion() {
        let mut engine = M4Engine::new();

        // Process a simple configure.ac through M4
        let _result = engine.process(
            "AC_INIT([trace_test], [1.0], [bugs@test.org])\n\
             AC_SUBST([CC], [gcc])\n\
             AC_DEFINE([HAVE_FOO], [1], [Define to 1 if foo is available])\n\
             AC_CONFIG_FILES([Makefile])\n\
             AC_CHECK_FUNC([malloc])\n\
             AC_OUTPUT\n",
        );

        let log = &engine.trace_log;

        // Verify trace events were populated
        println!("Trace events captured: {}", log.events.len());
        assert!(
            !log.events.is_empty(),
            "Trace log must have events after process() — panel mandate: M4 expansion is source of truth"
        );

        // Verify key event types
        let has_init = log
            .events
            .iter()
            .any(|e| matches!(e, AutoconfEvent::Init { .. }));
        let has_subst = log
            .events
            .iter()
            .any(|e| matches!(e, AutoconfEvent::Subst { .. }));
        let has_define = log
            .events
            .iter()
            .any(|e| matches!(e, AutoconfEvent::Define { .. }));
        let has_config = log
            .events
            .iter()
            .any(|e| matches!(e, AutoconfEvent::ConfigFile { .. }));

        assert!(has_init, "must have Init event");
        assert!(has_subst, "must have Subst event");
        assert!(has_define, "must have Define event");
        assert!(has_config, "must have ConfigFile event");

        println!(
            "  Init: {}, Subst: {}, Define: {}, ConfigFile: {}",
            has_init, has_subst, has_define, has_config
        );

        // Verify autom4te trace format works
        let traces = log.emit_autom4te_traces();
        println!("  autom4te traces: {}", traces.len());
        for t in &traces {
            println!("    {}", t);
        }
        assert!(!traces.is_empty(), "autom4te traces must be non-empty");
    }

    #[test]
    fn test_autom4te_trace_format() {
        use autoconf_rs_core::trace::{Span, TraceLog};

        let mut log = TraceLog::new();
        log.push(AutoconfEvent::Init {
            package: "test".into(),
            version: "1.0".into(),
            bug_report: Some("bugs@test.org".into()),
            tarname: None,
            origin: Span::new("configure.ac", 1, 1),
        });
        log.push(AutoconfEvent::Subst {
            name: "CC".into(),
            value: Some("gcc".into()),
            origin: Span::new("configure.ac", 3, 1),
        });
        log.push(AutoconfEvent::Define {
            name: "HAVE_FOO".into(),
            value: Some("1".into()),
            description: Some("Define to 1".into()),
            origin: Span::new("configure.ac", 5, 1),
        });

        let traces = log.emit_autom4te_traces();

        assert_eq!(traces.len(), 3);
        assert_eq!(traces[0], "configure.ac:1:AC_INIT:test:1.0");
        assert_eq!(traces[1], "configure.ac:3:AC_SUBST:CC:gcc");
        assert_eq!(traces[2], "configure.ac:5:AC_DEFINE:HAVE_FOO:1:Define to 1");

        println!("autom4te trace format verified:");
        for t in &traces {
            println!("  {}", t);
        }
    }

    #[test]
    fn test_trace_log_roundtrip() {
        // Verify TraceLog can be created from events and serialized
        let log = create_sample_log();
        let traces = log.emit_autom4te_traces();

        // Verify we can parse back (basic check)
        for trace in &traces {
            let parts: Vec<&str> = trace.split(':').collect();
            assert!(
                parts.len() >= 4,
                "trace should have at least 4 parts: {}",
                trace
            );
            println!(
                "  Parsed: file={}, line={}, macro={}, args={}..",
                parts[0],
                parts[1],
                parts[2],
                parts.get(3).unwrap_or(&"?")
            );
        }
    }

    fn create_sample_log() -> TraceLog {
        use autoconf_rs_core::trace::{Span, TraceLog};
        let mut log = TraceLog::new();
        log.push(AutoconfEvent::Init {
            package: "sample".into(),
            version: "2.0".into(),
            bug_report: None,
            tarname: None,
            origin: Span::new("configure.ac", 1, 1),
        });
        log.push(AutoconfEvent::ConfigFile {
            output: "Makefile".into(),
            inputs: vec!["Makefile.in".into()],
            origin: Span::new("configure.ac", 3, 1),
        });
        log.push(AutoconfEvent::Output {
            origin: Span::new("configure.ac", 5, 1),
        });
        log
    }
}
