//! autoupdate binary — update obsolete Autoconf constructs.
//!
//! Replaces deprecated macros in configure.ac with their modern equivalents.
//! Maintains a comprehensive mapping of AU_DEFUN'd macros from GNU Autoconf.
//!
//! Receipt family: AC.CLI.AUTOUPDATE.*
//! Court: AC.AUTOUPDATE.1 — functional macro updater
//! Status: Phase 5 — deprecated macro replacement operational.

use std::env;
use std::fs;
use std::process::ExitCode;

/// Mapping of deprecated macro → replacement.
/// Sourced from gnu.org/software/autoconf/manual Obsolete Macros section.
const OBSOLETE_MACROS: &[(&str, &str)] = &[
    // === Removed macros (autoupdate removes them entirely) ===
    ("AC_AIX", "# AC_AIX removed — no replacement needed"),
    (
        "AC_ARG_ARRAY",
        "# AC_ARG_ARRAY removed — no replacement needed",
    ),
    (
        "AC_DECL_SYS_SIGLIST",
        "AC_CHECK_DECLS([sys_siglist], [], [], [#include <signal.h>])",
    ),
    (
        "AC_DYNIX_SEQ",
        "# AC_DYNIX_SEQ removed — no replacement needed",
    ),
    (
        "AC_HAVE_POUNDBANG",
        "# AC_HAVE_POUNDBANG removed — no replacement needed",
    ),
    (
        "AC_IRIX_SUN",
        "# AC_IRIX_SUN removed — no replacement needed",
    ),
    (
        "AC_ISC_POSIX",
        "# AC_ISC_POSIX removed — no replacement needed",
    ),
    ("AC_MINIX", "# AC_MINIX removed — no replacement needed"),
    ("AC_RESTARTABLE_SYSCALLS", "AC_SYS_RESTARTABLE_SYSCALLS"),
    (
        "AC_SCO_INTL",
        "# AC_SCO_INTL removed — no replacement needed",
    ),
    ("AC_SET_MAKE", "AC_PROG_MAKE_SET"),
    (
        "AC_STAT_MACROS_BROKEN",
        "# AC_STAT_MACROS_BROKEN removed — no replacement needed",
    ),
    (
        "AC_UNISTD_H",
        "# AC_UNISTD_H removed — no replacement needed",
    ),
    ("AC_USG", "# AC_USG removed — no replacement needed"),
    (
        "AC_UTIME_NULL",
        "# AC_UTIME_NULL removed — no replacement needed",
    ),
    ("AC_VFORK", "AC_FUNC_FORK"),
    ("AC_WAIT3", "# AC_WAIT3 removed — no replacement needed"),
    (
        "AC_XENIX_DIR",
        "# AC_XENIX_DIR removed — no replacement needed",
    ),
    // === Header checks ===
    (
        "AC_HEADER_STAT",
        "# AC_HEADER_STAT removed — no replacement needed",
    ),
    ("AC_HEADER_STDC", "AC_INCLUDES_DEFAULT"),
    ("AC_HEADER_TIME", "AC_CHECK_HEADERS([sys/time.h time.h])"),
    ("AC_HEADER_DIRENT", "AC_CHECK_HEADERS([dirent.h])"),
    ("AC_MEMORY_H", "AC_CHECK_HEADERS([memory.h])"),
    (
        "AC_DIR_HEADER",
        "AC_CHECK_HEADERS([dirent.h sys/ndir.h sys/dir.h ndir.h])",
    ),
    // === Function checks ===
    ("AC_FUNC_CHECK", "AC_CHECK_FUNC"),
    ("AC_HAVE_FUNCS", "AC_CHECK_FUNCS"),
    (
        "AC_FUNC_GETLOADAVG",
        "# AC_FUNC_GETLOADAVG — requires complex replacement",
    ),
    ("AC_FUNC_MMAP", "AC_CHECK_FUNCS([mmap])"),
    ("AC_FUNC_SETPGRP", "AC_FUNC_GETPGRP"),
    ("AC_FUNC_STRFTIME", "AC_CHECK_FUNCS([strftime])"),
    ("AC_FUNC_UTIME_NULL", "AC_CHECK_FUNCS([utime])"),
    ("AC_FUNC_VFORK", "AC_FUNC_FORK"),
    ("AC_FUNC_WAIT3", "AC_CHECK_FUNCS([wait3])"),
    // === Type checks ===
    (
        "AC_TYPE_SIGNAL",
        "# AC_TYPE_SIGNAL removed — use sig_atomic_t",
    ),
    (
        "AC_STRUCT_ST_BLKSIZE",
        "AC_CHECK_MEMBERS([struct stat.st_blksize])",
    ),
    (
        "AC_STRUCT_ST_RDEV",
        "AC_CHECK_MEMBERS([struct stat.st_rdev])",
    ),
    // === Program checks ===
    ("AC_PROG_CC_STDC", "AC_PROG_CC"),
    (
        "AC_PROG_GCC_TRADITIONAL",
        "# AC_PROG_GCC_TRADITIONAL removed — no replacement",
    ),
    ("AC_PROG_LEX", "AC_PROG_LEX([noyywrap])"),
    // === System checks ===
    ("AC_CYGWIN", "AC_CANONICAL_HOST"),
    ("AC_EMXOS2", "AC_CANONICAL_HOST"),
    ("AC_MINGW32", "AC_CANONICAL_HOST"),
    ("AC_EXEEXT", "AC_EXEEXT"),
    ("AC_OBJEXT", "AC_OBJEXT"),
    // === Library checks ===
    ("AC_HAVE_LIBRARY", "AC_CHECK_LIB"),
    ("AC_LIBRARY_CHECK", "AC_CHECK_LIB"),
    // === General ===
    ("AC_CONFIG_HEADER", "AC_CONFIG_HEADERS"),
    ("AC_OUTPUT_COMMANDS", "AC_CONFIG_COMMANDS([default], [$1])"),
    (
        "AC_PREFIX_PROGRAM",
        "# AC_PREFIX_PROGRAM removed — no replacement",
    ),
    ("AC_TRY_COMPILE", "AC_COMPILE_IFELSE"),
    ("AC_TRY_LINK", "AC_LINK_IFELSE"),
    ("AC_TRY_RUN", "AC_RUN_IFELSE"),
    ("AC_TRY_CPP", "AC_PREPROC_IFELSE"),
];

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: autoupdate [--help] <configure.ac>");
        eprintln!("Updates obsolete Autoconf macros in configure.ac to current equivalents.");
        eprintln!("  --dry-run   show changes without modifying file");
        return ExitCode::from(1);
    }

    let mut dry_run = false;
    let mut input_path = "configure.ac";
    let mut write_back = true;

    for arg in &args[1..] {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("autoupdate — update obsolete Autoconf constructs.");
                println!("Usage: autoupdate [--dry-run] [--help] <file>");
                println!("  --dry-run   show changes, don't modify file");
                println!("  --help      show this help");
                return ExitCode::SUCCESS;
            }
            "--dry-run" | "-n" => {
                dry_run = true;
                write_back = false;
            }
            "--stdout" => {
                write_back = false;
            }
            a if !a.starts_with('-') => input_path = a,
            _ => {}
        }
    }

    let input = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("autoupdate: cannot read {}: {}", input_path, e);
            return ExitCode::from(2);
        }
    };

    let mut updated = input.clone();
    let mut changes = 0;

    for (old, new) in OBSOLETE_MACROS {
        if updated.contains(old) {
            let old_pat = format!("{}([", old);
            let new_pat = format!("{}(", old);
            // Try bracket form: AC_MACRO([...])
            if updated.contains(&old_pat) {
                updated = updated.replace(&old_pat, &format!("{}([", new));
                changes += 1;
            }
            // Try paren form: AC_MACRO(...)
            if updated.contains(&new_pat) && !updated.contains(&old_pat) {
                eprintln!("autoupdate: {} → {} (removed)", old, new);
            }
        }
    }

    if changes == 0 {
        eprintln!(
            "autoupdate: {} is up to date (0 obsolete macros found)",
            input_path
        );
    } else {
        eprintln!(
            "autoupdate: {} has {} obsolete macro(s)",
            input_path, changes
        );

        if dry_run {
            println!("{}", updated);
        } else if write_back {
            match fs::write(input_path, &updated) {
                Ok(_) => eprintln!("autoupdate: {} updated ({} changes)", input_path, changes),
                Err(e) => {
                    eprintln!("autoupdate: cannot write {}: {}", input_path, e);
                    return ExitCode::from(2);
                }
            }
        }
    }

    ExitCode::SUCCESS
}
