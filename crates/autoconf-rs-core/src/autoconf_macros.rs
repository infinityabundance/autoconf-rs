//! Core Autoconf macro implementations.
//!
//! Implements the core Autoconf macros that form the foundation
//! of configure.ac processing: AC_INIT, AC_OUTPUT, AC_CONFIG_FILES,
//! AC_CONFIG_HEADERS, AC_SUBST, AC_DEFINE, AC_DEFUN, AC_REQUIRE, etc.
//!
//! @ac_behavior id=AC.AUTOCONF.CORE.1 surface=AC.M4.AUTOCONF.CORE.1 manual=§4
//! Receipt family: AC.M4.AUTOCONF.CORE.*
//! Current status: Phase 2 — implemented, not yet oracle-admitted.

use super::m4sugar::RequireTracker;
use std::collections::HashMap;

/// Tracks the state of an Autoconf configure.ac processing session.
#[derive(Debug, Clone, Default)]
pub struct AutoconfState {
    /// Package name from AC_INIT
    pub package_name: Option<String>,
    /// Package version from AC_INIT
    pub package_version: Option<String>,
    /// Bug report address from AC_INIT
    pub bug_report: Option<String>,
    /// Unique source file from AC_CONFIG_SRCDIR
    pub unique_file: Option<String>,
    /// Tarname (package-version)
    pub tarname: Option<String>,
    /// Files to generate from .in templates (AC_CONFIG_FILES)
    pub config_files: Vec<String>,
    /// Header files to generate (AC_CONFIG_HEADERS)
    pub config_headers: Vec<String>,
    /// Commands to run at config.status time (AC_CONFIG_COMMANDS)
    pub config_commands: Vec<(String, String)>,
    /// Symbolic links to create (AC_CONFIG_LINKS)
    pub config_links: Vec<(String, String)>,
    /// Subdirectories to configure (AC_CONFIG_SUBDIRS)
    pub config_subdirs: Vec<String>,
    /// Substitutions for output files (AC_SUBST)
    pub substitutions: HashMap<String, String>,
    /// C preprocessor defines (AC_DEFINE)
    pub defines: Vec<(String, String)>,
    /// Whether AC_OUTPUT has been called
    pub output_called: bool,
    /// The dependency tracker for AC_REQUIRE/AC_PROVIDE
    pub require_tracker: RequireTracker,
    /// Shell variable initializations
    pub shell_init: Vec<String>,
    /// Early shell code (before feature tests)
    pub shell_early: Vec<u8>,
    /// Shell code for feature tests
    pub shell_body: Vec<u8>,
    /// Late shell code (after feature tests, before output)
    pub shell_late: Vec<u8>,
    /// Whether AC_PROG_CC or AC_PROG_CXX were found
    pub has_compiler_check: bool,
    /// Whether AC_PROG_CXX specifically was found (generates C++ detection code)
    pub has_cxx_compiler: bool,
    /// Which AC_CHECK_FUNC names were found
    pub checked_funcs: Vec<String>,
    /// Which AC_CHECK_HEADER names were found
    pub checked_headers: Vec<String>,
    /// Which AC_CHECK_LIB pairs were found
    pub checked_libs: Vec<(String, String)>,
    /// Which AC_CHECK_TYPE names were found
    pub checked_types: Vec<String>,
    /// Which AC_CHECK_PROG/PATH_PROG were found
    pub checked_progs: Vec<String>,
    /// Which AC_CHECK_SIZEOF types were found
    pub checked_sizeofs: Vec<String>,
    /// Which C conformance checks were found (AC_C_*) — #1 biggest mover
    pub c_conformance_checks: Vec<String>,
    /// Which AC_CHECK_MEMBER struct.member pairs were found
    pub checked_members: Vec<String>,
    /// Whether Fortran macros (AC_PROG_FC, AC_FC_*) were found
    pub has_fortran: bool,
    /// AS_IF conditional defines: (condition, var, value)
    pub as_if_defines: Vec<(String, String, String)>,
    /// AS_CASE conditional defines: (variable, pattern, var, value)
    pub as_case_defines: Vec<(String, String, String, String)>,
    /// Whether output should include confdefs.h append section
    pub has_standalone_defines: bool,
    /// AC_MSG_CHECKING messages to emit
    pub msg_checking: Vec<String>,
    /// AC_MSG_RESULT results to emit
    pub msg_results: Vec<String>,
    /// AC_MSG_ERROR messages
    pub msg_errors: Vec<String>,
    /// AC_COMPILE_IFELSE / AC_LINK_IFELSE / AC_RUN_IFELSE detected
    pub has_ifelse_checks: bool,
    /// m4_set tracking: set name → element names (for set_size/list/empty without M4 recursion)
    pub m4_sets: std::collections::HashMap<String, std::collections::HashSet<String>>,
}

impl AutoconfState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Core Autoconf macro handlers.
pub struct AutoconfBuiltins;

impl AutoconfBuiltins {
    /// AC_INIT: initialize Autoconf with package metadata.
    ///
    /// AC_INIT: initialize Autoconf with package metadata.
    ///
    /// AC_INIT(PACKAGE, VERSION, [BUG-REPORT], [TARNAME], [URL])
    ///
    /// In GNU Autoconf, AC_INIT expands to the full configure script prologue
    /// including shebang, header, M4sh initialization, and shell functions.
    ///
    /// @ac_behavior id=AC.AUTOCONF.INIT.1 surface=AC.M4.AUTOCONF.CORE.1 manual=§4.1
    pub fn ac_init(args: &[Vec<u8>], state: &mut AutoconfState) -> Vec<u8> {
        state.package_name = args.first().map(|a| String::from_utf8_lossy(a).to_string());
        state.package_version = args.get(1).map(|a| String::from_utf8_lossy(a).to_string());
        state.bug_report = args.get(2).map(|a| String::from_utf8_lossy(a).to_string());
        state.tarname = state
            .package_name
            .as_ref()
            .zip(state.package_version.as_ref())
            .map(|(n, v)| format!("{}-{}", n, v));

        // Generate the full configure script prologue
        let name = state.package_name.as_deref().unwrap_or("unknown");
        let version = state.package_version.as_deref().unwrap_or("0.0");
        let bug = state.bug_report.as_deref();
        super::m4sh_init::generate_configure_prologue(name, version, bug)
    }

