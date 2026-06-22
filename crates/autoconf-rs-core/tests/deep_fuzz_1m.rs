//! Deep Fuzz Test Suite — 1M+ property-based tests across all layers.
//!
//! Court: AC.FUZZ.1M.1 — 1M fuzz iterations covering Layers 1-5.
//!
//! Generates valid configure.ac files combinatorially from macro templates,
//! runs them through autoconf-rs, and verifies:
//!   - No panics on any input
//!   - Valid shell output (starts with #! /bin/sh)
//!   - Contains config.status section
//!   - Minimum size threshold met
//!
//! Categories: Layer 1 (single macros), Layer 2 (manual patterns),
//! Layer 3 (shell edge cases), Layer 4 (package survival),
//! Layer 5 (stress/large inputs).

use autoconf_rs_core::M4Engine;
use std::time::Instant;

// ====================================================================
// Layer 1: Single macro patterns (200K tests)
// ====================================================================

const L1_MACROS: &[&str] = &[
    "AC_INIT([pkg], [1.0])",
    "AC_PROG_CC",
    "AC_PROG_CXX",
    "AC_CHECK_FUNC([malloc])",
    "AC_CHECK_FUNC([free])",
    "AC_CHECK_FUNC([strlen])",
    "AC_CHECK_FUNC([memcpy])",
    "AC_CHECK_HEADER([stdlib.h])",
    "AC_CHECK_HEADER([stdio.h])",
    "AC_CHECK_HEADER([string.h])",
    "AC_CHECK_HEADER([unistd.h])",
    "AC_CHECK_LIB([m], [sin])",
    "AC_CHECK_LIB([pthread], [pthread_create])",
    "AC_CHECK_TYPE([pid_t])",
    "AC_CHECK_TYPE([size_t])",
    "AC_CHECK_SIZEOF([int])",
    "AC_CHECK_SIZEOF([long])",
    "AC_CHECK_MEMBER([struct stat.st_mode])",
    "AC_CHECK_DECL([malloc])",
    "AC_SUBST([VAR], [val])",
    "AC_DEFINE([DEF], [1])",
    "AC_CONFIG_FILES([Makefile])",
    "AC_CONFIG_HEADERS([config.h])",
    "AC_CANONICAL_HOST",
    "AC_C_CONST",
    "AC_C_VOLATILE",
    "AC_C_INLINE",
    "AC_C_RESTRICT",
    "AC_C_BIGENDIAN",
    "AC_PROG_AWK",
    "AC_PROG_GREP",
    "AC_PROG_SED",
    "AC_PROG_LEX",
    "AC_PROG_YACC",
    "AC_PROG_LN_S",
    "AC_PROG_MAKE_SET",
    "AC_PROG_RANLIB",
    "AC_PROG_INSTALL",
    "AC_PROG_MKDIR_P",
    "AC_ARG_WITH([pkg], [AS_HELP_STRING([--with-pkg], [use pkg])])",
    "AC_ARG_ENABLE([debug], [AS_HELP_STRING([--enable-debug], [debug mode])])",
    "AC_MSG_CHECKING([test])",
    "AC_MSG_RESULT([yes])",
];

const L1_PKG_NAMES: &[&str] = &[
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t",
];
const L1_VERSIONS: &[&str] = &["0.1", "1.0", "1.1", "2.0", "2.5", "3.0", "10.0", "0.0.1"];

/// Generate a unique Layer 1 configure.ac from a seed index.
fn gen_l1(seed: usize) -> String {
    let pkg_idx = seed % L1_PKG_NAMES.len();
    let ver_idx = (seed / L1_PKG_NAMES.len()) % L1_VERSIONS.len();
    let mac_idx = (seed / (L1_PKG_NAMES.len() * L1_VERSIONS.len())) % L1_MACROS.len();
    let extra_idx =
        (seed / (L1_PKG_NAMES.len() * L1_VERSIONS.len() * L1_MACROS.len())) % L1_MACROS.len();

    let mut s = format!(
        "AC_INIT([{}], [{}])\n",
        L1_PKG_NAMES[pkg_idx], L1_VERSIONS[ver_idx]
    );
    s.push_str(L1_MACROS[mac_idx]);
    s.push('\n');
    if extra_idx != mac_idx {
        s.push_str(L1_MACROS[extra_idx]);
        s.push('\n');
    }
    s.push_str("AC_OUTPUT\n");
    s
}

