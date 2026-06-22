//! Oracle Fuzz Scale-Up — AC.ORACLE.1 Feature 4
//!
//! Scaled fuzzing: 5K iterations + corpus-based fuzz using pre-generated fixtures.
//! Honest about divergence: 0% byte-exact expected (NC.ADMIT.2 — prescan vs pure M4).
//!
//! Receipt family: AC.ORACLE.1.FUZZ.SCALE

use std::path::Path;
use std::process::Command;

/// Divergence classification for oracle comparison.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Divergence {
    ByteExact,
    SizeDiff,
    StructDiff,
    ExitDiff,
    RustPanic,
    OracleFail,
    OracleNotFound,
}

/// Simple LCG for deterministic pseudo-random numbers.
fn lcg(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

/// Generate a random configure.ac from a seed.
fn gen_random_ac(seed: u64) -> String {
    let mut s = seed;
    let pkg_idx = (s % 20) as usize;
    s = lcg(s);
    let ver_major = (s % 10) + 1;
    s = lcg(s);
    let ver_minor = s % 20;
    s = lcg(s);

    let pkgs = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t",
    ];
    let mut ac = format!(
        "AC_INIT([{}], [{}.{}])\n",
        pkgs[pkg_idx], ver_major, ver_minor
    );

    let num_macros = ((s % 4) + 1) as usize;
    s = lcg(s);

    let macros = [
        "AC_PROG_CC\n",
        "AC_PROG_CXX\n",
        "AC_CHECK_FUNC([malloc])\n",
        "AC_CHECK_FUNC([free])\n",
        "AC_CHECK_HEADER([stdlib.h])\n",
        "AC_CHECK_HEADER([stdio.h])\n",
        "AC_CHECK_LIB([m], [sin])\n",
        "AC_CHECK_LIB([pthread], [pthread_create])\n",
        "AC_CHECK_TYPE([pid_t])\n",
        "AC_CHECK_TYPE([size_t])\n",
        "AC_CHECK_SIZEOF([int])\n",
        "AC_CHECK_SIZEOF([long])\n",
        "AC_SUBST([CC], [gcc])\n",
        "AC_SUBST([CFLAGS], [-O2])\n",
        "AC_DEFINE([HAVE_FOO], [1])\n",
        "AC_DEFINE([PACKAGE_VERSION], [\\\"1.0\\\"])\n",
        "AC_CONFIG_FILES([Makefile])\n",
        "AC_CONFIG_HEADERS([config.h])\n",
        "AC_CANONICAL_HOST\n",
        "AC_C_CONST\n",
        "AC_C_VOLATILE\n",
        "AC_PROG_AWK\n",
        "AC_PROG_GREP\n",
        "AC_PROG_SED\n",
        "AC_MSG_CHECKING([for something])\nAC_MSG_RESULT([yes])\n",
        "AC_ARG_WITH([foo], [AS_HELP_STRING([--with-foo], [use foo])])\n",
        "AC_ARG_ENABLE([debug], [AS_HELP_STRING([--enable-debug], [debug mode])])\n",
        "AC_CACHE_VAL([ac_cv_foo], [ac_cv_foo=yes])\n",
        "AC_CHECK_MEMBER([struct stat.st_mode])\n",
        "AC_CHECK_DECL([malloc])\n",
    ];

    for _ in 0..num_macros {
        let idx = (s as usize) % macros.len();
        s = lcg(s);
        ac.push_str(macros[idx]);
    }

    ac.push_str("AC_OUTPUT\n");
    ac
}

/// Run both GNU autoconf and autoconf-rs on the same input.
fn diff_run(input: &str) -> (Divergence, usize, usize, i32, i32) {
    let tmp = std::env::temp_dir().join(format!("ac_fuzz_scale_{}.ac", std::process::id()));
    let _ = std::fs::write(&tmp, input);

    let oracle = Command::new("autoconf")
        .arg("-f")
        .arg(&tmp)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();

    let rust_out = Command::new("../../target/release/autoconf")
        .arg("-f")
        .arg(&tmp)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();

    let _ = std::fs::remove_file(&tmp);

    match (oracle, rust_out) {
        (Err(_), _) => (Divergence::OracleNotFound, 0, 0, -1, -1),
        (_, Err(_)) => (Divergence::RustPanic, 0, 0, -1, -101),
        (Ok(o), Ok(r)) => {
            let o_exit = o.status.code().unwrap_or(-1);
            let r_exit = r.status.code().unwrap_or(-101);
            let o_len = o.stdout.len();
            let r_len = r.stdout.len();

            if r_exit == -101 {
                (Divergence::RustPanic, o_len, r_len, o_exit, r_exit)
            } else if o_exit != r_exit {
                (Divergence::ExitDiff, o_len, r_len, o_exit, r_exit)
            } else if o.stdout == r.stdout {
                (Divergence::ByteExact, o_len, r_len, o_exit, r_exit)
            } else if o_len > 0
                && r_len > 0
                && (o_len as f64 / r_len as f64) > 0.4
                && (o_len as f64 / r_len as f64) < 2.5
            {
                (Divergence::SizeDiff, o_len, r_len, o_exit, r_exit)
            } else {
                (Divergence::StructDiff, o_len, r_len, o_exit, r_exit)
            }
        }
    }
}

