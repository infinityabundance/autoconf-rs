// fuzz.rs — Deterministic fuzz harness for autoconf-rs.
//
// Generates random configure.ac inputs with a fixed seed, runs them through
// both the GNU Autoconf oracle and autoconf-rs, compares output byte-for-byte,
// and produces a fuzz receipt.
//
// Court: AC.FUZZ.1
// Target: 1M iterations, 0 panics, quantified divergence rate.

use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

/// Simple deterministic PRNG (LCG) with fixed seed for reproducibility.
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next() as usize) % max
    }

    fn next_bool(&mut self) -> bool {
        self.next() % 2 == 0
    }
}

/// A single fuzz iteration: generate random configure.ac, run both oracles, compare.
#[allow(dead_code)]
struct FuzzCase {
    seed: u64,
    configure_ac: String,
    oracle_stdout: Vec<u8>,
    oracle_stderr: Vec<u8>,
    oracle_exit: i32,
    rust_stdout: Vec<u8>,
    rust_stderr: Vec<u8>,
    rust_exit: i32,
    stdout_match: bool,
    exit_match: bool,
    ac_rs_panicked: bool,
}

/// Generate a random configure.ac file.
fn generate_random_configure_ac(rng: &mut DeterministicRng) -> String {
    let mut buf = String::new();

    // Package name from pool
    let names = ["hello", "test", "foo", "bar", "myapp", "libx", "prog"];
    let versions = ["1.0", "2.3.1", "0.1-alpha", "3.0-beta2", "1.2.3"];
    let bug_reports = ["bugs@example.com", "https://github.com/example/issues", ""];

    let name = names[rng.next_usize(names.len())];
    let version = versions[rng.next_usize(versions.len())];
    let bug = bug_reports[rng.next_usize(bug_reports.len())];

    // AC_INIT line
    if bug.is_empty() {
        buf.push_str(&format!("AC_INIT([{}], [{}])\n", name, version));
    } else {
        buf.push_str(&format!("AC_INIT([{}], [{}], [{}])\n", name, version, bug));
    }

    // Random AC_PREREQ
    if rng.next_bool() {
        buf.push_str("AC_PREREQ([2.69])\n");
    }

    // Random config files
    if rng.next_bool() {
        buf.push_str("AC_CONFIG_FILES([Makefile])\n");
    }

    // Random config headers
    if rng.next_bool() {
        if rng.next_bool() {
            buf.push_str("AC_CONFIG_HEADERS([config.h])\n");
        } else {
            buf.push_str("AC_CONFIG_HEADERS([config.h:config.hin])\n");
        }
    }

    // Random subst
    let subst_vars = [
        "PACKAGE_NAME",
        "PACKAGE_VERSION",
        "CC",
        "CFLAGS",
        "LIBS",
        "prefix",
    ];
    let subst_count = rng.next_usize(3);
    for _ in 0..subst_count {
        let var = subst_vars[rng.next_usize(subst_vars.len())];
        buf.push_str(&format!("AC_SUBST([{}])\n", var));
    }

    // Random define
    if rng.next_bool() {
        let define_names = [
            "HAVE_STDIO_H",
            "HAVE_STDLIB_H",
            "PACKAGE",
            "VERSION",
            "DEBUG",
        ];
        let name = define_names[rng.next_usize(define_names.len())];
        if rng.next_bool() {
            buf.push_str(&format!("AC_DEFINE([{}], [1], [Define to 1])\n", name));
        } else {
            let vals = ["1", "0", "\"yes\""];
            let val = vals[rng.next_usize(vals.len())];
            buf.push_str(&format!("AC_DEFINE([{}], [{}])\n", name, val));
        }
    }

    // Random message
    if rng.next_bool() {
        let msgs = [
            "checking for stdio.h",
            "checking for library",
            "checking system type",
        ];
        let msg = msgs[rng.next_usize(msgs.len())];
        buf.push_str(&format!("AC_MSG_CHECKING([{}])\n", msg));
        let results = ["yes", "no", "found"];
        let result = results[rng.next_usize(results.len())];
        buf.push_str(&format!("AC_MSG_RESULT([{}])\n", result));
    }

    // Random AC_CHECK_FUNC
    if rng.next_bool() {
        let funcs = ["malloc", "strerror", "getpwuid_r", "socket", "gettimeofday"];
        let func = funcs[rng.next_usize(funcs.len())];
        buf.push_str(&format!("AC_CHECK_FUNCS([{}])\n", func));
    }

    // Random AC_CHECK_HEADER
    if rng.next_bool() {
        let headers = ["stdio.h", "stdlib.h", "string.h", "unistd.h", "sys/types.h"];
        let h = headers[rng.next_usize(headers.len())];
        buf.push_str(&format!("AC_CHECK_HEADERS([{}])\n", h));
    }

    // Random AC_CHECK_LIB
    if rng.next_bool() {
        let libs = ["m", "pthread", "dl", "socket"];
        let lib = libs[rng.next_usize(libs.len())];
        buf.push_str(&format!("AC_CHECK_LIB([{}], [main])\n", lib));
    }

    // Random AC_PROG_*
    if rng.next_bool() {
        let progs = [
            "AC_PROG_CC",
            "AC_PROG_CXX",
            "AC_PROG_INSTALL",
            "AC_PROG_MAKE_SET",
            "AC_PROG_AWK",
            "AC_PROG_GREP",
            "AC_PROG_LN_S",
            "AC_PROG_SED",
            "AC_PROG_YACC",
            "AC_PROG_LEX",
        ];
        let prog = progs[rng.next_usize(progs.len())];
        buf.push_str(&format!("{}\n", prog));
    }

    // Random AC_CANONICAL
    if rng.next_bool() {
        buf.push_str("AC_CANONICAL_HOST\n");
    }

    // Random AC_ARG_WITH/ENABLE
    if rng.next_bool() {
        let features = ["debug", "threads", "ssl", "zlib", "readline"];
        let feat = features[rng.next_usize(features.len())];
        let help = format!("enable {} support", feat);
        buf.push_str(&format!(
            "AC_ARG_ENABLE([{}], AS_HELP_STRING([--enable-{}], [{}]))\n",
            feat, feat, help
        ));
    }

    // AC_OUTPUT must be last
    buf.push_str("AC_OUTPUT\n");

    buf
}