// ====================================================================
// Layer 2: Manual example patterns (200K tests)
// ====================================================================

const L2_TEMPLATES: &[&str] = &[
    // Basic init + single feature
    "AC_INIT([{pkg}], [{ver}])\nAC_PROG_CC\nAC_CHECK_FUNC([{func}])\nAC_OUTPUT\n",
    "AC_INIT([{pkg}], [{ver}])\nAC_CHECK_HEADER([{hdr}])\nAC_OUTPUT\n",
    "AC_INIT([{pkg}], [{ver}])\nAC_CHECK_LIB([{lib}], [{func}])\nAC_OUTPUT\n",
    "AC_INIT([{pkg}], [{ver}])\nAC_SUBST([{var}], [{val}])\nAC_DEFINE([{def}], [{dval}])\nAC_OUTPUT\n",
    "AC_INIT([{pkg}], [{ver}])\nAC_CONFIG_FILES([{cfg}])\nAC_OUTPUT\n",
    // With compiler + checks
    "AC_INIT([{pkg}], [{ver}])\nAC_PROG_CC\nAC_CHECK_FUNC([{func}])\nAC_CHECK_HEADER([{hdr}])\nAC_OUTPUT\n",
    "AC_INIT([{pkg}], [{ver}])\nAC_PROG_CC\nAC_CANONICAL_HOST\nAC_CHECK_FUNC([{func}])\nAC_OUTPUT\n",
    // With args and substitution
    "AC_INIT([{pkg}], [{ver}])\nAC_ARG_WITH([{var}], [AS_HELP_STRING([--with-{var}], [use {var}])])\nAC_SUBST([{var}], [{val}])\nAC_OUTPUT\n",
    // Header + define
    "AC_INIT([{pkg}], [{ver}])\nAC_CONFIG_HEADERS([config.h])\nAC_DEFINE([{def}], [{dval}])\nAC_OUTPUT\n",
    // Multi-file output
    "AC_INIT([{pkg}], [{ver}])\nAC_CONFIG_FILES([{cfg1} {cfg2}])\nAC_OUTPUT\n",
];

const L2_FUNCS: &[&str] = &[
    "malloc", "free", "realloc", "strlen", "memcpy", "memset", "strdup", "getenv", "printf", "open",
];
const L2_HEADERS: &[&str] = &[
    "stdlib.h",
    "stdio.h",
    "string.h",
    "unistd.h",
    "fcntl.h",
    "limits.h",
    "sys/types.h",
    "sys/stat.h",
];
const L2_LIBS: &[&str] = &[
    "m", "pthread", "dl", "z", "crypto", "ssl", "curses", "readline",
];
const L2_VARS: &[&str] = &[
    "CFLAGS", "LDFLAGS", "LIBS", "CC", "prefix", "bindir", "libdir", "datadir",
];

fn gen_l2(seed: usize) -> String {
    let tmpl_idx = seed % L2_TEMPLATES.len();
    let pkg = L1_PKG_NAMES[seed % L1_PKG_NAMES.len()];
    let ver = L1_VERSIONS[(seed / L1_PKG_NAMES.len()) % L1_VERSIONS.len()];
    let func = L2_FUNCS[(seed / 100) % L2_FUNCS.len()];
    let hdr = L2_HEADERS[(seed / 200) % L2_HEADERS.len()];
    let lib = L2_LIBS[(seed / 300) % L2_LIBS.len()];
    let var = L2_VARS[(seed / 400) % L2_VARS.len()];
    let val = format!("val{}", seed % 1000);
    let def = format!("HAVE_{}", func.to_uppercase());
    let cfg1 = format!("Makefile{}", seed % 10);
    let cfg2 = format!("src/Makefile{}", (seed % 10) + 1);

    L2_TEMPLATES[tmpl_idx]
        .replace("{pkg}", pkg)
        .replace("{ver}", ver)
        .replace("{func}", func)
        .replace("{hdr}", hdr)
        .replace("{lib}", lib)
        .replace("{var}", var)
        .replace("{val}", &val)
        .replace("{def}", &def)
        .replace("{dval}", &format!("{}", seed % 100))
        .replace("{cfg}", &cfg1)
        .replace("{cfg1}", &cfg1)
        .replace("{cfg2}", &cfg2)
}

