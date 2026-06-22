// xtask — project maintenance tasks for autoconf-rs.
//
// Usage: cargo xtask <command>
//
// Commands:
//   check       Run all acceptance gate checks (fmt, clippy, test, freshness, oracle, claims, cleanroom)
//   fmt         Run rustfmt
//   clippy      Run clippy with warnings denied
//   test        Run all tests
//   oracle      Run oracle admission
//   generate    (Re)generate all documents from JSON sources
//   receipts    Verify receipt freshness
//   claims      Verify claim ladder freshness
//   ast-verify  Run AST parity verification bridge against oracle
//   behaviors   Scan source for @ac_behavior witnesses
//   cleanroom   Run GPL contamination scan
//   fuzz        Run deterministic fuzz harness
//   smoke       Run synthetic smoke test harness
//   gnu-compare Compare against GNU Autoconf test suite
//   bench       Performance baseline
//   status      Print current status summary
//
// The check command runs all gates and is the standard CI entry point.

mod ast_verify;
mod bench;
mod cleanroom;
mod compare;
mod consistency;
mod docgen;
mod fuzz;
mod gnu_compare;
mod rules_gate;
mod smoke;
mod truth;

use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("check");

    match command {
        "check" => run_check(),
        "fmt" => run_fmt(),
        "clippy" => run_clippy(),
        "test" => run_test(),
        "oracle" => run_oracle_admission(),
        "compare" => compare::run(),
        "generate" => run_generate(),
        "receipts" => run_receipt_check(),
        "claims" => run_claim_check(),
        "ast-verify" => run_ast_verify(),
        "behaviors" => run_behaviors_scan(),
        "cleanroom" => run_cleanroom_scan(),
        "fuzz" => fuzz::run(),
        "smoke" => smoke::run(),
        "gnu-compare" => gnu_compare::run(),
        "bench" => bench::run(),
        "status" => run_status(),
        _ => {
            eprintln!("xtask: unknown command: {}", command);
            eprintln!(
                "Available: check, fmt, clippy, test, oracle, compare, generate, \
                 receipts, claims, ast-verify, behaviors, cleanroom, fuzz, smoke, \
                 gnu-compare, bench, status"
            );
            ExitCode::FAILURE
        }
    }
}

