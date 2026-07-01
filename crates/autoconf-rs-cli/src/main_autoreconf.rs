//! autoreconf binary — orchestrate autotools chain.
//!
//! Runs autoconf, autoheader, aclocal, and automake in the correct order.
//! Handles directory traversal, force regeneration, and verbose output.
//!
//! Receipt family: AC.CLI.AUTORECONF.*
//! Court: AC.CLI.1 — sealed (all 8 binaries operational)
//! Status: Phase 5 — full orchestration with proper tool detection.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut dir = ".".to_string();
    let mut verbose = false;
    let mut force = false;
    let mut install = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "-v" | "--verbose" => verbose = true,
            "-f" | "--force" => force = true,
            "-i" | "--install" => install = true,
            "-h" | "--help" => {
                println!("autoreconf-rs — autotools orchestration (Rust native)");
                println!("Usage: autoreconf [OPTIONS] [DIRECTORY]");
                println!("  -v, --verbose  Verbose output");
                println!("  -f, --force    Force regeneration of all files");
                println!("  -i, --install  Copy missing auxiliary files");
                println!("  -h, --help     Show this help");
                return ExitCode::SUCCESS;
            }
            s if !s.starts_with('-') => dir = s.to_string(),
            _ => {}
        }
    }

    let target_dir = Path::new(&dir);
    if !target_dir.exists() {
        eprintln!("autoreconf: directory '{}' not found", dir);
        return ExitCode::from(2);
    }

    let configure_ac = target_dir.join("configure.ac");
    let configure_in = target_dir.join("configure.in");

    if !configure_ac.exists() && !configure_in.exists() {
        eprintln!(
            "autoreconf: no configure.ac or configure.in found in '{}'",
            dir
        );
        return ExitCode::from(2);
    }

    let input_file = if configure_ac.exists() {
        "configure.ac"
    } else {
        "configure.in"
    };

    if verbose {
        println!("autoreconf: processing {} in {}", input_file, dir);
    }

    // Find binary paths — look relative to our own binary first
    let our_bin = std::env::current_exe().unwrap_or_default();
    let bin_dir = our_bin.parent().unwrap_or(Path::new("."));

    fn find_tool(bin_dir: &Path, name: &str) -> PathBuf {
        let local = bin_dir.join(name);
        if local.exists() {
            return local;
        }
        // Try PATH
        PathBuf::from(name)
    }

    let autoconf_bin = find_tool(bin_dir, "autoconf");
    let autoheader_bin = find_tool(bin_dir, "autoheader");
    let aclocal_bin = find_tool(bin_dir, "aclocal");

    // Step 1: Run aclocal if m4/ directory exists or force
    let m4_dir = target_dir.join("m4");
    let aclocal_m4 = target_dir.join("aclocal.m4");
    if m4_dir.exists() || force || !aclocal_m4.exists() {
        if verbose {
            println!("autoreconf: running aclocal...");
        }
        let mut cmd = Command::new(&aclocal_bin);
        cmd.arg("-I").arg("m4");
        if force {
            cmd.arg("--force");
        }
        if install {
            cmd.arg("--install");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.current_dir(target_dir);
        let status = cmd.status();
        if verbose {
            match status {
                Ok(s) => println!("  aclocal exit: {:?}", s.code()),
                Err(e) => eprintln!("  aclocal error: {}", e),
            }
        }
    }

    // Step 2: Run autoconf
    if verbose {
        println!("autoreconf: running autoconf-rs...");
    }
    let mut ac_cmd = Command::new(&autoconf_bin);
    ac_cmd.arg(input_file);
    // Capture autoconf's output into an executable `configure` file (GNU autoreconf always leaves a
    // `configure` behind). Without `-o`, autoconf-rs prints the script to stdout, which autoreconf's
    // inherited stdout dumped to the terminal -> no `configure` was ever written, so every git-checkout
    // repo (no shipped tarball `configure`) failed at the "run ./configure" step.
    ac_cmd.arg("-o").arg("configure");
    if force {
        ac_cmd.arg("--force");
    }
    ac_cmd.current_dir(target_dir);
    let status = ac_cmd.status();

    match status {
        Ok(s) if s.success() => {
            if verbose {
                println!("  autoconf-rs: OK");
            }
        }
        Ok(s) => {
            eprintln!("autoreconf: autoconf failed with exit code {:?}", s.code());
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("autoreconf: autoconf error: {}", e);
            return ExitCode::from(1);
        }
    }

    // Step 3: Run autoheader if AC_CONFIG_HEADERS present
    let ac_content = std::fs::read_to_string(target_dir.join(input_file)).unwrap_or_default();
    // `AC_CONFIG_HEADER` (no S) matches BOTH the modern plural and the legacy singular / `AM_CONFIG_HEADER`
    // forms, so autoheader runs for old-style configure.ac too (else config.h.in is never made ->
    // `make` dies with `config.h: No such file`, e.g. redir/rmark's `AC_CONFIG_HEADER([config.h])`).
    if ac_content.contains("AC_CONFIG_HEADER") || force {
        if verbose {
            println!("autoreconf: running autoheader...");
        }
        let mut ah_cmd = Command::new(&autoheader_bin);
        ah_cmd.arg(input_file);
        ah_cmd.current_dir(target_dir);
        let status = ah_cmd.status();
        if verbose {
            match status {
                Ok(s) => println!("  autoheader exit: {:?}", s.code()),
                Err(e) => eprintln!("  autoheader error: {}", e),
            }
        }
    }

    // Step 4: Run automake if Makefile.am exists
    let makefile_am = target_dir.join("Makefile.am");
    if makefile_am.exists() || force {
        if verbose {
            println!("autoreconf: running automake...");
        }
        let mut am_cmd = Command::new("automake");
        am_cmd.arg("--add-missing");
        am_cmd.arg("--copy");
        if force {
            am_cmd.arg("--force-missing");
        }
        am_cmd.current_dir(target_dir);
        let status = am_cmd.status();
        if verbose {
            match status {
                Ok(s) => println!("  automake exit: {:?}", s.code()),
                Err(e) => eprintln!("  automake error: {}", e),
            }
        }
    }

    if verbose {
        println!("autoreconf: done");
    }

    ExitCode::SUCCESS
}
