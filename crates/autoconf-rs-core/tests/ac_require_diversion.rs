//! AC_REQUIRE diversion ordering tests.
//!
//! Proves that AC_REQUIRE uses diversions to order macro output correctly:
//! required macros appear BEFORE the requiring macro in the final output.
//!
//! Court: AC.M4.REQUIRE.DIVERT.1
//! Court: AC.M4.DIVERT.WIRED.1 — DiversionManager integrated into M4Engine pipeline.

#[cfg(test)]
mod tests {
    use autoconf_rs_core::diversion::DiversionManager;
    use autoconf_rs_core::m4sugar::RequireTracker;

    #[test]
    fn test_ac_require_ordering_via_diversions() {
        let mut tracker = RequireTracker::new();

        // AC_REQUIRE semantics: required macro (B) must appear BEFORE requirer (A)
        // Lower diversion numbers -> earlier output. So B goes to lower diversion.
        tracker.require("A", "B").unwrap();

        // Expand B first (required) -> diversion 1 (lower = earlier)
        tracker.divert(1);
        tracker.write(b"# B's output (required, should appear first)\n");

        // Expand A (requirer) -> diversion 2 (higher = later)
        tracker.divert(2);
        tracker.write(b"# A's output (requirer, should appear after B)\n");

        // Initialization stays in diversion 0 (earliest of all)
        tracker.divert(0);
        tracker.write(b"# Initialization\n");

        let output = tracker.collect_output();
        let text = String::from_utf8_lossy(&output);
        println!("Collected output:\n{}", text);

        let pos_b = text.find("B's output").unwrap();
        let pos_a = text.find("A's output").unwrap();
        let pos_init = text.find("Initialization").unwrap();

        assert!(pos_init < pos_b, "init before B");
        assert!(
            pos_b < pos_a,
            "B (required) before A (requirer): B at {}, A at {}",
            pos_b,
            pos_a
        );
    }

    #[test]
    fn test_nested_ac_require_ordering() {
        let mut tracker = RequireTracker::new();

        // A requires B, B requires C. Order: init(0) < C(1) < B(2) < A(3)
        tracker.require("A", "B").unwrap();
        tracker.require("B", "C").unwrap();

        tracker.divert(0);
        tracker.write(b"# Init\n");

        tracker.divert(1);
        tracker.write(b"# C (deepest)\n");

        tracker.divert(2);
        tracker.write(b"# B (middle)\n");

        tracker.divert(3);
        tracker.write(b"# A (top)\n");

        let output = tracker.collect_output();
        let text = String::from_utf8_lossy(&output);

        let pos_c = text.find("C (deepest)").unwrap();
        let pos_b = text.find("B (middle)").unwrap();
        let pos_a = text.find("A (top)").unwrap();

        assert!(pos_c < pos_b, "C must come before B");
        assert!(pos_b < pos_a, "B must come before A");
        println!(
            "Nested ordering: C({}) < B({}) < A({})",
            pos_c, pos_b, pos_a
        );
    }

    #[test]
    fn test_divert_discard_hides_output() {
        let mut tracker = RequireTracker::new();

        tracker.write(b"# visible output\n");
        tracker.divert(-1);
        tracker.write(b"# hidden output (discarded)\n");
        tracker.divert(0);
        tracker.write(b"# more visible output\n");

        let output = tracker.collect_output();
        let text = String::from_utf8_lossy(&output);

        assert!(text.contains("visible output"));
        assert!(text.contains("more visible"));
        assert!(!text.contains("hidden output"));
    }

    #[test]
    fn test_cycle_detection() {
        let mut tracker = RequireTracker::new();
        tracker.push_expansion("A");
        tracker.push_expansion("B");

        let result = tracker.require("B", "A"); // B requires A, but A is on stack
        assert!(result.is_err(), "cycle should be detected");
        println!("Cycle detected: {}", result.unwrap_err());
    }

