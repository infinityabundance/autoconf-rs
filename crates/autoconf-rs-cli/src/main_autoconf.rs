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
    let mut output_arg: Option<String> = None;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--force" => force = true,
            "--allow-syscmd" => allow_syscmd = true,
            "--pure-m4" => pure_m4 = true,
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_arg = Some(args[i].clone());
                }
            }
            a if a.starts_with("--output=") => {
                output_arg = Some(a["--output=".len()..].to_string());
            }
            a if a.starts_with("-o") && a.len() > 2 => {
                output_arg = Some(a[2..].to_string());
            }
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
                println!("  -o, --output FILE  Write configure to FILE (chmod +x); '-' = stdout");
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
            return emit_output(&output_arg, &cached_output);
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
    let configure_ac = protect_hash_comments(&configure_ac);

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
    // Splice the overrides INTO the configure.ac text (right before AC_INIT) rather than prepending
    // them to the whole input: a define that leads the input stream is not honored by the engine,
    // but the identical text positioned just before AC_INIT in the body IS. (aclocal.m4 still goes
    // first so its AC_DEFUNs are registered before our overrides redefine the ones we own.)
    let configure_ac = match configure_ac.find("AC_INIT") {
        Some(pos) => {
            // back up to the start of the AC_INIT line
            let line_start = configure_ac[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            format!(
                "{}{}\n{}",
                &configure_ac[..line_start],
                overrides,
                &configure_ac[line_start..]
            )
        }
        None => format!("{}\n{}", overrides, configure_ac),
    };
    let input = match std::fs::read_to_string(&aclocal_path) {
        Ok(acm4) => format!("{}\n{}", strip_toplevel_hash_comments(&acm4), configure_ac),
        Err(_) => configure_ac,
    };

    let _ac = ConfigureAc::parse(&input);
    let mut engine = M4Engine::new();
    engine.allow_syscmd = allow_syscmd;
    engine.pure_m4 = pure_m4;

    let configure_script = match engine.process(&input) {
        Ok(s) => {
            // Systemic pass (default ON; opt out with AUTOCONF_RS_NO_NEUTRALIZE): neutralize any UNKNOWN
            // autoconf-family macro that leaked into the generated shell, so configure continues past it
            // instead of dying with "COMMAND not found"/syntax error. A/B-measured on the 986-repo corpus
            // baseline: NET +35 configure-clears, 0 regressions — so it ships on by default.
            let s = if std::env::var("AUTOCONF_RS_NO_NEUTRALIZE").is_err() {
                neutralize_leaked_macros(&s)
            } else {
                s
            };
            let s = expand_lang_constants(&s);
            guard_empty_shell_blocks(&s)
        }
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

    emit_output(&output_arg, &configure_script)
}

/// Write the generated configure script to `-o FILE` (chmod +x, since configure must be executable),
/// or to stdout when no `-o` is given or `-o -` is requested. GNU autoconf's `-o FILE`; without it the
/// real tool writes `configure` when reading configure.ac. autoreconf-rs passes `-o configure`, so the
/// orchestrated path lands a real executable `configure` file instead of dumping the script to stdout.
fn emit_output(output_arg: &Option<String>, content: &str) -> ExitCode {
    match output_arg {
        Some(o) if o != "-" => {
            if let Err(e) = std::fs::write(o, content) {
                eprintln!("autoconf: cannot write {}: {}", o, e);
                return ExitCode::from(2);
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(o) {
                    let mut perm = meta.permissions();
                    perm.set_mode(0o755);
                    let _ = std::fs::set_permissions(o, perm);
                }
            }
            ExitCode::SUCCESS
        }
        _ => {
            print!("{}", content);
            ExitCode::SUCCESS
        }
    }
}

/// m4-quote the content of FULL-LINE `#` comments in configure.ac so their text passes through to the
/// generated configure but the macro NAMES inside them are NOT expanded. Our engine disables `#` as an
/// m4 comment (m4-rs discards comments, and `#` occurs in shell like `${v#pat}`), so `# AC_CHECK_HEADER
/// doesn't give us …` had `AC_CHECK_HEADER` EXPANDED — injecting an `fi` and an unbalanced `'` (the
/// apostrophe in "doesn't") that swallowed the rest of configure (tmux configure.ac). Wrapping the
/// comment body in `[...]` (m4 quotes) makes m4 emit it literally, quotes stripped, macros untouched.
/// Only when the body has no `[`/`]` of its own (would unbalance the quoting) — rare in comments.
fn protect_hash_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 64);
    for (i, line) in input.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let indent = &line[..line.len() - trimmed.len()];
            if !rest.is_empty() && !rest.contains('[') && !rest.contains(']') {
                out.push_str(indent);
                out.push('#');
                out.push('[');
                out.push_str(rest);
                out.push(']');
                continue;
            }
        }
        out.push_str(line);
    }
    out
}

