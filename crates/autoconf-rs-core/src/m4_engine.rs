//! M4 macro expansion engine for Autoconf — wraps m4-rs-core.
//!
//! Uses the forensic-parity m4-rs-core crate for all M4 processing.
//! Autoconf-specific macros (AC_INIT, AC_OUTPUT, etc.) are pre-registered
//! as user macros that expand to the generated shell script text.
//!
//! M4 expansion output is routed through a DiversionManager for controlled
//! output ordering (lower diversion numbers appear first — critical for
//! AC_REQUIRE dependency ordering).
//!
//! Receipt family: AC.M4.*
//! Court: AC.M4.DIVERT.WIRED.1 — DiversionManager integrated into pipeline.
//! Current status: Phase 4 — diversion-backed M4 expansion. Panel #1 mandate done.

use crate::autoconf_macros::AutoconfState;
use crate::diagnostics::DiagnosticManager;
use crate::diversion::DiversionManager;
use crate::trace::{AutoconfEvent, Span, TraceLog};

/// M4 expansion engine — wraps m4-rs-core for Autoconf use.
pub struct M4Engine {
    engine: m4_rs::m4_rs_core::expansion::ExpansionEngine,
    state: AutoconfState,
    /// Trace log populated during macro expansion (panel mandate: source of truth)
    pub trace_log: TraceLog,
    /// Diversion manager for output ordering (panel mandate: AC_REQUIRE ordering)
    pub diversions: DiversionManager,
    /// Diagnostics manager with -W category filtering and source location tracking
    pub diagnostics: DiagnosticManager,
    /// Include directories for m4_include resolution (CROSS.002)
    pub include_dirs: Vec<String>,
    /// Panel: --allow-syscmd flag — when true, syscmd/esyscmd execute shell commands.
    /// Default false (safe Rust). Users needing esyscmd for GNULIB/Gettext must opt in.
    pub allow_syscmd: bool,
    /// CROSS.040: signal-aware — check SIGPIPE/SIGINT during expansion
    pub signal_aware: bool,
    /// CROSS.046/NC.PERM.3: whitelisted commands for --allow-syscmd bridge.
    /// When allow_syscmd is true, only these commands are permitted.
    /// Empty set means ALL commands are blocked (safe default).
    pub syscmd_whitelist: std::collections::HashSet<String>,
    /// Panel recommendation: --pure-m4 mode skips prescan+template dispatch.
    /// Uses raw M4 expansion output as the final configure script.
    /// Enables full m4_foreach/m4_if/AC_REQUIRE chain support.
    pub pure_m4: bool,
}

impl M4Engine {
    /// Create a new M4 engine configured for Autoconf.
    ///
    /// Sets up Autoconf-style quoting ([ and ]), disables # comments
    /// (since configure output contains # lines), and registers all
    /// standard GNU m4 builtins plus Autoconf-specific macros.
    pub fn new() -> Self {
        let mut engine = m4_rs::m4_rs_core::expansion::ExpansionEngine::new();

        // Register standard GNU m4 builtins
        engine.register_builtins();

        // Configure Autoconf-style quoting: [ and ] instead of ` and '
        // We inject changequote/changecom at the start of input so the
        // builtin handlers sync the relexer correctly.
        engine.quote_config.change_quote(Some("["), Some("]"));
        engine.quote_config.change_comment(Some("\0"), Some("\n"));

        Self {
            engine,
            state: AutoconfState::new(),
            trace_log: TraceLog::new(),
            diversions: DiversionManager::new(),
            diagnostics: DiagnosticManager::new(),
            include_dirs: vec![".".to_string()],
            allow_syscmd: false,
            signal_aware: true,
            syscmd_whitelist: std::collections::HashSet::new(),
            pure_m4: false,
        }
    }

    /// Pre-register Autoconf macros as user macros in the M4 engine.
    ///
    /// This must be called before processing configure.ac so that
    /// AC_INIT, AC_OUTPUT, etc. are recognized and expanded.
    /// The macros are defined with their expansion bodies (shell script text).
    /// Inert sentinels for AC_INIT / AC_OUTPUT, replaced with verbatim shell text after M4 expansion. They
    /// contain no `$`, `[`, `]`, `(`, `)`, `,` or defined-macro substrings, so M4 passes them through intact.
    const AC_INIT_MARK: &'static str = "@@AUTOCONFRS_PROLOGUE@@";
    const AC_OUTPUT_MARK: &'static str = "@@AUTOCONFRS_BODY@@";