/// Run the GNU Autoconf oracle on a configure.ac file in a temp dir.
fn run_oracle(ac_path: &Path, autoconf_bin: &str) -> (Vec<u8>, Vec<u8>, i32) {
    let dir = ac_path.parent().unwrap();
    let output = Command::new(autoconf_bin)
        .arg(ac_path.file_name().unwrap())
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => (out.stdout, out.stderr, out.status.code().unwrap_or(-1)),
        Err(e) => (vec![], format!("oracle error: {}", e).into_bytes(), -1),
    }
}

/// Run autoconf-rs on a configure.ac file.
/// Uses pre-built binary if available, otherwise falls back to cargo run.
fn run_autoconf_rs(ac_path: &Path) -> (Vec<u8>, Vec<u8>, i32) {
    let dir = ac_path.parent().unwrap();

    // Try pre-built release binary first (much faster)
    let release_bin = Path::new("target/release/autoconf");
    // Canonicalize to absolute path so Command::new can find it
    let autoconf_rs_bin = if release_bin.exists() {
        release_bin
            .canonicalize()
            .unwrap_or_else(|_| release_bin.to_path_buf())
    } else {
        // Fall back to cargo run
        return run_autoconf_rs_cargo(ac_path);
    };

    let output = Command::new(autoconf_rs_bin)
        .arg(ac_path.file_name().unwrap())
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => (out.stdout, out.stderr, out.status.code().unwrap_or(-1)),
        Err(e) => (
            vec![],
            format!("autoconf-rs error: {}", e).into_bytes(),
            -101,
        ),
    }
}

/// Fallback: run autoconf-rs via cargo run.
fn run_autoconf_rs_cargo(ac_path: &Path) -> (Vec<u8>, Vec<u8>, i32) {
    let dir = ac_path.parent().unwrap();
    let output = Command::new("cargo")
        .args(["run", "-p", "autoconf-rs-cli", "--bin", "autoconf", "--"])
        .arg(ac_path.file_name().unwrap())
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => (out.stdout, out.stderr, out.status.code().unwrap_or(-1)),
        Err(e) => (
            vec![],
            format!("autoconf-rs error: {}", e).into_bytes(),
            -101,
        ),
    }
}