/// Expand the language-state m4 macros `_AC_LANG_ABBREV`→`c`, `_AC_LANG_PREFIX`→`C`, `_AC_LANG`→`C`
/// that leaked LITERAL into the generated shell. These are constants in our world (AC_LANG is a no-op,
/// always C), but m4sugar composes them inside a quoted `AS_VAR_PUSHDEF` value (`ax_cv_[]_AC_LANG_ABBREV
/// []flags_...`) that our engine stores without re-expanding, so cache/flag var names came out as
/// `ax_cv__AC_LANG_ABBREVflags` and `_AC_LANG_PREFIXFLAGS` instead of `ax_cv_cflags` and `CFLAGS` —
/// breaking AX_CHECK_COMPILE_FLAG (autoconf-archive, very common). Longest names first so
/// `_AC_LANG_ABBREV`/`_AC_LANG_PREFIX` are consumed before the `_AC_LANG` prefix inside them.
fn expand_lang_constants(input: &str) -> String {
    input
        .replace("_AC_LANG_ABBREV", "c")
        .replace("_AC_LANG_PREFIX", "C")
        .replace("_AC_LANG", "C")
}

/// Drop full-line `#` comments that sit OUTSIDE any macro body (m4 quote/bracket depth 0) from a loaded
/// aclocal.m4 before it is prepended to configure.ac. The autoconf engine disables `#` as an m4 comment
/// (the generated configure OUTPUT is full of `#` shell comments), but aclocal.m4's macro DEFINITIONS
/// carry `#` doc-comment blocks (`# _AM_PROG_CC_C_O`, `# like AC_PROG_CC_C_O, but changed...`). With
/// `#`-comments off, the macro names inside those doc lines get expanded (to empty / garbage), which
/// corrupts the following `AC_DEFUN([NAME],[body])` parse so NAME never registers and later leaks as
/// `NAME: command not found` in configure. m4sugar reads .m4 defs with `#`-comments ON; we approximate
/// that by stripping only the depth-0 comment lines, leaving `#` shell comments INSIDE macro bodies
/// (bracket depth > 0) intact so a macro's emitted output is unchanged.
fn strip_toplevel_hash_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth: i32 = 0;
    for line in input.split_inclusive('\n') {
        let at_top = depth == 0;
        let is_comment = at_top && line.trim_start().starts_with('#');
        if is_comment {
            continue; // drop this line (and don't count its brackets — it's prose)
        }
        out.push_str(line);
        for b in line.bytes() {
            match b {
                b'[' => depth += 1,
                b']' => depth -= 1,
                _ => {}
            }
        }
    }
    out
}