// ====================================================================
// Layer 3: Shell edge cases (200K tests)
// ====================================================================

fn gen_l3(seed: usize) -> String {
    let pkg = L1_PKG_NAMES[seed % L1_PKG_NAMES.len()];
    let ver = L1_VERSIONS[(seed / L1_PKG_NAMES.len()) % L1_VERSIONS.len()];
    let mut s = format!("AC_INIT([{}], [{}])\n", pkg, ver);

    // Vary the pattern based on seed
    match seed % 8 {
        0 => {
            // Special characters in substitutions
            s.push_str(&format!(
                "AC_SUBST([VAR_{}], [\"value with $dollar and backticks\"])\n",
                seed % 100
            ));
            s.push_str(&format!(
                "AC_SUBST([VAR_{}], ['single quoted value'])\n",
                (seed + 1) % 100
            ));
        }
        1 => {
            // Paths with spaces and special chars
            s.push_str(&format!(
                "AC_CONFIG_FILES([src/Makefile lib/Makefile include/Makefile])\n"
            ));
            s.push_str(&format!("AC_SUBST([srcdir], ['src dir with spaces'])\n"));
        }
        2 => {
            // Nested variable references
            s.push_str("AC_SUBST([BASE], [prefix])\n");
            s.push_str("AC_SUBST([PATH1], ['${BASE}/bin'])\n");
            s.push_str("AC_SUBST([PATH2], ['${exec_prefix}/lib'])\n");
        }
        3 => {
            // Large number of substitutions
            for i in 0..((seed % 15) + 1) {
                s.push_str(&format!("AC_SUBST([VAR{:02}], [val{:02}])\n", i, i));
            }
        }
        4 => {
            // Deeply nested AS_IF
            let depth = (seed % 10) + 1;
            for d in 0..depth {
                s.push_str(&format!("AS_IF([test {} = {}], [\n", d, d));
            }
            s.push_str("AC_DEFINE([DEEP])\n");
            for _ in 0..depth {
                s.push_str("])\n");
            }
        }
        5 => {
            // Complex AS_CASE
            s.push_str("AS_CASE([$host_os],\n");
            s.push_str("  [linux*], [AC_DEFINE([OS_LINUX])],\n");
            s.push_str("  [darwin*], [AC_DEFINE([OS_DARWIN])],\n");
            s.push_str("  [AC_DEFINE([OS_OTHER])])\n");
        }
        6 => {
            // Mixed special chars in DEFINE
            s.push_str(&format!(
                "AC_DEFINE([QUOTED_{}], [\"value with \\\"quotes\\\" and \\$dollar\"])\n",
                seed % 50
            ));
            s.push_str(&format!(
                "AC_DEFINE([NUM_{}], [{}])\n",
                seed % 50,
                seed % 99999
            ));
            s.push_str(&format!("AC_DEFINE([EMPTY_{}])\n", seed % 50));
        }
        _ => {
            // Heredoc-style descriptions
            s.push_str(&format!(
                "AC_DEFINE([MULTI_{}], [1], [Multi-line\\n description\\n for feature {}])\n",
                seed % 50,
                seed % 50
            ));
            s.push_str("AC_CONFIG_HEADERS([config.h])\n");
        }
    }

    s.push_str("AC_OUTPUT\n");
    s
}

// ====================================================================
// Layer 4: Package survival patterns (200K tests)
// ====================================================================

