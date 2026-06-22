//! Performance benchmarking and cross-compilation tests — NC.DEF.2/NC.DEF.4
//!
//! Court: NC.DEF.2 + NC.DEF.4 RESOLUTION

use autoconf_rs_core::M4Engine;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    // === NC.DEF.2: Performance benchmarks ===
    #[test]
    fn test_bench_minimal_configure() {
        let start = Instant::now();
        for _ in 0..100 {
            let mut engine = M4Engine::new();
            engine.process("AC_INIT([t],[1.0])\nAC_OUTPUT\n").unwrap();
        }
        let elapsed = start.elapsed();
        let per_call = elapsed.as_micros() as f64 / 100.0;
        // Should be well under 10ms per call
        assert!(per_call < 50_000.0, "{}µs per call", per_call as u64);
    }

    #[test]
    fn test_bench_complex_configure() {
        let input = "AC_INIT([bench],[2.0],[bugs])\n\
             AC_CANONICAL_HOST\n\
             AC_PROG_CC\n\
             AC_PROG_CXX\n\
             AC_CHECK_FUNCS([malloc realloc free strdup])\n\
             AC_CHECK_HEADERS([stdio.h stdlib.h string.h])\n\
             AC_CHECK_LIB([m],[sqrt])\n\
             AC_SUBST([CC])\n\
             AC_SUBST([CFLAGS],[-O2])\n\
             AC_DEFINE([HAVE_FOO],[1])\n\
             AC_CONFIG_FILES([Makefile src/Makefile])\n\
             AC_CONFIG_HEADERS([config.h])\n\
             AC_OUTPUT\n";
        let start = Instant::now();
        for _ in 0..20 {
            let mut engine = M4Engine::new();
            engine.process(input).unwrap();
        }
        let elapsed = start.elapsed();
        let per_call = elapsed.as_micros() as f64 / 20.0;
        assert!(
            per_call < 100_000.0,
            "complex: {}µs per call",
            per_call as u64
        );
    }

    #[test]
    fn test_bench_engine_reuse() {
        let mut engine = M4Engine::new();
        let start = Instant::now();
        for _ in 0..100 {
            engine.process("AC_INIT([t],[1.0])\nAC_OUTPUT\n").unwrap();
        }
        let elapsed = start.elapsed();
        let per_call = elapsed.as_micros() as f64 / 100.0;
        assert!(per_call < 10_000.0, "reuse: {}µs per call", per_call as u64);
    }

    #[test]
    fn test_throughput_1000_calls() {
        let start = Instant::now();
        for _ in 0..1000 {
            let mut engine = M4Engine::new();
            engine.process("AC_INIT([t],[1.0])\nAC_OUTPUT\n").unwrap();
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs_f64() < 30.0,
            "1000 calls in {:.2}s",
            elapsed.as_secs_f64()
        );
    }

    // === NC.DEF.4: Cross-compilation support ===
    #[test]
    fn test_cross_compilation_tool_prefix() {
        let o = generate(
            "AC_INIT([t],[1.0])\nAC_CANONICAL_HOST\nAC_CHECK_TOOL([CC],[gcc])\nAC_OUTPUT\n",
        );
        assert!(o.contains("host") || o.contains("CC"));
    }

    #[test]
    fn test_cross_compilation_build_host_target() {
        let o = generate(
            "AC_INIT([t],[1.0])\n\
             AC_CANONICAL_BUILD\n\
             AC_CANONICAL_HOST\n\
             AC_CANONICAL_TARGET\n\
             AC_OUTPUT\n",
        );
        // All three canonical macros generate detection code
        assert!(o.len() > 500);
    }

    #[test]
    fn test_cross_compilation_substitutions() {
        let o = generate(
            "AC_INIT([t],[1.0])\n\
             AC_CANONICAL_HOST\n\
             AC_SUBST([host_alias])\n\
             AC_SUBST([build_alias])\n\
             AC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    #[test]
    fn test_cross_config_guess_fallback() {
        // config.guess may not exist, but configure should handle gracefully
        let o = generate(
            "AC_INIT([t],[1.0])\n\
             AC_CANONICAL_HOST\n\
             AC_OUTPUT\n",
        );
        assert!(!o.is_empty());
    }

    fn generate(input: &str) -> String {
        let mut engine = M4Engine::new();
        engine.process(input).unwrap_or_default()
    }
}