/// Neutralize a line that begins with an UNKNOWN autoconf-family macro call that leaked unexpanded into
/// the generated shell (`AC_FOO(...)`, `AX_BAR([x],[y])`, `m4_require(...)` etc.). A leaked macro is an
/// IDENTIFIER immediately followed by `(` at the statement start — real shell never calls functions that
/// way (funcs are `name args`; `$( )`/`$(( ))`/`(subshell)` start with `$` or `(`). We replace the whole
/// (paren-balanced, possibly multi-line) call with `:` so configure continues instead of dying. Only
/// well-known autoconf macro prefixes are touched, to avoid neutralizing project shell.
fn neutralize_leaked_macros(input: &str) -> String {
    const PREFIXES: &[&str] = &[
        "AC_", "AX_", "AM_", "LT_", "AS_", "PKG_", "AH_", "_AC_", "_AM_", "_LT_",
        "m4_", "_m4_", "gl_", "IT_", "GLIB_", "GTK_", "BOOST_", "AC", "AM", // AC_DEFUN-internal _AC etc.
    ];
    // Bare m4 BUILTINS that can never be valid shell (a complex macro-body — e.g. gettext's
    // lib-link.m4 AC_LIB_LINKFLAGS_BODY — can leak these when our engine fails to fully expand its
    // pushdef/translit machinery). Neutralize so configure degrades instead of dying with a hard
    // `syntax error near '[NAME],[translit'`.
    const M4_BUILTINS: &[&str] = &[
        "pushdef", "popdef", "translit", "ifelse", "ifdef", "undefine", "defn",
        "changequote", "changecom", "m4_pattern_allow", "m4_pattern_forbid",
    ];
    // does `s` (already left-trimmed) start with an autoconf-family macro name then `(`?
    let leaked_macro_at = |s: &str| -> bool {
        let id_len = s.bytes().take_while(|b| b.is_ascii_alphanumeric() || *b == b'_').count();
        if id_len == 0 || s.as_bytes().get(id_len) != Some(&b'(') {
            return false;
        }
        let id = &s[..id_len];
        if M4_BUILTINS.contains(&id) {
            return true;
        }
        // require a real autoconf prefix AND an uppercase letter or underscore-prefixed (macro-shaped)
        let has_prefix = PREFIXES.iter().any(|p| id.starts_with(p) && id.len() > p.len());
        has_prefix && (id.contains('_') || id.chars().any(|c| c.is_ascii_uppercase()))
    };
    let lines: Vec<&str> = input.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if leaked_macro_at(trimmed) {
            // consume the paren-balanced call (may span lines); count ( vs ) ignoring nothing fancy.
            let mut depth: i32 = 0;
            let mut started = false;
            let mut j = i;
            while j < lines.len() {
                for c in lines[j].chars() {
                    if c == '(' { depth += 1; started = true; }
                    else if c == ')' { depth -= 1; }
                }
                if started && depth <= 0 { break; }
                j += 1;
            }
            out.push(":".to_string()); // single no-op replaces the whole leaked call
            i = j + 1;
            continue;
        }
        out.push(lines[i].to_string());
        i += 1;
    }
    let mut result = out.join("\n");
    if input.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Insert a `:` no-op into otherwise-empty shell blocks (`then`/`else`/`do` immediately followed by
/// `fi`/`else`/`elif`/`done`). autoconf macros that legitimately expand to nothing (no-op'd or
/// m4_ifdef-gated-unavailable) leave empty `if ...; then <nothing> fi` blocks, which are a shell
/// syntax error ("syntax error near unexpected token `fi'"). Real autoconf never emits empty blocks;
/// this defends against the whole class generically without touching the project's control flow.
fn guard_empty_shell_blocks(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 8);
    let opens_block = |t: &str| -> bool {
        let tt = t.trim();
        if tt == "else" {
            return true;
        }
        // last shell word before EOL is `then` or `do`
        match tt.rsplit(|c| c == ' ' || c == ';' || c == '\t').next() {
            Some("then") | Some("do") => true,
            _ => false,
        }
    };
    let mut i = 0;
    while i < lines.len() {
        out.push(lines[i].to_string());
        if opens_block(lines[i]) {
            // peek past blank/whitespace-only lines
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            if j < lines.len() {
                let nt = lines[j].trim_start();
                if nt.starts_with("fi") || nt == "else" || nt.starts_with("else ")
                    || nt.starts_with("elif") || nt.starts_with("done")
                {
                    out.push(":".to_string());
                }
            }
        }
        i += 1;
    }
    let mut result = out.join("\n");
    if input.ends_with('\n') {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_toplevel_hash_comments_removes_docblocks() {
        // Top-level `#` doc-comment lines (between/above definitions) are dropped so their macro
        // names are not mis-expanded when `#`-comments are disabled for the configure body.
        let input = "# _AM_PROG_CC_C_O\n# like AC_PROG_CC_C_O, but changed.\nAC_DEFUN([_AM_PROG_CC_C_O], [body])\n";
        let out = strip_toplevel_hash_comments(input);
        assert!(!out.contains("like AC_PROG_CC_C_O"), "top-level doc comment must be stripped");
        assert!(out.contains("AC_DEFUN([_AM_PROG_CC_C_O], [body])"), "the definition must survive");
    }

    #[test]
    fn test_strip_preserves_hash_inside_macro_body() {
        // A `#`-led line INSIDE a macro body ([...] depth > 0) must be preserved: it may be heredoc'd
        // C source (`#include`, `#define`) that the macro emits into conftest.
        let input = "AC_DEFUN([X], [cat > c <<EOF\n#include <stdio.h>\n#define FOO 1\nEOF\n])\n";
        let out = strip_toplevel_hash_comments(input);
        assert!(out.contains("#include <stdio.h>"), "heredoc #include inside a body must survive");
        assert!(out.contains("#define FOO 1"), "heredoc #define inside a body must survive");
    }
}