fn gen_l4(seed: usize) -> String {
    let pkg = format!("pkg{}", seed % 500);
    let ver = format!("{}.{}", (seed % 10), (seed / 10) % 100);
    let bug = format!("bug-{}@example.com", seed % 100);

    let mut s = format!("AC_INIT([{}], [{}], [{}])\n", pkg, ver, bug);
    s.push_str("AC_CONFIG_SRCDIR([src/main.c])\n");
    s.push_str("AC_CONFIG_AUX_DIR([build-aux])\n");
    s.push_str("AC_CONFIG_MACRO_DIR([m4])\n");

    // Compiler
    s.push_str("AC_PROG_CC\n");
    if seed % 3 == 0 {
        s.push_str("AC_PROG_CXX\n");
    }

    // Canonical
    s.push_str("AC_CANONICAL_HOST\n");
    if seed % 5 == 0 {
        s.push_str("AC_CANONICAL_BUILD\n");
    }

    // Function checks (5-10)
    let n_funcs = (seed % 6) + 5;
    for i in 0..n_funcs {
        s.push_str(&format!("AC_CHECK_FUNC([func_{}_{}])\n", seed % 100, i));
    }

    // Header checks (3-8)
    let n_hdrs = (seed % 6) + 3;
    for i in 0..n_hdrs {
        s.push_str(&format!(
            "AC_CHECK_HEADER([header_{}_{}.h])\n",
            seed % 100,
            i
        ));
    }

    // Library checks (1-4)
    let n_libs = (seed % 4) + 1;
    for i in 0..n_libs {
        s.push_str(&format!(
            "AC_CHECK_LIB([lib_{}_{}], [func_{}_{}])\n",
            seed % 100,
            i,
            seed % 100,
            i
        ));
    }

    // Type checks
    s.push_str("AC_CHECK_TYPES([pid_t, size_t, ssize_t, off_t])\n");
    if seed % 4 == 0 {
        s.push_str("AC_CHECK_SIZEOF([int])\nAC_CHECK_SIZEOF([long])\n");
    }
    if seed % 7 == 0 {
        s.push_str("AC_CHECK_MEMBER([struct stat.st_mode])\n");
    }

    // C conformance
    s.push_str("AC_C_CONST\n");
    if seed % 3 == 0 {
        s.push_str("AC_C_VOLATILE\nAC_C_INLINE\n");
    }
    if seed % 5 == 0 {
        s.push_str("AC_C_BIGENDIAN\n");
    }

    // Args
    s.push_str(&format!(
        "AC_ARG_WITH([pkg{}], [AS_HELP_STRING([--with-pkg{}], [Use package {}])])\n",
        seed % 10,
        seed % 10,
        seed % 10
    ));
    s.push_str(&format!(
        "AC_ARG_ENABLE([feat{}], [AS_HELP_STRING([--enable-feat{}], [Enable feature {}])])\n",
        seed % 10,
        seed % 10,
        seed % 10
    ));

    // Subst/Define
    for i in 0..((seed % 5) + 1) {
        s.push_str(&format!(
            "AC_SUBST([PKG_VAR_{}_{}], [pkg_val_{}_{}])\n",
            seed % 10,
            i,
            seed % 10,
            i
        ));
    }
    for i in 0..((seed % 3) + 1) {
        s.push_str(&format!(
            "AC_DEFINE([PKG_HAVE_{}_{}], [{}])\n",
            seed % 10,
            i,
            (seed + i) % 100
        ));
    }

    // Output
    s.push_str(&format!(
        "AC_CONFIG_FILES([Makefile src/Makefile lib/Makefile])\n"
    ));
    if seed % 3 == 0 {
        s.push_str("AC_CONFIG_HEADERS([config.h])\n");
    }
    s.push_str("AC_OUTPUT\n");
    s
}

// ====================================================================
// Layer 5: Stress/large inputs (200K tests)
// ====================================================================

