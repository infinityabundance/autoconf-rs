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
    // Deeply-nested M4 quotes/macros expand via recursion; run on a large stack so a pathological but
    // finite input fails gracefully (or completes) instead of overflowing the 8 MB default and aborting.
    std::thread::Builder::new()
        .stack_size(1024 * 1024 * 1024)
        .spawn(run)
        .ok()
        .and_then(|h| h.join().ok())
        .unwrap_or(ExitCode::from(2))
}

fn run() -> ExitCode {
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
    let configure_ac = match read_input(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("autoconf: {}", e);
            return ExitCode::from(2);
        }
    };

    // Prepend aclocal.m4 (if present beside configure.ac). GNU autoconf always includes aclocal.m4
    // before the configure.ac body: it carries the AC_DEFUN definitions for AM_*, AX_*, gl_*, PKG_*,
    // LT_* and other third-party macros gathered from the project's m4/ dir. Without this, those
    // macro calls were left literal -> shell "syntax error" / "COMMAND: command not found". The
    // AC_DEFUN bodies expand to nothing, so prepending only registers macros (no stray output).
    let aclocal_path = Path::new(&path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("aclocal.m4");
    // Macro OVERRIDES injected AFTER aclocal.m4 but BEFORE configure.ac, so they win over the
    // project's third-party definitions (pkg.m4 etc.) that autoconf-rs cannot yet expand correctly
    // (the real pkg.m4 leaks `pkg_default`/`glib_minimum` -> shell syntax error). We emit clean,
    // self-contained shell instead. Real pkg-config runs at configure time and sets PFX_CFLAGS/LIBS.
    let overrides = autoconf_rs_core::macro_overrides();
    let input = match std::fs::read_to_string(&aclocal_path) {
        Ok(acm4) => format!("{}\n{}\n{}", acm4, overrides, configure_ac),
        Err(_) => format!("{}\n{}", overrides, configure_ac),
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
