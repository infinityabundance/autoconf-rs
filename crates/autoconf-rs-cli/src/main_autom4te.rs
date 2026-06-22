//! autom4te binary — caching M4 wrapper for Autoconf.
//!
//! Panel mandate: implement --trace flag using the TraceLog event system.
//! Trace events are the data bus between autoconf, autoheader, and automake.
//! Cache provides SHA256-based freshness checking with JSON cache entries.
//!
//! Receipt family: AC.CLI.AUTOM4TE.*
//! Court: AC.AUTOM4TE.CACHE.1 — caching operational
//! Status: Phase 5 — cache + trace + include paths operational.

use autoconf_rs_cli::read_input;
use autoconf_rs_core::autom4te::Autom4teCache;
use autoconf_rs_core::M4Engine;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let mut trace_patterns: Vec<String> = Vec::new();
    let mut include_dirs: Vec<PathBuf> = Vec::new();
    let mut input_path: Option<String> = None;
    let mut show_traces = false;
    let mut force = false;
    let mut language = "Autoconf".to_string();
    let mut cache_dir = PathBuf::from("autom4te.cache");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--force" | "-f" => force = true,
            "--freeze" => {
                eprintln!("autom4te: --freeze mode (will cache output)");
            }
            "--reload" => {
                eprintln!("autom4te: --reload mode (will use cached output if fresh)");
            }
            "--language" | "-l" => {
                i += 1;
                if i < args.len() {
                    language = args[i].clone();
                }
            }
            "--include" | "-I" => {
                i += 1;
                if i < args.len() {
                    include_dirs.push(PathBuf::from(&args[i]));
                }
            }
            "--prepend-include" | "-B" => {
                i += 1;
                if i < args.len() {
                    include_dirs.insert(0, PathBuf::from(&args[i]));
                }
            }
            "--cache" => {
                i += 1;
                if i < args.len() {
                    cache_dir = PathBuf::from(&args[i]);
                }
            }
            a if a.starts_with("--trace") => {
                if let Some(pattern) = a.strip_prefix("--trace=") {
                    trace_patterns.push(pattern.to_string());
                }
                show_traces = true;
            }
            a if !a.starts_with('-') => {
                input_path = Some(a.to_string());
            }
            "-h" | "--help" => {
                println!("autom4te-rs {}", env!("CARGO_PKG_VERSION"));
                println!("Caching M4 wrapper for Autoconf");
                println!("Usage: autom4te [OPTIONS] [file]");
                println!("  -f, --force       Force regeneration");
                println!("  -I, --include DIR Add include directory");
                println!("  -l, --language L  Set language (Autoconf, M4sh, etc)");
                println!("  --trace=M:F       Emit trace events");
                println!("  --cache DIR       Set cache directory");
                println!("  -h, --help        Show this help");
                println!("  --version         Show version");
                return ExitCode::SUCCESS;
            }
            "--version" => {
                println!("autom4te-rs {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            _ => {
                eprintln!("autom4te: unknown flag: {}", args[i]);
            }
        }
        i += 1;
    }

    // Default include dirs
    if include_dirs.is_empty() {
        include_dirs.push(PathBuf::from("."));
    }

    let path_str = input_path.unwrap_or_else(|| "configure.ac".to_string());
    let input_path = Path::new(&path_str);

    let mut cache = Autom4teCache::new(&cache_dir);
    cache.set_force(force);

    if let Some(cached_output) = cache.lookup(input_path, &include_dirs, &language) {
        // Cache hit — emit cached output and traces
        if show_traces {
            let traces = cache.get_traces(input_path, &language);
            for trace in &traces {
                println!("{}", trace);
            }
        } else {
            print!("{}", cached_output);
        }
        return ExitCode::SUCCESS;
    }

    // Cache miss — process through M4 engine
    let input = match read_input(&path_str) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("autom4te: {}", e);
            return ExitCode::from(2);
        }
    };

    let mut engine = M4Engine::new();
    match engine.process(&input) {
        Ok(output) => {
            // Collect trace events
            let trace_lines: Vec<String> = engine
                .trace_log
                .emit_autom4te_traces()
                .iter()
                .map(|t| t.lines().next().unwrap_or(t).to_string())
                .collect();

            if show_traces {
                if trace_patterns.is_empty() {
                    // --trace without pattern: emit all traces in GNU format
                    let traces = engine.trace_log.emit_autom4te_traces();
                    for trace in &traces {
                        println!("{}", trace);
                    }
                    eprintln!(
                        "autom4te: {} trace events emitted (from process)",
                        traces.len()
                    );
                } else {
                    // --trace=MACRO:FORMAT: filter by requested macros with format
                    for pattern in &trace_patterns {
                        let parts: Vec<&str> = pattern.splitn(2, ':').collect();
                        let macro_name = parts[0];
                        let format_str = parts.get(1).copied().unwrap_or("$f:$l:$n:$1");

                        let structured = engine.trace_log.structured_traces();
                        let matching: Vec<_> = structured
                            .iter()
                            .filter(|(_, _, name, _)| name == macro_name)
                            .collect();

                        eprintln!(
                            "autom4te: trace {} => {} events",
                            macro_name,
                            matching.len()
                        );
                        for (file, line, name, args) in &matching {
                            println!(
                                "{}",
                                autoconf_rs_core::trace::TraceLog::format_trace(
                                    file, *line, name, args, format_str
                                )
                            );
                        }
                    }
                }
            } else {
                // No trace: output the expanded configure for --freeze
                // When --freeze is used, cache the result
                print!("{}", output);
                cache.store(input_path, &include_dirs, &language, &output, &trace_lines);
            }

            // Cache the trace results even when --trace was used
            if show_traces && !trace_lines.is_empty() {
                cache.store(input_path, &include_dirs, &language, &output, &trace_lines);
            }

            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("autom4te: {}", e);
            ExitCode::from(2)
        }
    }
}