fn run_check() -> ExitCode {
    println!("=== autoconf-rs acceptance gate check ===\n");

    let mut failed = false;

    // 0. .RULES compliance gate — verifies work COMPLIES with .RULES, not just file exists
    println!("[0/7] .RULES compliance gate...");
    let (rules_ok, rules_report) = rules_gate::check_rules_compliance();
    if rules_ok {
        println!(
            "  PASS: .RULES compliance verified ({} warnings)",
            rules_report.warnings.len()
        );
        for w in &rules_report.warnings {
            println!("  WARN: {}", w);
        }
    } else {
        for e in &rules_report.errors {
            eprintln!("  {}", e);
        }
        for w in &rules_report.warnings {
            eprintln!("  WARN: {}", w);
        }
        eprintln!(
            "  FAIL: {} .RULES compliance errors found",
            rules_report.errors.len()
        );
        failed = true;
    }

    // 1. Format
    println!("[1/7] rustfmt...");
    let fmt = Command::new("cargo")
        .args(["fmt", "--", "--check"])
        .status();
    if fmt.map(|s| !s.success()).unwrap_or(true) {
        eprintln!("  FAIL: formatting issues");
        failed = true;
    } else {
        println!("  PASS");
    }

    // 2. Clippy
    println!("[2/7] clippy...");
    let clippy = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .status();
    if clippy.map(|s| !s.success()).unwrap_or(true) {
        eprintln!("  FAIL: clippy issues");
        failed = true;
    } else {
        println!("  PASS");
    }

    // 3. Tests
    println!("[3/7] tests...");
    // Fast suite runs in parallel; the real-compiler integration tests (test_runtime_*,
    // test_layer4_*) are skipped here and run single-threaded below. Those tests fork many
    // concurrent `cc` processes — at full thread-per-core parallelism cc gets oversubscribed
    // and transient compile probes fail nondeterministically. Running them serially keeps the
    // gate deterministic; every test still runs and must pass (no test is ignored or weakened).
    let test = Command::new("cargo")
        .args([
            "test", "--all", "--", "--skip", "test_runtime", "--skip", "test_layer4",
        ])
        .status();
    let rt = Command::new("cargo")
        .args([
            "test",
            "-p",
            "autoconf-rs-core",
            "--release",
            "--test",
            "runtime_sandbox",
            "--",
            "--test-threads=1",
        ])
        .status();
    let test_ok =
        test.map(|s| s.success()).unwrap_or(false) && rt.map(|s| s.success()).unwrap_or(false);
    if !test_ok {
        eprintln!("  FAIL: tests failed");
        failed = true;
    } else {
        println!("  PASS");
    }

    // 4. Document freshness + internal consistency
    println!("[4/7] document freshness + internal consistency...");
    let registry_path = Path::new("reports/doc-registry.json");
    if registry_path.exists() {
        match std::fs::read_to_string(registry_path) {
            Ok(json) => match serde_json::from_str::<docgen::DocumentRegistry>(&json) {
                Ok(registry) => match registry.verify_freshness() {
                    Ok(msgs) => {
                        for m in &msgs {
                            println!("  {}", m);
                        }
                    }
                    Err(stale) => {
                        for s in &stale {
                            eprintln!("  {}", s);
                        }
                        eprintln!("  FAIL: stale documents. Run 'cargo xtask generate'.");
                        failed = true;
                    }
                },
                Err(e) => {
                    eprintln!("  WARN: invalid registry: {}", e);
                }
            },
            Err(e) => {
                eprintln!("  WARN: cannot read registry: {}", e);
            }
        }
    } else {
        println!("  INFO: no doc registry yet. Run 'cargo xtask generate'.");
    }

    // 4b. JSON source internal consistency
    match consistency::validate_all() {
        Ok(()) => {}
        Err(errors) => {
            eprintln!("  FAIL: {} JSON consistency errors found:", errors.len());
            for e in &errors {
                eprintln!("    {}", e);
            }
            eprintln!("  ACTION: Fix the stale/inconsistent data in source JSON files and re-run 'cargo xtask generate'.");
            failed = true;
        }
    }

    // 4c. TRUTH GATE — runs actual tests and compares against JSON claims
    // This gate prevents fabricated data. If JSON claims 1355 tests but only
    // 104 exist, this gate FAILS HARD.
    let (truth_ok, truth_report) = truth::run_truth_gate();
    if truth_ok {
        println!(
            "  TRUTH: test count verified (claimed {} ≤ actual {})",
            truth_report.claim_test_count, truth_report.actual_test_count
        );
    } else {
        for e in &truth_report.errors {
            eprintln!("  {}", e);
        }
        failed = true;
    }

    // 5. Oracle
    println!("[5/7] oracle profile...");
    if Path::new("reports/oracle-profile.json").exists() {
        println!("  PASS: oracle profile present");
    } else {
        println!("  WARN: no oracle profile. Run 'cargo xtask oracle'.");
    }

    // 6. Claim ladder
    println!("[6/7] claim ladder...");
    if Path::new("reports/claim-ladder.json").exists() {
        println!("  PASS: claim ladder present");
    } else {
        println!("  WARN: no claim ladder.");
    }

    // 7. Clean-room contamination scan
    println!("[7/7] clean-room scan...");
    match cleanroom::scan_source_tree() {
        Ok(receipt) => {
            if receipt.verdict == "FAIL" {
                eprintln!(
                    "  FAIL: {} GPL contamination errors found",
                    receipt.errors.len()
                );
                for e in &receipt.errors {
                    eprintln!("    {}:{} — {}: {}", e.file, e.line, e.pattern, e.matched);
                }
                failed = true;
            } else {
                println!(
                    "  PASS: {} files scanned, {} warnings, {} info markers",
                    receipt.files_scanned,
                    receipt.warnings.len(),
                    receipt.infos.len()
                );
                if let Ok(json) = serde_json::to_string_pretty(&receipt) {
                    let _ = std::fs::create_dir_all("reports/receipts");
                    let _ = std::fs::write("reports/receipts/cleanroom-receipt.json", &json);
                }
            }
        }
        Err(e) => {
            eprintln!("  FAIL: scan error: {}", e);
            failed = true;
        }
    }

    println!();
    if failed {
        eprintln!("=== ACCEPTANCE GATE FAILED ===");
        ExitCode::FAILURE
    } else {
        println!("=== ACCEPTANCE GATE PASSED ===");
        ExitCode::SUCCESS
    }
}