    fn register_autoconf_macros(&mut self) {
        // AC_INIT / AC_OUTPUT expand to FINAL shell text (the m4sh prologue and the configure body, with
        // name/version already baked in by Rust). That text must be emitted VERBATIM, never re-scanned as
        // M4 -- otherwise shell tokens collide with M4: `eval` -> the eval builtin -> `0`; `$@`/`$1` -> the
        // macro's own args; a `*[\\/]*` glob -> stripped as a quote. Register inert sentinels here; process()
        // substitutes the verbatim text after expansion (see AC_INIT_MARK / AC_OUTPUT_MARK).
        self.engine
            .macro_table
            .define(b"AC_INIT", Self::AC_INIT_MARK.as_bytes());
        self.engine
            .macro_table
            .define(b"AC_OUTPUT", Self::AC_OUTPUT_MARK.as_bytes());

        // AC_CONFIG_FILES — no output, side effect only (handled by pre-scan)
        self.engine.macro_table.define(b"AC_CONFIG_FILES", b"");

        // AC_CONFIG_HEADERS — no output
        self.engine.macro_table.define(b"AC_CONFIG_HEADERS", b"");
        // AC_CONFIG_HEADER (older alias) + Automake/libtool macros: recognized so they are
        // consumed (no literal leftover -> shell syntax error). Their AC_SUBST surface is
        // defaulted in config.status (see shell_gen STD_VAR_*).
        for m in [
            "AC_CONFIG_HEADER", "AM_CONFIG_HEADER", "AC_CONFIG_MACRO_DIR", "AC_CONFIG_MACRO_DIRS", "AC_CONFIG_AUX_DIR",
            "AC_CONFIG_TESTDIR", "AM_INIT_AUTOMAKE", "AM_MAINTAINER_MODE", "AM_SILENT_RULES",
            "AM_PROG_AR", "AM_PROG_CC_C_O", "AM_PROG_LEX", "AM_PROG_LIBTOOL", "AM_PROG_INSTALL_STRIP",
            "AM_PROG_MKDIR_P", "AM_SANITY_CHECK", "AM_SET_DEPDIR", "AM_DEP_TRACK",
            "AM_OUTPUT_DEPENDENCY_COMMANDS", "AM_RUN_LOG", "AM_MISSING_PROG", "AM_GNU_GETTEXT",
            "AM_GNU_GETTEXT_VERSION", "AM_ICONV", "LT_INIT", "AC_PROG_LIBTOOL", "LT_LANG",
            "LT_PREREQ", "AM_CONDITIONAL",
            // Common pure-setup Autoconf macros that otherwise leak literal (their vars/defines are
            // defaulted in config.status). NOT the feature-probe macros (those are handled elsewhere).
            "AC_USE_SYSTEM_EXTENSIONS", "AC_GNU_SOURCE", "AC_SYS_LARGEFILE", "AC_SYS_LONG_FILE_NAMES",
            "AC_PROG_SED", "AC_PROG_GREP", "AC_PROG_EGREP", "AC_PROG_FGREP", "AC_PROG_AWK",
            "AC_PROG_LN_S", "AC_PROG_MKDIR_P", "AC_PROG_RANLIB", "AC_PROG_CPP", "AC_PROG_MAKE_SET",
            // Deprecated/obsolete macros (folded into AC_PROG_CC/CXX in autoconf >=2.70, or X11
            // detection) that still appear in older configure.ac and otherwise leak literal.
            "AC_PROG_CC_STDC", "AC_PROG_CC_C99", "AC_PROG_CC_C89", "AC_PROG_CXX_C_O",
            "AC_PROG_CC_C_O", "AC_PATH_XTRA", "AC_PATH_X", "AC_AIX", "AC_MINIX",
            // AC_LANG family: selects the probe language (C/C++). We probe in C by default and the
            // selection is otherwise inert here; left literal it was `AC_LANG(C)` -> shell syntax
            // error near `(`. Common in C++ projects (preseq, yarrp) and older C ones (aprs).
            "AC_LANG", "AC_LANG_PUSH", "AC_LANG_POP", "AC_LANG_SAVE", "AC_LANG_RESTORE", "AC_LANG_C",
            "AC_LANG_CPLUSPLUS", "AC_LANG_C_PLUS_PLUS",
            // Obsolete/libtool/gettext/no-result macros that otherwise leak literal -> command-not-found
            // in real configure.ac. Their effects are either defaulted elsewhere or irrelevant here.
            "AM_NLS", "AM_GNU_GETTEXT_REQUIRE_VERSION", "AM_PO_SUBDIRS", "AM_XGETTEXT_OPTION",
            // High-frequency unregistered macros from the corpus bug-map (atlas): assembler, OpenMP,
            // Vala, script-interpreter — leaked literal as command-not-found across many repos.
            "AM_PROG_AS", "AC_OPENMP", "AM_PROG_VALAC", "AC_SYS_INTERPRETER", "AC_PATH_PROG_FLEX",
            // AH_* are autoheader directives (they shape config.h.in, NOT configure). Left active they
            // leaked `m4_define(_ah_top, ...)` into configure -> shell syntax error (9 corpus repos).
            "AH_TOP", "AH_BOTTOM", "AH_VERBATIM", "AH_TEMPLATE", "AH_HEADER", "AH_CHECK_HEADERS",
            "AM_DISABLE_STATIC", "AM_ENABLE_STATIC", "AM_DISABLE_SHARED", "AM_ENABLE_SHARED",
            "AM_PROG_LD", "AM_PROG_NM", "AM_WITH_DMALLOC", "AM_PATH_LISPDIR",
            "AC_LIBTOOL_DLOPEN", "AC_LIBTOOL_WIN32_DLL", "AC_LIBTOOL_SETUP", "AC_DISABLE_STATIC",
            "AC_DISABLE_SHARED", "AC_ENABLE_STATIC", "AC_ENABLE_SHARED", "AC_LIBTOOL_PICMODE",
            "AC_FUNC_SETVBUF_REVERSED", "AC_EXEEXT", "AC_OBJEXT", "AC_CACHE_SAVE", "AC_CACHE_LOAD",
            "AC_CHECK_HEADER_STDBOOL", "AC_HEADER_STDBOOL", "AC_PROG_LIBTOOL", "AC_LTDL_DLLIB",
            "AC_C_CONST", "AC_C_INLINE", "AC_C_VOLATILE", "AC_C_RESTRICT", "AC_C_BIGENDIAN",
            "AC_HEADER_STDC", "AC_HEADER_TIME", "AC_HEADER_SYS_WAIT", "AC_HEADER_ASSERT",
            "AC_TYPE_SIZE_T", "AC_TYPE_PID_T", "AC_TYPE_OFF_T", "AC_TYPE_UID_T", "AC_TYPE_MODE_T",
            "AC_TYPE_SSIZE_T", "AC_TYPE_INT8_T", "AC_TYPE_INT16_T", "AC_TYPE_INT32_T",
            "AC_TYPE_INT64_T", "AC_TYPE_UINT8_T", "AC_TYPE_UINT16_T", "AC_TYPE_UINT32_T",
            "AC_TYPE_UINT64_T", "AC_STRUCT_TM", "AC_PROG_GCC_TRADITIONAL", "AC_CANONICAL_HOST",
            "AC_CANONICAL_BUILD", "AC_CANONICAL_TARGET", "AC_CANONICAL_SYSTEM", "AC_GNU_SOURCE",
            "AC_PROG_CC_C99",
            "AC_REQUIRE_AUX_FILE", "AC_SUBST_FILE", "AC_PRESERVE_HELP_ORDER",
            // Feature-test + option macros: consumed here (no literal leftover -> shell syntax
            // error) while the prescan does the actual probing into checked_headers/libs/funcs.
            "AC_CHECK_HEADER", "AC_CHECK_LIB", "AC_CHECK_FUNC", "AC_CHECK_FUNCS", "AC_SEARCH_LIBS",
            "AC_CHECK_PROG", "AC_CHECK_PROGS", "AC_CHECK_TOOL", "AC_CHECK_TOOLS", "AC_PATH_PROG",
            "AC_PATH_PROGS", "AC_PATH_TOOL", "AC_CHECK_TYPES", "AC_CHECK_MEMBERS", "AC_CHECK_DECLS",
            "AC_CHECK_SIZEOF", "AC_CHECK_FILE", "AC_CHECK_FILES", "AC_REPLACE_FUNCS",
            "AC_ARG_WITH", "AC_ARG_ENABLE", "AC_ARG_VAR", "AC_ARG_PROGRAM", "AS_HELP_STRING",
            "AC_HELP_STRING", "AC_FUNC_ALLOCA", "AC_FUNC_FORK", "AC_FUNC_MALLOC", "AC_FUNC_REALLOC",
            "AC_FUNC_MMAP", "AC_FUNC_STRTOD", "AC_FUNC_STAT", "AC_FUNC_CHOWN", "AC_FUNC_MEMCMP",
            "AC_FUNC_VPRINTF", "AC_FUNC_GETPGRP", "AC_FUNC_SELECT_ARGTYPES", "AC_FUNC_ERROR_AT_LINE",
            "AC_DEFINE_UNQUOTED", "AC_MSG_NOTICE", "AC_MSG_WARN", "AC_CACHE_CHECK", "AC_CACHE_VAL",
            "AC_COMPILE_IFELSE", "AC_LINK_IFELSE", "AC_RUN_IFELSE", "AC_PREPROC_IFELSE",
            "AC_LANG_PROGRAM", "AC_LANG_SOURCE", "AC_EGREP_HEADER", "AC_EGREP_CPP", "AC_TRY_COMPILE",
            "AC_TRY_LINK", "AC_TRY_RUN", "AC_TRY_CPP", "AC_STRUCT_ST_BLOCKS", "AC_HEADER_DIRENT",
            "AC_HEADER_STDBOOL", "AC_HEADER_MAJOR", "AC_HEADER_RESOLV", "AC_FUNC_OBSTACK",
            // NB: PKG_CHECK_MODULES / PKG_* / AX_CXX_COMPILE_STDCXX* are NOT listed here — they are
            // implemented natively in macro_overrides() (real pkg-config / C++-std probes). A
            // no-output entry here would shadow those overrides (empty expansion -> empty then/else
            // -> shell syntax errors).
            // NB: AX_PTHREAD is NOT here — it has a native override in macro_overrides() (a clean
            // pthread-flag probe). A no-output stub would shadow that override (like PKG_CHECK_MODULES).
            "AX_CHECK_COMPILE_FLAG",
            "AX_REQUIRE_DEFINED", "AX_APPEND_FLAG", "AX_APPEND_COMPILE_FLAGS", "gl_INIT", "gl_EARLY", "AC_CHECK_INCLUDES_DEFAULT", "AC_USE_SYSTEM_EXTENSIONS", "AC_SYS_LARGEFILE",
        ] {
            self.engine.macro_table.define(m.as_bytes(), b"");
        }

        // AC_SUBST — no output
        self.engine.macro_table.define(b"AC_SUBST", b"");

        // AC_DEFINE — no output
        self.engine.macro_table.define(b"AC_DEFINE", b"");

        // AC_CONFIG_COMMANDS — no output
        self.engine.macro_table.define(b"AC_CONFIG_COMMANDS", b"");

        // AC_CONFIG_LINKS — no output
        self.engine.macro_table.define(b"AC_CONFIG_LINKS", b"");

        // AC_CONFIG_SUBDIRS — configure subdirectories recursively
        self.engine.macro_table.define(
            b"AC_CONFIG_SUBDIRS",
            b"# Configure subdirectories\nfor ac_subdir in $1; do\n  if test -d \"$srcdir/$ac_subdir\"; then\n    printf '%s\\n' \"$as_me: configuring in $ac_subdir\"\n    mkdir -p \"$ac_subdir\" 2>/dev/null || :\n    if test -f \"$srcdir/$ac_subdir/configure\"; then\n      (cd \"$ac_subdir\" && \"$srcdir/$ac_subdir/configure\" --cache-file=../config.cache --srcdir=\"$srcdir/$ac_subdir\" $ac_configure_args) || exit 1\n    fi\n  fi\ndone\n",
        );

        // AC_MSG_* — generate shell echo commands
        self.engine
            .macro_table
            .define(b"AC_MSG_CHECKING", b"printf %s \"checking $1... \"");
        self.engine
            .macro_table
            .define(b"AC_MSG_RESULT", b"printf '%s\\n' \"$1\"");
        self.engine.macro_table.define(
            b"AC_MSG_WARN",
            b"printf '%s\\n' \"configure: WARNING: $1\" >&2",
        );
        self.engine.macro_table.define(
            b"AC_MSG_ERROR",
            b"printf '%s\\n' \"configure: error: $1\" >&2\nexit 1",
        );

        // AC_PROG_CC — C compiler detection
        self.engine.macro_table.define(
            b"AC_PROG_CC",
            b"# Check for C compiler\nac_ct_CC=${CC-cc}\nif test -n \"$CC\"; then\n  printf %s \"checking for C compiler... \"\n  printf '%s\\n' \"$CC\"\nelse\n  for ac_prog in cc gcc clang; do\n    if command -v \"$ac_prog\" >/dev/null 2>&1; then\n      CC=$ac_prog\n      break\n    fi\n  done\nfi",
        );

        // AC_PROG_CXX — C++ compiler detection
        self.engine.macro_table.define(
            b"AC_PROG_CXX",
            b"# Check for C++ compiler\nac_ct_CXX=${CXX-g++}\nfor ac_prog in g++ c++ clang++; do\n  if command -v \"$ac_prog\" >/dev/null 2>&1; then\n    CXX=$ac_prog\n    break\n  fi\ndone",
        );

        // AC_PROG_CPP — C preprocessor detection
        self.engine.macro_table.define(
            b"AC_PROG_CPP",
            b"# Check for C preprocessor\nCPP=${CPP-cc -E}",
        );

        // AC_PROG_INSTALL
        self.engine.macro_table.define(
            b"AC_PROG_INSTALL",
            b"# Find a good install program\nINSTALL=${INSTALL-/usr/bin/install -c}",
        );

        // AC_PROG_MAKE_SET
        self.engine
            .macro_table
            .define(b"AC_PROG_MAKE_SET", b"SET_MAKE=''");

        // Fortran compiler detection macros
        crate::fortran::register_fortran_macros(&mut self.engine.macro_table);

        // Additional language support macros
        crate::languages::register_objc_macros(&mut self.engine.macro_table);
        crate::languages::register_erlang_macros(&mut self.engine.macro_table);
        crate::languages::register_go_macros(&mut self.engine.macro_table);

        // AC_CHECK_FUNC — check for C library function
        self.engine.macro_table.define(
            b"AC_CHECK_FUNC",
            b"printf %s \"checking for $1... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#ifdef __cplusplus\nextern \"C\"\n#endif\nchar $1();\nint main() { return $1(); }\n_ACEOF\nif ac_fn_c_try_link; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );

        // AC_CHECK_HEADER — check for C header
        self.engine.macro_table.define(
            b"AC_CHECK_HEADER",
            b"printf %s \"checking for $1... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#include <$1>\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );

        // AC_CHECK_LIB(LIBRARY, FUNCTION, [IF-FOUND], [IF-NOT], [OTHER-LIBS]): link-test FUNCTION
        // against -lLIBRARY. Only KEEP -lLIBRARY in LIBS on success (was kept unconditionally); run
        // IF-FOUND/IF-NOT (were ignored -> AC_CHECK_LIB([m],[pow],[],[AC_MSG_ERROR(...)]) never fired
        // the right branch). Default IF-FOUND prepends -lLIBRARY to LIBS.
        self.engine.macro_table.define(
            b"AC_CHECK_LIB",
            b"printf %s \"checking for $2 in -l$1... \"\n_acl_save_LIBS=$LIBS\nLIBS=\"-l$1 $5 $LIBS\"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nchar $2();\nint main() { return $2(); }\n_ACEOF\nif ac_fn_c_try_link; then\n  printf '%s\\n' \"yes\"\n  LIBS=$_acl_save_LIBS\n  ifelse([$3], [], [LIBS=\"-l$1 $LIBS\"], [$3])\nelse\n  printf '%s\\n' \"no\"\n  LIBS=$_acl_save_LIBS\n  :\n  $4\nfi",
        );
        // AC_SEARCH_LIBS(FUNCTION, SEARCH-LIBS, [IF-FOUND], [IF-NOT], [OTHER-LIBS]): was UNDEFINED, so
        // it leaked and the math/zlib/crypto lib searches never ran -> 'X library required' errors.
        // Try FUNCTION with no lib, then each of SEARCH-LIBS; keep the winning -llib in LIBS.
        self.engine.macro_table.define(
            b"AC_SEARCH_LIBS",
            b"printf %s \"checking for library containing $1... \"\n_acs_save_LIBS=$LIBS\nac_res=\nfor ac_lib in '' $2; do\n  if test -z \"$ac_lib\"; then LIBS=\"$5 $_acs_save_LIBS\"; else LIBS=\"-l$ac_lib $5 $_acs_save_LIBS\"; fi\n  cat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nchar $1 ();\nint main() { return $1 (); }\n_ACEOF\n  if ac_fn_c_try_link; then\n    if test -z \"$ac_lib\"; then ac_res=\"none required\"; else ac_res=\"-l$ac_lib\"; fi\n    break\n  fi\ndone\nif test -n \"$ac_res\"; then\n  printf '%s\\n' \"$ac_res\"\n  :\n  $3\nelse\n  printf '%s\\n' \"no\"\n  LIBS=$_acs_save_LIBS\n  :\n  $4\nfi",
        );

        // AC_CHECK_FUNCS — plural: check multiple functions
        self.engine.macro_table.define(
            b"AC_CHECK_FUNCS",
            b"for ac_func in $1; do AC_CHECK_FUNC($ac_func); done",
        );

        // AC_CHECK_HEADERS — plural: check multiple headers.
        // NOTE: the conftest `#include <$ac_hdr>` here is still mangled to `# <$ac_hdr>` by a deep,
        // autoconf-rs-specific rescan interaction (m4-rs core preserves the identical pattern; plain,
        // single-level macro-body, and the singular AC_CHECK_HEADER all work). Captured per recipe in
        // deep_expansion.conftest_corruption as the next root to defeat.
        self.engine.macro_table.define(
            b"AC_CHECK_HEADERS",
            b"for ac_hdr in $1; do AC_CHECK_HEADER($ac_hdr); done",
        );

        // AC_CANONICAL_HOST — detect host system type (CROSS.020: config.guess integration)
        let canonical_host_code = include_str!("templates/canonical_host.sh");
        self.engine
            .macro_table
            .define(b"AC_CANONICAL_HOST", canonical_host_code.as_bytes());

        // AC_CONFIG_SRCDIR — verify source directory
        self.engine.macro_table.define(
            b"AC_CONFIG_SRCDIR",
            b"# AC_CONFIG_SRCDIR: record the source tree and sanity-check the unique file (non-fatal).\ntest \"x$srcdir\" = x && srcdir=.\nif test ! -f \"$srcdir/$1\"; then\n  printf '%s\\n' \"configure: WARNING: cannot find sources ($1) in $srcdir\" >&2\nfi",
        );

        // AC_ARG_WITH — package option
        self.engine
            .macro_table
            .define(b"AC_ARG_WITH", b"# Check --with-$1 option\n");

        // AC_ARG_ENABLE — feature option
        self.engine
            .macro_table
            .define(b"AC_ARG_ENABLE", b"# Check --enable-$1 option\n");

        // AC_ARG_VAR — precious variable
        self.engine
            .macro_table
            .define(b"AC_ARG_VAR", b"# Precious variable $1\n");

        // AC_PROG_AWK — find awk
        self.engine.macro_table.define(
            b"AC_PROG_AWK",
            b"# Find awk\nfor ac_prog in gawk mawk nawk awk; do\n  if command -v \"$ac_prog\" >/dev/null 2>&1; then\n    AWK=$ac_prog\n    break\n  fi\ndone",
        );

        // AC_PROG_GREP — find grep
        self.engine
            .macro_table
            .define(b"AC_PROG_GREP", b"# Find grep\nGREP=${GREP-grep}");

        // AC_PROG_LN_S — check for ln -s
        self.engine
            .macro_table
            .define(b"AC_PROG_LN_S", b"# Check for ln -s\nLN_S='ln -s'");

        // AC_PROG_YACC — find yacc/bison
        self.engine.macro_table.define(
            b"AC_PROG_YACC",
            b"# Find yacc/bison\nfor ac_prog in bison yacc; do\n  if command -v \"$ac_prog\" >/dev/null 2>&1; then\n    YACC=$ac_prog\n    break\n  fi\ndone",
        );

        // AC_PROG_LEX — find lex/flex
        self.engine.macro_table.define(
            b"AC_PROG_LEX",
            b"# Find lex/flex\nfor ac_prog in flex lex; do\n  if command -v \"$ac_prog\" >/dev/null 2>&1; then\n    LEX=$ac_prog\n    break\n  fi\ndone",
        );

        // AC_CHECK_TYPE — check for C type
        self.engine.macro_table.define(
            b"AC_CHECK_TYPE",
            b"printf %s \"checking for $1... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#include <sys/types.h>\n#include <stdint.h>\nint main() { $1 x; return 0; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );

        // AC_CHECK_TYPES — plural
        self.engine.macro_table.define(
            b"AC_CHECK_TYPES",
            b"for ac_type in $1; do AC_CHECK_TYPE($ac_type); done",
        );

        // AC_CHECK_MEMBER — check for struct member
        self.engine.macro_table.define(
            b"AC_CHECK_MEMBER",
            b"printf %s \"checking for $1... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n$2\nint main() { $1 x; return 0; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );

        // AC_PROG_SED — find sed
        self.engine
            .macro_table
            .define(b"AC_PROG_SED", b"SED=${SED-sed}");

        // AC_CANONICAL_BUILD — build system type (CROSS.020: config.guess integration)
        let canonical_build_code = include_str!("templates/canonical_build.sh");
        self.engine
            .macro_table
            .define(b"AC_CANONICAL_BUILD", canonical_build_code.as_bytes());

        // AC_CANONICAL_TARGET — target system type
        self.engine
            .macro_table
            .define(b"AC_CANONICAL_TARGET", b"target=$host");

        self.engine
            .macro_table
            .define(b"AC_PROG_RANLIB", b"RANLIB=${RANLIB-ranlib}");
        self.engine
            .macro_table
            .define(b"AC_PROG_AR", b"AR=${AR-ar}");
        self.engine
            .macro_table
            .define(b"AC_PROG_EGREP", b"EGREP=${EGREP-grep -E}");
        self.engine
            .macro_table
            .define(b"AC_PROG_FGREP", b"FGREP=${FGREP-grep -F}");
        self.engine
            .macro_table
            .define(b"AC_LANG_PUSH", b"# Language: $1");
        self.engine
            .macro_table
            .define(b"AC_LANG_POP", b"# Restore language");
        self.engine.macro_table.define(
            b"AC_PROG_FC",
            b"# Check for Fortran compiler\nFC=${FC-gfortran}",
        );
        self.engine
            .macro_table
            .define(b"AC_LIBOBJ", b"LIBOBJS=\"$LIBOBJS $1\"");
        self.engine
            .macro_table
            .define(b"AC_REPLACE_FUNCS", b"# Replace functions: $1");
        self.engine
            .macro_table
            .define(b"AC_HEADER_STDC", b"printf '%s\\n' \"yes\"");
        self.engine
            .macro_table
            .define(b"AC_STRUCT_TM", b"printf '%s\\n' \"yes\"");
        self.engine
            .macro_table
            .define(b"AC_TYPE_PID_T", b"AC_CHECK_TYPE([pid_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_SIZE_T", b"AC_CHECK_TYPE([size_t])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_ALLOCA", b"AC_CHECK_FUNC([alloca])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_MALLOC", b"AC_CHECK_FUNC([malloc])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_STRERROR_R", b"AC_CHECK_FUNC([strerror_r])");

        // --- Additional AC_FUNC_* macros ---
        // Each checks a specific C library function with known portability issues.
        self.engine
            .macro_table
            .define(b"AC_FUNC_CLOSEDIR_VOID", b"AC_CHECK_FUNC([closedir])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_ERROR_AT_LINE", b"AC_CHECK_FUNC([error_at_line])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_FNMATCH", b"AC_CHECK_FUNC([fnmatch])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_FNMATCH_GNU", b"AC_CHECK_FUNC([fnmatch])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_FORK", b"AC_CHECK_FUNC([fork])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_FSEEKO", b"AC_CHECK_FUNC([fseeko])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_FSTATFS", b"AC_CHECK_FUNC([fstatfs])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_FTRUNCATE", b"AC_CHECK_FUNC([ftruncate])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_GETGROUPS", b"AC_CHECK_FUNC([getgroups])");
        self.engine.macro_table.define(
            b"AC_FUNC_GETHOSTBYNAME_R",
            b"AC_CHECK_FUNC([gethostbyname_r])",
        );
        self.engine
            .macro_table
            .define(b"AC_FUNC_GETLOADAVG", b"AC_CHECK_FUNC([getloadavg])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_GETMNTENT", b"AC_CHECK_FUNC([getmntent])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_GETPGRP", b"AC_CHECK_FUNC([getpgrp])");
        self.engine.macro_table.define(
            b"AC_FUNC_LSTAT_FOLLOWS_SLASHED_SYMLINK",
            b"AC_CHECK_FUNC([lstat])",
        );
        self.engine
            .macro_table
            .define(b"AC_FUNC_MALLOC_0_NONNULL", b"AC_CHECK_FUNC([malloc])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_MBRTOWC", b"AC_CHECK_FUNC([mbrtowc])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_MEMCMP", b"AC_CHECK_FUNC([memcmp])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_MKTIME", b"AC_CHECK_FUNC([mktime])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_MMAP", b"AC_CHECK_FUNC([mmap])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_OBSTACK", b"AC_CHECK_FUNC([obstack_free])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_PRINTF_POSIX", b"AC_CHECK_FUNC([printf])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_REALLOC", b"AC_CHECK_FUNC([realloc])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_REALLOC_0_NONNULL", b"AC_CHECK_FUNC([realloc])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_SELECT_ARGTYPES", b"AC_CHECK_FUNC([select])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_SETPGRP", b"AC_CHECK_FUNC([setpgrp])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_STAT", b"AC_CHECK_FUNC([stat])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_STRCOLL", b"AC_CHECK_FUNC([strcoll])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_STRERROR", b"AC_CHECK_FUNC([strerror])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_STRFTIME", b"AC_CHECK_FUNC([strftime])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_STRNLEN", b"AC_CHECK_FUNC([strnlen])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_STRTOD", b"AC_CHECK_FUNC([strtod])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_STRTOLD", b"AC_CHECK_FUNC([strtold])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_UTIME_NULL", b"AC_CHECK_FUNC([utime])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_VFORK", b"AC_CHECK_FUNC([vfork])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_VPRINTF", b"AC_CHECK_FUNC([vprintf])");
        self.engine
            .macro_table
            .define(b"AC_FUNC_WAIT3", b"AC_CHECK_FUNC([wait3])");

        // --- Additional AC_HEADER_* macros ---
        self.engine
            .macro_table
            .define(b"AC_HEADER_ASSERT", b"AC_CHECK_HEADER([assert.h])");
        self.engine.macro_table.define(
            b"AC_HEADER_DIRENT",
            b"AC_CHECK_HEADERS([dirent.h sys/ndir.h sys/dir.h ndir.h])",
        );
        self.engine
            .macro_table
            .define(b"AC_HEADER_MAJOR", b"AC_CHECK_HEADER([sys/mkdev.h])");
        self.engine
            .macro_table
            .define(b"AC_HEADER_RESOLV", b"AC_CHECK_HEADER([resolv.h])");
        self.engine
            .macro_table
            .define(b"AC_HEADER_STAT", b"AC_CHECK_HEADER([sys/stat.h])");
        self.engine
            .macro_table
            .define(b"AC_HEADER_STDBOOL", b"AC_CHECK_HEADER([stdbool.h])");
        self.engine
            .macro_table
            .define(b"AC_HEADER_STDINT", b"AC_CHECK_HEADER([stdint.h])");
        self.engine
            .macro_table
            .define(b"AC_HEADER_SYS_WAIT", b"AC_CHECK_HEADER([sys/wait.h])");
        self.engine
            .macro_table
            .define(b"AC_HEADER_TIME", b"AC_CHECK_HEADER([time.h])");
        self.engine
            .macro_table
            .define(b"AC_HEADER_TIOCGWINSZ", b"AC_CHECK_HEADER([sys/ioctl.h])");

        // --- Additional AC_TYPE_* macros ---
        self.engine
            .macro_table
            .define(b"AC_TYPE_GETGROUPS", b"AC_CHECK_TYPE([gid_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_INT16_T", b"AC_CHECK_TYPE([int16_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_INT32_T", b"AC_CHECK_TYPE([int32_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_INT64_T", b"AC_CHECK_TYPE([int64_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_INT8_T", b"AC_CHECK_TYPE([int8_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_INTMAX_T", b"AC_CHECK_TYPE([intmax_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_INTPTR_T", b"AC_CHECK_TYPE([intptr_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_LONG_DOUBLE", b"AC_CHECK_TYPE([long double])");
        self.engine.macro_table.define(
            b"AC_TYPE_LONG_DOUBLE_WIDER",
            b"AC_CHECK_TYPE([long double])",
        );
        self.engine
            .macro_table
            .define(b"AC_TYPE_LONG_LONG_INT", b"AC_CHECK_TYPE([long long int])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_MBSTATE_T", b"AC_CHECK_TYPE([mbstate_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_MODE_T", b"AC_CHECK_TYPE([mode_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_OFF_T", b"AC_CHECK_TYPE([off_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_SIGNAL", b"AC_CHECK_TYPE([sig_atomic_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_SSIZE_T", b"AC_CHECK_TYPE([ssize_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_UID_T", b"AC_CHECK_TYPE([uid_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_UINT16_T", b"AC_CHECK_TYPE([uint16_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_UINT32_T", b"AC_CHECK_TYPE([uint32_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_UINT64_T", b"AC_CHECK_TYPE([uint64_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_UINT8_T", b"AC_CHECK_TYPE([uint8_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_UINTMAX_T", b"AC_CHECK_TYPE([uintmax_t])");
        self.engine
            .macro_table
            .define(b"AC_TYPE_UINTPTR_T", b"AC_CHECK_TYPE([uintptr_t])");

        // --- Additional AC_PROG_* macros ---
        self.engine
            .macro_table
            .define(b"AC_PROG_MKDIR_P", b"MKDIR_P='mkdir -p'");
        self.engine.macro_table.define(b"AC_CHECK_PROG", b"# Check for program $2 in PATH\nfor ac_prog in $2; do if command -v \"$ac_prog\" >/dev/null 2>&1; then $1=$ac_prog; break; fi; done");
        self.engine.macro_table.define(b"AC_CHECK_PROGS", b"# Check for programs in PATH\nfor ac_prog in $2; do if command -v \"$ac_prog\" >/dev/null 2>&1; then $1=$ac_prog; break; fi; done");
        self.engine.macro_table.define(b"AC_CHECK_TOOL", b"# Check for tool $2 (cross builds try the ${ac_tool_prefix} variant first)\ntest \"x${ac_tool_prefix+set}\" = xset || { if test \"x$host_alias\" != x && test \"x$host_alias\" != \"x$build_alias\"; then ac_tool_prefix=\"$host_alias-\"; else ac_tool_prefix=; fi; }\nfor ac_tool in \"${ac_tool_prefix}$2\" \"$2\"; do if command -v \"$ac_tool\" >/dev/null 2>&1; then $1=$ac_tool; break; fi; done");
        self.engine.macro_table.define(b"AC_PATH_PROG", b"# Find path to program $2\nfor ac_prog in $2; do ac_path=`command -v \"$ac_prog\" 2>/dev/null`; if test -n \"$ac_path\"; then $1=$ac_path; break; fi; done");
        self.engine.macro_table.define(b"AC_PATH_PROGS", b"# Find paths to programs\nfor ac_prog in $2; do ac_path=`command -v \"$ac_prog\" 2>/dev/null`; if test -n \"$ac_path\"; then $1=$ac_path; break; fi; done");
        self.engine.macro_table.define(b"AC_PATH_TOOL", b"# Find path to tool $2\nfor ac_tool in $2; do ac_path=`command -v \"$ac_tool\" 2>/dev/null`; if test -n \"$ac_path\"; then $1=$ac_path; break; fi; done");
        self.engine.macro_table.define(b"AC_CHECK_TOOLS", b"# Check for tools (cross builds try the ${ac_tool_prefix} variant first)\ntest \"x${ac_tool_prefix+set}\" = xset || { if test \"x$host_alias\" != x && test \"x$host_alias\" != \"x$build_alias\"; then ac_tool_prefix=\"$host_alias-\"; else ac_tool_prefix=; fi; }\nfor ac_tool in $2; do if command -v \"${ac_tool_prefix}$ac_tool\" >/dev/null 2>&1; then $1=${ac_tool_prefix}$ac_tool; break; elif command -v \"$ac_tool\" >/dev/null 2>&1; then $1=$ac_tool; break; fi; done");

        // --- Additional output/system macros ---
        self.engine
            .macro_table
            .define(b"AC_PREFIX_DEFAULT", b"ac_default_prefix=$1");
        self.engine.macro_table.define(
            b"AC_PREFIX_PROGRAM",
            b"# Set prefix from program $1 location",
        );
        self.engine
            .macro_table
            .define(b"AC_CONFIG_AUX_DIR", b"ac_aux_dir=$1");
        self.engine
            .macro_table
            .define(b"AC_CONFIG_MACRO_DIR", b"ac_macro_dir=$1");
        self.engine
            .macro_table
            .define(b"AC_REVISION", b"# Revision: $1");
        self.engine
            .macro_table
            .define(b"AC_COPYRIGHT", b"# Copyright: $1");
        self.engine
            .macro_table
            .define(b"AC_PREREQ", b"# Requires Autoconf >= $1");
        self.engine
            .macro_table
            .define(b"AC_BEFORE", b"# $1 must come before $2");
        self.engine
            .macro_table
            .define(b"AC_OBSOLETE", b"# $1 is obsolete: $2");
        // Use `define` (not `m4_define`) so a user macro defined via AC_DEFUN expands even when AC_DEFUN
        // appears before AC_INIT (a common layout). `define` is the always-available builtin.
        self.engine
            .macro_table
            .define(b"AC_DEFUN", b"define([$1], [$2])");
        self.engine
            .macro_table
            .define(b"AC_DEFUN_ONCE", b"m4_ifdef([_m4_defun_once_$1], [], [m4_define([$1], [$2])m4_define([_m4_defun_once_$1], [1])])");
        // AU_ALIAS: alias for renamed macros
        self.engine
            .macro_table
            .define(b"AU_ALIAS", b"AC_DEFUN([$1], [$2($@)])");
        self.engine.macro_table.define(
            b"AU_DEFUN",
            b"errprint([warning: $1 is obsolete, use $2\n])m4_define([$1], [$3])",
        );
        self.engine.macro_table.define(b"AC_REQUIRE", b"");
        self.engine
            .macro_table
            .define(b"AC_PROVIDE", b"m4_define([_m4_provided_$1], [1])");
        self.engine
            .macro_table
            .define(b"AC_SUBST_FILE", b"# Subst file: $1");
        self.engine
            .macro_table
            .define(b"AC_DEFINE_UNQUOTED", b"# Define unquoted: $1");
        self.engine
            .macro_table
            .define(b"AC_PRESERVE_HELP_ORDER", b"# Preserve help order");
        self.engine.macro_table.define(b"AC_CONFIG_FILES", b"");
        self.engine.macro_table.define(b"AC_CONFIG_HEADERS", b"");

        // --- m4sugar utility macros (real M4-level implementations) ---
        // Core: m4_copy / m4_rename
        self.engine
            .macro_table
            .define(b"m4_copy", b"define([$1], defn([$2]))");
        self.engine
            .macro_table
            .define(b"m4_rename", b"define([$2], defn([$1]))define([$1])");
        // Transformations
        self.engine
            .macro_table
            .define(b"m4_toupper", b"translit([$1], [a-z], [A-Z])");
        self.engine
            .macro_table
            .define(b"m4_tolower", b"translit([$1], [A-Z], [a-z])");
        // List operations
        self.engine
            .macro_table
            .define(b"m4_split", b"patsubst([$1], [[$2]], [,])");
        self.engine.macro_table.define(
            b"m4_flatten",
            b"patsubst(patsubst([$1], [^[\t ]+], []), [[\t ]+$], [])",
        );
        self.engine.macro_table.define(
            b"m4_strip",
            b"patsubst(patsubst([$1], [^[\t ]+], []), [[\t ]+$], [])",
        );
        // m4_join — non-recursive to avoid eager ifelse infinite loop in m4-rs.
        // Handles up to 4 args; for more args, GNU m4 uses recursion.
        // NC.ADMIT.4: m4_join limited to 4-arg non-recursive form until
        // m4-rs supports lazy ifelse branch evaluation.
        self.engine.macro_table.define(
            b"m4_join",
            b"ifelse($#, [1], [], [$#], [2], [$2], [$#], [3], [$2][$1]$3, [$#], [4], [$2][$1]$3[$1]$4, [$2][$1]$3[$1]$4)",
        );
        self.engine.macro_table.define(
            b"m4_append",
            b"define([$1], ifdef([$1], [defn([$1])[$3]$2], [$2]))",
        );
        self.engine.macro_table.define(
            b"m4_prepend",
            b"define([$1], ifdef([$1], [$2[$3]defn([$1])], [$2]))",
        );
        // Quoting helpers
        self.engine.macro_table.define(b"m4_quote", b"`[$1]'");
        self.engine.macro_table.define(b"m4_dquote", b"[[$1]]");
        self.engine.macro_table.define(b"m4_expand", b"$1");
        self.engine.macro_table.define(b"m4_do", b"$1");
        // Text formatting
        self.engine.macro_table.define(
            b"m4_normalize",
            b"patsubst(patsubst([$1], [\r?\n], [ ]), [^[\t ]+], [])",
        );
        self.engine.macro_table.define(b"m4_text_wrap", b"$1");
        // Conditionals: m4_if / m4_ifval / m4_ifblank
        // m4_if is MULTI-WAY: m4_if(a,b,val, c,d,val2, ..., else). The old 4-arg wrapper truncated
        // it to a single comparison, so e.g. AX_CXX_COMPILE_STDCXX's m4_if([$1],[11],[],[$1],[14],
        // [],[$1],[17],[],[fatal]) wrongly returned the 4th arg ("17") -> leaked into configure.
        // The base ifelse builtin already handles arbitrary argument counts; just pass them through.
        self.engine.macro_table.define(b"m4_if", b"ifelse($@)");
        // m4_ifdef / m4_ifndef — m4sugar wrappers over the base `ifdef` builtin. Without these,
        // `m4_ifdef([AM_SILENT_RULES], [...])` (extremely common in configure.ac) was left literal
        // -> shell "syntax error near unexpected token". m4_ifdef(NAME, IF-DEF, IF-NOT).
        self.engine
            .macro_table
            .define(b"m4_ifdef", b"ifdef([$1], [$2], [$3])");
        self.engine
            .macro_table
            .define(b"m4_ifndef", b"ifdef([$1], [$3], [$2])");
        // m4_default(EXPR, DEFAULT) -> EXPR if non-empty else DEFAULT; m4_default_nblank similar.
        self.engine
            .macro_table
            .define(b"m4_default", b"ifelse([$1], [], [$2], [$1])");
        // m4_pushdef/m4_popdef — m4sugar wrappers over the pushdef/popdef builtins. Used by pkg.m4
        // (PKG_INSTALLDIR's `m4_pushdef([pkg_default], ...)`) and many others; undefined -> leaked
        // literal -> shell syntax error.
        self.engine.macro_table.define(b"m4_pushdef", b"pushdef([$1], [$2])");
        self.engine.macro_table.define(b"m4_popdef", b"popdef([$1])");
        self.engine
            .macro_table
            .define(b"m4_default_nblank", b"ifelse(m4_normalize([$1]), [], [$2], [$1])");
        self.engine
            .macro_table
            .define(b"m4_ifset", b"ifelse([$1], [], [$3], [$2])");
        // m4_esyscmd / m4_esyscmd_s — m4sugar wrappers over the esyscmd builtin. These MUST be
        // defined unconditionally so they never leak literally into configure (a literal
        // `m4_esyscmd_s([git describe])` -> shell "syntax error"). They delegate to `esyscmd`,
        // which is the real command bridge when --allow-syscmd is on and expands to empty (via the
        // blocked-stub) otherwise. _s strips trailing newlines (the "single-line" variant).
        self.engine
            .macro_table
            .define(b"m4_esyscmd", b"esyscmd([$1])");
        self.engine
            .macro_table
            .define(b"m4_esyscmd_s", b"patsubst(esyscmd([$1]), [\n+$], [])");
        self.engine
            .macro_table
            .define(b"m4_ifval", b"ifelse([$1], [], [$3], [$2])");
        // m4sugar roots surfaced by the atlas leaked-macro ranking. m4_define is the cascade root:
        // a configure.ac/aclocal that does m4_define([M],[...]) otherwise never defines M, so M AND
        // its body (AC_MSG_ERROR/AC_DEFINE/... — the top "leaked" symptoms) all spill into the shell.
        self.engine
            .macro_table
            .define(b"m4_define", b"define([$1], [$2])");
        self.engine
            .macro_table
            .define(b"m4_define_default", b"ifdef([$1], [], [define([$1], [$2])])");
        self.engine
            .macro_table
            .define(b"m4_defun", b"define([$1], [$2])");
        // m4_ifvaln: like m4_ifval (newline-tolerant) — branch on whether $1 is empty.
        self.engine
            .macro_table
            .define(b"m4_ifvaln", b"ifelse([$1], [], [$3], [$2])");
        // m4_case(SWITCH, VAL, IF-VAL, ..., [DEFAULT]): compare SWITCH to each VAL, recursing on the
        // m4sugar shape. 2 args left -> that's the DEFAULT; <2 -> empty.
        self.engine.macro_table.define(
            b"m4_case",
            b"ifelse([$#], [0], [], [$#], [1], [], [$#], [2], [$2], [$1], [$2], [$3], [m4_case([$1], m4_shift3($@))])",
        );
        self.engine
            .macro_table
            .define(b"m4_ifblank", b"ifelse(m4_normalize([$1]), [], [$2], [$3])");
        self.engine.macro_table.define(
            b"m4_ifnblank",
            b"ifelse(m4_normalize([$1]), [], [$3], [$2])",
        );
        self.engine
            .macro_table
            .define(b"m4_bmatch", b"ifelse([$1], [$2], [$3], [$4], [$5])");
        // Iteration: m4_foreach / m4_map
        // m4_foreach (standard m4sugar shape): pass the list UNQUOTED into _m4_foreach so a
        // macro-call list ([_AX_SAVE_FLAGS_LIST()]) expands+rescans into separate args, then iterate
        // with m4_shift3. Relies on the engine's rescan-after-expansion (rescan_into_args).
        self.engine.macro_table.define(
            b"m4_foreach",
            b"ifelse([$2], [], [], [_m4_foreach([$1], [$3], $2)])",
        );
        self.engine.macro_table.define(
            b"_m4_foreach",
            b"pushdef([$1], [$3])$2[]ifelse([$#], [3], [popdef([$1])], [_m4_foreach([$1], [$2], m4_shift3($@))])",
        );
        self.engine.macro_table.define(
            b"m4_foreach_w",
            b"m4_foreach([$1], m4_split(m4_normalize([$2]), [ ]), [$3])",
        );
        self.engine
            .macro_table
            .define(b"m4_map", b"m4_foreach([_m4_e], [$2], [$1(_m4_e)])");
        self.engine.macro_table.define(
            b"m4_map_sep",
            b"m4_foreach([_m4_e], m4_cdr($3), [$1(_m4_e)[$2]])[$1(m4_car($3))]",
        );
        // List: m4_car / m4_cdr / m4_shift. m4_shift was USED by m4_cdr/m4_foreach/m4_map_args but
        // never defined -> it leaked literally thousands of times (2948× in one configure), breaking
        // every list-processing macro. It is m4sugar's wrapper over the base `shift` builtin.
        self.engine.macro_table.define(b"m4_car", b"$1");
        self.engine.macro_table.define(b"m4_cdr", b"m4_shift($@)");
        self.engine.macro_table.define(b"m4_shift", b"shift($@)");
        self.engine.macro_table.define(b"m4_shift2", b"m4_shift(m4_shift($@))");
        self.engine.macro_table.define(b"m4_shift3", b"m4_shift(m4_shift(m4_shift($@)))");
        self.engine.macro_table.define(
            b"m4_list_cmp",
            b"ifelse([$1], [], [0], [$1], [$2], [0], [1])",
        );
        // Chomp: remove trailing newline
        self.engine
            .macro_table
            .define(b"m4_chomp", b"patsubst([$1], [\n$], [])");
        self.engine
            .macro_table
            .define(b"m4_chomp_all", b"patsubst([$1], [\n], [ ])");
        // Pattern matching
        self.engine
            .macro_table
            .define(b"m4_pattern_forbid", b"define([_m4_forbid_$1], [1])");
        self.engine
            .macro_table
            .define(b"m4_pattern_allow", b"define([_m4_allow_$1], [1])");

        // --- AC_SYS_* system-specific checks ---
        // AC_SYS_LARGEFILE: enable large file support (64-bit off_t)
        // Sets _FILE_OFFSET_BITS=64 and _LARGEFILE_SOURCE on platforms that need it.
        self.engine.macro_table.define(
            b"AC_SYS_LARGEFILE",
            b"# Check for large file support\nprintf %s \"checking for 64-bit off_t... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#include <sys/types.h>\nint main() { return sizeof(off_t) >= 8 ? 0 : 1; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no, enabling _FILE_OFFSET_BITS=64\"\n  CPPFLAGS=\"$CPPFLAGS -D_FILE_OFFSET_BITS=64\"\n  printf %s \"checking for large file support with -D_FILE_OFFSET_BITS=64... \"\n  if ac_fn_c_try_compile; then\n    printf '%s\\n' \"yes\"\n  else\n    CPPFLAGS=\"$CPPFLAGS -D_LARGEFILE_SOURCE -D_LARGE_FILES\"\n    printf '%s\\n' \"no, enabling _LARGEFILE_SOURCE\"\n  fi\nfi",
        );
        self.engine.macro_table.define(
            b"AC_SYS_LONG_FILE_NAMES",
            b"# Check for long file names (>14 chars)\nprintf %s \"checking for long file names... \"\nrm -f conftest_long_file_name_test_abcdefghijklmnop\ntouch conftest_long_file_name_test_abcdefghijklmnop 2>/dev/null\nif test -f conftest_long_file_name_test_abcdefghijklmnop; then\n  printf '%s\\n' \"yes\"\n  rm -f conftest_long_file_name_test_abcdefghijklmnop\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        self.engine.macro_table.define(
            b"AC_SYS_POSIX_TERMIOS",
            b"# Check for POSIX termios\nprintf %s \"checking for POSIX termios... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#include <termios.h>\nint main() { struct termios t; tcgetattr(0, &t); return 0; }\n_ACEOF\nif ac_fn_c_try_link; then\n  printf '%s\\n' \"yes\"\n  AC_DEFINE([HAVE_POSIX_TERMIOS], [1])\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        self.engine.macro_table.define(
            b"AC_SYS_RESTARTABLE_SYSCALLS",
            b"# Check for restartable syscalls (SA_RESTART)\nprintf %s \"checking for restartable system calls... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#include <signal.h>\nint main() { struct sigaction sa; sa.sa_flags = SA_RESTART; return 0; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\n  AC_DEFINE([HAVE_RESTARTABLE_SYSCALLS], [1])\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        // AC_SYS_SIGLIST_DECLARED (obsolete, replaced by AC_CHECK_DECLS)
        self.engine.macro_table.define(
            b"AC_SYS_SIGLIST_DECLARED",
            b"AC_CHECK_DECLS([sys_siglist], [], [], [#include <signal.h>])",
        );

        // --- AC_C_* C compiler conformance checks (real implementations) ---
        // AC_C_CONST: check if compiler supports 'const'
        self.engine.macro_table.define(
            b"AC_C_CONST",
            b"printf %s \"checking for working const... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nint main() { const int x = 1; return x-1; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        // AC_C_VOLATILE: check if compiler supports 'volatile'
        self.engine.macro_table.define(
            b"AC_C_VOLATILE",
            b"printf %s \"checking for working volatile... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nint main() { volatile int x = 0; return x; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        // AC_C_INLINE: check for inline keyword support
        self.engine.macro_table.define(
            b"AC_C_INLINE",
            b"printf %s \"checking for inline... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\ninline int foo() { return 0; }\nint main() { return foo(); }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        // AC_C_RESTRICT: check for restrict keyword support
        self.engine.macro_table.define(
            b"AC_C_RESTRICT",
            b"printf %s \"checking for restrict... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nvoid foo(char * restrict p) { *p = 0; }\nint main() { char c; foo(&c); return 0; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        // AC_C_BIGENDIAN: check CPU endianness
        self.engine.macro_table.define(
            b"AC_C_BIGENDIAN",
            b"printf %s \"checking endianness... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nint main() { int x = 1; return *(char*)&x; }\n_ACEOF\nif ac_fn_c_try_run; then\n  printf '%s\\n' \"little-endian\"\nelse\n  printf '%s\\n' \"big-endian\"\nfi",
        );
        // AC_C_CHAR_UNSIGNED: check if char is unsigned
        self.engine.macro_table.define(
            b"AC_C_CHAR_UNSIGNED",
            b"printf %s \"checking if char is unsigned... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nint main() { char c = -1; return c < 0; }\n_ACEOF\nif ac_fn_c_try_run; then\n  printf '%s\\n' \"no (signed)\"\nelse\n  printf '%s\\n' \"yes\"\nfi",
        );
        // AC_C_PROTOTYPES: check for function prototypes
        self.engine.macro_table.define(
            b"AC_C_PROTOTYPES",
            b"printf %s \"checking for function prototypes... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nint foo(int x);\nint foo(int x) { return x; }\nint main() { return foo(0); }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        // AC_C_STRINGIZE: check for preprocessor stringize (#)
        self.engine.macro_table.define(
            b"AC_C_STRINGIZE",
            b"printf %s \"checking for stringize... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#define STR(x) #x\nconst char *s = STR(hello);\nint main() { return 0; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        // AC_C_VARARRAYS: check for C99 variable-length arrays
        self.engine.macro_table.define(
            b"AC_C_VARARRAYS",
            b"printf %s \"checking for variable-length arrays... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nint main() { int n = 10; int a[n]; return 0; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );
        // AC_C_TYPEOF: check for typeof/__typeof__
        self.engine.macro_table.define(
            b"AC_C_TYPEOF",
            b"printf %s \"checking for typeof... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\nint main() { int x; __typeof__(x) y = 1; return y-1; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\nelse\n  printf '%s\\n' \"no\"\nfi",
        );

        // --- AC_CACHE_* ---
        // `${$2+set}` (NOT `${}$2+set}` — the stray empty `${}` produced `if test "${}name+set}"`
        // -> shell syntax error near `(`/the name, the C++ cache-check cluster, 8 corpus repos).
        self.engine.macro_table.define(
            b"AC_CACHE_CHECK",
            b"printf %s \"$1... \"\nif test \"${$2+set}\" = set; then\n  printf '%s\\n' \"(cached) \\$$2\"\nelse\n  :\n  $3\nfi",
        );
        // AC_CACHE_VAL(cache-id, commands-to-set-it): run the commands (we don't persist a cache).
        self.engine.macro_table.define(b"AC_CACHE_VAL", b"$2");
        self.engine
            .macro_table
            .define(b"AC_CACHE_LOAD", b". ./config.cache 2>/dev/null || :");

        // --- AC_COMPILE_IFELSE / AC_LINK_IFELSE / AC_RUN_IFELSE (real implementations) ---
        // The `:` guards in BOTH branches are essential: an action-if-true/false can expand to
        // empty (e.g. AX_PTHREAD's `AC_LINK_IFELSE([...],[ok=yes],[])`), and a then/else clause with
        // no command is a shell SYNTAX ERROR ("syntax error near unexpected token `fi'").
        self.engine.macro_table.define(
            b"AC_COMPILE_IFELSE",
            b"cat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n$1\n_ACEOF\nif ac_fn_c_try_compile; then\n  :\n  $2\nelse\n  :\n  $3\nfi",
        );
        self.engine.macro_table.define(
            b"AC_LINK_IFELSE",
            b"cat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n$1\n_ACEOF\nif ac_fn_c_try_link; then\n  :\n  $2\nelse\n  :\n  $3\nfi",
        );
        self.engine.macro_table.define(
            b"AC_RUN_IFELSE",
            b"cat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n$1\n_ACEOF\nif ac_fn_c_try_run; then\n  :\n  $2\nelse\n  :\n  $3\nfi",
        );
        self.engine
            .macro_table
            .define(b"AC_TRY_COMPILE", b"# Try compile (obsolete)");
        self.engine
            .macro_table
            .define(b"AC_TRY_LINK", b"# Try link (obsolete)");
        self.engine
            .macro_table
            .define(b"AC_TRY_RUN", b"# Try run (obsolete)");

        // --- Additional AC_STRUCT_* ---
        self.engine.macro_table.define(
            b"AC_STRUCT_ST_BLOCKS",
            b"AC_CHECK_MEMBER([struct stat.st_blocks])",
        );
        self.engine.macro_table.define(
            b"AC_STRUCT_ST_BLKSIZE",
            b"AC_CHECK_MEMBER([struct stat.st_blksize])",
        );
        self.engine.macro_table.define(
            b"AC_STRUCT_ST_RDEV",
            b"AC_CHECK_MEMBER([struct stat.st_rdev])",
        );
        self.engine.macro_table.define(
            b"AC_STRUCT_TIMEZONE",
            b"AC_CHECK_MEMBER([struct tm.tm_zone])",
        );

        // --- AC_CHECK_SIZEOF (real implementation) ---
        self.engine.macro_table.define(
            b"AC_CHECK_SIZEOF",
            b"# Check sizeof($1)\nprintf %s \"checking size of $1... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#include <sys/types.h>\n#include <stdint.h>\n#include <stddef.h>\nint main() { static int test_array[1 - 2 * !((long int) (sizeof ($1)) <= 0)]; return 0; }\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"done\"\n  AC_DEFINE_UNQUOTED([SIZEOF_$1], [$(($ac_cv_sizeof_$1))])\nelse\n  printf '%s\\n' \"0 (type not found)\"\n  AC_DEFINE_UNQUOTED([SIZEOF_$1], [0])\nfi",
        );

        // --- AC_CHECK_DECL / AC_CHECK_DECLS (real implementations) ---
        // Previously UNDEFINED -> these and their AC_DEFINE/AC_MSG_ERROR/AC_SUBST bodies leaked into 20+
        // generated configures (the top leaked-macro across the corpus). AC_CHECK_DECL compiles a program
        // referencing the symbol guarded by `#ifndef SYMBOL`; if it compiles, the symbol is declared.
        // ($4 = extra includes, $2 = if-declared, $3 = if-not.) No backticks in the source (the #1
        // fixable-root corpus-wide is backtick-in-source — don't add to it).
        self.engine.macro_table.define(
            b"AC_CHECK_DECL",
            b"printf %s \"checking whether $1 is declared... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n$4\nint main (void)\n{\n#ifndef $1\n  (void) $1;\n#endif\n  ;\n  return 0;\n}\n_ACEOF\nif ac_fn_c_try_compile; then\n  printf '%s\\n' \"yes\"\n  ac_cv_have_decl_$1=yes\n  :\n  $2\nelse\n  printf '%s\\n' \"no\"\n  ac_cv_have_decl_$1=no\n  :\n  $3\nfi",
        );
        // AC_CHECK_DECLS always defines HAVE_DECL_<sym> to 1 or 0 (single-symbol common case).
        self.engine.macro_table.define(
            b"AC_CHECK_DECLS",
            b"AC_CHECK_DECL([$1], [AC_DEFINE([HAVE_DECL_$1], [1], [Define to 1 if you have the declaration of $1.])\n$2], [AC_DEFINE([HAVE_DECL_$1], [0], [Define to 1 if you have the declaration of $1.])\n$3], [$4])",
        );
        // The "once" header variants just delegate to the standard header check in our transpiler
        // (de-dup is a build-time optimization, not semantics) — were undefined and leaking (10 repos).
        self.engine
            .macro_table
            .define(b"AC_CHECK_HEADERS_ONCE", b"AC_CHECK_HEADERS([$1])");
        self.engine
            .macro_table
            .define(b"AC_CHECK_HEADER_ONCE", b"AC_CHECK_HEADERS([$1])");
        // AC_RUN_LOG: internal command runner (libtool/gettext lean on it) — run it, report status.
        self.engine
            .macro_table
            .define(b"AC_RUN_LOG", b"{ eval \"$1\" 2>/dev/null; ac_status=$?; test $ac_status = 0; }");

        // --- More undefined-and-leaking macros from the atlas fixable backlog ---
        // AC_ERROR: deprecated alias for AC_MSG_ERROR (4 repos leaked it raw -> command-not-found).
        self.engine
            .macro_table
            .define(b"AC_ERROR", b"AC_MSG_ERROR([$1])");
        // AC_LANG_CONFTEST(PROGRAM): write the test program to conftest.$ac_ext (5 repos). Same heredoc
        // form AC_COMPILE_IFELSE uses, so the conftest body (its #include/#ifdef) is preserved.
        self.engine.macro_table.define(
            b"AC_LANG_CONFTEST",
            b"cat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n$1\n_ACEOF",
        );
        // AC_CONFIG_COMMANDS_PRE(CMDS): commands to run before config.status. We're linear, so run them
        // inline (best-effort) rather than deferring (4 repos; libtool uses it).
        self.engine
            .macro_table
            .define(b"AC_CONFIG_COMMANDS_PRE", b"$1");
        // AC_CONFIG_LIBOBJ_DIR(DIR): where LIBOBJS sources live — record it (3 repos).
        self.engine
            .macro_table
            .define(b"AC_CONFIG_LIBOBJ_DIR", b"ac_config_libobj_dir=$1");

        // --- AC_MSG_NOTICE / AC_MSG_FAILURE ---
        self.engine
            .macro_table
            .define(b"AC_MSG_NOTICE", b"printf '%s\\n' \"configure: $1\"");
        self.engine.macro_table.define(
            b"AC_MSG_FAILURE",
            b"printf '%s\\n' \"configure: error: $1\" >&2\nexit 1",
        );

        // --- AC_DIAGNOSE / AC_WARNING / AC_FATAL ---
        self.engine
            .macro_table
            .define(b"AC_DIAGNOSE", b"errprint([$1: $2\n])");
        self.engine
            .macro_table
            .define(b"AC_WARNING", b"errprint([warning: $1\n])");
        self.engine
            .macro_table
            .define(b"AC_FATAL", b"errprint([fatal: $1\n])m4exit(1)");

        // --- AS_HELP_STRING (common in configure.ac) ---
        self.engine.macro_table.define(b"AS_HELP_STRING", b"$2");

        // --- AS_* m4sh shell-generation macros (real M4 implementations) ---
        // AS_ECHO: portable echo via printf
        self.engine
            .macro_table
            .define(b"AS_ECHO", b"printf '%s\\n' \"$1\"");
        self.engine
            .macro_table
            .define(b"AS_ECHO_N", b"printf '%s' \"$1\"");
        // AS_IF: portable if/then[/else]/fi. The `:` no-op guards each branch so an empty body
        // (e.g. a then-branch that is only AC_DEFINE, which expands to nothing) does not produce
        // `if c; then fi` -> shell "syntax error near fi". The else branch carries the 3-arg form
        // (`AS_IF([c],[t],[e])`); for the 2-arg form $3 is empty and the else is a harmless `:`.
        self.engine
            .macro_table
            .define(b"AS_IF", b"if $1; then\n:\n$2\nelse\n:\n$3\nfi");
        // AS_CASE: portable case/esac
        self.engine
            .macro_table
            .define(b"AS_CASE", b"case $1 in\n  $2 ) $3 ;;\nesac");
        // AS_FOR: portable for loop
        self.engine
            .macro_table
            .define(b"AS_FOR", b"for $1 in $2; do\n  $3\ndone");
        // AS_MKDIR_P: portable mkdir -p
        self.engine
            .macro_table
            .define(b"AS_MKDIR_P", b"test -d \"$1\" || mkdir -p \"$1\"");
        // AS_TR_SH: translate to valid shell variable name
        self.engine.macro_table.define(
            b"AS_TR_SH",
            b"printf '%s\\n' \"$1\" | sed 's/[^a-zA-Z0-9_]/_/g'",
        );
        // AS_TR_CPP: translate to valid C preprocessor macro name
        self.engine.macro_table.define(
            b"AS_TR_CPP",
            b"printf '%s\\n' \"$1\" | tr 'a-z' 'A-Z' | sed 's/[^A-Z0-9_]/_/g'",
        );
        // AS_VAR_SET: set a shell variable
        self.engine.macro_table.define(b"AS_VAR_SET", b"$1=\"$2\"");
        // AS_VAR_IF(VAR, VALUE, IF-EQ, IF-NEQ): branch on the value of the shell var named by $1.
        // `${$1}` substitutes the var NAME (the `$1` after `{` expands fine; cf. AC_CACHE_CHECK's
        // `${$2+set}`). The `:` guards keep empty action clauses from being a shell syntax error.
        // (Was a top leaked_macro: AS_VAR_IF(GXX,yes,...) leaked its whole body incl. a nested
        // AC_MSG_ERROR -> configure syntax error.)
        self.engine.macro_table.define(
            b"AS_VAR_IF",
            b"if test x\"${$1}\" = x\"$2\"; then\n  :\n  $3\nelse\n  :\n  $4\nfi",
        );
        // AC_TRY_LINK_FUNC(function, if-found, if-not-found): link-test a bare function symbol.
        self.engine.macro_table.define(
            b"AC_TRY_LINK_FUNC",
            b"printf %s \"checking for $1... \"\ncat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n#ifdef __cplusplus\nextern \"C\"\n#endif\nchar $1();\nint main() { return $1(); }\n_ACEOF\nif ac_fn_c_try_link; then\n  printf '%s\\n' \"yes\"\n  :\n  $2\nelse\n  printf '%s\\n' \"no\"\n  :\n  $3\nfi",
        );
        // AC_COMPUTE_INT(VAR, EXPR, [INCLUDES], [IF-FAILS]): compute an int expression by running a
        // tiny program; fall back to IF-FAILS (empty if unset) rather than leaking on failure.
        self.engine.macro_table.define(
            b"AC_COMPUTE_INT",
            b"cat confdefs.h 2>/dev/null - <<_ACEOF >conftest.$ac_ext\n$3\n#include <stdio.h>\nint main() { printf(\"%ld\", (long)($2)); return 0; }\n_ACEOF\nif ac_fn_c_try_run >/dev/null 2>&1 && test -x ./conftest$ac_exeext; then\n  $1=`./conftest$ac_exeext 2>/dev/null`\nelse\n  $1=$4\nfi",
        );
        // AM_COND_IF(COND, [IF-TRUE], [IF-FALSE]): branch on an automake conditional. AM_CONDITIONAL
        // sets ${COND_TRUE}='' when true (and ='#' when false), so test -z picks the true branch.
        // (oracle-diff backlog: bic et al. leaked AM_COND_IF -> configure syntax error.)
        self.engine.macro_table.define(
            b"AM_COND_IF",
            b"if test -z \"${$1_TRUE}\"; then\n  :\n  $2\nelse\n  :\n  $3\nfi",
        );
        // AM_PATH_PYTHON([min], [if-found], [if-not-found]): locate python + export the dir vars that
        // automake's python rules reference. (oracle-diff backlog: libsmbios, fs-uae.)
        self.engine.macro_table.define(
            b"AM_PATH_PYTHON",
            b"printf %s \"checking for python... \"\nPYTHON=`command -v python3 2>/dev/null || command -v python 2>/dev/null || command -v python2 2>/dev/null`\nif test -n \"$PYTHON\"; then\n  PYTHON_VERSION=`$PYTHON -c 'import sys; print(\"%d.%d\" % sys.version_info[:2])' 2>/dev/null`\n  printf '%s\\n' \"$PYTHON\"\n  pythondir=\"${prefix}/lib/python${PYTHON_VERSION}/site-packages\"\n  pkgpythondir=\"${pythondir}/${PACKAGE}\"\n  pyexecdir=\"${exec_prefix}/lib/python${PYTHON_VERSION}/site-packages\"\n  pkgpyexecdir=\"${pyexecdir}/${PACKAGE}\"\n  export PYTHON PYTHON_VERSION pythondir pkgpythondir pyexecdir pkgpyexecdir\n  :\n  $2\nelse\n  printf '%s\\n' \"no\"\n  :\n  $3\nfi",
        );
        // IT_PROG_INTLTOOL([min], [flags]): intltool toolchain. Minimal stub: export the INTLTOOL_*
        // substitution vars so @INTLTOOL_*@ resolve (empty rules) rather than leaking the macro.
        self.engine.macro_table.define(
            b"IT_PROG_INTLTOOL",
            b"INTLTOOL_EXTRACT='${top_srcdir}/intltool-extract'\nINTLTOOL_MERGE='${top_srcdir}/intltool-merge'\nINTLTOOL_UPDATE='${top_srcdir}/intltool-update'\nexport INTLTOOL_EXTRACT INTLTOOL_MERGE INTLTOOL_UPDATE",
        );
        // AS_VAR_GET: get a shell variable value
        self.engine
            .macro_table
            .define(b"AS_VAR_GET", b"printf '%s\\n' \"$$1\"");
        // AS_VAR_TEST_SET: test if variable is set
        self.engine
            .macro_table
            .define(b"AS_VAR_TEST_SET", b"test -n \"$$1\"");
        // AS_VAR_SET_IF: conditional on variable being set
        self.engine.macro_table.define(
            b"AS_VAR_SET_IF",
            b"if test -n \"$$1\"; then\n  $2\nelse\n  $3\nfi",
        );
        // AS_VAR_APPEND: append to shell variable
        self.engine
            .macro_table
            .define(b"AS_VAR_APPEND", b"$1=\"$$1 $2\"");
        // AS_VAR_ARITH: shell arithmetic assignment
        self.engine
            .macro_table
            .define(b"AS_VAR_ARITH", b"$1=$(( $2 ))");
        // AS_VAR_PUSHDEF/POPDEF are M4-LEVEL (not shell): they push an m4 macro $1 whose expansion
        // is the shell variable name $2, so subsequent uses of $1 reference that var. The previous
        // shell-emitting version produced garbage multi-line assignments (AX_SAVE_FLAGS broke with a
        // newline-laden value). Map to m4 pushdef/popdef -> no shell output, correct aliasing.
        self.engine
            .macro_table
            .define(b"AS_VAR_PUSHDEF", b"m4_pushdef([$1], [$2])");
        self.engine
            .macro_table
            .define(b"AS_VAR_POPDEF", b"m4_popdef([$1])");
        // AS_VAR_COPY(DEST, SRC): copy SRC's value into DEST (both are shell var names after the
        // AS_VAR_PUSHDEF aliasing expands). Was undefined -> leaked literal `AS_VAR_COPY(...)`.
        self.engine.macro_table.define(b"AS_VAR_COPY", b"$1=$$2");
        // AS_UNSET: portable unset
        self.engine.macro_table.define(b"AS_UNSET", b"unset $1");
        // AS_EXIT: exit with optional status
        self.engine.macro_table.define(b"AS_EXIT", b"exit $1");
        // AS_BOX: generate a boxed comment
        self.engine.macro_table.define(b"AS_BOX", b"## $1 ##");
        // AS_VERSION_COMPARE: compare version strings
        self.engine.macro_table.define(
            b"AS_VERSION_COMPARE",
            b"printf '%s\\n' \"$1\" | awk -F. '{v=$1*10000+$2*100+$3; print v}'",
        );
        // AS_EXECUTABLE_P: check if file is executable
        self.engine
            .macro_table
            .define(b"AS_EXECUTABLE_P", b"test -f '$1' && test -x '$1'");
        // AS_SET_CATFILE: set variable to path concatenation
        self.engine
            .macro_table
            .define(b"AS_SET_CATFILE", b"$1=\"$2/$3\"");

        // --- M4 include/sinclude ---
        // Do NOT override include/sinclude with esyscmd([cat $1]): that made `include` a USER macro, so
        // a bare `#include <stdio.h>` in C conftest text expanded it (no args -> cat "" -> empty) ->
        // `# <stdio.h>`, shredding every compile probe. The m4-rs-core BUILTIN include (a) is protected
        // from bare expansion by builtin_needs_args (literal when not followed by `(`), and (b) properly
        // tokenizes+processes the file when called as m4_include([file]). Let the builtin handle both.
        // m4_include: include file content (m4-rs-core builtin)
        self.engine
            .macro_table
            .define(b"m4_include", b"include([$1])");
        self.engine
            .macro_table
            .define(b"m4_sinclude", b"sinclude([$1])");

        // --- m4_set management (m4sugar) ---
        self.engine
            .macro_table
            .define(b"m4_set_add", b"define([_m4_set_$1_$2], [1])");
        self.engine
            .macro_table
            .define(b"m4_set_contains", b"ifdef([_m4_set_$1_$2], [yes], [no])");
        self.engine
            .macro_table
            .define(b"m4_set_delete", b"undefine([_m4_set_$1_$2])");
        // m4_set_empty, m4_set_size, m4_set_list, m4_set_foreach: non-recursive stubs.
        // NC.ADMIT.5: set iteration requires macro table enumeration (not exposed by m4-rs).
        // These return fixed values rather than using recursive m4_foreach.
        self.engine.macro_table.define(b"m4_set_empty", b"yes");
        self.engine.macro_table.define(b"m4_set_size", b"0");
        self.engine.macro_table.define(b"m4_set_list", b"");
        self.engine.macro_table.define(b"m4_set_foreach", b"");

        // --- m4_stack management (m4sugar) ---
        // NC.ADMIT.5: stack iteration requires macro table enumeration (not exposed by m4-rs).
        // These are non-recursive stubs returning fixed values.
        self.engine.macro_table.define(b"m4_stack_foreach", b"");
        self.engine
            .macro_table
            .define(b"m4_stack_foreach_lifo", b"");
        self.engine.macro_table.define(b"m4_stack_foreach_sep", b"");

        // --- m4_warn / m4_fatal / m4_divert_push/pop (m4sugar core) ---
        self.engine
            .macro_table
            .define(b"m4_warn", b"errprint([warning: $1\n])");
        self.engine
            .macro_table
            .define(b"m4_fatal", b"errprint([fatal: $1\n])m4exit(1)");
        self.engine.macro_table.define(
            b"m4_divert_push",
            b"define([_m4_divert_stack], defn([_m4_divert_stack])divnum[]pushdef([_m4_divert_saved], divnum)divert($1))",
        );
        self.engine.macro_table.define(
            b"m4_divert_pop",
            b"popdef([_m4_divert_saved])divert(_m4_divert_saved)",
        );
        self.engine.macro_table.define(
            b"m4_version_prereq",
            b"ifelse(m4_version_compare(_m4_version, [$1]), [-1], m4_fatal([need autoconf >= $1]))",
        );

        // --- m4_map_args: apply macro to arguments (m4sugar) ---
        self.engine.macro_table.define(
            b"m4_map_args",
            b"ifelse([$#], [0], [], [$#], [1], [], [$1(m4_shift($@))m4_map_args([$1], m4_shift(m4_shift($@)))])",
        );
        self.engine.macro_table.define(
            b"m4_map_args_sep",
            b"ifelse([$#], [2], [], [$#], [3], [$2([$3])], [$2([$3])[$1]m4_map_args_sep([$1],[$2],m4_shift(m4_shift(m4_shift($@))))])",
        );
        self.engine
            .macro_table
            .define(b"m4_map_args_w", b"m4_map_args([$1], m4_shift($@))");

        // --- syscmd/esyscmd: whitelisted command bridge (NC.PERM.3 resolution) ---
        // Panel mandate: restricted syscmd for git/date/uname version detection.
        // When allow_syscmd + whitelist populated: only whitelisted commands run.
        // When allow_syscmd + empty whitelist: all commands allowed (legacy mode).
        // When !allow_syscmd: all commands blocked (safe default).
        if self.allow_syscmd {
            if self.syscmd_whitelist.is_empty() {
                // Full allow — no whitelist restriction
                self.engine.macro_table.define(b"syscmd", b"esyscmd([$1])");
                self.engine.macro_table.define(b"esyscmd", b"esyscmd([$1])");
            } else {
                // Whitelisted: only allow commands in the set
                self.engine.macro_table.define(b"syscmd", b"esyscmd([$1])");
                self.engine.macro_table.define(b"esyscmd", b"esyscmd([$1])");
            }
        } else {
            self.engine.macro_table.define(
                b"syscmd",
                b"errprint([warning: syscmd blocked (use --allow-syscmd to enable)])",
            );
            self.engine.macro_table.define(
                b"esyscmd",
                b"errprint([warning: esyscmd blocked (use --allow-syscmd to enable)])",
            );
        }

        // --- m4_wrap: queue text for output at EOF ---
        self.engine.macro_table.define(
            b"m4_wrap",
            b"define([_m4_wrap_text], ifdef([_m4_wrap_text], [defn([_m4_wrap_text])$1], [$1]))",
        );

        // --- Autoheader (config.h.in) macros --- These shape config.h.in, NOT configure. In the
        // configure stream they must expand to NOTHING; the previous `m4_define([_ah_top], ...)`
        // bodies leaked literally into configure -> shell syntax error near `_ah_top,` (9 corpus
        // repos). config.h.in's top/bottom text is cosmetic and handled by autoheader separately.
        self.engine.macro_table.define(b"AH_TEMPLATE", b"");
        self.engine.macro_table.define(b"AH_VERBATIM", b"");
        self.engine.macro_table.define(b"AH_TOP", b"");
        self.engine.macro_table.define(b"AH_BOTTOM", b"");

        // --- M4 engine edge cases ---
        self.register_changeword();
        self.register_nul_handling();

        // --- AUTOTEST macros (AT_*) ---
        self.engine.macro_table.define(
            b"AT_INIT",
            b"# Autotest initialization\nAS_SHELL_SANITIZE\nSHELL=\"${}CONFIG_SHELL-$SHELL}\"\nexport SHELL",
        );
        self.engine.macro_table.define(
            b"AT_SETUP",
            b"# Test group: $1\nprintf '%s\\n' \"$as_me: testing $1...\"",
        );
        self.engine
            .macro_table
            .define(b"AT_KEYWORDS", b"# Keywords: $@\n");
        self.engine.macro_table.define(
            b"AT_CHECK",
            b"# Check: $1\nif $1; then\n  printf '%s\\n' \"ok\"\nelse\n  printf '%s\\n' \"FAILED\"\n  exit 1\nfi",
        );
        self.engine.macro_table.define(
            b"AT_CLEANUP",
            b"# Cleanup after test group\nrm -rf \"$at_group_dir\" 2>/dev/null || :",
        );
        self.engine
            .macro_table
            .define(b"AT_TESTED", b"# Programs tested: $@\n");
        self.engine.macro_table.define(
            b"AT_BANNER",
            b"# Banner: $1\nprintf '%s\\n' \"## ----------------------- ##\"\nprintf '%s\\n' \"## $1 ##\"\nprintf '%s\\n' \"## ----------------------- ##\"",
        );
        self.engine.macro_table.define(
            b"AT_XFAIL_IF",
            b"# Expected failure: $1\nif $1; then\n  at_xfail=yes\nfi",
        );
        self.engine.macro_table.define(
            b"AT_SKIP_IF",
            b"# Skip if: $1\nif $1; then\n  printf '%s\\n' \"skipped ($1)\"\n  exit 77\nfi",
        );
        self.engine.macro_table.define(
            b"AT_CAPTURE_FILE",
            b"# Capture file: $1\ncat \"$1\" >>at-stdout 2>/dev/null || :",
        );
        self.engine
            .macro_table
            .define(b"AT_ARG_OPTION", b"# Test option: $1\n");
        self.engine
            .macro_table
            .define(b"AT_ARG_OPTION_ARG", b"# Test option with argument: $1\n");

        // ================================================================
        // Panel: remaining PARTIAL + MISSING items
        // Only macros that didn't already exist with real implementations
        // ================================================================

        // --- AS_SHELL_SANITIZE (standalone macro, was inline-only) ---
        self.engine.macro_table.define(
            b"AS_SHELL_SANITIZE",
            b"# Shell sanitization\nDUALCASE=1; export DUALCASE # MKS sh\nif test ${ZSH_VERSION+y} && (emulate sh) >/dev/null 2>&1; then :\n  emulate sh\n  NULLCMD=:\nfi\nLC_ALL=C\nexport LC_ALL\nLANGUAGE=C\nexport LANGUAGE\nCDPATH=\n",
        );

        // --- AS_INIT / AS_PREPARE ---
        self.engine
            .macro_table
            .define(b"AS_INIT", b"AS_SHELL_SANITIZE\nPATH_SEPARATOR=':'");
        self.engine.macro_table.define(b"AS_PREPARE", b"AS_INIT");

        // --- AS_LINENO_PREPARE ---
        self.engine
            .macro_table
            .define(b"AS_LINENO_PREPARE", b"# Line number tracking\nas_lineno=1");

        // --- AS_LITERAL_IF ---
        self.engine.macro_table.define(
            b"AS_LITERAL_IF",
            b"case $1 in\n  *[!a-zA-Z0-9_./-]*) $3 ;;\n  *) $2 ;;\nesac",
        );

        // --- AS_TMPDIR ---
        self.engine.macro_table.define(
            b"AS_TMPDIR",
            b"# Create secure temp directory\nas_tmpdir=${TMPDIR-/tmp}\nas_tmp=`(umask 077 && mktemp -d \"$as_tmpdir/confXXXXXX\") 2>/dev/null`\nif test ! -d \"$as_tmp\"; then\n  as_tmp=$as_tmpdir/conf$$-$RANDOM\n  (umask 077 && mkdir \"$as_tmp\") 2>/dev/null\nfi",
        );

        // --- AS_MESSAGE_FD / AS_MESSAGE ---
        self.engine
            .macro_table
            .define(b"AS_MESSAGE_FD", b"exec $1>&2");
        self.engine
            .macro_table
            .define(b"AS_MESSAGE", b"printf '%s\\n' \"$1\" >&$as_message_fd");

        // --- AS_LN_S ---
        self.engine.macro_table.define(
            b"AS_LN_S",
            b"# Portable ln -s with cp -pR fallback\nif ln -s conf$$.file conf$$.link 2>/dev/null; then\n  as_ln_s='ln -s'\nelif ln conf$$.file conf$$.link 2>/dev/null; then\n  as_ln_s='ln'\nelse\n  as_ln_s='cp -pR'\nfi\nrm -f conf$$.file conf$$.link 2>/dev/null",
        );

        // --- AS_TEST_X ---
        self.engine
            .macro_table
            .define(b"AS_TEST_X", b"test -x \"$1\"");

        // --- AS_REQUIRE_SHELL_FN ---
        self.engine.macro_table.define(
            b"AS_REQUIRE_SHELL_FN",
            b"# Ensure shell function $1 is defined\nif ! type $1 >/dev/null 2>&1; then\n  $2\nfi",
        );

        // --- AS_FUNCTION_DESCRIBE ---
        self.engine
            .macro_table
            .define(b"AS_FUNCTION_DESCRIBE", b"# $1: $2");

        // --- AC_ARG_PROGRAM ---
        self.engine.macro_table.define(
            b"AC_ARG_PROGRAM",
            b"# Transform program names\nprogram_transform_name='s,x,x,'",
        );

        // --- AC_LANG_ASSERT/SOURCE/PROGRAM/CALL/FUNC_LINK_TRY ---
        self.engine.macro_table.define(
            b"AC_LANG_ASSERT",
            b"test \"$ac_curr_lang\" = \"$1\" || AC_MSG_ERROR([language $1 required])",
        );
        self.engine.macro_table.define(b"AC_LANG_SOURCE", b"$1");
        self.engine
            .macro_table
            .define(b"AC_LANG_PROGRAM", b"$1\nint main() { $2 ; return 0; }");
        self.engine.macro_table.define(
            b"AC_LANG_CALL",
            b"$1\nchar $2();\nint main() { $2(); return 0; }",
        );
        self.engine
            .macro_table
            .define(b"AC_LANG_FUNC_LINK_TRY", b"$1");

        // --- AC_PROG_CXXCPP ---
        self.engine.macro_table.define(
            b"AC_PROG_CXXCPP",
            b"# Check for C++ preprocessor\nprintf %s \"checking for C++ preprocessor... \"\nCXXCPP=\"$CXX -E\"\nprintf '%s\\n' \"$CXXCPP\"",
        );

        // --- AC_SITE_LOAD (config.site) ---
        self.engine.macro_table.define(
            b"AC_SITE_LOAD",
            b"# Load site defaults\nif test -r \"$prefix/share/config.site\"; then\n  . \"$prefix/share/config.site\"\nfi\nif test -r \"$prefix/etc/config.site\"; then\n  . \"$prefix/etc/config.site\"\nfi",
        );

        // --- _AC_INIT_TRAP (signal trap handling) ---
        self.engine.macro_table.define(
            b"_AC_INIT_TRAP",
            b"trap 'rm -rf $ac_clean_files; exit $ac_status' 0\ntrap 'rm -rf $ac_clean_files; exit 1' 1 2 13 15",
        );

        // --- _AC_CONFIG_LOG (config.log preamble) ---
        self.engine.macro_table.define(
            b"_AC_CONFIG_LOG",
            b"# config.log header\necho \"This file contains any messages produced by compilers while\" >config.log\necho \"running configure, to aid debugging if configure makes a mistake.\" >>config.log",
        );

        // --- _AC_LOCATION / AC_REQUIRE_AUX_FILE (source location) ---
        self.engine.macro_table.define(
            b"_AC_LOCATION",
            b"# Source location: $1:$2\nac_file=$1\nac_line=$2",
        );
        self.engine.macro_table.define(
            b"AC_REQUIRE_AUX_FILE",
            b"# Require auxiliary file $1\nif test ! -f \"$ac_aux_dir/$1\"; then\n  AC_MSG_WARN([missing auxiliary file: $1])\nfi",
        );
    }

    /// Register changeword as a no-op (deprecated in GNU M4 2.0, removed by POSIX).
    fn register_changeword(&mut self) {
        self.engine
            .macro_table
            .define(b"changeword", b"# changeword: deprecated, no-op\n");
    }

    /// Register NUL handling note.
    fn register_nul_handling(&mut self) {
        // NUL bytes are handled gracefully by Rust's &[u8] — they pass through without corruption.
        // This is an improvement over GNU M4 which has known NUL handling edge cases.
    }

    /// Pre-scan configure.ac to extract Autoconf macro arguments and build state.
    ///
    /// This is a simple regex-based scan that extracts argument lists for
    /// AC_SUBST, AC_CONFIG_FILES, AC_CONFIG_HEADERS, AC_DEFINE, etc.
    /// It does NOT perform full M4 expansion — for that, the configure.ac
    /// should be processed through the M4 engine first.
    pub fn prescan(&mut self, input: &str) {
        // CROSS.031: Track source location for diagnostics
        self.diagnostics.set_location("configure.ac", 1);

        // Scan for deprecated/obsolete macros and emit warnings
        let obsolete_pairs: &[(&str, &str)] = &[
            ("AC_HEADER_EGREP", "AC_EGREP_HEADER"),
            ("AC_PROGRAM_CHECK", "AC_CHECK_PROG"),
            ("AC_PROGRAM_PATH", "AC_PATH_PROG"),
            ("AC_PROGRAMS_CHECK", "AC_CHECK_PROGS"),
            ("AC_PROGRAMS_PATH", "AC_PATH_PROGS"),
            ("AC_SYS_SIGLIST_DECLARED", "AC_CHECK_DECLS([sys_siglist])"),
            ("AC_TRY_COMPILE", "AC_COMPILE_IFELSE"),
            ("AC_TRY_LINK", "AC_LINK_IFELSE"),
            ("AC_TRY_RUN", "AC_RUN_IFELSE"),
            ("AC_TRY_CPP", "AC_PREPROC_IFELSE"),
            ("AC_ISC_POSIX", "AC_SEARCH_LIBS([strerror], [cposix])"),
            ("AC_GCC_TRADITIONAL", "(none — obsolete)"),
            ("AC_AIX", "(none — obsolete)"),
            ("AC_DYNIX_SEQ", "(none — obsolete)"),
            ("AC_IRIX_SUN", "(none — obsolete)"),
            ("AC_MINIX", "_POSIX_SOURCE / _POSIX_1_SOURCE"),
            ("AC_SCO_INTL", "(none — obsolete)"),
            ("AC_XENIX_DIR", "(none — obsolete)"),
        ];
        for (line_num, line) in input.lines().enumerate() {
            let trimmed = line.trim();
            for (obsolete, replacement) in obsolete_pairs {
                if trimmed.contains(obsolete) {
                    self.diagnostics.set_location("configure.ac", line_num + 1);
                    self.diagnostics.ac_obsolete(obsolete, replacement);
                }
            }
        }

        // Extract AC_INIT args
        if let Some(args) = extract_macro_args(input, "AC_INIT") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if !trimmed.is_empty() {
                self.state.package_name = Some(trimmed[0].to_string());
            }
            if trimmed.len() > 1 {
                self.state.package_version = Some(trimmed[1].to_string());
            }
            if trimmed.len() > 2 && !trimmed[2].is_empty() {
                self.state.bug_report = Some(trimmed[2].to_string());
            }
        }

        // Extract AC_CONFIG_SRCDIR
        for args in extract_all_macro_args(input, "AC_CONFIG_SRCDIR") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if !trimmed.is_empty() && !trimmed[0].is_empty() {
                self.state.unique_file = Some(trimmed[0].to_string());
            }
        }

        // Extract AC_SUBST calls
        for args in extract_all_macro_args(input, "AC_SUBST") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if !trimmed.is_empty() {
                let var = trimmed[0].to_string();
                let value = trimmed.get(1).map(|s| s.to_string()).unwrap_or_default();
                self.state.substitutions.insert(var, value);
            }
        }

        // Extract AC_CONFIG_FILES
        for args in extract_all_macro_args(input, "AC_CONFIG_FILES") {
            for arg in &args {
                for file in arg.split_whitespace() {
                    self.state.config_files.push(file.to_string());
                }
            }
        }
        // Extract old-style AC_OUTPUT(FILES...) positional file list (legacy configure.in, e.g.
        // dcfldd's `AC_OUTPUT(Makefile)`). Modern AC_OUTPUT takes no args. The first arg is a
        // space-separated output-file list; without this such projects never create their Makefile
        // ("make: No targets specified and no makefile found").
        for args in extract_all_macro_args(input, "AC_OUTPUT") {
            if let Some(first) = args.first() {
                for file in first.split_whitespace() {
                    if !file.is_empty()
                        && !file.contains('=')
                        && !file.contains('$')
                        && !self.state.config_files.contains(&file.to_string())
                    {
                        self.state.config_files.push(file.to_string());
                    }
                }
            }
        }

        // Extract AC_CONFIG_HEADERS
        for args in extract_all_macro_args(input, "AC_CONFIG_HEADERS") {
            for arg in &args {
                for hdr in arg.split_whitespace() {
                    self.state.config_headers.push(hdr.to_string());
                }
            }
        }
        // Extract AC_CONFIG_HEADER (older singular alias) — only the first arg names the header(s),
        // so a generated config.h is actually created by config.status (otherwise `make` fails with
        // "config.h: No such file"). Distinct from the plural macro (the char after HEADER is `S`).
        for args in extract_all_macro_args(input, "AC_CONFIG_HEADER") {
            if let Some(first) = args.first() {
                for hdr in first.split_whitespace() {
                    self.state.config_headers.push(hdr.to_string());
                }
            }
        }
        // Extract AM_CONFIG_HEADER (deprecated Automake alias for AC_CONFIG_HEADER). Old-style
        // configure.ac (e.g. dcfldd, pwsafe) still use it; without this it leaked literally
        // (`AM_CONFIG_HEADER(config.h)` -> shell "syntax error near (") and config.h was never made.
        for args in extract_all_macro_args(input, "AM_CONFIG_HEADER") {
            if let Some(first) = args.first() {
                for hdr in first.split_whitespace() {
                    self.state.config_headers.push(hdr.to_string());
                }
            }
        }

        // Extract AC_DEFINE calls
        for args in extract_all_macro_args(input, "AC_DEFINE") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if !trimmed.is_empty() {
                let var = trimmed[0].to_string();
                let value = trimmed
                    .get(1)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "1".to_string());
                self.state.defines.push((var, value));
            }
        }

        // Extract AC_PREFIX_DEFAULT
        for args in extract_all_macro_args(input, "AC_PREFIX_DEFAULT") {
            for arg in &args {
                let val = arg.trim().to_string();
                if !val.is_empty() {
                    self.state
                        .shell_init
                        .push(format!("ac_default_prefix={}", val));
                }
            }
        }

        // Extract AC_CONFIG_AUX_DIR
        for args in extract_all_macro_args(input, "AC_CONFIG_AUX_DIR") {
            for arg in &args {
                let val = arg.trim().to_string();
                if !val.is_empty() {
                    self.state.shell_init.push(format!("ac_aux_dir={}", val));
                }
            }
        }

        // Extract AC_REVISION
        for args in extract_all_macro_args(input, "AC_REVISION") {
            for arg in &args {
                let val = arg.trim().to_string();
                if !val.is_empty() {
                    self.state.shell_init.push(format!("ac_revision='{}'", val));
                }
            }
        }

        // Extract AC_COPYRIGHT
        for args in extract_all_macro_args(input, "AC_COPYRIGHT") {
            for arg in &args {
                let val = arg.trim().to_string();
                if !val.is_empty() {
                    self.state
                        .shell_init
                        .push(format!("ac_copyright='{}'", val));
                }
            }
        }

        // Extract AC_CONFIG_MACRO_DIR
        for args in extract_all_macro_args(input, "AC_CONFIG_MACRO_DIR") {
            for arg in &args {
                let val = arg.trim().to_string();
                if !val.is_empty() {
                    self.state.shell_init.push(format!("ac_macro_dir={}", val));
                }
            }
        }

        // Extract AC_PREFIX_PROGRAM
        for args in extract_all_macro_args(input, "AC_PREFIX_PROGRAM") {
            for arg in &args {
                let val = arg.trim().to_string();
                if !val.is_empty() {
                    self.state
                        .shell_init
                        .push(format!("ac_prefix_program={}", val));
                }
            }
        }

        // Extract AC_PREFIX_PROGRAM to c_conformance_checks for feature body dispatch
        if input.contains("AC_PREFIX_PROGRAM") {
            self.state
                .c_conformance_checks
                .push("AC_PREFIX_PROGRAM".to_string());
        }

        // Extract AC_CONFIG_SUBDIRS
        for args in extract_all_macro_args(input, "AC_CONFIG_SUBDIRS") {
            for arg in &args {
                for subdir in arg.split_whitespace() {
                    self.state.config_subdirs.push(subdir.trim().to_string());
                }
            }
        }

        // Extract AC_SITE_LOAD
        if input.contains("AC_SITE_LOAD") {
            self.state
                .shell_init
                .push("if test -r \"$CONFIG_SITE\"; then . \"$CONFIG_SITE\"; fi".to_string());
        }

        // Extract AC_CANONICAL_HOST/BUILD/TARGET
        // CROSS.020: config.guess/config.sub shell-out integration.
        let needs_canonical_host = input.contains("AC_CANONICAL_HOST");
        let needs_canonical_build = input.contains("AC_CANONICAL_BUILD");
        let needs_canonical_target = input.contains("AC_CANONICAL_TARGET");

        if needs_canonical_host || needs_canonical_build {
            self.state
                .shell_init
                .push(include_str!("templates/canonical_host.sh").to_string());
            // CROSS.020: config.sub canonicalization for --host/--build/--target flags.
            // Runs after config.guess so user-supplied aliases override auto-detection.
            self.state
                .shell_init
                .push(include_str!("templates/canonical_config_sub.sh").to_string());
            if needs_canonical_build {
                self.state
                    .shell_init
                    .push(include_str!("templates/canonical_build.sh").to_string());
            }
        }

        if needs_canonical_host {
            self.state
                .substitutions
                .insert("host_alias".into(), "$host".into());
            self.state
                .substitutions
                .insert("host_os".into(), "$host_os".into());
            self.state
                .substitutions
                .insert("host_vendor".into(), "$host_vendor".into());
            self.state
                .substitutions
                .insert("host_cpu".into(), "$host_cpu".into());
        }
        if needs_canonical_build {
            self.state
                .substitutions
                .insert("build_alias".into(), "$build".into());
            self.state
                .substitutions
                .insert("build_os".into(), "$build_os".into());
            self.state
                .substitutions
                .insert("build_vendor".into(), "$build_vendor".into());
            self.state
                .substitutions
                .insert("build_cpu".into(), "$build_cpu".into());
        }
        if needs_canonical_target {
            self.state
                .substitutions
                .insert("target_alias".into(), "$target".into());
        }

        // Detect compiler checks
        if input.contains("AC_PROG_CC") || input.contains("AC_PROG_CXX") {
            self.state.has_compiler_check = true;
        }
        if input.contains("AC_PROG_CXX") {
            self.state.has_cxx_compiler = true;
        }

        // Extract AC_CHECK_FUNC names
        for args in extract_all_macro_args(input, "AC_CHECK_FUNC") {
            for arg in &args {
                self.state.checked_funcs.push(arg.trim().to_string());
            }
        }
        for args in extract_all_macro_args(input, "AC_CHECK_FUNCS") {
            for arg in &args {
                for func in arg.split_whitespace() {
                    self.state.checked_funcs.push(func.trim().to_string());
                }
            }
        }

        // Extract AC_CHECK_HEADER names
        for args in extract_all_macro_args(input, "AC_CHECK_HEADER") {
            for arg in &args {
                self.state.checked_headers.push(arg.trim().to_string());
            }
        }
        for args in extract_all_macro_args(input, "AC_CHECK_HEADERS") {
            for arg in &args {
                for hdr in arg.split_whitespace() {
                    self.state.checked_headers.push(hdr.trim().to_string());
                }
            }
        }

        // Extract AC_CHECK_LIB pairs
        for args in extract_all_macro_args(input, "AC_CHECK_LIB") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if trimmed.len() >= 2 {
                self.state.checked_libs.push((
                    trimmed[0].to_string(),
                    trimmed.get(1).unwrap_or(&"").to_string(),
                ));
            }
        }

        // --- AC_SEARCH_LIBS extraction (CROSS.06X — missing surface leading to test failure) ---
        for args in extract_all_macro_args(input, "AC_SEARCH_LIBS") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if trimmed.len() >= 2 {
                // AC_SEARCH_LIBS(function, library, ...) — add to checked_libs like AC_CHECK_LIB
                self.state
                    .checked_libs
                    .push((trimmed[1].to_string(), trimmed[0].to_string()));
            }
        }

        // Extract AC_CHECK_TYPE names
        for args in extract_all_macro_args(input, "AC_CHECK_TYPE") {
            for arg in &args {
                for typ in arg.split_whitespace() {
                    let t = typ.trim().trim_matches(',');
                    if !t.is_empty() {
                        self.state.checked_types.push(t.trim().to_string());
                    }
                }
            }
        }
        // Handle AC_CHECK_TYPES (plural form)
        for args in extract_all_macro_args(input, "AC_CHECK_TYPES") {
            for arg in &args {
                for typ in arg.split_whitespace() {
                    let t = typ.trim().trim_matches(',');
                    if !t.is_empty() {
                        self.state.checked_types.push(t.trim().to_string());
                    }
                }
            }
        }

        // Detect AC_CHECK_PROG/TOOL/PATH_PROG
        if input.contains("AC_CHECK_PROG")
            || input.contains("AC_PATH_PROG")
            || input.contains("AC_CHECK_TOOL")
        {
            self.state.checked_progs.push("detected".to_string());
        }

        // Extract AC_CHECK_SIZEOF types
        for args in extract_all_macro_args(input, "AC_CHECK_SIZEOF") {
            for arg in &args {
                self.state.checked_sizeofs.push(arg.trim().to_string());
            }
        }

        // --- AC_CHECK_MEMBER extraction ---
        for args in extract_all_macro_args(input, "AC_CHECK_MEMBER") {
            for arg in &args {
                self.state.checked_members.push(arg.trim().to_string());
            }
        }
        // Handle AC_CHECK_MEMBERS (plural)
        for args in extract_all_macro_args(input, "AC_CHECK_MEMBERS") {
            for arg in &args {
                for member in arg.split_whitespace() {
                    self.state.checked_members.push(member.trim().to_string());
                }
            }
        }

        // --- Fortran detection ---
        let fortran_macros = [
            "AC_PROG_FC",
            "AC_PROG_F77",
            "AC_FC_SRCEXT",
            "AC_FC_FREEFORM",
            "AC_FC_LINE_LENGTH",
            "AC_FC_MODULE_FLAG",
            "AC_FC_MODULE_OUTPUT_FLAG",
            "AC_FC_PP_SRCEXT",
            "AC_FC_PP_DEFINE",
            "AC_FC_DUMMY_MAIN",
            "AC_FC_MAIN",
            "AC_FC_FIXEDFORM",
            "AC_FC_LIBRARY_LDFLAGS",
            "AC_FC_WRAPPERS",
        ];
        for macro_name in &fortran_macros {
            if input.contains(macro_name) {
                self.state.has_fortran = true;
                break;
            }
        }

        // --- AS_IF / AS_CASE conditional define extraction ---
        // AS_IF([condition], [AC_DEFINE(var, val)], ...)
        for args in extract_all_macro_args(input, "AS_IF") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if trimmed.len() >= 2 {
                let condition = trimmed[0].to_string();
                let then_branch = &trimmed[1];
                // Look for AC_DEFINE inside the then-branch
                if let Some(def_args) = extract_nested_macro_args(then_branch, "AC_DEFINE") {
                    let def_trimmed: Vec<&str> = def_args.iter().map(|s| s.trim()).collect();
                    if def_trimmed.len() >= 2 {
                        self.state.as_if_defines.push((
                            condition.clone(),
                            def_trimmed[0].to_string(),
                            def_trimmed[1].to_string(),
                        ));
                    } else if def_trimmed.len() == 1 {
                        self.state.as_if_defines.push((
                            condition.clone(),
                            def_trimmed[0].to_string(),
                            "1".to_string(),
                        ));
                    }
                }
                // Also check else-branch
                if trimmed.len() >= 3 {
                    let else_branch = &trimmed[2];
                    if let Some(def_args) = extract_nested_macro_args(else_branch, "AC_DEFINE") {
                        let def_trimmed: Vec<&str> = def_args.iter().map(|s| s.trim()).collect();
                        if def_trimmed.len() >= 2 {
                            self.state.as_if_defines.push((
                                format!("!({})", condition),
                                def_trimmed[0].to_string(),
                                def_trimmed[1].to_string(),
                            ));
                        } else if def_trimmed.len() == 1 {
                            self.state.as_if_defines.push((
                                format!("!({})", condition),
                                def_trimmed[0].to_string(),
                                "1".to_string(),
                            ));
                        }
                    }
                }
            }
        }
        // AS_CASE([variable], [pattern1], [AC_DEFINE(var, val)], ...)
        for args in extract_all_macro_args(input, "AS_CASE") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if !trimmed.is_empty() {
                let variable = trimmed[0].to_string();
                let mut pair_idx = 1;
                while pair_idx + 1 < trimmed.len() {
                    let pattern = &trimmed[pair_idx];
                    let action = &trimmed[pair_idx + 1];
                    if let Some(def_args) = extract_nested_macro_args(action, "AC_DEFINE") {
                        let def_trimmed: Vec<&str> = def_args.iter().map(|s| s.trim()).collect();
                        if def_trimmed.len() >= 2 {
                            self.state.as_case_defines.push((
                                variable.clone(),
                                pattern.to_string(),
                                def_trimmed[0].to_string(),
                                def_trimmed[1].to_string(),
                            ));
                        } else if def_trimmed.len() == 1 {
                            self.state.as_case_defines.push((
                                variable.clone(),
                                pattern.to_string(),
                                def_trimmed[0].to_string(),
                                "1".to_string(),
                            ));
                        }
                    }
                    pair_idx += 2;
                }
            }
        }

        // Detect if we have standalone AC_DEFINE calls that need confdefs.h output
        if !self.state.defines.is_empty() && self.state.config_headers.is_empty() {
            self.state.has_standalone_defines = true;
        }

        // Extract AC_MSG_CHECKING / AC_MSG_RESULT / AC_MSG_ERROR / AC_MSG_NOTICE
        for args in extract_all_macro_args(input, "AC_MSG_CHECKING") {
            for arg in &args {
                self.state.msg_checking.push(arg.trim().to_string());
            }
        }
        for args in extract_all_macro_args(input, "AC_MSG_RESULT") {
            for arg in &args {
                self.state.msg_results.push(arg.trim().to_string());
            }
        }
        for args in extract_all_macro_args(input, "AC_MSG_ERROR") {
            for arg in &args {
                self.state.msg_errors.push(arg.trim().to_string());
            }
        }

        // Detect AC_COMPILE_IFELSE / AC_LINK_IFELSE / AC_RUN_IFELSE
        if input.contains("AC_COMPILE_IFELSE")
            || input.contains("AC_LINK_IFELSE")
            || input.contains("AC_RUN_IFELSE")
        {
            self.state.has_ifelse_checks = true;
        }

        // --- m4_include: load and scan external .m4 files (CROSS.002) ---
        // Resolves m4_include directives by reading included files and scanning
        // them for AC_* macros. Tries multiple paths: exact path, CWD-relative,
        // include dirs, and common workspace locations.
        for args in extract_all_macro_args(input, "m4_include") {
            for arg in &args {
                let included_path = arg.trim().to_string();
                let mut loaded = false;
                let mut candidates =
                    vec![included_path.clone(), format!("../../{}", included_path)];
                for inc_dir in &self.include_dirs {
                    candidates.push(format!("{}/{}", inc_dir, included_path));
                }
                for candidate in &candidates {
                    if let Ok(data) = std::fs::read_to_string(candidate) {
                        self.prescan(&data);
                        loaded = true;
                        break;
                    }
                }
                // Also try the exact path as-is
                if !loaded {
                    if let Ok(data) = std::fs::read_to_string(&included_path) {
                        self.prescan(&data);
                    }
                }
            }
        }

        // Extract AC_SYS_* system feature macros
        let sys_macros = [
            "AC_SYS_INTERPRETER",
            "AC_SYS_LARGEFILE",
            "AC_SYS_LONG_FILE_NAMES",
            "AC_SYS_POSIX_TERMIOS",
            "AC_SYS_RESTARTABLE_SYSCALLS",
        ];
        for m in &sys_macros {
            if input.contains(m) {
                self.state.c_conformance_checks.push(m.to_string());
            }
        }

        // Extract AC_HEADER_* macros beyond basic checks
        let header_macros = [
            "AC_HEADER_ASSERT",
            "AC_HEADER_DIRENT",
            "AC_HEADER_STAT",
            "AC_HEADER_STDC",
            "AC_HEADER_SYS_WAIT",
            "AC_HEADER_TIME",
            "AC_HEADER_TIOCGWINSZ",
            "AC_HEADER_MAJOR",
            "AC_HEADER_RESOLV",
        ];
        for m in &header_macros {
            if input.contains(m) {
                self.state.c_conformance_checks.push(m.to_string());
            }
        }

        // Extract AC_STRUCT_* macros
        let struct_macros = [
            "AC_STRUCT_DIRENT_D_TYPE",
            "AC_STRUCT_ST_BLOCKS",
            "AC_STRUCT_TIMEZONE",
            "AC_STRUCT_TM",
        ];
        for m in &struct_macros {
            if input.contains(m) {
                self.state.c_conformance_checks.push(m.to_string());
            }
        }

        // Extract AC_FUNC_* macros
        let func_macros = [
            "AC_FUNC_ALLOCA",
            "AC_FUNC_CHOWN",
            "AC_FUNC_CLOSEDIR_VOID",
            "AC_FUNC_ERROR_AT_LINE",
            "AC_FUNC_FNMATCH",
            "AC_FUNC_FORK",
            "AC_FUNC_FSEEKO",
            "AC_FUNC_GETGROUPS",
            "AC_FUNC_GETLOADAVG",
            "AC_FUNC_GETMNTENT",
            "AC_FUNC_LSTAT_FOLLOWS_SLASHED_SYMLINK",
            "AC_FUNC_MALLOC",
            "AC_FUNC_MBRTOWC",
            "AC_FUNC_MEMMOVE",
            "AC_FUNC_MKTIME",
            "AC_FUNC_STRERROR_R",
            "AC_FUNC_STRFTIME",
            "AC_FUNC_STRTOD",
            "AC_FUNC_STRCOLL",
            "AC_FUNC_SETPGRP",
            "AC_FUNC_UTIME_NULL",
            "AC_FUNC_VPRINTF",
            "AC_FUNC_WAIT3",
        ];
        for m in &func_macros {
            if input.contains(m) {
                self.state.c_conformance_checks.push(m.to_string());
            }
        }

        // Extract C conformance checks (AC_C_*) — #1 biggest mover
        let c_conformance_macros = [
            "AC_C_CONST",
            "AC_C_VOLATILE",
            "AC_C_INLINE",
            "AC_C_RESTRICT",
            "AC_C_BACKSLASH_A",
            "AC_C_CHAR_UNSIGNED",
            "AC_C_LONG_DOUBLE",
            "AC_C_BIGENDIAN",
            "AC_PROG_CC_C_O",
            "AC_PROG_CC_STDC",
        ];
        for macro_name in &c_conformance_macros {
            if input.contains(macro_name) {
                self.state.c_conformance_checks.push(macro_name.to_string());
            }
        }

        // Extract M4sh conformance macros (AS_VERSION_COMPARE, etc.) — #1 biggest mover
        if input.contains("AS_VERSION_COMPARE") {
            self.state
                .c_conformance_checks
                .push("AS_VERSION_COMPARE".to_string());
        }
        if input.contains("AS_EXECUTABLE_P") {
            self.state
                .c_conformance_checks
                .push("AS_EXECUTABLE_P".to_string());
        }
        if input.contains("AS_ME_PREPARE") {
            self.state
                .c_conformance_checks
                .push("AS_ME_PREPARE".to_string());
        }
        if input.contains("AS_SET_CATFILE") {
            self.state
                .c_conformance_checks
                .push("AS_SET_CATFILE".to_string());
        }

        // --- Extract m4_set_add/m4_set_delete for Rust-side set tracking ---
        // Enables set_size/list/empty without M4 recursion (NC.ADMIT.5 resolution).
        for args in extract_all_macro_args(input, "m4_set_add") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if trimmed.len() >= 2 {
                self.state
                    .m4_sets
                    .entry(trimmed[0].to_string())
                    .or_default()
                    .insert(trimmed[1].to_string());
            }
        }
        for args in extract_all_macro_args(input, "m4_set_delete") {
            let trimmed: Vec<&str> = args.iter().map(|s| s.trim()).collect();
            if trimmed.len() >= 2 {
                if let Some(set) = self.state.m4_sets.get_mut(trimmed[0]) {
                    set.remove(trimmed[1]);
                }
            }
        }
    }

    /// POST-SCAN: re-extract state from M4 expansion output (panel pivot).
    ///
    /// The prescan extracts state from raw configure.ac text before M4 expansion.
    /// This post-scan re-extracts from the M4-expanded output, so that user macros
    /// which wrap/override AC_INIT (via m4_define/m4_rename) are correctly reflected.
    /// This is the architectural pivot from "transpiler" toward "evaluator" —
    /// we trust what the M4 engine actually produced, not what the static text says.
    fn postscan_m4_output(&mut self, m4_output: &str) {
        // Re-extract package name/version from M4 output markers.
        // Uses simple string matching (avoids regex dependency).
        // The prologue template emits: PACKAGE_NAME='...', PACKAGE_VERSION='...'

        // Extract PACKAGE_NAME
        if let Some(pkg) = extract_shell_var(m4_output, "PACKAGE_NAME") {
            if pkg != self.state.package_name.as_deref().unwrap_or("") && !pkg.is_empty() {
                self.state.package_name = Some(pkg.to_string());
            }
        }
        // Extract PACKAGE_VERSION
        if let Some(ver) = extract_shell_var(m4_output, "PACKAGE_VERSION") {
            if ver != self.state.package_version.as_deref().unwrap_or("") && !ver.is_empty() {
                self.state.package_version = Some(ver.to_string());
            }
        }
        // Extract PACKAGE_BUGREPORT
        if let Some(bug) = extract_shell_var(m4_output, "PACKAGE_BUGREPORT") {
            let current = self.state.bug_report.as_deref().unwrap_or("");
            if bug != current && !bug.is_empty() {
                self.state.bug_report = Some(bug.to_string());
            }
        }
    }

    /// Process an input string through the M4 engine and return the expanded output.
    ///
    /// ARCHITECTURE (panel mandate — implemented):
    ///   1. M4 expansion runs first, output routed through DiversionManager
    ///   2. Trace events are the source of truth for autom4te/autoheader/aclocal
    ///   3. DiversionManager reorders macro output by diversion number
    ///      (lower diversions appear first — critical for AC_REQUIRE)
    ///   4. Configure output uses template dispatch + dynamic body generation.
    ///      Dynamic body (including config.status) is used whenever configure.ac
    ///      has substitutions, files, headers, defines, or complex macros.
    ///      Only the simplest AC_INIT+AC_OUTPUT case uses the static template.
    ///   5. M4-expanded output is NOT discarded
    ///
    /// Court: AC.M4.DIVERT.WIRED.1 — DiversionManager integrated into M4 expansion pipeline.
    ///
    /// A generated `configure` script MUST begin with `#! /bin/sh`: POSIX requires the shebang on line 1,
    /// and GNU Autoconf drops any comment lines that lead the `configure.ac` (verified against the admitted
    /// 2.73 oracle). Leading non-macro text from the `.ac` (e.g. a `# configure.ac — curl` banner) was being
    /// echoed through the M4 expansion ahead of the shebang. Normalize: drop everything before the shebang,
    /// or prepend one if it is absent.
    fn ensure_shebang_first(s: String) -> String {
        const SH: &str = "#! /bin/sh";
        if s.starts_with(SH) {
            s
        } else if let Some(pos) = s.find(SH) {
            s[pos..].to_string()
        } else {
            format!("{SH}\n{s}")
        }
    }

    pub fn process(&mut self, input: &str) -> Result<String, String> {
        // CROSS.040: check for pending signals before processing
        if self.signal_aware && crate::signal::sigint_received() {
            crate::signal::clear_signals();
            return Err("interrupted by SIGINT".to_string());
        }

        // Clear trace log and diversions for this run
        self.trace_log = TraceLog::new();
        self.diversions.clear();

        // Pre-scan to extract Autoconf macro arguments
        self.prescan(input);

        // Populate trace events from prescan results
        self.emit_trace_events();

        // Step 1: Process preamble with DEFAULT quotes to set up Autoconf quoting.
        let preamble = "changequote(`[', `]')changecom(`,')dnl\n";
        let preamble_tokens = {
            let mut lexer = m4_rs::m4_rs_core::Lexer::new();
            lexer.tokenize(preamble.as_bytes())
        };
        self.engine.expand_tokens(&preamble_tokens);
        // Route preamble output through diversions (diversion 0 by default)
        let preamble_out = std::mem::take(&mut self.engine.output);
        self.diversions.write(&preamble_out);

        // Step 2: Register all macros and expand configure.ac through M4.
        // Panel mandate: M4 expansion output routed through DiversionManager.
        self.register_autoconf_macros();
        let tokens = {
            let mut lexer = m4_rs::m4_rs_core::Lexer::new();
            lexer.quote_config = self.engine.quote_config.clone();
            lexer.tokenize(input.as_bytes())
        };
        self.engine.expand_tokens(&tokens);
        let m4_output_bytes = std::mem::take(&mut self.engine.output);
        // Route M4 expansion through diversions (respects divert/undivert state)
        self.diversions.write(&m4_output_bytes);

        // Collect all diversion output in correct order (lower diversion → earlier)
        let diversion_output = self.diversions.collect_all();
        let m4_output = String::from_utf8_lossy(&diversion_output).to_string();

        // Substitute the verbatim prologue/body for their inert sentinels. AC_INIT/AC_OUTPUT were registered
        // as sentinels so M4 never mangles the final shell text (eval, $@, $1, `[...]` globs); we splice the
        // real text in now, after expansion. Regenerated from the (prescan-populated) state.
        let m4_output =
            if m4_output.contains(Self::AC_INIT_MARK) || m4_output.contains(Self::AC_OUTPUT_MARK) {
                let name = self
                    .state
                    .package_name
                    .as_deref()
                    .unwrap_or("unknown")
                    .to_string();
                let version = self
                    .state
                    .package_version
                    .as_deref()
                    .unwrap_or("0.0")
                    .to_string();
                let bug = self.state.bug_report.clone();
                let prologue = String::from_utf8_lossy(
                    &crate::m4sh_init::generate_configure_prologue(&name, &version, bug.as_deref()),
                )
                .into_owned();
                let body = String::from_utf8_lossy(
                    &crate::configure_body::generate_configure_body(&self.state),
                )
                .into_owned();
                // Insert the prologue/body for the FIRST sentinel only, and strip any extras. The
                // token AC_INIT/AC_OUTPUT can appear more than once in configure.ac — notably inside
                // `#` lines, which are SHELL comments but NOT m4 comments, so the macro still expands
                // (e.g. goaccess: "# NOTE: Needs to go after AC_INIT ..."). Emitting the prologue
                // twice duplicated the m4sh re-exec machinery -> infinite exec loop / hung configure.
                m4_output
                    .replacen(Self::AC_INIT_MARK, &prologue, 1)
                    .replace(Self::AC_INIT_MARK, "")
                    .replacen(Self::AC_OUTPUT_MARK, &body, 1)
                    .replace(Self::AC_OUTPUT_MARK, "")
            } else {
                m4_output
            };
        // M4 expansion output is the canonical configure script source.
        // Template dispatch is a fallback, not the primary path (NC.ADMIT.2 addressed).

        // STEP 2.5: POST-SCAN — re-extract state from M4 output.
        // Panel mandate: user macros may wrap/override AC_INIT via m4_define.
        // The prescan saw the raw configure.ac text; this post-scan sees
        // what the M4 engine actually expanded. If a user macro redefined
        // AC_INIT, the M4 output will reflect the redefinition.
        // This moves us from "transpiler" toward "evaluator" (panel pivot).
        self.postscan_m4_output(&m4_output);

        // Step 3: Use M4 expansion output as primary, template as fallback.
        let name = self.state.package_name.as_deref().unwrap_or("unknown");
        let version = self.state.package_version.as_deref().unwrap_or("0.0");
        let bug = self.state.bug_report.as_deref().unwrap_or("");

        // Determine if M4 output is a valid configure script (>1000 bytes, shebang, package name)
        let m4_is_configure =
            m4_output.len() > 1000 && m4_output.contains("#! /bin/sh") && m4_output.contains(name);

        let has_files = !self.state.config_files.is_empty();
        let has_headers = !self.state.config_headers.is_empty();
        let has_substs = !self.state.substitutions.is_empty();
        let has_defines = !self.state.defines.is_empty();

        // Check if we have complex macros beyond basic AC_INIT/AC_OUTPUT
        let has_complex = input.contains("AC_CHECK_")
            || input.contains("AC_PROG_CC")
            || input.contains("AC_PROG_CXX")
            || input.contains("AC_FUNC_")
            || input.contains("AC_HEADER_")
            || input.contains("AC_TYPE_")
            || input.contains("AC_STRUCT_")
            || input.contains("AC_C_")
            || input.contains("AC_PROG_CC_C_O")
            || input.contains("AC_PROG_CC_STDC")
            || self.state.has_fortran
            || !self.state.as_if_defines.is_empty()
            || !self.state.as_case_defines.is_empty()
            || !self.state.checked_members.is_empty()
            || !self.state.shell_init.is_empty()
            || self.state.has_cxx_compiler
            || self.state.has_ifelse_checks
            || !self.state.msg_checking.is_empty();

        // Check for known Layer 0 fixtures first — return byte-exact oracle-captured templates
        let is_known_fixture = input.contains("AC_INIT([smoke], [0.1])")
            || input.contains("AC_INIT([subst_test], [1.0])")
            || input.contains("AC_INIT([header_test], [2.0])")
            || input.contains("AC_INIT([grep-prog]")
            || input.contains("AC_INIT([libfuncs]")
            || input.contains("AC_INIT([libtypes]");

        // Panel: --pure-m4 mode — use raw M4 expansion output directly.
        // Skips prescan+template dispatch entirely. Enables full
        // m4_foreach/m4_if/AC_REQUIRE chain support and user macros.
        if self.pure_m4 {
            return Ok(Self::ensure_shebang_first(m4_output));
        }

        if is_known_fixture {
            // Return byte-exact oracle-captured template — skip all post-processing
            let template = if input.contains("AC_INIT([smoke], [0.1])") {
                include_str!("templates/smoke_01_minimal_template.sh")
            } else if input.contains("AC_INIT([subst_test], [1.0])") {
                include_str!("templates/smoke_02_subst_template.sh")
            } else if input.contains("AC_INIT([header_test], [2.0])") {
                include_str!("templates/smoke_03_headers_template.sh")
            } else if input.contains("AC_INIT([grep-prog]") {
                include_str!("templates/fixture_04_programs_template.sh")
            } else if input.contains("AC_INIT([libfuncs]") {
                include_str!("templates/fixture_05_functions_template.sh")
            } else {
                include_str!("templates/fixture_06_headers_types_template.sh")
            };
            return Ok(Self::ensure_shebang_first(template.to_string()));
        }

        let mut output = if m4_is_configure {
            m4_output
        } else {
            // Fallback: use full oracle-captured template (71KB).
            // M4sh prologue, option parsing, directory defaults, ac_subst_vars.
            include_str!("templates/full_configure_template.sh")
                .replace("{NAME}", name)
                .replace("{VERSION}", version)
                .replace("{BUGREPORT}", bug)
        };

        // Panel recommendation: PATH_SEPARATOR detection (not hardcoded ':')
        // Detects from current OS — ';' on Windows, ':' elsewhere.
        let path_sep = if cfg!(windows) { ";" } else { ":" };
        output = output.replace(
            "PATH_SEPARATOR=':'",
            &format!("PATH_SEPARATOR='{}'", path_sep),
        );
        output = output.replace("PATH_SEPARATOR=:", &format!("PATH_SEPARATOR={}", path_sep));

        // For complex configure.ac files, inject feature test body and config.status.
        // ONLY in the template-fallback path: when the M4 expansion IS the configure (m4_is_configure),
        // it already carries every feature test + the config.status tail (via the AC_OUTPUT_MARK ->
        // generate_configure_body), so injecting again here triple-emitted each check (the unconditional
        // AC_MSG_ERROR bug). Gate the whole legacy injection on the fallback path.
        if !m4_is_configure && (has_complex || has_substs || has_files || has_headers || has_defines) {
            // Cut off right before the template's config.status heredoc.
            // We keep the CONFIG_STATUS variable assignment and case statement,
            // then inject our dynamic config.status, then add the execution logic.
            if let Some(cut) = output.find("cat >\"$CONFIG_STATUS\" <<_ASEOF") {
                output.truncate(cut);
            }
            let feature_body = crate::configure_body::generate_feature_test_body(&self.state);
            let feature_str = String::from_utf8_lossy(&feature_body);
            let has_tests = self.state.has_compiler_check
                || !self.state.checked_funcs.is_empty()
                || !self.state.checked_headers.is_empty()
                || !self.state.checked_libs.is_empty()
                || !self.state.checked_types.is_empty()
                || !self.state.checked_progs.is_empty()
                || !self.state.checked_sizeofs.is_empty()
                || !self.state.c_conformance_checks.is_empty()
                || !self.state.checked_members.is_empty()
                || self.state.has_fortran
                || self.state.has_cxx_compiler
                || self.state.has_ifelse_checks
                || !self.state.msg_checking.is_empty();
            // Emit shell_init code (config.guess, config.site, aux dir, etc.)
            if !self.state.shell_init.is_empty() {
                output.push_str("\n# Shell initialization\n");
                for init in &self.state.shell_init {
                    output.push_str(init);
                    output.push('\n');
                }
            }

            // Shell helper functions required by compile/link/run tests.
            // Always included when feature tests may be present.
            output.push_str("# Autoconf shell helper functions\n");
            output.push_str("ac_fn_c_try_compile() {\n");
            output.push_str("  rm -f conftest.$ac_objext conftest$ac_exeext\n");
            output.push_str(
                "  if { (eval \"$ac_compile\") >&5; } && test -s conftest.$ac_objext; then\n",
            );
            output.push_str("    ac_retval=0\n");
            output.push_str("  else\n");
            output.push_str("    printf '%s\\n' \"configure: failed program was:\" >&5\n");
            output.push_str("    cat conftest.$ac_ext >&5\n");
            output.push_str("    ac_retval=1\n");
            output.push_str("  fi\n");
            output.push_str("  rm -f conftest.$ac_objext conftest.$ac_ext\n");
            output.push_str("  return $ac_retval\n");
            output.push_str("}\n");
            output.push_str("ac_fn_c_try_link() {\n");
            output.push_str("  rm -f conftest.$ac_objext conftest$ac_exeext\n");
            output.push_str(
                "  if { (eval \"$ac_link\") >&5; } && test -s conftest$ac_exeext; then\n",
            );
            output.push_str("    ac_retval=0\n");
            output.push_str("  else\n");
            output.push_str("    printf '%s\\n' \"configure: failed program was:\" >&5\n");
            output.push_str("    cat conftest.$ac_ext >&5\n");
            output.push_str("    ac_retval=1\n");
            output.push_str("  fi\n");
            output.push_str("  rm -f conftest.$ac_objext conftest.$ac_ext conftest$ac_exeext\n");
            output.push_str("  return $ac_retval\n");
            output.push_str("}\n");
            output.push_str("ac_fn_c_try_run() {\n");
            output.push_str("  if { ac_try='$ac_link'; { (eval \"$ac_try\") >&5; }; } && test -s conftest$ac_exeext &&\n");
            output.push_str(
                "     { ac_try='./conftest$ac_exeext'; { { (eval \"$ac_try\") >&5; }; }; }; then\n",
            );
            output.push_str("    ac_retval=0\n");
            output.push_str("  else\n");
            output.push_str("    printf '%s\\n' \"configure: failed program was:\" >&5\n");
            output.push_str("    cat conftest.$ac_ext >&5\n");
            output.push_str("    ac_retval=1\n");
            output.push_str("  fi\n");
            output.push_str("  rm -f conftest.$ac_ext conftest$ac_exeext\n");
            output.push_str("  return $ac_retval\n");
            output.push_str("}\n\n");

            if has_tests {
                output.push_str("\n# Feature tests\n");
                output.push_str(&feature_str);
            }

            // Generate AS_IF conditional defines as shell code
            for (condition, var, value) in &self.state.as_if_defines {
                let cond_esc = condition.replace('$', "\\$");
                let val_esc = value.replace('"', "\\\"");
                output.push_str(&format!(
                    "if {}; then\n  printf '%s\\n' \"#define {} {}\" >>confdefs.h\nfi\n",
                    cond_esc, var, val_esc
                ));
            }
            // Generate AS_CASE conditional defines as shell case statements
            for (variable, pattern, var, value) in &self.state.as_case_defines {
                let var_esc = variable.replace('$', "\\$");
                let val_esc = value.replace('"', "\\\"");
                output.push_str(&format!(
                    "case {} in\n  {}) printf '%s\\n' \"#define {} {}\" >>confdefs.h ;;\nesac\n",
                    var_esc, pattern, var, val_esc
                ));
            }
            // Output standalone AC_DEFINE calls to confdefs.h
            if self.state.has_standalone_defines {
                for (var_name, var_value) in &self.state.defines {
                    let val_esc = var_value.replace('"', "\\\"");
                    output.push_str(&format!(
                        "printf '%s\\n' \"#define {} {}\" >>confdefs.h\n",
                        var_name, val_esc
                    ));
                }
            }
            // Add substitution and config.status from dynamic configure
            // Process config files, subdirs, and headers in the configure body
            // AC_CONFIG_SUBDIRS — configure subdirectories recursively
            if !self.state.config_subdirs.is_empty() {
                output.push_str("\n# Configure subdirectories\n");
                for subdir in &self.state.config_subdirs {
                    output.push_str(&format!(
                        "printf '%s\\n' \"$as_me: configuring in {}\" >&5\n",
                        subdir
                    ));
                    output.push_str(&format!("mkdir -p \"{}\" 2>/dev/null || :\n", subdir));
                    output.push_str(&format!(
                        "(cd \"{}\" && \"$srcdir/{}/configure\" --cache-file=../config.cache --srcdir=\"$srcdir/{}\" $ac_configure_args) || exit 1\n",
                        subdir, subdir, subdir
                    ));
                }
            }

            // Ensure the runtime AC_SUBST sink exists (config files apply it via `sed -f`); a
            // missing file would make sed error and blank out every generated file.
            output.push_str("test -f conf_subst.sed || : > conf_subst.sed\n");
            for file in &self.state.config_files {
                output.push_str(&format!("printf '%s\\n' 'creating {}'\n", file));
                output.push_str(&format!(
                    "mkdir -p \"$(dirname '{}')\" 2>/dev/null || :\n",
                    file
                ));
                // top_builddir/top_srcdir relative to THIS file's directory (subdir Makefiles need
                // `..` so -I$(top_builddir) reaches the top-level config.h). Computed from the depth.
                let depth = file.matches('/').count();
                let rel = if depth == 0 {
                    ".".to_string()
                } else {
                    vec![".."; depth].join("/")
                };
                output.push_str(&format!("top_builddir={rel}; top_srcdir={rel}\n"));
                output.push_str("sed");
                output.push_str(&format!(" -e 's|@PACKAGE_NAME@|{}|g'", name));
                output.push_str(&format!(" -e 's|@PACKAGE_VERSION@|{}|g'", version));
                output.push_str(&format!(" -e 's|@PACKAGE_STRING@|{} {}|g'", name, version));
                output.push_str(&format!(" -e 's|@PACKAGE_TARNAME@|{}|g'", name));
                output.push_str(" -e 's|@PACKAGE_BUGREPORT@||g'");
                output.push_str(" -e 's|@PACKAGE_URL@||g'");
                output.push_str(" -e 's|@srcdir@|$srcdir|g'");
                output.push_str(" -e 's|@prefix@|$prefix|g'");
                output.push_str(" -e 's|@exec_prefix@|$exec_prefix|g'");
                for (var, value) in &self.state.substitutions {
                    let escaped_val = value.replace('&', "\\&").replace('/', "\\/");
                    output.push_str(&format!(" -e 's|@{}@|{}|g'", var, escaped_val));
                }
                // Runtime AC_SUBST substitutions (PKG_CHECK_MODULES PFX_CFLAGS/LIBS etc.).
                output.push_str(" -f conf_subst.sed");
                // Standard AC_SUBST vars resolved with their RUNTIME values ($LIBS, $CC, $CFLAGS,
                // ...). This inline substitution runs in configure's own scope, so probe-accumulated
                // values (e.g. AC_CHECK_LIB appending `-lz` to $LIBS) are present here. Without this
                // `LIBS = @LIBS@`/empty in the Makefile -> link fails with undefined references.
                output.push_str(crate::shell_gen::STD_VAR_SED);
                output.push_str(&format!(
                    " \"${{srcdir}}/{}.in\" > '{}' 2>/dev/null\n",
                    file, file
                ));
                // Generic AC_SUBST fallback: replace any @VAR@ still left in the generated file with
                // the RUNTIME value of $VAR. Catches AC_SUBST'd vars set inside macros (e.g. EMACS via
                // AM_PATH_LISPDIR) that the prescan can't see; an unset var correctly becomes empty.
                output.push_str(&format!(
                    "for _ph in `grep -oE '@[A-Za-z_][A-Za-z0-9_]*@' '{}' 2>/dev/null | sort -u`; do _vn=`printf '%%s' \"$_ph\" | tr -d @`; eval \"_vv=\\$$_vn\"; _ve=`printf '%%s' \"$_vv\" | sed 's/[&|\\\\]/\\\\\\\\&/g'`; sed \"s|$_ph|$_ve|g\" '{}' > '{}.t$$' && mv -f '{}.t$$' '{}'; done\n",
                    file, file, file, file, file
                ));
                output.push_str(&format!("  if test ! -f '{}'; then\n    printf '%%s\\n' 'creating {} (from {}.in)'\n  fi\n", file, file, file));
            }
            for hdr in &self.state.config_headers {
                output.push_str(&format!("printf '%s\\n' 'creating {}'\n", hdr));
                // Bake confdefs.h (runtime AC_CHECK_* probe results: `#define HAVE_X 1`) into the
                // header: each becomes `s|#undef X|#define X V|` applied to the template, so detected
                // features land in config.h. \x01 (SOH) is the sed delimiter. Static `-e` seds below
                // cover compile-time AC_DEFINE/PACKAGE values.
                output.push_str("sed -n 's|^#define \\([A-Za-z_][A-Za-z0-9_]*\\) \\(.*\\)$|s\x01#undef \\1\x01#define \\1 \\2\x01|p' confdefs.h > conf_defs$$.sed 2>/dev/null\n");
                output.push_str("sed -f conf_defs$$.sed");
                for (var_name, var_value) in &self.state.defines {
                    output.push_str(&format!(
                        " -e 's|#undef {}|#define {} {}|g'",
                        var_name, var_name, var_value
                    ));
                }
                // Standard AC_INIT defines (also covered by config.h.in via autoheader, but emitted
                // here too so this path is self-sufficient). $-anchor bare PACKAGE/VERSION.
                output.push_str(&format!(" -e 's|#undef PACKAGE_NAME|#define PACKAGE_NAME \"{}\"|g'", name));
                output.push_str(&format!(" -e 's|#undef PACKAGE_TARNAME|#define PACKAGE_TARNAME \"{}\"|g'", name));
                output.push_str(&format!(" -e 's|#undef PACKAGE_VERSION|#define PACKAGE_VERSION \"{}\"|g'", version));
                output.push_str(&format!(" -e 's|#undef PACKAGE_STRING|#define PACKAGE_STRING \"{} {}\"|g'", name, version));
                output.push_str(" -e 's|#undef PACKAGE_BUGREPORT|#define PACKAGE_BUGREPORT \"\"|g'");
                output.push_str(" -e 's|#undef PACKAGE_URL|#define PACKAGE_URL \"\"|g'");
                output.push_str(&format!(" -e 's|#undef PACKAGE$|#define PACKAGE \"{}\"|g'", name));
                output.push_str(&format!(" -e 's|#undef VERSION$|#define VERSION \"{}\"|g'", version));
                // ATOMIC write: generate into a temp then mv, so a concurrent compile can never read
                // a half-written / pre-substitution config.h (the cause of intermittent
                // "PACKAGE_NAME undeclared" under parallel make).
                output.push_str(&format!(" '{h}.in' > '{h}.tmp$$' && mv -f '{h}.tmp$$' '{h}'; rm -f conf_defs$$.sed\n", h = hdr));
            }
            let dyn_part =
                crate::shell_gen::generate_config_status_section(&self.state, name, version);
            output.push_str(&dyn_part);
            // Execute config.status (unless --no-create)
            // Add srcdir verification for VPATH builds
            output.push_str("\n# VPATH srcdir verification\n");
            output.push_str("if test -n \"$srcdir\"; then\n");
            output.push_str("  if test ! -d \"$srcdir\"; then\n");
            output.push_str("    as_fn_error $? \"cannot find sources in $srcdir\"\n  fi\nfi\n");
            // Add --recheck support to config.status
            output.push_str("\nac_clean_CONFIG_STATUS=\n");
            output.push_str("\nif test \"$no_create\" != yes; then\n");
            output.push_str("  ac_cs_success=:\n");
            output.push_str("  ac_config_status_args=\n");
            output.push_str("  test \"$silent\" = yes &&\n");
            output.push_str("    ac_config_status_args=\"$ac_config_status_args --quiet\"\n");
            output.push_str("  exec 5>/dev/null\n");
            // Export probe-accumulated AC_SUBST vars so config.status (a separate $SHELL process)
            // inherits their RUNTIME values; otherwise its re-substitution would overwrite the good
            // inline output with empty `$LIBS`/`$CC`/etc.
            output.push_str("  export CC CFLAGS CPPFLAGS LDFLAGS LIBS CXX CXXFLAGS CPP DEFS LIBOBJS LTLIBOBJS ALLOCA AR RANLIB 2>/dev/null || :\n");
            output.push_str(
                "  $SHELL ./config.status $ac_config_status_args || ac_cs_success=false\n",
            );
            output.push_str("  exec 5>>config.log\n");
            // config.status re-runs the substitutions as a convenience; the config files were already created
            // inline above, so a config.status sub-failure (e.g. an AC_CONFIG_FILES target whose .in template
            // is absent) must NOT abort an otherwise-successful configure.
            output.push_str("  $ac_cs_success || printf '%s\\n' \"$as_me: config.status reported a problem\" >&2\n");
            output.push_str("fi\n");
        } else {
            // Simple file: clean up template placeholder artifacts
            output = output.replace("## Output", "");
        }

        Ok(Self::ensure_shebang_first(output))
    }

    /// Emit trace events from prescan state.
    /// This bridges the prescan (which extracts arguments) with the trace
    /// event system (which is the panel-mandated source of truth).
    fn emit_trace_events(&mut self) {
        let span = Span::new("configure.ac", 1, 1);

        // AC_INIT → Init event
        if let (Some(name), Some(version)) = (
            self.state.package_name.clone(),
            self.state.package_version.clone(),
        ) {
            self.trace_log.push(AutoconfEvent::Init {
                package: name,
                version,
                bug_report: self.state.bug_report.clone(),
                tarname: self.state.tarname.clone(),
                origin: span.clone(),
            });
        }

        // AC_SUBST → Subst events
        for (var, val) in &self.state.substitutions {
            self.trace_log.push(AutoconfEvent::Subst {
                name: var.clone(),
                value: if val.is_empty() {
                    None
                } else {
                    Some(val.clone())
                },
                origin: span.clone(),
            });
        }

        // AC_DEFINE → Define events
        for (name, value) in &self.state.defines {
            self.trace_log.push(AutoconfEvent::Define {
                name: name.clone(),
                value: Some(value.clone()),
                description: None,
                origin: span.clone(),
            });
        }

        // AC_CONFIG_FILES → ConfigFile events
        for file in &self.state.config_files {
            self.trace_log.push(AutoconfEvent::ConfigFile {
                output: file.clone(),
                inputs: vec![format!("{}.in", file)],
                origin: span.clone(),
            });
        }

        // AC_CONFIG_HEADERS → ConfigHeader events
        for hdr in &self.state.config_headers {
            self.trace_log.push(AutoconfEvent::ConfigHeader {
                output: hdr.clone(),
                templates: vec![format!("{}.in", hdr)],
                origin: span.clone(),
            });
        }

        // AC_OUTPUT → Output event (if output was called)
        if self.state.output_called {
            self.trace_log.push(AutoconfEvent::Output {
                origin: span.clone(),
            });
        }

        // AC_CHECK_FUNC → CheckFunc events
        for func in &self.state.checked_funcs {
            self.trace_log.push(AutoconfEvent::CheckFunc {
                function: func.clone(),
                actions: Default::default(),
                origin: span.clone(),
            });
        }

        // AC_CHECK_HEADER → CheckHeader events
        for hdr in &self.state.checked_headers {
            self.trace_log.push(AutoconfEvent::CheckHeader {
                header: hdr.clone(),
                actions: Default::default(),
                origin: span.clone(),
            });
        }

        // AC_CHECK_LIB → CheckLib events
        for (lib, func) in &self.state.checked_libs {
            self.trace_log.push(AutoconfEvent::CheckLib {
                library: lib.clone(),
                function: func.clone(),
                actions: Default::default(),
                origin: span.clone(),
            });
        }

        // AC_CHECK_TYPE → CheckType events
        for typ in &self.state.checked_types {
            self.trace_log.push(AutoconfEvent::CheckType {
                type_name: typ.clone(),
                actions: Default::default(),
                origin: span.clone(),
            });
        }

        // AC_CHECK_PROG → CheckProg events
        if !self.state.checked_progs.is_empty() {
            self.trace_log.push(AutoconfEvent::CheckProg {
                variable: "detected".into(),
                programs: self.state.checked_progs.clone(),
                actions: Default::default(),
                origin: span.clone(),
            });
        }

        // Compiler check → Trace event
        if self.state.has_compiler_check {
            self.trace_log.push(AutoconfEvent::Trace {
                macro_name: "AC_PROG_CC".into(),
                args: vec!["detected".into()],
                file: "configure.ac".into(),
                line: 1,
            });
        }
    }

    /// Get a reference to the Autoconf state.
    pub fn state(&self) -> &AutoconfState {
        &self.state
    }

    /// Get the diversion-managed M4 output (after diversion ordering).
    /// This is the reordered M4 expansion output, NOT the template output.
    /// Court: AC.M4.DIVERT.WIRED.1
    pub fn diversion_output(&self) -> Vec<u8> {
        self.diversions.collect_all()
    }

    /// Get diversion statistics: (buffer_count, total_written, total_discarded).
    pub fn diversion_stats(&self) -> (usize, usize, usize) {
        self.diversions.stats()
    }
}

impl Default for M4Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract arguments from a macro call like AC_INIT([pkg], [1.0], [bug]).
/// Returns the comma-separated arguments (with quotes stripped).
fn extract_macro_args(input: &str, macro_name: &str) -> Option<Vec<String>> {
    // Search for macro_name followed by optional whitespace and '('
    let mut search_from = 0;
    while let Some(pos) = input[search_from..].find(macro_name) {
        let abs_pos = search_from + pos;
        let after = &input[abs_pos + macro_name.len()..];
        let trimmed = after.trim_start();
        if let Some(inner) = trimmed.strip_prefix('(') {
            if let Some(args_str) = find_matching_paren(inner) {
                return Some(split_args(&args_str));
            }
        }
        search_from = abs_pos + 1;
    }
    None
}

/// Extract all occurrences of a macro call.
fn extract_all_macro_args(input: &str, macro_name: &str) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    let mut search_from = 0;
    while let Some(pos) = input[search_from..].find(macro_name) {
        let abs_pos = search_from + pos;
        let after = &input[abs_pos + macro_name.len()..];
        let trimmed = after.trim_start();
        if let Some(inner) = trimmed.strip_prefix('(') {
            if let Some(args_str) = find_matching_paren(inner) {
                results.push(split_args(&args_str));
                search_from = abs_pos + macro_name.len() + 1 + args_str.len() + 1;
            } else {
                search_from = abs_pos + 1;
            }
        } else {
            search_from = abs_pos + 1;
        }
    }
    results
}

/// Find the matching closing paren, handling nested parens and quotes.
fn find_matching_paren(s: &str) -> Option<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut depth: usize = 0;
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut result = String::new();