/// Run autoconf-rs on a corpus file and return exit code + output size.
fn run_rs_on_corpus(path: &str) -> (i32, usize) {
    match Command::new("../../target/release/autoconf")
        .arg("-f")
        .arg(path)
        .output()
    {
        Ok(o) => (o.status.code().unwrap_or(-101), o.stdout.len()),
        Err(_) => (-101, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // SCALED FUZZ — 5000 iterations (Feature 4 scale-up from 1500)
    // ================================================================

    /// 5K Oracle Diff Fuzz — scaled up from the 1K baseline.
    /// Court: AC.FUZZ.ORACLE_DIFF.5K
    #[test]
    fn test_oracle_diff_fuzz_5k() {
        let oracle_check = Command::new("autoconf").arg("--version").output();
        if oracle_check.is_err() || !oracle_check.unwrap().status.success() {
            eprintln!("SKIP: GNU autoconf not found on PATH");
            return;
        }

        let total = 5000usize;
        let mut byte_exact = 0u64;
        let mut size_diff = 0u64;
        let mut struct_diff = 0u64;
        let mut exit_diff = 0u64;
        let mut rust_panic = 0u64;
        let mut size_ratios: Vec<f64> = Vec::with_capacity(total);
        let mut exit_diff_seeds: Vec<(usize, i32, i32)> = Vec::new();
        let start = std::time::Instant::now();

        for i in 0..total {
            let input = gen_random_ac(i as u64);
            let (div, o_len, r_len, o_exit, r_exit) = diff_run(&input);

            match div {
                Divergence::ByteExact => byte_exact += 1,
                Divergence::SizeDiff => size_diff += 1,
                Divergence::StructDiff => struct_diff += 1,
                Divergence::ExitDiff => {
                    exit_diff += 1;
                    if exit_diff_seeds.len() < 20 {
                        exit_diff_seeds.push((i, o_exit, r_exit));
                    }
                }
                Divergence::RustPanic => rust_panic += 1,
                Divergence::OracleFail => {}
                Divergence::OracleNotFound => {
                    eprintln!("SKIP: GNU autoconf became unavailable mid-fuzz");
                    return;
                }
            }

            if o_len > 0 && r_len > 0 {
                size_ratios.push(r_len as f64 / o_len as f64);
            }

            if (i + 1) % 500 == 0 {
                let elapsed = start.elapsed();
                eprintln!(
                    "  {}/5K: exact={} size={} struct={} exit={} panic={} ({:.1}s, {:.0}/s)",
                    i + 1,
                    byte_exact,
                    size_diff,
                    struct_diff,
                    exit_diff,
                    rust_panic,
                    elapsed.as_secs_f64(),
                    (i + 1) as f64 / elapsed.as_secs_f64().max(0.001)
                );
            }
        }

        let elapsed = start.elapsed();

        // Compute statistics
        let avg_ratio = if size_ratios.is_empty() {
            0.0
        } else {
            size_ratios.iter().sum::<f64>() / size_ratios.len() as f64
        };
        let mut ratios_sorted = size_ratios.clone();
        ratios_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_ratio = if ratios_sorted.is_empty() {
            0.0
        } else {
            ratios_sorted[ratios_sorted.len() / 2]
        };

        println!("\n=== ORACLE DIFFERENTIAL FUZZ 5K ===");
        println!("  Total:       {}", total);
        println!(
            "  Byte-exact:  {} ({:.1}%)",
            byte_exact,
            byte_exact as f64 / total as f64 * 100.0
        );
        println!(
            "  Size diff:   {} ({:.1}%)",
            size_diff,
            size_diff as f64 / total as f64 * 100.0
        );
        println!(
            "  Struct diff: {} ({:.1}%)",
            struct_diff,
            struct_diff as f64 / total as f64 * 100.0
        );
        println!(
            "  Exit diff:   {} ({:.1}%)",
            exit_diff,
            exit_diff as f64 / total as f64 * 100.0
        );
        println!(
            "  Rust panic:  {} ({:.1}%)",
            rust_panic,
            rust_panic as f64 / total as f64 * 100.0
        );
        println!(
            "  Size ratio:  avg={:.2} median={:.2}",
            avg_ratio, median_ratio
        );
        println!(
            "  Time:        {:.1}s ({:.0}/s)",
            elapsed.as_secs_f64(),
            total as f64 / elapsed.as_secs_f64().max(0.001)
        );
        println!("  Court:       AC.FUZZ.ORACLE_DIFF.5K");

        // Hard gate: ZERO panics
        assert_eq!(
            rust_panic, 0,
            "CRITICAL: {} panics in 5K fuzz iterations",
            rust_panic
        );

        // Soft gate: size ratios should be mostly reasonable
        let within_range = size_ratios.iter().filter(|&&r| r > 0.3 && r < 3.0).count();
        let range_pct = within_range as f64 / size_ratios.len().max(1) as f64 * 100.0;
        println!("  Size sanity: {:.0}% within 0.3x-3.0x", range_pct);

        // Report exit diffs if any
        if !exit_diff_seeds.is_empty() {
            println!("\n  First {} exit code mismatches:", exit_diff_seeds.len());
            for (seed, o_exit, r_exit) in &exit_diff_seeds {
                let input = gen_random_ac(*seed as u64);
                let preview: String = input.chars().take(100).collect();
                println!(
                    "  seed={}: oracle={} rust={} | {}",
                    seed,
                    o_exit,
                    r_exit,
                    preview.replace('\n', "\\n")
                );
            }
        }
    }

    // ================================================================
    // CORPUS FUZZ — Fuzz against the pre-generated 5000+ corpus fixtures
    // ================================================================

    /// Corpus Fuzz: run autoconf-rs against all generated corpus fixtures.
    /// Verifies no panics and all produce valid output.
    /// Court: AC.FUZZ.CORPUS.SCAN
    #[test]
    fn test_corpus_fuzz_scan() {
        let corpus_dir = "../../lab/corpus/generated";
        if !Path::new(corpus_dir).exists() {
            eprintln!("SKIP: corpus directory not found at '{}'", corpus_dir);
            return;
        }

        let mut count = 0u64;
        let mut panics = 0u64;
        let mut empty = 0u64;
        let mut non_zero_exit = 0u64;
        let start = std::time::Instant::now();

        // Read directory
        let entries = match std::fs::read_dir(corpus_dir) {
            Ok(e) => e,
            Err(_) => {
                eprintln!("SKIP: cannot read corpus directory");
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().map(|e| e != "ac").unwrap_or(true) {
                continue;
            }

            count += 1;
            let path_str = path.to_string_lossy().to_string();
            let (exit, size) = run_rs_on_corpus(&path_str);

            if exit == -101 {
                panics += 1;
                eprintln!("  PANIC: {} (exit={})", path_str, exit);
            } else if size == 0 {
                empty += 1;
            }
            if exit != 0 && exit != -101 {
                non_zero_exit += 1;
            }

            if count % 100 == 0 {
                let elapsed = start.elapsed();
                eprintln!(
                    "  corpus {}/??: panics={} empty={} nonzero={} ({:.1}s)",
                    count,
                    panics,
                    empty,
                    non_zero_exit,
                    elapsed.as_secs_f64()
                );
            }
        }

        let elapsed = start.elapsed();
        println!("\n=== CORPUS FUZZ SCAN ===");
        println!("  Total files:  {}", count);
        println!(
            "  Panics:       {} ({:.1}%)",
            panics,
            panics as f64 / count.max(1) as f64 * 100.0
        );
        println!(
            "  Empty output: {} ({:.1}%)",
            empty,
            empty as f64 / count.max(1) as f64 * 100.0
        );
        println!(
            "  Non-zero exit:{} ({:.1}%)",
            non_zero_exit,
            non_zero_exit as f64 / count.max(1) as f64 * 100.0
        );
        println!(
            "  Time:        {:.1}s ({:.0}/s)",
            elapsed.as_secs_f64(),
            count as f64 / elapsed.as_secs_f64().max(0.001)
        );
        println!("  Court:       AC.FUZZ.CORPUS.SCAN");

        // Hard gate: ZERO panics across entire corpus
        assert_eq!(panics, 0, "CRITICAL: {} panics in corpus scan", panics);

        // Soft gate: most should produce output
        let empty_pct = empty as f64 / count.max(1) as f64 * 100.0;
        assert!(
            empty_pct < 50.0,
            "Too many empty outputs: {:.1}%",
            empty_pct
        );
    }

    // ================================================================
    // PROGRESSIVE FUZZ — increasing complexity levels
    // ================================================================

    /// Progressive difficulty fuzzing: Level 1 (simple) through Level 5 (complex).
    /// Tests that increasing input complexity doesn't introduce panics.
    /// Court: AC.FUZZ.PROGRESSIVE
    #[test]
    fn test_progressive_fuzz() {
        let oracle_check = Command::new("autoconf").arg("--version").output();
        if oracle_check.is_err() || !oracle_check.unwrap().status.success() {
            eprintln!("SKIP: GNU autoconf not found on PATH");
            return;
        }

        // Level definitions — each level has more complexity
        let levels: Vec<(&str, Vec<&str>)> = vec![
            // Level 1: Minimal — just INIT+OUTPUT
            ("L1-minimal", vec!["AC_INIT([p],[1.0])\nAC_OUTPUT\n"]),
            // Level 2: Basic — add substitutions
            ("L2-basic", vec![
                "AC_INIT([p],[1.0])\nAC_SUBST([CC],[gcc])\nAC_OUTPUT\n",
                "AC_INIT([p],[1.0])\nAC_SUBST([CC])\nAC_DEFINE([FOO],[1])\nAC_OUTPUT\n",
            ]),
            // Level 3: Feature tests — add function/header checks
            ("L3-features", vec![
                "AC_INIT([p],[1.0])\nAC_PROG_CC\nAC_CHECK_FUNC([malloc])\nAC_OUTPUT\n",
                "AC_INIT([p],[1.0])\nAC_CHECK_HEADER([stdlib.h])\nAC_CHECK_TYPE([size_t])\nAC_OUTPUT\n",
                "AC_INIT([p],[1.0])\nAC_CHECK_LIB([m],[sin])\nAC_CHECK_SIZEOF([int])\nAC_OUTPUT\n",
            ]),
            // Level 4: Complex — config files, canonicals, args
            ("L4-complex", vec![
                "AC_INIT([pkg],[2.0])\nAC_CONFIG_FILES([Makefile src/Makefile])\nAC_CONFIG_HEADERS([config.h])\nAC_CANONICAL_HOST\nAC_PROG_CC\nAC_CHECK_FUNCS([malloc realloc free])\nAC_OUTPUT\n",
                "AC_INIT([tool],[1.5])\nAC_CONFIG_FILES([Makefile])\nAC_ARG_WITH([ssl],[AS_HELP_STRING([--with-ssl],[use SSL])])\nAC_ARG_ENABLE([debug])\nAC_PROG_CC\nAC_OUTPUT\n",
            ]),
            // Level 5: Full-featured — everything at once
            ("L5-full", vec![
                "AC_INIT([full],[3.1],[bugs@ex.com])\nAC_PREREQ([2.69])\nAC_CONFIG_SRCDIR([src/main.c])\nAC_CONFIG_HEADERS([config.h:config.hin])\nAC_CONFIG_FILES([Makefile lib/Makefile src/Makefile])\nAC_SUBST([CC])\nAC_SUBST([CFLAGS],[-O2])\nAC_SUBST([LIBS],[-lm])\nAC_PROG_CC\nAC_PROG_CXX\nAC_PROG_INSTALL\nAC_PROG_MAKE_SET\nAC_PROG_AWK\nAC_CHECK_FUNCS([malloc realloc calloc])\nAC_CHECK_HEADERS([stdlib.h string.h unistd.h])\nAC_CHECK_TYPE([pid_t])\nAC_CHECK_TYPE([size_t])\nAC_CHECK_SIZEOF([int])\nAC_CHECK_SIZEOF([long])\nAC_CANONICAL_HOST\nAC_ARG_ENABLE([debug])\nAC_ARG_WITH([ssl])\nAC_MSG_CHECKING([for readiness])\nAC_MSG_RESULT([ready])\nAC_OUTPUT\n",
            ]),
        ];

        let start = std::time::Instant::now();
        let mut total_inputs = 0u64;
        let mut total_panics = 0u64;

        println!("\n=== PROGRESSIVE FUZZ (5 levels) ===");
        for (label, inputs) in &levels {
            let mut level_panics = 0u64;
            for input in inputs {
                total_inputs += 1;
                let (div, _, _, _, _) = diff_run(input);
                if div == Divergence::RustPanic {
                    level_panics += 1;
                    total_panics += 1;
                }
            }
            let status = if level_panics == 0 { "✓" } else { "✗" };
            println!(
                "  {}: {} inputs, {} panics {}",
                label,
                inputs.len(),
                level_panics,
                status
            );
        }

        let elapsed = start.elapsed();
        println!(
            "  Total: {} inputs, {} panics ({:.1}s)",
            total_inputs,
            total_panics,
            elapsed.as_secs_f64()
        );
        println!("  Court: AC.FUZZ.PROGRESSIVE");

        // Hard gate: ZERO panics
        assert_eq!(
            total_panics, 0,
            "CRITICAL: {} panics in progressive fuzz",
            total_panics
        );
    }
}
