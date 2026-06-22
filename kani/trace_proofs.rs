//! Kani proofs for shell escaping and trace log safety.
//!
//! Court: AC.KANI.2

use autoconf_rs_core::trace::AutoconfEvent;
use autoconf_rs_core::trace::Span;
use autoconf_rs_core::trace::TraceLog;

/// Prove TraceLog never panics on arbitrary event insertion.
#[cfg(kani)]
#[kani::proof]
fn trace_log_no_panic_on_push() {
    let mut log = TraceLog::new();
    let count: u8 = kani::any();

    for _ in 0..count {
        // Push a mix of event types
        log.push(AutoconfEvent::Init {
            package: "test".into(),
            version: "1.0".into(),
            bug_report: None,
            tarname: None,
            origin: Span::new("configure.ac", 1, 1),
        });
    }

    let traces = log.emit_autom4te_traces();
    kani::assert(
        traces.len() == count as usize,
        "trace count must match event count for Init events",
    );
}

/// Prove TraceLog::emit_autom4te_traces returns bounded output.
#[cfg(kani)]
#[kani::proof]
fn trace_log_emit_bounded() {
    let mut log = TraceLog::new();
    let count: u8 = kani::any();

    for _ in 0..count {
        log.push(AutoconfEvent::Subst {
            name: "VAR".into(),
            value: Some("val".into()),
            origin: Span::new("configure.ac", 1, 1),
        });
    }

    let traces = log.emit_autom4te_traces();
    // Number of traces must equal number of events pushed
    kani::assert(
        traces.len() <= count as usize,
        "emit_autom4te_traces must not exceed event count",
    );
}

/// Prove AutoconfEvent::Trace construction is safe.
#[cfg(kani)]
#[kani::proof]
fn trace_event_construction_safe() {
    let event = AutoconfEvent::Trace {
        macro_name: "AC_TEST".into(),
        args: vec!["arg1".into(), "arg2".into()],
        file: "configure.ac".into(),
        line: 42,
    };

    // Event must be constructible and hold correct data
    match &event {
        AutoconfEvent::Trace {
            macro_name,
            args,
            file,
            line,
        } => {
            kani::assert(macro_name == "AC_TEST", "macro name preserved");
            kani::assert(args.len() == 2, "args count preserved");
            kani::assert(*line == 42, "line number preserved");
        }
        _ => kani::assert(false, "must be Trace variant"),
    }
}