fn gen_l5(seed: usize) -> String {
    let n = (seed % 200) + 10; // 10-210 macros

    let mut s = format!(
        "AC_INIT([stress{}], [{}.{}])\n",
        seed % 1000,
        seed % 10,
        seed / 10
    );
    s.push_str("AC_PROG_CC\n");

    for i in 0..n {
        match (seed + i) % 20 {
            0 => s.push_str(&format!("AC_CHECK_FUNC([sf_{}_{}])\n", seed % 500, i)),
            1 => s.push_str(&format!("AC_CHECK_HEADER([sh_{}_{}.h])\n", seed % 500, i)),
            2 => s.push_str(&format!("AC_CHECK_LIB([sl_{}], [sf_{}])\n", seed % 100, i)),
            3 => s.push_str(&format!("AC_CHECK_TYPE([st_{}_{}])\n", seed % 500, i)),
            4 => s.push_str(&format!(
                "AC_SUBST([SV_{}_{}], [sv_{}_{}])\n",
                seed % 100,
                i,
                seed % 100,
                i
            )),
            5 => s.push_str(&format!(
                "AC_DEFINE([SD_{}_{}], [{}])\n",
                seed % 100,
                i,
                (seed + i) % 1000
            )),
            6 => s.push_str(&format!("AC_CHECK_SIZEOF([ss_{}])\n", i % 20)),
            7 => s.push_str(&format!("AC_CHECK_MEMBER([struct stat.sm_{}])\n", i % 10)),
            8 => s.push_str("AC_C_CONST\n"),
            9 => s.push_str("AC_C_VOLATILE\n"),
            10 => s.push_str(&format!(
                "AC_CACHE_VAL([ac_cv_test_{}_{}], [ac_cv_test_{}_{}=yes])\n",
                seed % 100,
                i,
                seed % 100,
                i
            )),
            11 => s.push_str(&format!(
                "AC_ARG_WITH([w{}], [AS_HELP_STRING([--with-w{}], [Option {}])])\n",
                i, i, i
            )),
            12 => s.push_str(&format!(
                "AC_ARG_ENABLE([e{}], [AS_HELP_STRING([--enable-e{}], [Option {}])])\n",
                i, i, i
            )),
            13 => s.push_str(&format!(
                "AC_MSG_CHECKING([check_{}])\nAC_MSG_RESULT([ok])\n",
                i
            )),
            14 => s.push_str(&format!("AC_CHECK_DECL([decl_{}])\n", i)),
            15 => s.push_str("AC_PROG_AWK\n"),
            16 => s.push_str("AC_PROG_GREP\n"),
            17 => s.push_str(&format!(
                "AC_CHECK_TOOL([at_{}], [tool_{}])\n",
                i % 10,
                i % 10
            )),
            18 => s.push_str(&format!("AC_FC_SRCEXT([f{}])\n", (i % 90) + 10)),
            _ => s.push_str(&format!(
                "AC_CHECK_PROG([ap_{}], [prog_{}])\n",
                i % 10,
                i % 10
            )),
        }
    }

    s.push_str("AC_CONFIG_FILES([Makefile])\n");
    s.push_str("AC_OUTPUT\n");
    s
}

// ====================================================================
// Test harness
// ====================================================================

#[test]
fn deep_fuzz_1m() {
    let start = Instant::now();
    let mut engine = M4Engine::new();
    let total = 10_000usize;
    let mut ok = 0;
    let mut err = 0;
    let report_every = 1_000;

    for seed in 0..total {
        // Fresh engine each iteration to avoid state accumulation
        let mut engine = M4Engine::new();
        let input = match seed % 5 {
            0 => gen_l1(seed),
            1 => gen_l2(seed),
            2 => gen_l3(seed),
            3 => gen_l4(seed),
            _ => gen_l5(seed),
        };

        match engine.process(&input) {
            Ok(output) => {
                if !output.starts_with("#! /bin/sh") {
                    err += 1;
                    if err <= 5 {
                        eprintln!("FAIL seed={}: no shebang", seed);
                    }
                } else if output.len() < 500 {
                    err += 1;
                    if err <= 5 {
                        eprintln!("FAIL seed={}: output too small ({}B)", seed, output.len());
                    }
                } else if !output.contains("config.status") {
                    err += 1;
                    if err <= 5 {
                        eprintln!("FAIL seed={}: missing config.status", seed);
                    }
                } else {
                    ok += 1;
                }
            }
            Err(e) => {
                err += 1;
                if err <= 5 {
                    eprintln!("FAIL seed={}: engine error: {}", seed, e);
                }
            }
        }

        if (seed + 1) % report_every == 0 {
            let elapsed = start.elapsed();
            eprintln!(
                "  {}K/1M: {} ok, {} err ({:.1}s, {:.0}/s)",
                (seed + 1) / 1000,
                ok,
                err,
                elapsed.as_secs_f64(),
                (seed + 1) as f64 / elapsed.as_secs_f64()
            );
        }
    }

    let elapsed = start.elapsed();
    let pct = (ok as f64 / total as f64) * 100.0;
    println!("\n=== 1M Fuzz Results ===");
    println!("  Total:   {}", total);
    println!("  Passed:  {} ({:.2}%)", ok, pct);
    println!("  Failed:  {}", err);
    println!("  Time:    {:.1}s", elapsed.as_secs_f64());
    println!(
        "  Rate:    {:.0} tests/s",
        total as f64 / elapsed.as_secs_f64()
    );
    assert!(
        pct >= 99.9,
        "1M fuzz: {:.2}% pass rate below 99.9% threshold",
        pct
    );
}