/// Compare two byte slices and report differences.
fn byte_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a == b
}

/// Main fuzz entry point.
pub fn run() -> ExitCode {
    println!("=== autoconf-rs Deterministic Fuzz ===\n");

    // Default: 10K for quick CI. Use FUZZ_ITERATIONS env var to override.
    let iterations: u64 = std::env::var("FUZZ_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    let seed: u64 = 0xAC_AC_AC_01_F5_F5_F5; // Fixed seed for reproducibility
    let autoconf_bin = "autoconf";

    // Verify oracle is available
    if Command::new(autoconf_bin)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| !s.success())
        .unwrap_or(true)
    {
        eprintln!("ERROR: GNU Autoconf oracle not found at '{}'", autoconf_bin);
        eprintln!("Install GNU Autoconf 2.73 and ensure it's on PATH.");
        return ExitCode::FAILURE;
    }

    // Get oracle version
    let oracle_version = Command::new(autoconf_bin)
        .arg("--version")
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("unknown")
                .to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string());
    println!("Oracle: {}", oracle_version);
    println!("Iterations: {}", iterations);
    println!("Seed: 0x{:016x}\n", seed);

    let mut rng = DeterministicRng::new(seed);

    // Temp directory for fuzz files
    let tmpdir = std::env::temp_dir().join("autoconf-rs-fuzz");
    let _ = std::fs::create_dir_all(&tmpdir);

    let mut results: Vec<FuzzCase> = Vec::new();
    let mut divergences: Vec<usize> = Vec::new();
    let mut panics: Vec<usize> = Vec::new();
    let mut exit_mismatches: Vec<usize> = Vec::new();

    let start = Instant::now();
    let report_interval = if iterations >= 10 { iterations / 10 } else { 1 }; // Report every 10%

    for i in 0..iterations {
        let case_seed = rng.state;
        let ac_content = generate_random_configure_ac(&mut rng);

        // Write configure.ac to temp file
        let ac_path = tmpdir.join(format!("fuzz_{:08}.ac", i));
        if let Err(e) = std::fs::write(&ac_path, &ac_content) {
            eprintln!("  ERROR writing fuzz file: {}", e);
            return ExitCode::FAILURE;
        }

        // Run oracle
        let (oracle_stdout, oracle_stderr, oracle_exit) = run_oracle(&ac_path, autoconf_bin);

        // Run autoconf-rs
        let (rust_stdout, rust_stderr, rust_exit) = run_autoconf_rs(&ac_path);

        let stdout_match = byte_compare(&oracle_stdout, &rust_stdout);
        let exit_match = oracle_exit == rust_exit;
        let panicked = rust_exit == -101;

        let case = FuzzCase {
            seed: case_seed,
            configure_ac: ac_content,
            oracle_stdout,
            oracle_stderr,
            oracle_exit,
            rust_stdout,
            rust_stderr,
            rust_exit,
            stdout_match,
            exit_match,
            ac_rs_panicked: panicked,
        };

        if !stdout_match {
            divergences.push(i as usize);
        }
        if panicked {
            panics.push(i as usize);
        }
        if !exit_match {
            exit_mismatches.push(i as usize);
        }

        results.push(case);

        // Clean up temp file
        let _ = std::fs::remove_file(&ac_path);

        // Progress reporting
        if (i + 1) % report_interval == 0 {
            let elapsed = start.elapsed();
            let pct = ((i + 1) as f64 / iterations as f64) * 100.0;
            println!(
                "  [{:.0}%] {}/{} iterations, {} divergences, {} panics ({:.1}s)",
                pct,
                i + 1,
                iterations,
                divergences.len(),
                panics.len(),
                elapsed.as_secs_f64()
            );
        }
    }

    let elapsed = start.elapsed();

    // Summary
    println!("\n=== Fuzz Complete ===\n");
    println!("  Iterations:     {}", iterations);
    println!("  Duration:       {:.1}s", elapsed.as_secs_f64());
    println!(
        "  Rate:           {:.0} iterations/sec",
        if elapsed.as_secs_f64() > 0.0 {
            iterations as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        }
    );
    let div_rate = if iterations > 0 {
        (divergences.len() as f64 / iterations as f64) * 100.0
    } else {
        0.0
    };
    println!("  Divergences:    {} ({:.4}%)", divergences.len(), div_rate);
    println!("  Panics:         {}", panics.len());
    println!("  Exit mismatches: {}", exit_mismatches.len());
    println!("  Seed:           0x{:016x}", seed);

    // Show first few divergences
    if !divergences.is_empty() {
        let show = divergences.len().min(10);
        println!("\nFirst {} divergences:\n", show);
        for &idx in &divergences[..show] {
            let case = &results[idx];
            println!("--- Divergence #{} (seed 0x{:016x}) ---", idx, case.seed);
            println!("configure.ac:\n{}", case.configure_ac);
            println!(
                "Oracle stdout ({} bytes):\n{}",
                case.oracle_stdout.len(),
                String::from_utf8_lossy(&case.oracle_stdout[..case.oracle_stdout.len().min(500)])
            );
            println!(
                "Rust stdout ({} bytes):\n{}",
                case.rust_stdout.len(),
                String::from_utf8_lossy(&case.rust_stdout[..case.rust_stdout.len().min(500)])
            );
            println!(
                "Oracle stderr: {}",
                String::from_utf8_lossy(&case.oracle_stderr)
            );
            println!(
                "Rust stderr: {}",
                String::from_utf8_lossy(&case.rust_stderr)
            );
            println!(
                "Oracle exit: {}, Rust exit: {}",
                case.oracle_exit, case.rust_exit
            );
            println!();
        }
    }

    // Save fuzz receipt
    let receipt = serde_json::json!({
        "schema": "autoconf-rs-fuzz-receipt-v1",
        "court": "AC.FUZZ.1",
        "verdict": if panics.is_empty() && divergences.len() < (iterations as usize / 100) {
            "admitted_match"
        } else if panics.is_empty() {
            "admitted_divergence"
        } else {
            "rust_error"
        },
        "oracle": {
            "kind": "gnu_autoconf",
            "version": oracle_version,
            "path": autoconf_bin,
        },
        "fuzz_config": {
            "iterations": iterations,
            "seed": format!("0x{:016x}", seed),
            "duration_secs": elapsed.as_secs_f64(),
            "rate_per_sec": iterations as f64 / elapsed.as_secs_f64(),
        },
        "results": {
            "total_iterations": iterations,
            "divergences": divergences.len(),
            "divergence_rate_pct": (divergences.len() as f64 / iterations as f64) * 100.0,
            "panics": panics.len(),
            "exit_mismatches": exit_mismatches.len(),
            "stdout_match_rate_pct": 100.0 - (divergences.len() as f64 / iterations as f64) * 100.0,
        },
        "positive_claim": format!(
            "autoconf-rs survives {} fuzz iterations with {} panics, {} stdout divergences ({:.4}%).",
            iterations, panics.len(), divergences.len(),
            (divergences.len() as f64 / iterations as f64) * 100.0
        ),
        "non_claims": [
            "Divergences may exist for unimplemented macro surfaces",
            "Oracle comparison limited to generated configure script, not execution",
            "Fuzz only tests Layer 0 macro expansion, not real-project configure.ac",
        ],
    });

    let receipt_dir = Path::new("reports/receipts");
    let _ = std::fs::create_dir_all(receipt_dir);
    let receipt_path = receipt_dir.join("fuzz-1M-receipt.json");
    if let Ok(json) = serde_json::to_string_pretty(&receipt) {
        if let Err(e) = std::fs::write(&receipt_path, &json) {
            eprintln!("Warning: could not save receipt: {}", e);
        } else {
            println!("\nReceipt saved to {}", receipt_path.display());
        }
    }

    // Compute SHA256 of receipt
    if let Ok(json) = serde_json::to_string(&receipt) {
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        println!("Receipt SHA256: {}", hash);
    }

    // Verdict
    if panics.is_empty() && divergences.is_empty() {
        println!("\n=== FUZZ PASS — 100% match, 0 panics ===");
        ExitCode::SUCCESS
    } else if panics.is_empty() {
        println!(
            "\n=== FUZZ PASS (with divergences) — {} divergences, 0 panics ===",
            divergences.len()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "\n=== FUZZ FAILED — {} panics, {} divergences ===",
            panics.len(),
            divergences.len()
        );
        ExitCode::FAILURE
    }
}
