//! Self-host and signal integration tests — CROSS.040/041/046
//!
//! Tests: signal handling, stack overflow protection, self-host configure.ac.
//!
//! Court: CROSS.040/CROSS.041/CROSS.046

use autoconf_rs_core::signal;
use autoconf_rs_core::M4Engine;

#[cfg(test)]
mod tests {
    use super::*;

    // === CROSS.040: Signal handling ===
    #[test]
    fn test_signal_registration() {
        signal::register_signal_handlers();
        signal::clear_signals();
        assert!(!signal::sigint_received());
        assert!(!signal::sigpipe_received());
    }

    #[test]
    fn test_signal_aware_engine() {
        let mut engine = M4Engine::new();
        engine.signal_aware = true;
        // No signal pending — should process normally
        let result = engine.process("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_with_signal_aware_disabled() {
        let mut engine = M4Engine::new();
        engine.signal_aware = false;
        let result = engine.process("AC_INIT([t],[1.0])\nAC_OUTPUT\n");
        assert!(result.is_ok());
    }

    // === CROSS.041: Stack overflow protection ===
    #[test]
    fn test_engine_recursion_bounded() {
        let mut engine = M4Engine::new();
        // Deeply nested macros should not overflow stack
        let o = engine
            .process(
                "define([L0],[0])define([L1],[L0])define([L2],[L1])define([L3],[L2])define([L4],[L3])define([L5],[L4])define([L6],[L5])define([L7],[L6])define([L8],[L7])define([L9],[L8])\nAC_INIT([t],[1.0])\nL9\nAC_OUTPUT\n",
            )
            .unwrap_or_default();
        assert!(!o.is_empty());
    }

    #[test]
    fn test_m4_join_bounded_recursion() {
        let mut engine = M4Engine::new();
        // m4_join limited to 4 args — no infinite recursion
        let o = engine
            .process("AC_INIT([t],[1.0])\nm4_join([-],[a],[b])\nAC_OUTPUT\n")
            .unwrap_or_default();
        assert!(!o.is_empty());
    }

    // === CROSS.046: Self-host configure.ac ===
    #[test]
    fn test_self_host_configure_ac() {
        let mut engine = M4Engine::new();
        let input = include_str!("../../../configure.ac");
        let result = engine.process(input);
        assert!(result.is_ok(), "self-host configure.ac: {:?}", result.err());
        let o = result.unwrap();
        assert!(o.contains("#!"));
        assert!(o.contains("autoconf-rs"));
        assert!(o.len() > 500);
    }
}