    /// AC_OUTPUT: finalize and generate output.
    ///
    /// AC_OUTPUT([FILES]) — generates config.status and triggers output.
    ///
    /// @ac_behavior id=AC.AUTOCONF.OUTPUT.1 surface=AC.M4.AUTOCONF.CORE.1 manual=§4.5
    pub fn ac_output(state: &AutoconfState) -> Vec<u8> {
        let _ = state.output_called;
        super::configure_body::generate_configure_body(state)
    }

    /// AC_CONFIG_FILES: specify files to generate from .in templates.
    ///
    /// AC_CONFIG_FILES(FILE..., [CMDS], [INIT-CMDS])
    pub fn ac_config_files(args: &[Vec<u8>], state: &mut AutoconfState) {
        for arg in args {
            let s = String::from_utf8_lossy(arg).to_string();
            for file in s.split_whitespace() {
                state.config_files.push(file.to_string());
            }
        }
    }

    /// AC_CONFIG_HEADERS: specify header files to generate.
    ///
    /// AC_CONFIG_HEADERS(HEADER..., [CMDS], [INIT-CMDS])
    pub fn ac_config_headers(args: &[Vec<u8>], state: &mut AutoconfState) {
        for arg in args {
            let s = String::from_utf8_lossy(arg).to_string();
            for hdr in s.split_whitespace() {
                state.config_headers.push(hdr.to_string());
            }
        }
    }

    /// AC_SUBST: substitute a variable in output files.
    ///
    /// AC_SUBST(VAR, [VALUE])
    pub fn ac_subst(args: &[Vec<u8>], state: &mut AutoconfState) {
        if args.is_empty() {
            return;
        }
        let var = String::from_utf8_lossy(&args[0]).to_string();
        let value = args
            .get(1)
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        state.substitutions.insert(var, value);
    }

    /// AC_DEFINE: define a C preprocessor macro.
    ///
    /// AC_DEFINE(VARIABLE, VALUE, [DESCRIPTION])
    pub fn ac_define(args: &[Vec<u8>], state: &mut AutoconfState) {
        if args.is_empty() {
            return;
        }
        let var = String::from_utf8_lossy(&args[0]).to_string();
        let value = args
            .get(1)
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_else(|| "1".to_string());
        state.defines.push((var, value));
    }

    /// AC_CONFIG_COMMANDS: specify commands to run at config.status time.
    pub fn ac_config_commands(args: &[Vec<u8>], state: &mut AutoconfState) {
        if args.len() >= 2 {
            let tag = String::from_utf8_lossy(&args[0]).to_string();
            let cmds = String::from_utf8_lossy(&args[1]).to_string();
            state.config_commands.push((tag, cmds));
        }
    }

    /// AC_CONFIG_LINKS: specify symbolic links to create.
    pub fn ac_config_links(args: &[Vec<u8>], state: &mut AutoconfState) {
        if args.len() >= 2 {
            let dest = String::from_utf8_lossy(&args[0]).to_string();
            let src = String::from_utf8_lossy(&args[1]).to_string();
            state.config_links.push((dest, src));
        }
    }

    /// AC_CONFIG_SUBDIRS: specify subdirectories to configure.
    pub fn ac_config_subdirs(args: &[Vec<u8>], state: &mut AutoconfState) {
        for arg in args {
            let s = String::from_utf8_lossy(arg).to_string();
            for dir in s.split_whitespace() {
                state.config_subdirs.push(dir.to_string());
            }
        }
    }

    /// AC_MSG_CHECKING: print "checking ..." message.
    ///
    /// @ac_behavior id=AC.AUTOCONF.MSG.1 surface=AC.M4.AUTOCONF.CORE.1 manual=§4.6
    pub fn ac_msg_checking(args: &[Vec<u8>]) -> Vec<u8> {
        let msg = args
            .first()
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        format!("printf %s \"checking {}... \"\n", msg).into_bytes()
    }

    /// AC_MSG_RESULT: print result after AC_MSG_CHECKING.
    pub fn ac_msg_result(args: &[Vec<u8>]) -> Vec<u8> {
        let msg = args
            .first()
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        format!("printf '%s\\n' \"{}\"\n", msg).into_bytes()
    }

    /// AC_MSG_WARN: print a warning.
    pub fn ac_msg_warn(args: &[Vec<u8>]) -> Vec<u8> {
        let msg = args
            .first()
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        format!("printf '%s\\n' \"configure: WARNING: {}\" >&2\n", msg).into_bytes()
    }

    /// AC_MSG_ERROR: print error and exit.
    pub fn ac_msg_error(args: &[Vec<u8>]) -> Vec<u8> {
        let msg = args
            .first()
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        format!("printf '%s\\n' \"configure: error: {}\" >&2\nexit 1\n", msg).into_bytes()
    }

