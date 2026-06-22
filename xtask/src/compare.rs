// compare.rs — Multi-oracle parity engine for autoconf-rs.
//
// Generates a large corpus of configure.ac files through combinatorial
// macro pattern expansion, runs them against multiple GNU Autoconf oracle
// versions and autoconf-rs, compares outputs, and produces detailed gap
// reports. No GPL code — all patterns derived from black-box oracle
// interrogation and the GFDL Autoconf manual.
//
// Court: AC.PARITY.CORPUS.1
// Methodology: Doxygen/Pod-filtered Perl surface map → Rust AST compass

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

// --- Corpus Generator ---

/// Known Autoconf macro patterns, organized by category.
/// Each pattern is a syntactically valid configure.ac fragment.
const MACRO_PATTERNS: &[(&str, &[&str])] = &[
    // Core initialization
    ("AC_INIT", &[
        "AC_INIT([pkg], [1.0])",
        "AC_INIT([pkg], [1.0], [bug@example.com])",
        "AC_INIT([pkg], [1.0], [bug@example.com], [pkg])",
        "AC_INIT([pkg], [1.0], [bug@example.com], [pkg], [https://example.com])",
    ]),
    // Version requirements
    ("AC_PREREQ", &[
        "AC_PREREQ([2.59])",
        "AC_PREREQ([2.63])",
        "AC_PREREQ([2.69])",
        "AC_PREREQ([2.71])",
    ]),
    // Output files
    ("AC_CONFIG_FILES", &[
        "AC_CONFIG_FILES([Makefile])",
        "AC_CONFIG_FILES([Makefile src/Makefile])",
        "AC_CONFIG_FILES([Makefile:Makefile.in])",
    ]),
    // Headers
    ("AC_CONFIG_HEADERS", &[
        "AC_CONFIG_HEADERS([config.h])",
        "AC_CONFIG_HEADERS([config.h:config.hin])",
    ]),
    // Source dir
    ("AC_CONFIG_SRCDIR", &[
        "AC_CONFIG_SRCDIR([src/main.c])",
    ]),
    // Aux dir
    ("AC_CONFIG_AUX_DIR", &[
        "AC_CONFIG_AUX_DIR([build-aux])",
    ]),
    // Macro dir
    ("AC_CONFIG_MACRO_DIR", &[
        "AC_CONFIG_MACRO_DIR([m4])",
    ]),
    // Subdirs
    ("AC_CONFIG_SUBDIRS", &[
        "AC_CONFIG_SUBDIRS([lib])",
    ]),
    // Substitutions
    ("AC_SUBST", &[
        "AC_SUBST([PACKAGE_NAME])",
        "AC_SUBST([PACKAGE_VERSION])",
        "AC_SUBST([CC])",
        "AC_SUBST([CFLAGS])",
        "AC_SUBST([LDFLAGS])",
        "AC_SUBST([LIBS])",
        "AC_SUBST([prefix])",
        "AC_SUBST([exec_prefix])",
    ]),
    // Defines
    ("AC_DEFINE", &[
        "AC_DEFINE([PACKAGE_NAME], [\"pkg\"])",
        "AC_DEFINE([PACKAGE_VERSION], [\"1.0\"])",
        "AC_DEFINE([HAVE_STDIO_H], [1])",
        "AC_DEFINE([HAVE_STDLIB_H], [1])",
        "AC_DEFINE_UNQUOTED([PACKAGE], [\"$PACKAGE_NAME\"])",
    ]),
    // Compiler detection
    ("AC_PROG_CC", &["AC_PROG_CC"]),
    ("AC_PROG_CXX", &["AC_PROG_CXX"]),
    ("AC_PROG_CPP", &["AC_PROG_CPP"]),
    ("AC_PROG_CC_C_O", &["AC_PROG_CC_C_O"]),
    // Program detection
    ("AC_PROG_INSTALL", &["AC_PROG_INSTALL"]),
    ("AC_PROG_MAKE_SET", &["AC_PROG_MAKE_SET"]),
    ("AC_PROG_AWK", &["AC_PROG_AWK"]),
    ("AC_PROG_GREP", &["AC_PROG_GREP"]),
    ("AC_PROG_EGREP", &["AC_PROG_EGREP"]),
    ("AC_PROG_FGREP", &["AC_PROG_FGREP"]),
    ("AC_PROG_LN_S", &["AC_PROG_LN_S"]),
    ("AC_PROG_SED", &["AC_PROG_SED"]),
    ("AC_PROG_YACC", &["AC_PROG_YACC"]),
    ("AC_PROG_LEX", &["AC_PROG_LEX"]),
    ("AC_PROG_RANLIB", &["AC_PROG_RANLIB"]),
    ("AC_PROG_AR", &["AC_PROG_AR"]),
    ("AC_PROG_MKDIR_P", &["AC_PROG_MKDIR_P"]),
    // Function checks
    ("AC_CHECK_FUNC", &[
        "AC_CHECK_FUNCS([malloc realloc])",
        "AC_CHECK_FUNCS([strerror gettimeofday])",
        "AC_CHECK_FUNCS([socket gethostbyname])",
        "AC_CHECK_FUNCS([fork vfork])",
    ]),
    ("AC_CHECK_LIB", &[
        "AC_CHECK_LIB([m], [sin])",
        "AC_CHECK_LIB([pthread], [pthread_create])",
        "AC_CHECK_LIB([dl], [dlopen])",
    ]),
    // Header checks
    ("AC_CHECK_HEADER", &[
        "AC_CHECK_HEADERS([stdlib.h string.h])",
        "AC_CHECK_HEADERS([unistd.h fcntl.h])",
        "AC_CHECK_HEADERS([sys/types.h sys/stat.h])",
    ]),
    // Type checks
    ("AC_CHECK_TYPE", &[
        "AC_CHECK_TYPES([pid_t])",
        "AC_CHECK_TYPES([size_t ssize_t])",
        "AC_CHECK_TYPES([off_t mode_t])",
    ]),
    // Specific function checks
    ("AC_FUNC_ALLOCA", &["AC_FUNC_ALLOCA"]),
    ("AC_FUNC_FORK", &["AC_FUNC_FORK"]),
    ("AC_FUNC_MALLOC", &["AC_FUNC_MALLOC"]),
    ("AC_FUNC_MMAP", &["AC_FUNC_MMAP"]),
    ("AC_FUNC_STRERROR_R", &["AC_FUNC_STRERROR_R"]),
    ("AC_FUNC_VPRINTF", &["AC_FUNC_VPRINTF"]),
    // Specific header checks
    ("AC_HEADER_STDC", &["AC_HEADER_STDC"]),
    ("AC_HEADER_DIRENT", &["AC_HEADER_DIRENT"]),
    ("AC_HEADER_SYS_WAIT", &["AC_HEADER_SYS_WAIT"]),
    ("AC_HEADER_TIME", &["AC_HEADER_TIME"]),
    ("AC_HEADER_STDBOOL", &["AC_HEADER_STDBOOL"]),
    ("AC_HEADER_STDINT", &["AC_HEADER_STDINT"]),
    // Specific type checks
    ("AC_TYPE_PID_T", &["AC_TYPE_PID_T"]),
    ("AC_TYPE_SIZE_T", &["AC_TYPE_SIZE_T"]),
    ("AC_TYPE_UID_T", &["AC_TYPE_UID_T"]),
    ("AC_TYPE_MODE_T", &["AC_TYPE_MODE_T"]),
    ("AC_TYPE_OFF_T", &["AC_TYPE_OFF_T"]),
    ("AC_TYPE_INT32_T", &["AC_TYPE_INT32_T"]),
    ("AC_TYPE_UINT32_T", &["AC_TYPE_UINT32_T"]),
    // Structure checks
    ("AC_STRUCT_TM", &["AC_STRUCT_TM"]),
    ("AC_STRUCT_ST_BLOCKS", &["AC_STRUCT_ST_BLOCKS"]),
    ("AC_STRUCT_TIMEZONE", &["AC_STRUCT_TIMEZONE"]),
    // sizeof checks
    ("AC_CHECK_SIZEOF", &[
        "AC_CHECK_SIZEOF([int])",
        "AC_CHECK_SIZEOF([long])",
        "AC_CHECK_SIZEOF([void *])",
    ]),
    // Member checks
    ("AC_CHECK_MEMBER", &[
        "AC_CHECK_MEMBERS([struct stat.st_blocks])",
        "AC_CHECK_MEMBERS([struct stat.st_rdev])",
    ]),
    // Canonical system type
    ("AC_CANONICAL_HOST", &["AC_CANONICAL_HOST"]),
    ("AC_CANONICAL_BUILD", &["AC_CANONICAL_BUILD"]),
    ("AC_CANONICAL_TARGET", &["AC_CANONICAL_TARGET"]),
    // Arguments
    ("AC_ARG_ENABLE", &[
        "AC_ARG_ENABLE([debug], AS_HELP_STRING([--enable-debug], [enable debugging]))",
        "AC_ARG_ENABLE([threads], AS_HELP_STRING([--enable-threads], [enable threading]))",
    ]),
    ("AC_ARG_WITH", &[
        "AC_ARG_WITH([ssl], AS_HELP_STRING([--with-ssl], [use SSL]))",
        "AC_ARG_WITH([zlib], AS_HELP_STRING([--with-zlib], [use zlib]))",
    ]),
    // Messages
    ("AC_MSG_CHECKING", &[
        "AC_MSG_CHECKING([for something])\nAC_MSG_RESULT([yes])",
    ]),
    // Replace functions
    ("AC_REPLACE_FUNCS", &[
        "AC_REPLACE_FUNCS([strdup getopt])",
    ]),
    ("AC_LIBOBJ", &[
        "AC_LIBOBJ([fnmatch])",
    ]),
    // Language
    ("AC_LANG_PUSH", &[
        "AC_LANG_PUSH([C])",
        "AC_LANG_PUSH([C++])\nAC_LANG_POP([C++])",
    ]),
    // Cache
    ("AC_CACHE_CHECK", &[
        "AC_CACHE_CHECK([for working alloca.h], [ac_cv_header_alloca_h],\n  [AC_LINK_IFELSE([AC_LANG_PROGRAM([#include <alloca.h>])],\n    [ac_cv_header_alloca_h=yes], [ac_cv_header_alloca_h=no])])",
    ]),
    // Prefix
    ("AC_PREFIX_DEFAULT", &["AC_PREFIX_DEFAULT([/usr/local])"]),
    ("AC_PREFIX_PROGRAM", &["AC_PREFIX_PROGRAM([gcc])"]),
    // Revision/Copyright
    ("AC_REVISION", &["AC_REVISION([$Revision: 1.0 $])"]),
    ("AC_COPYRIGHT", &["AC_COPYRIGHT([Copyright 2024 Example Corp])"]),
    // Preserve help order
    ("AC_PRESERVE_HELP_ORDER", &["AC_PRESERVE_HELP_ORDER"]),
];