fn run_fmt() -> ExitCode {
    let s = Command::new("cargo")
        .args(["fmt"])
        .status()
        .unwrap_or_else(|_| std::process::exit(1));
    if s.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_clippy() -> ExitCode {
    let s = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .status()
        .unwrap_or_else(|_| std::process::exit(1));
    if s.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_test() -> ExitCode {
    let s = Command::new("cargo")
        .args(["test", "--all"])
        .status()
        .unwrap_or_else(|_| std::process::exit(1));
    if s.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_oracle_admission() -> ExitCode {
    println!("=== autoconf-rs oracle admission ===\n");

    match autoconf_oracle_rs::admit_oracle(&autoconf_oracle_rs::OracleConfig::default()) {
        Ok(profile) => {
            println!(
                "Oracle: {} (sha256: {})",
                profile.kind,
                &profile.sha256[..16.min(profile.sha256.len())]
            );
            println!("  autoconf: {}", profile.path);
            for (name, bp) in &profile.binaries {
                let status = if bp.smoke_passed { "✓" } else { "✗" };
                println!("  {} {} {}", status, name, bp.path);
            }
            if let Some(ref m4) = profile.m4_oracle {
                println!("  m4: {}", m4.path);
            }
            if let Err(e) =
                autoconf_oracle_rs::save_profile(&profile, Path::new("reports/oracle-profile.json"))
            {
                eprintln!("Error saving: {}", e);
                return ExitCode::FAILURE;
            }
            println!("Saved to reports/oracle-profile.json");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Oracle admission failed: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_generate() -> ExitCode {
    println!("=== Document Generation ===\n");
    let key = b"autoconf-rs-forensic-key-2026";

    let mut registry = docgen::DocumentRegistry::new();
    match docgen::generate::generate_all(&mut registry, key) {
        Ok(results) => {
            for r in &results {
                println!("  {}", r);
            }
            let json = serde_json::to_string_pretty(&registry).unwrap_or_default();
            if let Err(e) = std::fs::write("reports/doc-registry.json", &json) {
                eprintln!("Error saving registry: {}", e);
                return ExitCode::FAILURE;
            }
            println!("\nRegistry saved to reports/doc-registry.json");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for e in &errors {
                eprintln!("  {}", e);
            }
            ExitCode::FAILURE
        }
    }
}

fn run_receipt_check() -> ExitCode {
    println!("=== Receipt check ===\n");
    let dir = Path::new("reports/receipts");
    if !dir.exists() {
        println!("No receipts directory. Expected before courts are sealed.");
        return ExitCode::SUCCESS;
    }
    match std::fs::read_dir(dir) {
        Ok(entries) => {
            let count = entries
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
                .count();
            println!("Receipts: {}", count);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_claim_check() -> ExitCode {
    println!("=== Claim ladder check ===\n");
    let path = Path::new("reports/claim-ladder.json");
    if !path.exists() {
        println!("No claim-ladder.json. Expected before courts are sealed.");
        return ExitCode::SUCCESS;
    }
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            match serde_json::from_str::<autoconf_casefile_rs::ClaimLadder>(&contents) {
                Ok(ladder) => {
                    println!(
                        "Sealed: {}, Partial: {}, Unclaimed: {}, Known Failures: {}",
                        ladder.sealed_count,
                        ladder.partial_count,
                        ladder.unclaimed_count,
                        ladder.known_failure_count
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            eprintln!("Read error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_ast_verify() -> ExitCode {
    println!("=== AST Parity Verification ===\n");
    let profile_path = Path::new("reports/oracle-profile.json");
    if !profile_path.exists() {
        eprintln!("No oracle profile. Run 'cargo xtask oracle' first.");
        return ExitCode::FAILURE;
    }
    match ast_verify::AstParityBridge::new(profile_path) {
        Ok(bridge) => {
            let report = bridge.verify_all();
            report.print();
            if report.failed > 0 {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_behaviors_scan() -> ExitCode {
    println!("=== @ac_behavior Witness Scan ===\n");

    let src_dirs = &[
        "crates/autoconf-rs-core/src",
        "crates/autoconf-rs-cli/src",
        "crates/autoconf-oracle-rs/src",
    ];
    let mut total = 0;
    for dir in src_dirs {
        let path = Path::new(dir);
        if path.exists() {
            let witnesses = scan_directory_for_behaviors(path);
            total += witnesses.len();
            println!("{}: {} witness(es)", dir, witnesses.len());
            for w in &witnesses {
                println!(
                    "  - {} (surface: {}, manual: §{})",
                    w.id, w.surface, w.manual_section
                );
            }
        }
    }
    println!("\nTotal witnesses: {}", total);
    if total == 0 {
        println!("No @ac_behavior tags found. Add structured behavior docs to source files.");
    }
    ExitCode::SUCCESS
}

/// Scan a directory for @ac_behavior witness tags in source comments.
fn scan_directory_for_behaviors(dir: &Path) -> Vec<BehaviorWitness> {
    let mut witnesses = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    for line in contents.lines() {
                        if line.contains("@ac_behavior") {
                            let id = extract_attr(line, "id").unwrap_or("unknown");
                            let surface = extract_attr(line, "surface").unwrap_or("unknown");
                            let manual_section = extract_attr(line, "manual").unwrap_or("unknown");
                            witnesses.push(BehaviorWitness {
                                id: id.to_string(),
                                surface: surface.to_string(),
                                manual_section: manual_section.to_string(),
                                file: path.to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    witnesses
}

#[derive(Debug)]
#[allow(dead_code)]
struct BehaviorWitness {
    id: String,
    surface: String,
    manual_section: String,
    file: String,
}

fn extract_attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("{}=", key);
    if let Some(pos) = line.find(&pattern) {
        let rest = &line[pos + pattern.len()..];
        // Extract until whitespace, comma, or end
        let end = rest
            .find(|c: char| c.is_whitespace() || c == ',' || c == ')')
            .unwrap_or(rest.len());
        let value = rest[..end].trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

fn run_cleanroom_scan() -> ExitCode {
    cleanroom::run_scan()
}

fn run_status() -> ExitCode {
    println!("=== autoconf-rs project status ===\n");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Strategy: Clean-room behavioral reconstruction");
    println!("License: MIT OR Apache-2.0");
    println!("Methodology: Forensic parity — oracle admission → receipt-backed claims");
    println!();

    if Path::new("reports/oracle-profile.json").exists() {
        println!("Oracle: admitted ✓");
    } else {
        println!("Oracle: NOT YET ADMITTED — run 'cargo xtask oracle'");
    }

    let test_output = Command::new("cargo")
        .args(["test", "--all", "--", "--list"])
        .output();
    if let Ok(output) = test_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let test_count = stdout.lines().filter(|l| l.contains("test")).count();
        println!("Tests: {} found", test_count);
    }

    let receipts_dir = Path::new("reports/receipts");
    if receipts_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(receipts_dir) {
            let count = entries
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
                .count();
            println!("Receipts: {}", count);
        }
    } else {
        println!("Receipts: none yet");
    }

    if Path::new("reports/claim-ladder.json").exists() {
        println!("Claim ladder: present");
    } else {
        println!("Claim ladder: not yet generated");
    }

    if Path::new("reports/doc-registry.json").exists() {
        println!("\nDocument freshness: run 'cargo xtask check' for details.");
    }

    println!("\nIMPORTANT: autoconf-rs is NOT a GNU Autoconf replacement.");
    println!("See reports/FORENSIC-GAP-ANALYSIS.md for full gap details.");
    println!("See docs/negative-capabilities.md for the build roadmap.");
    println!("See docs/REVIEW-IN-10-MINUTES.md for a quick overview.");

    ExitCode::SUCCESS
}
