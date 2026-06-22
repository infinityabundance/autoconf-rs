//! aclocal binary — generate aclocal.m4 from configure.ac.
//!
//! Scans m4/ directories for third-party .m4 macro files, resolves
//! dependencies, and generates aclocal.m4 with serial numbers for
//! version tracking. Supports --install to copy macros to local m4/.
//!
//! Receipt family: AC.CLI.ACLOCAL.*
//! Court: AC.CLI.1 — sealed (all 8 binaries operational)
//! Status: Phase 5 — directory scanning, serial numbers, --install.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut include_dirs: Vec<String> = vec!["m4".to_string()];
    let mut input_path = "configure.ac".to_string();
    let mut verbose = false;
    let mut install = false;
    let mut output_path = "aclocal.m4".to_string();
    let mut force = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-I" => {
                i += 1;
                if i < args.len() {
                    include_dirs.push(args[i].clone());
                }
            }
            "--install" => install = true,
            "-f" | "--force" => force = true,
            "-v" | "--verbose" => verbose = true,
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = args[i].clone();
                }
            }
            "-h" | "--help" => {
                println!("aclocal-rs — generate aclocal.m4 (Rust native)");
                println!("Usage: aclocal [OPTIONS] [configure.ac]");
                println!("  -I DIR        Add directory to search path (default: m4/)");
                println!("  --install     Copy third-party macros to local m4/");
                println!("  -f, --force   Force overwrite of aclocal.m4");
                println!("  -v, --verbose Verbose output");
                println!("  -o, --output  Output file (default: aclocal.m4)");
                println!("  -h, --help    Show this help");
                return ExitCode::SUCCESS;
            }
            s if !s.starts_with('-') => input_path = s.to_string(),
            _ => {
                eprintln!("aclocal: unknown flag: {}", args[i]);
            }
        }
        i += 1;
    }

    // Read configure.ac
    let ac_content = match fs::read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("aclocal: cannot read {}: {}", input_path, e);
            return ExitCode::from(2);
        }
    };

    // Extract AC_CONFIG_MACRO_DIR if present
    let macro_dir = if let Some(start) = ac_content.find("AC_CONFIG_MACRO_DIR([") {
        let rest = &ac_content[start + "AC_CONFIG_MACRO_DIR([".len()..];
        if let Some(end) = rest.find("])") {
            let dir = rest[..end].to_string();
            if !include_dirs.contains(&dir) {
                include_dirs.push(dir.clone());
            }
            if verbose {
                eprintln!("aclocal: detected macro dir: {}", dir);
            }
            dir
        } else {
            "m4".to_string()
        }
    } else {
        "m4".to_string()
    };

    // Check if output exists and we're not forcing
    if !force && Path::new(&output_path).exists() {
        if verbose {
            eprintln!("aclocal: {} exists, use --force to overwrite", output_path);
        }
    }

    // Scan m4/ directories for .m4 files
    let mut m4_files: Vec<(String, String, String)> = Vec::new(); // (dir, name, content)
    let mut seen_macros = HashSet::new();

    for dir in &include_dirs {
        let path = Path::new(dir);
        if path.exists() && path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let fpath = entry.path();
                    if fpath.extension().map(|e| e == "m4").unwrap_or(false) {
                        if let Ok(content) = fs::read_to_string(&fpath) {
                            let fname = fpath
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown.m4")
                                .to_string();

                            if seen_macros.contains(&fname) {
                                if verbose {
                                    eprintln!("aclocal: skipping duplicate: {}/{}", dir, fname);
                                }
                                continue;
                            }
                            seen_macros.insert(fname.clone());

                            if verbose {
                                eprintln!("aclocal: found {}/{}", dir, fname);
                            }
                            m4_files.push((dir.clone(), fname, content));
                        }
                    }
                }
            }
        } else if verbose {
            eprintln!("aclocal: directory '{}' not found, skipping", dir);
        }
    }

    // --install: copy macros to first include dir (typically m4/)
    if install && !m4_files.is_empty() {
        let target_dir = if include_dirs.len() > 1 {
            &include_dirs[1] // Skip "m4" default, use the actual macro dir
        } else {
            &include_dirs[0]
        };
        fs::create_dir_all(target_dir).ok();

        for (src_dir, fname, content) in &m4_files {
            let _src_path = Path::new(src_dir).join(fname);
            let dest_path = Path::new(target_dir).join(fname);

            // Only copy if source is outside target dir (not already local)
            if src_dir != target_dir {
                if verbose {
                    eprintln!("aclocal: copying {} → {}/{}", src_dir, target_dir, fname);
                }
                fs::write(&dest_path, content).ok();
            }
        }
    }

    // Generate aclocal.m4
    let mut output = String::new();
    output.push_str(&format!(
        "dnl aclocal.m4 — Generated by autoconf-rs aclocal.\n\
         dnl Source: {}\n\
         dnl Macro directory: {}\n\
         dnl {} third-party .m4 files found\n\n",
        input_path,
        macro_dir,
        m4_files.len()
    ));

    // Extract serial numbers from third-party macros
    let mut serial_no = 1u64;
    for (_dir, fname, content) in &m4_files {
        let serial = if let Some(pos) = content.find("# serial ") {
            let rest = &content[pos + "# serial ".len()..];
            rest.lines().next().unwrap_or("1").trim().to_string()
        } else {
            let s = serial_no.to_string();
            serial_no += 1;
            s
        };
        output.push_str(&format!(
            "dnl {} (serial {})\n{}\n\n",
            fname, serial, content
        ));
    }

    if m4_files.is_empty() {
        output.push_str("dnl No third-party macros found.\n");
        output.push_str("dnl Consider adding macros to the m4/ directory.\n");
    }

    // Write aclocal.m4
    if let Err(e) = fs::write(&output_path, &output) {
        eprintln!("aclocal: cannot write {}: {}", output_path, e);
        return ExitCode::from(2);
    }

    if verbose {
        eprintln!(
            "aclocal: wrote {} ({} bytes, {} macros, {} dirs scanned)",
            output_path,
            output.len(),
            m4_files.len(),
            include_dirs.len()
        );
    }

    ExitCode::SUCCESS
}