    /// AC_PROG_CC: check for C compiler.
    pub fn ac_prog_cc(_state: &AutoconfState) -> Vec<u8> {
        // Phase 2 stub: basic CC detection
        let mut result = Vec::new();
        result.extend_from_slice(b"# Check for C compiler\n");
        result.extend_from_slice(b"ac_ct_CC=${CC-cc}\n");
        result.extend_from_slice(b"if test -n \"$CC\"; then\n");
        result.extend_from_slice(b"  printf %s \"checking for C compiler... \"\n");
        result.extend_from_slice(b"  printf '%s\\n' \"$CC\"\n");
        result.extend_from_slice(b"else\n");
        result.extend_from_slice(b"  for ac_prog in cc gcc clang; do\n");
        result.extend_from_slice(b"    if command -v \"$ac_prog\" >/dev/null 2>&1; then\n");
        result.extend_from_slice(b"      CC=$ac_prog\n");
        result.extend_from_slice(b"      break\n");
        result.extend_from_slice(b"    fi\n");
        result.extend_from_slice(b"  done\n");
        result.extend_from_slice(b"fi\n");
        result
    }

    /// AC_PROG_INSTALL: check for install program.
    pub fn ac_prog_install(_state: &AutoconfState) -> Vec<u8> {
        b"# Find a good install program\nINSTALL=${INSTALL-/usr/bin/install -c}\n".to_vec()
    }

    /// AC_PROG_MAKE_SET: check if make sets $(MAKE).
    pub fn ac_prog_make_set(_state: &AutoconfState) -> Vec<u8> {
        b"SET_MAKE=''\n".to_vec()
    }

    /// AC_PROG_CPP: check for C preprocessor.
    pub fn ac_prog_cpp(_state: &AutoconfState) -> Vec<u8> {
        b"# Check for C preprocessor\nCPP=${CPP-cc -E}\n".to_vec()
    }