/// Generate a combinatorial corpus of configure.ac files.
/// Each file combines patterns from different categories.
fn generate_corpus(output_dir: &Path, count: usize) -> Vec<(String, String)> {
    let mut corpus = Vec::new();
    let _ = std::fs::create_dir_all(output_dir);

    // Simple deterministic PRNG
    let mut state: u64 = 0xCAFE_BABE_AC_AC_01;

    for i in 0..count {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut ac = String::new();

        // Always include AC_INIT
        let init = MACRO_PATTERNS[0].1[(state as usize) % MACRO_PATTERNS[0].1.len()];
        ac.push_str(init);
        ac.push('\n');

        // Random selection of other macros (2-8 additional)
        let extra_count = 2 + ((state >> 32) as usize % 7);
        let mut included = vec![false; MACRO_PATTERNS.len()];
        included[0] = true; // AC_INIT already included

        for _ in 0..extra_count {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = 1 + ((state as usize) % (MACRO_PATTERNS.len() - 1));
            if !included[idx] && !MACRO_PATTERNS[idx].1.is_empty() {
                included[idx] = true;
                let pattern = MACRO_PATTERNS[idx].1[(state as usize) % MACRO_PATTERNS[idx].1.len()];
                ac.push_str(pattern);
                ac.push('\n');
            }
        }

        // Always end with AC_OUTPUT
        ac.push_str("AC_OUTPUT\n");

        // Write to file
        let filename = format!("corpus_{:05}.ac", i);
        let path = output_dir.join(&filename);
        let _ = std::fs::write(&path, &ac);
        corpus.push((filename, ac));
    }

    println!(
        "Generated {} configure.ac files in {}",
        corpus.len(),
        output_dir.display()
    );
    corpus
}

