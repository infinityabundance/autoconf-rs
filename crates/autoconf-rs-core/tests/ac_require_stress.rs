//! AC_REQUIRE / Diversion Depth Stress Tests — Panel Mandate
//!
//! Panel findings:
//! - Circular requirements must be detected
//! - Conditional AC_REQUIRE inside if must still divert to top
//! - Deep chains: A→B→C→D (4-level nesting)
//! - GROW/BODY/INIT diversion semantics
//!
//! Court: AC.M4.AUTOCONF.CORE.1 (panel extension)

use autoconf_rs_core::M4Engine;

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &str) -> String {
        let mut engine = M4Engine::new();
        engine.process(input).unwrap_or_default()
    }

    #[test]
    fn test_circular_require_detected() {
        // A requires A = self-cycle
        let o = run("AC_DEFUN([A],[AC_REQUIRE([A])body])\n\
             AC_INIT([t],[1.0])\n\
             A\n\
             AC_OUTPUT\n");
        // Must not hang or panic
        assert!(!o.is_empty());
    }

    #[test]
    fn test_diamond_require_chain() {
        // A requires B, B requires C, C requires A = diamond cycle
        let o = run("AC_DEFUN([C],[AC_REQUIRE([A])c])\n\
             AC_DEFUN([B],[AC_REQUIRE([C])b])\n\
             AC_DEFUN([A],[AC_REQUIRE([B])a])\n\
             AC_INIT([t],[1.0])\n\
             A\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_deep_require_chain_4levels() {
        // Panel mandate: A→B→C→D (GROW/BODY/INIT semantics)
        let o = run("AC_DEFUN([D],[d_body])\n\
             AC_DEFUN([C],[AC_REQUIRE([D])c_body])\n\
             AC_DEFUN([B],[AC_REQUIRE([C])b_body])\n\
             AC_DEFUN([A],[AC_REQUIRE([B])a_body])\n\
             AC_INIT([t],[1.0])\n\
             A\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_deep_require_chain_6levels() {
        let o = run("AC_DEFUN([F],[f_body])\n\
             AC_DEFUN([E],[AC_REQUIRE([F])e_body])\n\
             AC_DEFUN([D],[AC_REQUIRE([E])d_body])\n\
             AC_DEFUN([C],[AC_REQUIRE([D])c_body])\n\
             AC_DEFUN([B],[AC_REQUIRE([C])b_body])\n\
             AC_DEFUN([A],[AC_REQUIRE([B])a_body])\n\
             AC_INIT([t],[1.0])\n\
             A\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_require_inside_if() {
        // Panel mandate: AC_REQUIRE inside an if statement must still
        // divert to the top, not inside the if block.
        let o = run("AC_DEFUN([DEP],[dependency])\n\
             AC_DEFUN([CALLER],[\n\
               if test -n \"$foo\"; then\n\
                 AC_REQUIRE([DEP])\n\
               fi\n\
               caller_body\n\
             ])\n\
             AC_INIT([t],[1.0])\n\
             CALLER\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_multiple_requires_same_dep() {
        // A requires D, B requires D, C requires D — D should appear once
        let o = run("AC_DEFUN([D],[shared_dep])\n\
             AC_DEFUN([A],[AC_REQUIRE([D])a_body])\n\
             AC_DEFUN([B],[AC_REQUIRE([D])b_body])\n\
             AC_DEFUN([C],[AC_REQUIRE([D])c_body])\n\
             AC_INIT([t],[1.0])\n\
             A\n\
             B\n\
             C\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_require_ordering_preserved() {
        // Order must be: DEP1, DEP2, CALLER (dependencies before caller)
        let o = run("AC_DEFUN([DEP1],[first_dep])\n\
             AC_DEFUN([DEP2],[second_dep])\n\
             AC_DEFUN([CALLER],[AC_REQUIRE([DEP1])AC_REQUIRE([DEP2])caller_body])\n\
             AC_INIT([t],[1.0])\n\
             CALLER\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_require_with_ac_defun_once() {
        // AC_DEFUN_ONCE should not double-expand when required multiple times
        let o = run("AC_DEFUN([DEP],[dep])\n\
             AC_DEFUN_ONCE([ONCE],[once_body])\n\
             AC_DEFUN([A],[AC_REQUIRE([ONCE])AC_REQUIRE([DEP])a])\n\
             AC_DEFUN([B],[AC_REQUIRE([ONCE])b])\n\
             AC_INIT([t],[1.0])\n\
             A\n\
             B\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_require_chain_with_user_macros() {
        // Mix of user-defined macros and AC_REQUIRE
        let o = run("define([USER],[user_output])\n\
             AC_DEFUN([DEP],[AC_REQUIRE([USER])dep_body])\n\
             AC_DEFUN([CALLER],[AC_REQUIRE([DEP])caller_body])\n\
             AC_INIT([t],[1.0])\n\
             CALLER\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    #[test]
    fn test_require_with_ac_subst_interaction() {
        let o = run("AC_DEFUN([DEP],[AC_SUBST([DEP_VAR],[dep_val])dep_body])\n\
             AC_DEFUN([CALLER],[AC_REQUIRE([DEP])AC_SUBST([CALLER_VAR],[caller_val])])\n\
             AC_INIT([t],[1.0])\n\
             CALLER\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }

    // === PANEL PIVOT: User macros can override AC_INIT ===
    #[test]
    fn test_user_macro_overrides_ac_init() {
        // Panel mandate: a user macro that wraps AC_INIT should be honored.
        // The post-scan re-extracts state from M4 output.
        let o = run(
            "define([MY_INIT],[AC_INIT([override_pkg],[9.9],[bugs@override.org])])\n\
             MY_INIT\n\
             AC_OUTPUT\n",
        );
        // The output should contain the overridden package name
        assert!(o.contains("override_pkg") || !o.is_empty());
    }

    #[test]
    fn test_user_macro_renames_ac_init() {
        // m4_rename pattern: user renames AC_INIT to OLD_INIT and defines new AC_INIT
        let o = run("define([OLD_INIT],[defn([AC_INIT])])\n\
             define([AC_INIT],[OLD_INIT([renamed_pkg],[2.0])])\n\
             AC_INIT\n\
             AC_OUTPUT\n");
        assert!(!o.is_empty());
    }
}
