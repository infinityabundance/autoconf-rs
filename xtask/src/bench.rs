// bench.rs — Performance baseline: autoconf-rs vs GNU Autoconf.
//
// Measures wall-clock execution time for both autoconf-rs and the
// GNU Autoconf oracle on the same configure.ac files, reporting
// speedup ratio. Uses deterministic fixtures for reproducibility.
//
// Court: AC.PERF.1

use std::process::{Command, ExitCode};
use std::time::Instant;

/// Fixtures with varying complexity levels for benchmarking.
const BENCH_FIXTURES: &[(&str, &str)] = &[
    ("minimal", "AC_INIT([bench-min],[1.0])\nAC_OUTPUT\n"),
    (
        "with_subst",
        "AC_INIT([bench-subst],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_SUBST([CC],[gcc])\nAC_SUBST([LIBS],[-lm])\nAC_OUTPUT\n",
    ),
    (
        "with_checks",
        "AC_INIT([bench-chk],[1.0])\nAC_PROG_CC\nAC_CHECK_FUNCS([malloc free realloc])\nAC_CHECK_HEADERS([stdlib.h string.h])\nAC_OUTPUT\n",
    ),
];

pub fn run() -> ExitCode {
    println!("=== autoconf-rs Performance Benchmark ===");
    println!();

    let rs_bin = std::env::current_dir()
        .unwrap_or_default()
        .join("target/release/autoconf");
    let oracle_bin = find_oracle();

    if !rs_bin.exists() {
        println!("ERROR: autoconf-rs binary not found at {:?}", rs_bin);
        println!("Run: cargo build --release -p autoconf-rs-cli");
        return ExitCode::from(1);
    }

    if oracle_bin.is_none() {
        println!("WARNING: GNU Autoconf oracle not found. Only measuring autoconf-rs.");
        println!(
            "Install GNU Autoconf for comparison: apt install autoconf / brew install autoconf"
        );
        println!();
    }

    let tmp = std::env::temp_dir().join("ac_bench");
    let _ = std::fs::create_dir_all(&tmp);

    let mut rs_total_ms = 0f64;
    let mut oracle_total_ms = 0f64;
    let rounds = 5;

    println!(
        "{:<20} {:>8} {:>8} {:>10}",
        "Fixture", "autoconf-rs", "GNU AC", "Speedup"
    );
    println!("{:-<52}", "");

    for (name, content) in BENCH_FIXTURES {
        let ac_path = tmp.join(format!("{}.ac", name));
        std::fs::write(&ac_path, content).unwrap();

        // Benchmark autoconf-rs
        let mut rs_elapsed = 0f64;
        for _ in 0..rounds {
            let start = Instant::now();
            let _ = Command::new(&rs_bin).arg(&ac_path).output();
            rs_elapsed += start.elapsed().as_secs_f64();
        }
        let rs_avg = rs_elapsed / rounds as f64 * 1000.0;
        rs_total_ms += rs_avg;

        // Benchmark GNU oracle
        let mut oracle_avg = 0f64;
        if let Some(ref oracle) = oracle_bin {
            let mut oracle_elapsed = 0f64;
            for _ in 0..rounds {
                let start = Instant::now();
                let _ = Command::new(oracle).arg(&ac_path).output();
                oracle_elapsed += start.elapsed().as_secs_f64();
            }
            oracle_avg = oracle_elapsed / rounds as f64 * 1000.0;
            oracle_total_ms += oracle_avg;
        }

        let speedup = if oracle_avg > 0.0 {
            oracle_avg / rs_avg
        } else {
            0.0
        };

        let speedup_str = if speedup > 0.0 {
            format!("{:.1}x", speedup)
        } else {
            "N/A".to_string()
        };

        println!(
            "{:<20} {:>7.1}ms {:>7.1}ms {:>9}",
            name, rs_avg, oracle_avg, speedup_str
        );
    }

    println!("{:-<52}", "");
    println!(
        "{:<20} {:>7.1}ms {:>7.1}ms",
        "TOTAL", rs_total_ms, oracle_total_ms
    );

    if rs_total_ms > 0.0 && oracle_total_ms > 0.0 {
        println!("Overall speedup: {:.1}x", oracle_total_ms / rs_total_ms);
    }

    let _ = std::fs::remove_dir_all(&tmp);
    ExitCode::SUCCESS
}

fn find_oracle() -> Option<std::path::PathBuf> {
    for name in &["autoconf", "/usr/bin/autoconf", "/usr/local/bin/autoconf"] {
        let p = std::path::PathBuf::from(name);
        if p.exists() || Command::new("which").arg(name).output().is_ok() {
            // Try running it
            if let Ok(out) = Command::new(name).arg("--version").output() {
                let version = String::from_utf8_lossy(&out.stdout);
                println!("Oracle: {}", version.lines().next().unwrap_or("unknown"));
                return Some(p);
            }
        }
    }
    None
}