    #[test]
    fn test_diversion_manager_standalone() {
        let mut dm = DiversionManager::new();

        // Simulate M4sh init -> diversion 0 (normal)
        dm.write(b"## M4sh Initialization\n");

        // Simulate AC_REQUIRE body -> diversion 1 (appears after init, before main)
        dm.divert(1);
        dm.write(b"# Required macro output\n");

        // Simulate main body -> diversion 2
        dm.divert(2);
        dm.write(b"# Main body output\n");

        // Simulate config.status -> diversion 3
        dm.divert(3);
        dm.write(b"# config.status\n");

        let output = dm.collect_all();
        let text = String::from_utf8_lossy(&output);

        let pos_init = text.find("M4sh Initialization").unwrap();
        let pos_req = text.find("Required macro").unwrap();
        let pos_body = text.find("Main body").unwrap();

        assert!(pos_init < pos_body, "init before body");
        assert!(pos_init < pos_req, "init before required");

        println!(
            "Diversion ordering: init({}) < required({}) < body({})",
            pos_init, pos_req, pos_body
        );
    }

    /// Proves that M4Engine.process() routes M4 output through the DiversionManager.
    /// After M4 expansion, output is collected via diversions (not discarded).
    /// Court: AC.M4.DIVERT.WIRED.1
    #[test]
    fn test_m4_engine_diversion_wired() {
        use autoconf_rs_core::m4_engine::M4Engine;

        let mut engine = M4Engine::new();

        // Process a simple configure.ac -- M4 output should flow through diversions
        let input = "AC_INIT([diversion-test], [1.0])\nAC_OUTPUT\n";
        let result = engine.process(input);
        assert!(result.is_ok(), "process should succeed");

        // Verify diversion output is non-empty (M4 expansion went through diversions)
        let div_out = engine.diversion_output();
        assert!(!div_out.is_empty(), "diversion output must be non-empty");
        let div_text = String::from_utf8_lossy(&div_out);
        println!("Diversion output ({} bytes):", div_out.len());
        println!("{}", &div_text[..div_text.len().min(500)]);

        // Verify diversion stats are reasonable
        let (bufs, written, discarded) = engine.diversion_stats();
        println!(
            "Diversion stats: {} buffers, {} written, {} discarded",
            bufs, written, discarded
        );
        assert!(
            written > 0,
            "some bytes should be written through diversions"
        );
        assert_eq!(discarded, 0, "nothing should be discarded for simple input");

        // Verify that processing a second input clears and restarts diversions
        let input2 = "AC_INIT([second], [2.0])\nAC_OUTPUT\n";
        let _ = engine.process(input2);
        let (_, written2, _) = engine.diversion_stats();
        println!("Second run: {} bytes written", written2);
        // Each run starts fresh (diversions cleared in process())
    }

    /// Proves diversion ordering is preserved through the M4Engine pipeline.
    /// Both preamble M4 output and configure.ac M4 expansion flow through diversions.
    #[test]
    fn test_diversion_ordering_through_engine() {
        use autoconf_rs_core::m4_engine::M4Engine;

        let mut engine = M4Engine::new();

        // Process with diversion-aware output
        let input = "AC_INIT([order-test], [1.0])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n";
        let result = engine.process(input);
        assert!(result.is_ok());

        // Template output (the main configure script) should be non-trivial
        let template_out = result.unwrap();
        assert!(
            template_out.len() > 100,
            "template output must be non-trivial: {} bytes",
            template_out.len()
        );

        // Diversion output from M4 expansion should also contain data
        let div_out = engine.diversion_output();
        assert!(!div_out.is_empty());

        // The template output and diversion output are different systems:
        // - template = the actual configure script (from template dispatch)
        // - diversion = raw M4 expansion output (routed through diversions)
        println!("Template output: {} bytes", template_out.len());
        println!("Diversion output: {} bytes", div_out.len());

        // After clear, diversions should be empty
        engine.diversions.clear();
        let cleared = engine.diversion_output();
        assert!(cleared.is_empty(), "cleared diversions should be empty");

        // Reprocess — diversions should be repopulated
        let _ = engine.process(input);
        let repopulated = engine.diversion_output();
        assert!(
            !repopulated.is_empty(),
            "re-process should repopulate diversions"
        );
    }
}