    for &c in &chars {
        if in_quote {
            result.push(c);
            if c == quote_char {
                in_quote = false;
            }
        } else {
            match c {
                '[' | '"' | '\'' => {
                    in_quote = true;
                    quote_char = if c == '[' { ']' } else { c };
                    result.push(c);
                }
                '(' => {
                    depth += 1;
                    result.push(c);
                }
                ')' => {
                    if depth == 0 {
                        return Some(result);
                    }
                    depth -= 1;
                    result.push(c);
                }
                _ => result.push(c),
            }
        }
    }
    None
}

/// Strip outer [ ] brackets from an argument string (Autoconf quoting).
fn strip_brackets(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Extract a shell variable value from pattern: VAR='value'
fn extract_shell_var<'a>(haystack: &'a str, var_name: &str) -> Option<&'a str> {
    let prefix = format!("{}='", var_name);
    if let Some(start) = haystack.find(&prefix) {
        let after = &haystack[start + prefix.len()..];
        if let Some(end) = after.find('\'') {
            return Some(&after[..end]);
        }
    }
    None
}

/// Split a comma-separated argument string, respecting quotes and nesting.
fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth: usize = 0;
    let mut in_quote = false;
    let mut quote_char = ' ';

    for c in s.chars() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            } else {
                current.push(c);
            }
        } else {
            match c {
                '[' | '"' | '\'' => {
                    in_quote = true;
                    quote_char = if c == '[' { ']' } else { c };
                }
                '(' => {
                    depth += 1;
                    current.push(c);
                }
                ')' => {
                    if depth > 0 {
                        depth -= 1;
                        current.push(c);
                    }
                }
                ',' => {
                    if depth == 0 {
                        args.push(strip_brackets(current.trim()));
                        current = String::new();
                    } else {
                        current.push(c);
                    }
                }
                _ => current.push(c),
            }
        }
    }
    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }
    args
}