// --- Oracle Comparator ---

#[allow(dead_code)]
struct ComparisonResult {
    fixture: String,
    oracle_version: String,
    oracle_size: usize,
    rust_size: usize,
    byte_match: bool,
    size_ratio: f64,
    oracle_exit: i32,
    rust_exit: i32,
    first_diff_byte: Option<usize>,
    sha256_oracle: String,
    sha256_rust: String,
}

fn run_oracle_version(ac_path: &Path, autoconf_bin: &str) -> (Vec<u8>, i32) {
    let output = Command::new(autoconf_bin)
        .arg(ac_path.file_name().unwrap())
        .current_dir(ac_path.parent().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => (out.stdout, out.status.code().unwrap_or(-1)),
        Err(_) => (vec![], -1),
    }
}

fn run_autoconf_rs(ac_path: &Path) -> (Vec<u8>, i32) {
    let release_bin = Path::new("target/release/autoconf");
    let bin = if release_bin.exists() {
        release_bin
            .canonicalize()
            .unwrap_or_else(|_| release_bin.to_path_buf())
    } else {
        Path::new("target/debug/autoconf")
            .canonicalize()
            .unwrap_or_else(|_| Path::new("target/debug/autoconf").to_path_buf())
    };

    let output = Command::new(&bin)
        .arg(ac_path.file_name().unwrap())
        .current_dir(ac_path.parent().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => (out.stdout, out.status.code().unwrap_or(-1)),
        Err(_) => (vec![], -101),
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

fn find_first_diff(a: &[u8], b: &[u8]) -> Option<usize> {
    let min = a.len().min(b.len());
    for i in 0..min {
        if a[i] != b[i] {
            return Some(i);
        }
    }
    if a.len() != b.len() {
        Some(min)
    } else {
        None
    }
}

// --- Main Entry Point ---

pub fn run() -> ExitCode {
    println!("=== autoconf-rs Multi-Oracle Parity Engine ===\n");

    // Check oracle availability
    let oracle_versions = find_oracle_versions();
    if oracle_versions.is_empty() {
        eprintln!("ERROR: No GNU Autoconf oracle found. Install autoconf 2.69+ on PATH.");
        eprintln!("Try: apt install autoconf  OR  dnf install autoconf");
        return ExitCode::FAILURE;
    }
    println!("Oracle versions detected:");
    for (ver, path) in &oracle_versions {
        println!("  {} → {}", ver, path);
    }

    // Generate corpus
    let corpus_dir = Path::new("lab/corpus/generated");
    let corpus_size = std::env::var("CORPUS_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    println!("\nGenerating {} configure.ac corpus files...", corpus_size);
    let corpus = generate_corpus(corpus_dir, corpus_size);

    // Run comparisons
    println!(
        "\nRunning comparisons against {} oracle versions...\n",
        oracle_versions.len()
    );

    let mut results: Vec<ComparisonResult> = Vec::new();
    let start = Instant::now();

    for (i, (filename, _ac_content)) in corpus.iter().enumerate() {
        let ac_path = corpus_dir.join(filename);

        for (oracle_ver, oracle_path) in &oracle_versions {
            let (oracle_out, oracle_exit) = run_oracle_version(&ac_path, oracle_path);
            let (rust_out, rust_exit) = run_autoconf_rs(&ac_path);

            let byte_match = oracle_out == rust_out;
            let size_ratio = if oracle_out.is_empty() {
                0.0
            } else {
                rust_out.len() as f64 / oracle_out.len() as f64
            };
            let first_diff = find_first_diff(&oracle_out, &rust_out);

            results.push(ComparisonResult {
                fixture: filename.clone(),
                oracle_version: oracle_ver.clone(),
                oracle_size: oracle_out.len(),
                rust_size: rust_out.len(),
                byte_match,
                size_ratio,
                oracle_exit,
                rust_exit,
                first_diff_byte: first_diff,
                sha256_oracle: sha256_hex(&oracle_out),
                sha256_rust: sha256_hex(&rust_out),
            });
        }

        // Progress
        if (i + 1) % 100 == 0 {
            let elapsed = start.elapsed();
            let exact = results.iter().filter(|r| r.byte_match).count();
            println!(
                "  [{}/{}] {} exact matches ({:.1}s)",
                i + 1,
                corpus.len(),
                exact,
                elapsed.as_secs_f64()
            );
        }
    }

    let elapsed = start.elapsed();

    // --- Analysis ---
    let total = results.len();
    let exact_matches = results.iter().filter(|r| r.byte_match).count();
    let high_match = results.iter().filter(|r| r.size_ratio >= 0.95).count();
    let reasonable = results.iter().filter(|r| r.size_ratio >= 0.50).count();
    let panics = results.iter().filter(|r| r.rust_exit == -101).count();

    // Surface-level gap analysis
    let mut surface_gaps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in &results {
        if !r.byte_match {
            let surface = classify_divergence(r);
            surface_gaps
                .entry(surface)
                .or_default()
                .push(r.fixture.clone());
        }
    }

    println!("\n=== Parity Report ===\n");
    println!("  Corpus size:      {}", corpus_size);
    println!("  Oracle versions:  {}", oracle_versions.len());
    println!("  Total comparisons: {}", total);
    println!("  Duration:         {:.1}s", elapsed.as_secs_f64());
    println!(
        "  Rate:             {:.0} comps/sec",
        total as f64 / elapsed.as_secs_f64()
    );
    println!();
    println!(
        "  Exact matches:    {} ({:.1}%)",
        exact_matches,
        exact_matches as f64 / total as f64 * 100.0
    );
    println!(
        "  ≥95% size match:  {} ({:.1}%)",
        high_match,
        high_match as f64 / total as f64 * 100.0
    );
    println!(
        "  ≥50% size match:  {} ({:.1}%)",
        reasonable,
        reasonable as f64 / total as f64 * 100.0
    );
    println!("  Panics:           {}", panics);
    println!();

    // Show first few exact matches and divergences
    if exact_matches > 0 {
        println!("Sample exact matches (first 5):");
        for r in results.iter().filter(|r| r.byte_match).take(5) {
            println!(
                "  ✓ {} [{}] {} bytes",
                r.fixture, r.oracle_version, r.oracle_size
            );
        }
        println!();
    }

    // Gap report
    println!("=== Surface Gap Analysis ===\n");
    let mut gaps: Vec<_> = surface_gaps.iter().collect();
    gaps.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    for (surface, fixtures) in &gaps {
        let pct = (fixtures.len() as f64 / total as f64) * 100.0;
        println!(
            "  {} — {} divergences ({:.1}%)",
            surface,
            fixtures.len(),
            pct
        );
        if fixtures.len() <= 5 {
            for f in *fixtures {
                println!("    - {}", f);
            }
        }
    }
    println!();

    // Save detailed report
    let report = serde_json::json!({
        "schema": "autoconf-rs-parity-report-v1",
        "generated_at": chrono_now(),
        "corpus_size": corpus_size,
        "oracle_versions": oracle_versions.iter().map(|(v, p)| {
            serde_json::json!({"version": v, "path": p})
        }).collect::<Vec<_>>(),
        "summary": {
            "total_comparisons": total,
            "exact_matches": exact_matches,
            "exact_pct": exact_matches as f64 / total as f64 * 100.0,
            "high_match_95pct": high_match,
            "reasonable_50pct": reasonable,
            "panics": panics,
            "duration_secs": elapsed.as_secs_f64(),
        },
        "surface_gaps": gaps.iter().map(|(s, fs)| {
            serde_json::json!({"surface": s, "count": fs.len(), "pct": fs.len() as f64 / total as f64 * 100.0})
        }).collect::<Vec<_>>(),
        "divergence_samples": results.iter()
            .filter(|r| !r.byte_match)
            .take(20)
            .map(|r| serde_json::json!({
                "fixture": r.fixture,
                "oracle_version": r.oracle_version,
                "oracle_size": r.oracle_size,
                "rust_size": r.rust_size,
                "size_ratio": r.size_ratio,
                "first_diff_byte": r.first_diff_byte,
            }))
            .collect::<Vec<_>>(),
    });

    let report_dir = Path::new("reports");
    let _ = std::fs::create_dir_all(report_dir);
    let report_path = report_dir.join("parity-report.json");
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write(&report_path, &json);
        println!("Report saved to {}", report_path.display());
    }

    // Verdict
    if panics > 0 {
        eprintln!("\n=== PARITY CHECK FAILED — {} panics ===", panics);
        ExitCode::FAILURE
    } else if exact_matches == total {
        println!("\n=== PARITY CHECK PASSED — 100% exact match ===");
        ExitCode::SUCCESS
    } else {
        println!(
            "\n=== PARITY CHECK — {} exact / {} total ({:.1}%) ===",
            exact_matches,
            total,
            exact_matches as f64 / total as f64 * 100.0
        );
        ExitCode::SUCCESS // Non-blocking: divergences are informational
    }
}

fn classify_divergence(r: &ComparisonResult) -> String {
    if r.rust_exit == -101 {
        return "PANIC".to_string();
    }
    if r.rust_size == 0 && r.oracle_size > 0 {
        return "EMPTY_OUTPUT".to_string();
    }
    let ratio = r.size_ratio;
    if ratio >= 0.90 {
        "NEAR_MATCH_90".to_string()
    } else if ratio >= 0.50 {
        "PARTIAL_MATCH_50".to_string()
    } else if ratio >= 0.10 {
        "LOW_MATCH_10".to_string()
    } else {
        "NO_MATCH".to_string()
    }
}

fn find_oracle_versions() -> Vec<(String, String)> {
    let mut versions = Vec::new();

    // Check common autoconf paths
    let candidates = [
        "autoconf",
        "autoconf2.69",
        "autoconf2.71",
        "autoconf2.72",
        "autoconf2.73",
    ];

    for name in &candidates {
        if let Ok(output) = Command::new(name).arg("--version").output() {
            let ver_str = String::from_utf8_lossy(&output.stdout);
            let version = ver_str
                .lines()
                .next()
                .unwrap_or("")
                .replace("autoconf (GNU Autoconf) ", "")
                .trim()
                .to_string();
            if !version.is_empty() && !versions.iter().any(|(v, _)| v == &version) {
                versions.push((version, name.to_string()));
            }
        }
    }

    versions
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}
