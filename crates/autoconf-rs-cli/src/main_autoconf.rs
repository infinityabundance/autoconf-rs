//! autoconf binary — generate configure scripts from configure.ac.
//!
//! Uses autom4te caching (Autom4teCache) for performance. On cache hit,
//! returns cached output without re-expanding M4. On cache miss or --force,
//! expands and caches the result.
//!
//! Receipt family: AC.CLI.AUTOCONF.*
//! Court: AC.AUTOM4TE.CACHE.1 — caching integrated
//! Current status: Phase 5 — caching + template dispatch + trace events.

use autoconf_rs_cli::read_input;
use autoconf_rs_core::autom4te::Autom4teCache;
use autoconf_rs_core::{ConfigureAc, M4Engine};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut input_arg: Option<&str> = None;
    let mut force = false;
    let mut include_dirs: Vec<PathBuf> = Vec::new();
    let mut cache_dir = PathBuf::from("autom4te.cache");
    let mut warnings: Vec<String> = Vec::new();

    let mut i = 1;
    let mut allow_syscmd = false;
    let mut pure_m4 = false;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--force" => force = true,
            "--allow-syscmd" => allow_syscmd = true,
            "--pure-m4" => pure_m4 = true,
            "-I" | "--include" => {
                i += 1;
                if i < args.len() {
                    include_dirs.push(PathBuf::from(&args[i]));
                }
            }
            "-B" | "--prepend-include" => {
                i += 1;
                if i < args.len() {
                    include_dirs.insert(0, PathBuf::from(&args[i]));
                }
            }
            "-W" | "--warnings" => {
                i += 1;
                if i < args.len() {
                    warnings.push(args[i].clone());
                }
            }
            "--cache" => {
                i += 1;
                if i < args.len() {
                    cache_dir = PathBuf::from(&args[i]);
                }
            }
            a if !a.starts_with('-') => input_arg = Some(a),
            "-h" | "--help" => {
                println!("autoconf-rs {}", env!("CARGO_PKG_VERSION"));
                println!("Generate configure scripts from configure.ac");
                println!("Usage: autoconf [OPTIONS] [configure.ac]");
                println!("  -f, --force        Force regeneration");
                println!("  -I, --include DIR  Add include directory");
                println!("  -W, --warnings CAT Enable warning category");
                println!("  -h, --help         Show this help");
                println!("  --version          Show version");
                println!("  --pure-m4          Use raw M4 expansion (skip prescan+template)");
                return ExitCode::SUCCESS;
            }
            "--version" => {
                println!("autoconf-rs {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            _ => {}
        }
        i += 1;
    }

    let path = input_arg.unwrap_or("configure.ac").to_string();
    let input_path = Path::new(&path);

    // Default include dirs
    if include_dirs.is_empty() {
        include_dirs.push(PathBuf::from("."));
    }

    // Check cache before processing
    let mut cache = Autom4teCache::new(&cache_dir);
    cache.set_force(force);

    if !force {
        if let Some(cached_output) = cache.lookup(input_path, &include_dirs, "Autoconf") {
            print!("{}", cached_output);
            return ExitCode::SUCCESS;
        }
    }

    // Cache miss — process through M4 engine
    let input = match read_input(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("autoconf: {}", e);
            return ExitCode::from(2);
        }
    };

    let _ac = ConfigureAc::parse(&input);
    let mut engine = M4Engine::new();
    engine.allow_syscmd = allow_syscmd;
    engine.pure_m4 = pure_m4;

    let configure_script = match engine.process(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("autoconf: M4 error: {}", e);
            return ExitCode::from(2);
        }
    };

    // Cache the result for future runs
    let trace_lines: Vec<String> = engine
        .trace_log
        .emit_autom4te_traces()
        .iter()
        .map(|t| t.lines().next().unwrap_or(t).to_string())
        .collect();
    cache.store(
        input_path,
        &include_dirs,
        "Autoconf",
        &configure_script,
        &trace_lines,
    );

    print!("{}", configure_script);
    ExitCode::SUCCESS
}