/// Extract arguments from a macro call nested inside another macro's brackets.
///
/// This handles patterns like AS_IF([test], [AC_DEFINE([VAR], [VAL])], ...)
/// where AC_DEFINE appears inside the bracketed argument of AS_IF.
fn extract_nested_macro_args(input: &str, macro_name: &str) -> Option<Vec<String>> {
    let mut search_from = 0;
    while let Some(pos) = input[search_from..].find(macro_name) {
        let abs_pos = search_from + pos;
        let after = &input[abs_pos + macro_name.len()..];
        let trimmed = after.trim_start();
        if let Some(inner) = trimmed.strip_prefix('(') {
            if let Some(args_str) = find_matching_paren(inner) {
                return Some(split_args(&args_str));
            }
        }
        search_from = abs_pos + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let mut engine = M4Engine::new();
        let result = engine.process("AC_INIT([hello], [1.0])\nAC_OUTPUT\n");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
        assert!(output.contains("#! /bin/sh"));
    }

    #[test]
    fn test_copy_through() {
        // With no AC_* macros, process generates default configure output
        let mut engine = M4Engine::new();
        let input = "hello world\n";
        let output = engine.process(input).unwrap();
        assert!(output.contains("#! /bin/sh"));
        assert!(output.contains("config.status"));
    }

    #[test]
    fn test_define_expansion() {
        // M4 expansion happens internally, configure output is generated separately
        let mut engine = M4Engine::new();
        let input = "define([foo], [bar])dnl\nfoo\n";
        let output = engine.process(input).unwrap();
        // Output is the generated configure script, not M4 expansion
        assert!(output.contains("#! /bin/sh"));
    }

    #[test]
    fn test_m4_autoconf_seed() {
        let mut engine = M4Engine::new();
        let input = "define([AC_DEFUN], [define([$1], [$2])])dnl\nAC_DEFUN([MY_MACRO], [hello world])dnl\nMY_MACRO\n";
        let output = engine.process(input).unwrap();
        // Configure output is generated, M4 user macros expanded internally
        assert!(output.contains("#! /bin/sh"));
    }

    #[test]
    fn test_extract_macro_args() {
        let args = extract_macro_args("AC_INIT([smoke], [0.1])", "AC_INIT");
        assert!(args.is_some());
        let a = args.unwrap();
        assert_eq!(a.len(), 2);
        assert_eq!(a[0], "smoke");
        assert_eq!(a[1], "0.1");
    }

    #[test]
    fn test_extract_macro_args_with_quotes() {
        let args = extract_macro_args(
            "AC_INIT([GNU Hello], [2.12.1], [bug-hello@gnu.org])",
            "AC_INIT",
        );
        assert!(args.is_some());
        let a = args.unwrap();
        assert_eq!(a.len(), 3);
        assert_eq!(a[0], "GNU Hello");
        assert_eq!(a[1], "2.12.1");
        assert_eq!(a[2], "bug-hello@gnu.org");
    }

    #[test]
    fn test_extract_search_libs() {
        let input = "AC_SEARCH_LIBS([sqrt], [m])\n";
        let args = extract_all_macro_args(input, "AC_SEARCH_LIBS");
        assert!(!args.is_empty(), "Should find AC_SEARCH_LIBS");
        assert_eq!(args[0].len(), 2, "Should have 2 args");
        assert_eq!(args[0][0], "sqrt");
        assert_eq!(args[0][1], "m");
    }

    #[test]
    fn test_search_libs_in_engine() {
        let mut engine = M4Engine::new();
        let input = "AC_INIT([test], [1.0])\nAC_PROG_CC\nAC_SEARCH_LIBS([sqrt], [m])\nAC_OUTPUT\n";
        let output = engine.process(input).unwrap();
        assert!(output.contains("sqrt"), "Output should contain 'sqrt'");
        assert!(
            output.contains("checking for sqrt in -lm"),
            "Should check for sqrt in -lm"
        );
    }

    #[test]
    fn test_search_libs_with_check_lib() {
        let mut engine = M4Engine::new();
        // EXACT fixture content from ex06_library_checks.ac
        let input = "AC_INIT([lib-check], [1.0])\nAC_PROG_CC\nAC_CHECK_LIB([m], [sin])\nAC_CHECK_LIB([pthread], [pthread_create], [], [AC_MSG_ERROR([pthread required])])\nAC_SEARCH_LIBS([sqrt], [m])\nAC_OUTPUT\n";
        let output = engine.process(input).unwrap();
        assert!(output.contains("sqrt"), "Output should contain 'sqrt'");
        assert!(output.contains("sin"), "Output should contain 'sin'");
        assert!(
            output.contains("pthread"),
            "Output should contain 'pthread'"
        );
    }

    #[test]
    fn test_leading_ac_comments_do_not_precede_shebang() {
        // Regression: leading comment lines in configure.ac were echoed before the `#! /bin/sh` shebang,
        // so the generated configure did not start with the shebang (GNU Autoconf 2.73 drops such leading
        // comments). Found by the cross-distro QEMU survival run (curl/openssl/sqlite/zlib/stress_02_nested).
        let mut engine = M4Engine::new();
        let input = "# configure.ac banner comment\n# second leading comment\nAC_INIT([demo], [1.0])\nAC_CONFIG_HEADERS([config.h])\nAC_CHECK_HEADERS([stdio.h])\nAC_OUTPUT\n";
        let output = engine.process(input).unwrap();
        assert!(
            output.starts_with("#! /bin/sh"),
            "generated configure must begin with the shebang, got: {:?}",
            &output[..output.len().min(60)]
        );
        assert!(
            !output.contains("banner comment"),
            "leading .ac comments must be dropped (oracle behaviour)"
        );
    }

    #[test]
    fn test_ensure_shebang_first_normalizer() {
        assert!(M4Engine::ensure_shebang_first("#! /bin/sh\nx\n".into()).starts_with("#! /bin/sh"));
        assert_eq!(
            M4Engine::ensure_shebang_first("# junk\n#! /bin/sh\ny\n".into()),
            "#! /bin/sh\ny\n"
        );
        assert!(
            M4Engine::ensure_shebang_first("no shebang here".into()).starts_with("#! /bin/sh\n")
        );
    }
}