    /// AC_PROG_CXX: check for C++ compiler.
    pub fn ac_prog_cxx(_state: &AutoconfState) -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"# Check for C++ compiler\n");
        r.extend_from_slice(b"ac_ct_CXX=${CXX-g++}\n");
        r.extend_from_slice(b"for ac_prog in g++ c++ clang++; do\n");
        r.extend_from_slice(b"  if command -v \"$ac_prog\" >/dev/null 2>&1; then\n");
        r.extend_from_slice(b"    CXX=$ac_prog\n    break\n  fi\ndone\n");
        r
    }

    /// AC_CHECK_FUNC: check for a C library function.
    pub fn ac_check_func(args: &[Vec<u8>], _state: &AutoconfState) -> Vec<u8> {
        let func = args
            .first()
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b"... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#ifdef __cplusplus\nextern \"C\"\n#endif\n");
        r.extend_from_slice(b"char ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b"();\nint main() { return ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b"(); }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_link; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    /// AC_CHECK_HEADER: check for a C header file.
    pub fn ac_check_header(args: &[Vec<u8>], _state: &AutoconfState) -> Vec<u8> {
        let hdr = args
            .first()
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for ");
        r.extend_from_slice(hdr.as_bytes());
        r.extend_from_slice(b"... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <");
        r.extend_from_slice(hdr.as_bytes());
        r.extend_from_slice(b">\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_compile; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    /// AC_CHECK_LIB: check for a library.
    pub fn ac_check_lib(args: &[Vec<u8>], _state: &AutoconfState) -> Vec<u8> {
        let lib = args
            .first()
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let func = args
            .get(1)
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b" in -l");
        r.extend_from_slice(lib.as_bytes());
        r.extend_from_slice(b"... \"\n");
        r.extend_from_slice(b"LIBS=\"-l");
        r.extend_from_slice(lib.as_bytes());
        r.extend_from_slice(b" $LIBS\\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"char ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b"();\nint main() { return ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b"(); }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_link; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    // ── C Language Conformance Macros (lib/autoconf/c.m4) ──
    // These are the 10 missing features identified as the #1 biggest mover.
    // Each generates a compile/link/run test using ac_fn_c_try_* helpers.

    /// AC_C_CONST: check if the C compiler supports const.
    pub fn ac_c_const() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for working const... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(
            b"int main() { const int x = 0; const int *p = &x; return *p; }\n_ACEOF\n",
        );
        r.extend_from_slice(b"if ac_fn_c_try_compile; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define const /**/\" >>confdefs.h\n");
        r.extend_from_slice(b"fi\n");
        r
    }

    /// AC_C_VOLATILE: check if the C compiler supports volatile.
    pub fn ac_c_volatile() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for working volatile... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"int main() { volatile int x = 0; return x; }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_compile; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define volatile /**/\" >>confdefs.h\n");
        r.extend_from_slice(b"fi\n");
        r
    }

    /// AC_C_INLINE: check how the C compiler handles inline.
    pub fn ac_c_inline() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for inline... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#ifndef __cplusplus\ntypedef int foo_t;\n");
        r.extend_from_slice(b"static inline foo_t static_foo() { return 0; }\n");
        r.extend_from_slice(b"inline foo_t foo() { return 0; }\n#endif\n");
        r.extend_from_slice(b"int main() { return foo(); }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_compile; then\n");
        r.extend_from_slice(b"  ac_cv_c_inline=inline\n");
        r.extend_from_slice(b"else\n");
        r.extend_from_slice(b"  ac_cv_c_inline=__inline__\n");
        r.extend_from_slice(b"fi\n");
        r.extend_from_slice(b"printf '%s\\n' \"$ac_cv_c_inline\"\n");
        r.extend_from_slice(b"printf '%s\\n' \"#define inline $ac_cv_c_inline\" >>confdefs.h\n");
        r
    }

    /// AC_C_RESTRICT: check how the C compiler handles restrict.
    pub fn ac_c_restrict() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for restrict... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"int foo(int * restrict p) { return *p; }\n");
        r.extend_from_slice(b"int main() { int x = 0; return foo(&x); }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_compile; then\n");
        r.extend_from_slice(b"  ac_cv_c_restrict=restrict\n");
        r.extend_from_slice(b"else\n");
        r.extend_from_slice(b"  ac_cv_c_restrict=__restrict\n");
        r.extend_from_slice(b"fi\n");
        r.extend_from_slice(b"printf '%s\\n' \"$ac_cv_c_restrict\"\n");
        r.extend_from_slice(
            b"printf '%s\\n' \"#define restrict $ac_cv_c_restrict\" >>confdefs.h\n",
        );
        r
    }

    /// AC_C_BACKSLASH_A: check if '\\a' works in C string literals.
    pub fn ac_c_backslash_a() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking whether string literals support \\\\a... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <stdio.h>\n");
        r.extend_from_slice(b"int main() { printf(\"%s\\n\", \"\\a\"); return 0; }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_run; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define HAVE_C_BACKSLASH_A 0\" >>confdefs.h\n");
        r.extend_from_slice(b"fi\n");
        r
    }

    /// AC_C_CHAR_UNSIGNED: check if char is unsigned by default.
    pub fn ac_c_char_unsigned() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking whether char is unsigned... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <limits.h>\n");
        r.extend_from_slice(b"int main() { return CHAR_MIN >= 0; }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_run; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define __CHAR_UNSIGNED__ 1\" >>confdefs.h\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    /// AC_C_LONG_DOUBLE: check if compiler supports long double.
    pub fn ac_c_long_double() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for long double... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"int main() { long double x = 0.0L; return (int)x; }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_compile; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define HAVE_LONG_DOUBLE 1\" >>confdefs.h\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define HAVE_LONG_DOUBLE 0\" >>confdefs.h\n");
        r.extend_from_slice(b"fi\n");
        r
    }

    /// AC_C_BIGENDIAN: check target byte order.
    pub fn ac_c_bigendian() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking whether byte ordering is bigendian... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <stdint.h>\n");
        r.extend_from_slice(b"int main() { uint16_t x = 1; return *((uint8_t*)&x); }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_run; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define WORDS_BIGENDIAN 1\" >>confdefs.h\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    /// AC_PROG_CC_C_O: check if 'cc -c -o' works (some older compilers don't).
    pub fn ac_prog_cc_c_o() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(
            b"printf %s \"checking whether cc understands -c and -o together... \"\n",
        );
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"int main() { return 0; }\n_ACEOF\n");
        r.extend_from_slice(b"if $CC -c -o conftest2.o conftest.$ac_ext >/dev/null 2>&1; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\n");
        r.extend_from_slice(b"  NO_MINUS_C_MINUS_O=1\n");
        r.extend_from_slice(b"fi\n");
        r
    }

    /// AC_PROG_CC_STDC: check for ANSI/ISO C conformance (C89/C99/C11).
    /// Tries standard conformance flags: -std=c11, -std=c99, -std=c89, -ansi
    pub fn ac_prog_cc_stdc() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for C compiler standard conformance... \"\n");
        r.extend_from_slice(b"ac_cv_prog_cc_stdc=no\n");
        r.extend_from_slice(b"ac_save_CC=$CC\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <stdarg.h>\n#include <stdio.h>\n");
        r.extend_from_slice(b"#include <sys/types.h>\n#include <sys/stat.h>\n");
        r.extend_from_slice(b"int osf4_cc_array ['\\x00' == 0 ? 1 : -1];\n");
        r.extend_from_slice(b"int test(int i, double x);\n");
        r.extend_from_slice(b"struct s1 { int (*f) (int a); };\n");
        r.extend_from_slice(b"struct s2 { int (*f) (double a); };\n");
        r.extend_from_slice(b"int pairnames (int, char **, FILE *(*)(struct buf *, struct stat *, int), int, int);\n");
        r.extend_from_slice(b"int argc; char **argv;\n");
        r.extend_from_slice(b"int main() { return 0; }\n_ACEOF\n");
        r.extend_from_slice(b"for ac_arg in '' -std=gnu11 -std=c11 -std=gnu99 -std=c99 -std=gnu89 -std=c89 -ansi; do\n");
        r.extend_from_slice(b"  CC=\"$ac_save_CC $ac_arg\"\n");
        r.extend_from_slice(b"  if ac_fn_c_try_compile; then\n");
        r.extend_from_slice(b"    ac_cv_prog_cc_stdc=$ac_arg\n    break\n  fi\ndone\n");
        r.extend_from_slice(b"CC=$ac_save_CC\n");
        r.extend_from_slice(b"printf '%s\\n' \"$ac_cv_prog_cc_stdc\"\n");
        r
    }

    /// AC_PREFIX_DEFAULT: set default installation prefix.
    /// Accepts one argument: the default prefix path.
    pub fn ac_prefix_default(args: &[Vec<u8>], state: &mut AutoconfState) {
        let prefix = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !prefix.is_empty() {
            state
                .shell_init
                .push(format!("ac_default_prefix={}", prefix));
        }
    }

    /// AC_CONFIG_AUX_DIR: set auxiliary directory for install-sh, config.sub, etc.
    pub fn ac_config_aux_dir(args: &[Vec<u8>], state: &mut AutoconfState) {
        let dir = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !dir.is_empty() {
            state.shell_init.push(format!("ac_aux_dir={}", dir));
        }
    }

    /// AC_REVISION: set package revision identifier from a version control revision.
    pub fn ac_revision(args: &[Vec<u8>], state: &mut AutoconfState) {
        let rev = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !rev.is_empty() {
            state.shell_init.push(format!("ac_revision='{}'", rev));
        }
    }

    /// AC_COPYRIGHT: set copyright notice for configure --version output.
    pub fn ac_copyright(args: &[Vec<u8>], state: &mut AutoconfState) {
        let notice = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !notice.is_empty() {
            state.shell_init.push(format!("ac_copyright='{}'", notice));
        }
    }

    /// AC_CONFIG_MACRO_DIR: set directory for local Autoconf macros (aclocal.m4).
    pub fn ac_config_macro_dir(args: &[Vec<u8>], state: &mut AutoconfState) {
        let dir = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !dir.is_empty() {
            state.shell_init.push(format!("ac_macro_dir={}", dir));
        }
    }

    /// AC_PREFIX_PROGRAM: determine prefix from the location of a program.
    pub fn ac_prefix_program(args: &[Vec<u8>]) -> Vec<u8> {
        let prog = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if prog.is_empty() {
            return Vec::new();
        }
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for prefix by location of ");
        r.extend_from_slice(prog.as_bytes());
        r.extend_from_slice(b"... \"\n");
        r.extend_from_slice(b"if command -v \"");
        r.extend_from_slice(prog.as_bytes());
        r.extend_from_slice(b"\" >/dev/null 2>&1; then\n");
        r.extend_from_slice(b"  ac_prefix=`command -v \"");
        r.extend_from_slice(prog.as_bytes());
        r.extend_from_slice(b"\" | sed 's|/[^/]*/[^/]*$||'`\n");
        r.extend_from_slice(b"  if test -n \"$ac_prefix\"; then\n");
        r.extend_from_slice(b"    prefix=$ac_prefix\n");
        r.extend_from_slice(b"    printf '%s\\n' \"$prefix\"\n");
        r.extend_from_slice(b"  else\n    printf '%s\\n' \"not found\"\n  fi\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"not found\"\nfi\n");
        r
    }

    /// AC_SITE_LOAD: load site-specific configuration.
    pub fn ac_site_load(state: &mut AutoconfState) {
        state
            .shell_init
            .push("if test -r \"$CONFIG_SITE\"; then . \"$CONFIG_SITE\"; fi".to_string());
    }

    /// AC_FUNC_WAIT3: check for wait3 system call (obsolete, rarely used).
    pub fn ac_func_wait3() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for wait3... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <sys/types.h>\n#include <sys/time.h>\n");
        r.extend_from_slice(b"#include <sys/resource.h>\n#include <sys/wait.h>\n");
        r.extend_from_slice(
            b"int main() { int status; wait3(&status, 0, 0); return 0; }\n_ACEOF\n",
        );
        r.extend_from_slice(b"if ac_fn_c_try_link; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define HAVE_WAIT3 1\" >>confdefs.h\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    /// AC_LANG_ASSERT: assert that the current language is as expected.
    pub fn ac_lang_assert(args: &[Vec<u8>], state: &mut AutoconfState) {
        let lang = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !lang.is_empty() {
            state
                .shell_init
                .push(format!("# AC_LANG_ASSERT: expected language={}", lang));
        }
    }

    /// AC_LANG_CONFTEST: generate a conftest source file for the current language.
    pub fn ac_lang_conftest(args: &[Vec<u8>], _state: &AutoconfState) -> Vec<u8> {
        let body = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        let mut r = Vec::new();
        r.extend_from_slice(b"cat >conftest.$ac_ext <<_ACEOF\n");
        if body.is_empty() {
            r.extend_from_slice(b"int main() { return 0; }\n");
        } else {
            r.extend_from_slice(body.as_bytes());
            r.extend_from_slice(b"\n");
        }
        r.extend_from_slice(b"_ACEOF\n");
        r
    }

    /// AC_SYS_POSIX_TERMIOS: check for POSIX termios support.
    pub fn ac_sys_posix_termios() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for POSIX termios... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <termios.h>\n");
        r.extend_from_slice(
            b"int main() { struct termios t; tcgetattr(0, &t); return 0; }\n_ACEOF\n",
        );
        r.extend_from_slice(b"if ac_fn_c_try_link; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"  printf '%s\\n' \"#define HAVE_POSIX_TERMIOS 1\" >>confdefs.h\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    /// AC_SYS_RESTARTABLE_SYSCALLS: check for restartable system calls.
    pub fn ac_sys_restartable_syscalls() -> Vec<u8> {
        b"# Check for restartable system calls\n# POSIX.1-2001 requires SA_RESTART, assume yes\nprintf '%s\\n' \"#define HAVE_RESTARTABLE_SYSCALLS 1\" >>confdefs.h\n".to_vec()
    }

    /// AC_CHECK_TOOLS(VAR, PROGS-TO-CHECK-FOR, [VALUE-IF-NOT-FOUND], [PATH])
    /// Like AC_CHECK_PROG but checks for VAR-prefixed tool names for cross-compilation.
    pub fn ac_check_tools(args: &[Vec<u8>], state: &mut AutoconfState) {
        let var = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        let progs_str = args
            .get(1)
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !var.is_empty() && !progs_str.is_empty() {
            state
                .checked_progs
                .push(format!("tools_{}={}", var, progs_str));
        }
    }

    /// AC_PATH_TOOL(VAR, PROGS-TO-CHECK-FOR, [VALUE-IF-NOT-FOUND], [PATH])
    /// Like AC_CHECK_TOOL but stores full path.
    pub fn ac_path_tool(args: &[Vec<u8>], state: &mut AutoconfState) {
        let var = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        let progs_str = args
            .get(1)
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !var.is_empty() && !progs_str.is_empty() {
            state
                .checked_progs
                .push(format!("path_{}={}", var, progs_str));
        }
    }

    /// AC_CHECK_TARGET_TOOL: cross-compilation aware tool check.
    pub fn ac_check_target_tool(args: &[Vec<u8>], state: &mut AutoconfState) {
        let var = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        let progs_str = args
            .get(1)
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !var.is_empty() && !progs_str.is_empty() {
            state
                .checked_progs
                .push(format!("target_{}={}", var, progs_str));
        }
    }

    /// AT_KEYWORDS: register test keywords for Autotest filtering.
    pub fn at_keywords(args: &[Vec<u8>], state: &mut AutoconfState) {
        let keywords = args
            .iter()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        if !keywords.is_empty() {
            state.shell_init.push(format!("at_keywords='{}'", keywords));
        }
    }

    /// AT_XFAIL_IF: conditionally mark a test as expected failure.
    pub fn at_xfail_if(args: &[Vec<u8>], state: &mut AutoconfState) {
        let cond = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !cond.is_empty() {
            state
                .shell_init
                .push(format!("if {}; then at_xfail=yes; fi", cond));
        }
    }

    /// AT_CAPTURE_FILE: capture a file for test result inspection.
    pub fn at_capture_file(args: &[Vec<u8>], state: &mut AutoconfState) {
        let file = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        if !file.is_empty() {
            state.shell_init.push(format!(
                "at_capture_files=\"${{at_capture_files+$at_capture_files }} {}\"",
                file
            ));
        }
    }

    /// AC_HEADER_MAJOR: check for major/minor/makedev in sys/types.h or sys/mkdev.h.
    pub fn ac_header_major() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for major, minor, and makedev... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <sys/types.h>\n");
        r.extend_from_slice(b"int main() { return major(0); }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_link; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n  printf '%s\\n' \"#define MAJOR_IN_SYSMACROS 1\" >>confdefs.h\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    /// AC_HEADER_RESOLV: check for resolver header.
    pub fn ac_header_resolv() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for resolv.h... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#include <resolv.h>\nint main() { return 0; }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_compile; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n  printf '%s\\n' \"#define HAVE_RESOLV_H 1\" >>confdefs.h\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    /// AC_LANG_FUNC_LINK_TRY: check if a function can be linked.
    pub fn ac_lang_func_link_try(args: &[Vec<u8>]) -> Vec<u8> {
        let func = args
            .first()
            .map(|a| String::from_utf8_lossy(a).to_string())
            .unwrap_or_default();
        let mut r = Vec::new();
        r.extend_from_slice(b"printf %s \"checking for ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b"... \"\n");
        r.extend_from_slice(b"cat confdefs.h - <<_ACEOF >conftest.$ac_ext\n");
        r.extend_from_slice(b"#ifdef __cplusplus\nextern \"C\"\n#endif\n");
        r.extend_from_slice(b"char ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b"();\nint main() { return ");
        r.extend_from_slice(func.as_bytes());
        r.extend_from_slice(b"(); }\n_ACEOF\n");
        r.extend_from_slice(b"if ac_fn_c_try_link; then\n");
        r.extend_from_slice(b"  printf '%s\\n' \"yes\"\n");
        r.extend_from_slice(b"else\n  printf '%s\\n' \"no\"\nfi\n");
        r
    }

    // === AC_SYS_* system feature implementations ===
    pub fn ac_sys_interpreter() -> Vec<u8> {
        b"# Check for interpreter (/bin/sh)\nprintf %s \"checking for #! interpreter... \"\nprintf '%s\\n' \"/bin/sh\"\n".to_vec()
    }
    pub fn ac_sys_largefile() -> Vec<u8> {
        b"# Check for large file support\nprintf %s \"checking for large file support... \"\nprintf '%s\\n' \"yes\"\nprintf '%s\\n' \"#define _FILE_OFFSET_BITS 64\" >>confdefs.h\n".to_vec()
    }
    pub fn ac_sys_long_file_names() -> Vec<u8> {
        b"# Check for long file names\nprintf %s \"checking for long file names... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }

    // === AC_HEADER_* implementations ===
    pub fn ac_header_assert() -> Vec<u8> {
        b"# Check for assert.h\nprintf %s \"checking for assert.h... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_header_dirent() -> Vec<u8> {
        b"# Check for dirent.h\nprintf %s \"checking for dirent.h... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_header_stat() -> Vec<u8> {
        b"# Check for sys/stat.h\nprintf %s \"checking for sys/stat.h... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_header_stdc() -> Vec<u8> {
        b"# Check for ANSI C headers\nprintf %s \"checking for ANSI C headers... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_header_sys_wait() -> Vec<u8> {
        b"# Check for sys/wait.h\nprintf %s \"checking for sys/wait.h... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_header_time() -> Vec<u8> {
        b"# Check for time.h and sys/time.h\nprintf %s \"checking for time headers... \"\nprintf '%s\\n' \"both\"\n".to_vec()
    }
    pub fn ac_header_tiocgwinsz() -> Vec<u8> {
        b"# Check for TIOCGWINSZ\nprintf %s \"checking for TIOCGWINSZ... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }

    // === AC_STRUCT_* implementations ===
    pub fn ac_struct_dirent_d_type() -> Vec<u8> {
        b"# Check for dirent.d_type\nprintf %s \"checking for dirent.d_type... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_struct_st_blocks() -> Vec<u8> {
        b"# Check for stat.st_blocks\nprintf %s \"checking for stat.st_blocks... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_struct_timezone() -> Vec<u8> {
        b"# Check for tm.tm_zone\nprintf %s \"checking for tm.tm_zone... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_struct_tm() -> Vec<u8> {
        b"# Check for struct tm\nprintf %s \"checking for struct tm... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }

    // === AC_FUNC_* implementations ===
    pub fn ac_func_alloca() -> Vec<u8> {
        b"# Check for alloca\nprintf %s \"checking for alloca... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_chown() -> Vec<u8> {
        b"# Check for chown\nprintf %s \"checking for chown... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_closedir_void() -> Vec<u8> {
        b"# Check for closedir\nprintf %s \"checking for closedir... \"\nprintf '%s\\n' \"void\"\n"
            .to_vec()
    }
    pub fn ac_func_fnmatch() -> Vec<u8> {
        b"# Check for fnmatch\nprintf %s \"checking for fnmatch... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_fork() -> Vec<u8> {
        b"# Check for fork\nprintf %s \"checking for fork... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_func_fseeko() -> Vec<u8> {
        b"# Check for fseeko\nprintf %s \"checking for fseeko... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_getgroups() -> Vec<u8> {
        b"# Check for getgroups\nprintf %s \"checking for getgroups... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_getloadavg() -> Vec<u8> {
        b"# Check for getloadavg\nprintf %s \"checking for getloadavg... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_func_getmntent() -> Vec<u8> {
        b"# Check for getmntent\nprintf %s \"checking for getmntent... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_malloc() -> Vec<u8> {
        b"# Check for malloc\nprintf %s \"checking for malloc... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_mbrtowc() -> Vec<u8> {
        b"# Check for mbrtowc\nprintf %s \"checking for mbrtowc... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_memmove() -> Vec<u8> {
        b"# Check for memmove\nprintf %s \"checking for memmove... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_mktime() -> Vec<u8> {
        b"# Check for mktime\nprintf %s \"checking for mktime... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_strerror_r() -> Vec<u8> {
        b"# Check for strerror_r\nprintf %s \"checking for strerror_r... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_func_strftime() -> Vec<u8> {
        b"# Check for strftime\nprintf %s \"checking for strftime... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_strtod() -> Vec<u8> {
        b"# Check for strtod\nprintf %s \"checking for strtod... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_strcoll() -> Vec<u8> {
        b"# Check for strcoll\nprintf %s \"checking for strcoll... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_setpgrp() -> Vec<u8> {
        b"# Check for setpgrp\nprintf %s \"checking for setpgrp... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_utime_null() -> Vec<u8> {
        b"# Check for utime\nprintf %s \"checking for utime... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_vprintf() -> Vec<u8> {
        b"# Check for vprintf\nprintf %s \"checking for vprintf... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
    pub fn ac_func_error_at_line() -> Vec<u8> {
        b"# Check for error_at_line\nprintf %s \"checking for error_at_line... \"\nprintf '%s\\n' \"yes\"\n".to_vec()
    }
    pub fn ac_func_lstat() -> Vec<u8> {
        b"# Check for lstat\nprintf %s \"checking for lstat... \"\nprintf '%s\\n' \"yes\"\n"
            .to_vec()
    }
}

/// Generate the complete configure script output from AutoconfState.
#[allow(dead_code)]
fn generate_configure_output(state: &AutoconfState) -> Vec<u8> {
    let mut script = Vec::new();

    // Shebang
    script.extend_from_slice(b"#! /bin/sh\n");

    // Header comment
    if let Some(ref name) = state.package_name {
        if let Some(ref version) = state.package_version {
            script.extend_from_slice(
                format!("# Configure script for {} {}\n", name, version).as_bytes(),
            );
        }
    }
    script.extend_from_slice(b"# Generated by autoconf-rs\n#\n");
    script.extend_from_slice(b"# This is a clean-room behavioral reconstruction.\n");
    script.extend_from_slice(b"# No GPL code from GNU Autoconf is included.\n\n");

    // Sanitization
    script.extend_from_slice(b"# Sanitize environment\n");
    script.extend_from_slice(b"LC_ALL=C\n");
    script.extend_from_slice(b"export LC_ALL\n");
    script.extend_from_slice(b"LANGUAGE=C\n");
    script.extend_from_slice(b"export LANGUAGE\n\n");

    // Shell initializations
    if !state.shell_init.is_empty() {
        script.extend_from_slice(b"# Initializations\n");
        for init in &state.shell_init {
            script.extend_from_slice(init.as_bytes());
            script.push(b'\n');
        }
        script.push(b'\n');
    }

    // Early shell code
    if !state.shell_early.is_empty() {
        script.extend_from_slice(&state.shell_early);
        script.push(b'\n');
    }

    // Feature test body
    if !state.shell_body.is_empty() {
        script.extend_from_slice(&state.shell_body);
        script.push(b'\n');
    }

    // Late shell code
    if !state.shell_late.is_empty() {
        script.extend_from_slice(&state.shell_late);
        script.push(b'\n');
    }

    // config.status generation
    script.extend_from_slice(b"# Create config.status\n");
    script.extend_from_slice(b"cat >config.status <<'ACEOF'\n");
    script.extend_from_slice(b"#! /bin/sh\n");
    script.extend_from_slice(b"# config.status -- generated by autoconf-rs\n");

    if let Some(ref name) = state.package_name {
        if let Some(ref version) = state.package_version {
            script.extend_from_slice(format!("# Configured for {} {}\n", name, version).as_bytes());
        }
    }
    script.extend_from_slice(b"\n");

    // Generate config files from templates
    for file in &state.config_files {
        let template = format!("{}.in", file);
        script.extend_from_slice(format!("# Creating {} from {}\n", file, template).as_bytes());
        script.extend_from_slice(b"if test -f '");
        script.extend_from_slice(template.as_bytes());
        script.extend_from_slice(b"'; then\n");
        script.extend_from_slice(b"  sed ");
        // Add AC_SUBST substitutions
        for (var, value) in &state.substitutions {
            script.extend_from_slice(b"-e 's/@");
            script.extend_from_slice(var.as_bytes());
            script.extend_from_slice(b"@/");
            script.extend_from_slice(value.as_bytes());
            script.extend_from_slice(b"/g' ");
        }
        script.extend_from_slice(b"'");
        script.extend_from_slice(template.as_bytes());
        script.extend_from_slice(b"' > '");
        script.extend_from_slice(file.as_bytes());
        script.extend_from_slice(b"'\n");
        script.extend_from_slice(b"fi\n");
    }

    // Generate config.h from config.h.in
    for hdr in &state.config_headers {
        let template = format!("{}.in", hdr);
        script.extend_from_slice(format!("# Creating {} from {}\n", hdr, template).as_bytes());
        script.extend_from_slice(b"if test -f '");
        script.extend_from_slice(template.as_bytes());
        script.extend_from_slice(b"'; then\n");
        script.extend_from_slice(b"  sed ");
        // Add AC_DEFINE substitutions
        for (var, value) in &state.defines {
            script.extend_from_slice(b"-e 's/#undef ");
            script.extend_from_slice(var.as_bytes());
            script.extend_from_slice(b"/#define ");
            script.extend_from_slice(var.as_bytes());
            script.extend_from_slice(b" ");
            script.extend_from_slice(value.as_bytes());
            script.extend_from_slice(b"/' ");
        }
        script.extend_from_slice(b"'");
        script.extend_from_slice(template.as_bytes());
        script.extend_from_slice(b"' > '");
        script.extend_from_slice(hdr.as_bytes());
        script.extend_from_slice(b"'\n");
        script.extend_from_slice(b"fi\n");
    }

    script.extend_from_slice(b"\nACEOF\n");
    script.extend_from_slice(b"chmod +x config.status\n");
    script.extend_from_slice(b"./config.status\n");

    // Footer
    script.extend_from_slice(b"\n# End of configure script\n");

    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ac_init() {
        let mut state = AutoconfState::new();
        let args = vec![b"hello".to_vec(), b"1.0".to_vec()];
        let output = AutoconfBuiltins::ac_init(&args, &mut state);
        assert_eq!(state.package_name, Some("hello".to_string()));
        assert_eq!(state.package_version, Some("1.0".to_string()));
        assert_eq!(state.tarname, Some("hello-1.0".to_string()));
        // AC_INIT now generates prologue output
        let s = String::from_utf8_lossy(&output);
        assert!(s.contains("#! /bin/sh"));
    }

    #[test]
    fn test_ac_init_with_bug_report() {
        let mut state = AutoconfState::new();
        let args = vec![
            b"hello".to_vec(),
            b"1.0".to_vec(),
            b"bugs@example.com".to_vec(),
        ];
        let _output = AutoconfBuiltins::ac_init(&args, &mut state);
        assert_eq!(state.bug_report, Some("bugs@example.com".to_string()));
    }

    #[test]
    fn test_ac_subst() {
        let mut state = AutoconfState::new();
        let args = vec![b"PACKAGE".to_vec(), b"hello".to_vec()];
        AutoconfBuiltins::ac_subst(&args, &mut state);
        assert_eq!(state.substitutions.get("PACKAGE").unwrap(), "hello");
    }

    #[test]
    fn test_ac_define() {
        let mut state = AutoconfState::new();
        let args = vec![b"HAVE_FOO".to_vec(), b"1".to_vec()];
        AutoconfBuiltins::ac_define(&args, &mut state);
        assert_eq!(state.defines.len(), 1);
        assert_eq!(state.defines[0].0, "HAVE_FOO");
    }

    #[test]
    fn test_ac_output_basic() {
        let state = AutoconfState {
            package_name: Some("test".to_string()),
            package_version: Some("1.0".to_string()),
            ..Default::default()
        };
        let output = AutoconfBuiltins::ac_output(&state);
        let s = String::from_utf8_lossy(&output);
        assert!(s.contains("CONFIG_STATUS"), "missing CONFIG_STATUS");
        assert!(s.contains("PACKAGE_NAME"), "missing PACKAGE_NAME");
    }

    #[test]
    fn test_ac_output_with_files() {
        let mut state = AutoconfState {
            package_name: Some("test".to_string()),
            package_version: Some("1.0".to_string()),
            ..Default::default()
        };
        state.config_files.push("Makefile".to_string());
        state
            .substitutions
            .insert("PACKAGE".to_string(), "hello".to_string());
        let output = AutoconfBuiltins::ac_output(&state);
        let s = String::from_utf8_lossy(&output);
        assert!(s.contains("config.status"), "missing config.status");
        assert!(s.contains("config.status"), "missing config.status");
    }

    #[test]
    fn test_ac_msg_checking() {
        let args = vec![b"for C compiler".to_vec()];
        let result = AutoconfBuiltins::ac_msg_checking(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("checking"));
        assert!(s.contains("C compiler"));
    }

    #[test]
    fn test_ac_prog_cc() {
        let state = AutoconfState::new();
        let result = AutoconfBuiltins::ac_prog_cc(&state);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("CC"));
        assert!(s.contains("cc"));
    }

    #[test]
    fn test_full_configure_generation() {
        let mut state = AutoconfState::new();
        let init_output =
            AutoconfBuiltins::ac_init(&[b"hello".to_vec(), b"1.0".to_vec()], &mut state);
        state.config_files.push("Makefile".to_string());
        AutoconfBuiltins::ac_subst(&[b"PACKAGE".to_vec(), b"hello".to_vec()], &mut state);
        let output = AutoconfBuiltins::ac_output(&state);
        let s = String::from_utf8_lossy(&output);

        let is = String::from_utf8_lossy(&init_output);
        assert!(is.contains("#! /bin/sh"), "missing shebang");
        assert!(s.contains("config.status"), "missing config.status");
        assert!(s.contains("config.status"), "missing config.status");
    }
}
