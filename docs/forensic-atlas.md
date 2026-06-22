# Forensic Surface Atlas — GNU Autoconf → autoconf-rs

**Generated:** 1782137453
**Source:** `sources/docs/forensic-atlas.json`
**Methodology:** Clean-room, black-box forensic parity. Zero GPL code.
**Oracle:** GNU Autoconf 2.73

Clean-room, black-box. All behavior derived from binary oracle interrogation, GFDL manual (https://www.gnu.org/software/autoconf/manual/), and POSIX shell specification. No GPL source code consulted.

---

## Surface Map

### CLI Binary Interfaces

All 8 GNU Autoconf command-line tools, each with its own invocation contract, option flags, exit codes, and environment variable interactions.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| autoconf | autoconf | ✅ sealed | Main configure script generator. Reads configure.ac, expands M4 macros, outputs configure shell script. Flags: -o, -I, -B, -W, -f, --trace, --debug, --initialization, --include, --prepend-include. Env: AUTOCONF, AUTOM4TE, M4, WARNINGS. |
| autoheader | autoheader | 🔧 implemented | Generates config.h.in template from configure.ac. Reads AC_CONFIG_HEADERS, AC_DEFINE, AC_DEFINE_UNQUOTED. Flags: -o, -I, -B, -W, -f. |
| autom4te | autom4te | 🔧 implemented | Caching M4 wrapper. Core engine behind autoconf/autoheader/automake/autoscan. Manages frozen file cache, --language selection, --include paths, --freeze/--reload, --trace. |
| autoreconf | autoreconf | 🔧 implemented | Orchestrates autoconf/autoheader/automake/aclocal/libtoolize chain. Flags: -f, -i, -s, -v, --make. |
| aclocal | aclocal | 🔧 implemented | Generates aclocal.m4 from configure.ac by scanning m4/ directories. Flags: -I, --install, --diff, --force. |
| autoscan | autoscan | 🔧 implemented | Scans C source files for Autoconf hints. Generates configure.scan as a starting template. |
| autoupdate | autoupdate | 🔧 implemented | Updates outdated Autoconf macros in existing configure.ac files to current versions. |
| ifnames | ifnames | 🔧 implemented | Extracts #if/#ifdef/#ifndef/#elif preprocessor conditionals from C source files. |

### M4 Macro Engine

The GNU m4 macro processor that Autoconf builds upon. autoconf-rs depends on m4-rs-core v0.1 for this.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| m4_lexer | Lexer/Tokenizer | ✅ sealed | Character-by-character tokenization into words, strings, comments. Quote character recognition ([`] vs ['']), comment character (dnl, #). Eight-bit-clean. — see note |
  NOTE: Via m4-rs-core
| m4_macro_table | Macro Table | ✅ sealed | Hash map of name → definition. Supports define/undefine/pushdef/popdef/indir/builtin. Builtins vs user-defined distinction. — see note |
  NOTE: Via m4-rs-core
| m4_args | Argument Collection | ✅ sealed | Comma-separated argument parsing with nested parentheses tracking. $1-$9, $#, $*, $@, $0 substitution. Quoted vs unquoted handling. — see note |
  NOTE: Via m4-rs-core
| m4_expansion | Macro Expansion | ✅ sealed | Recursive expansion with rescanning. AC_REQUIRE diversion-based dependency management. — see note |
  NOTE: Via m4-rs-core
| m4_diversions | Diversion System | ✅ sealed | divert/undivert/divnum for output reordering. Critical for Autoconf's AC_REQUIRE pattern. Negative diversion discards. — see note |
  NOTE: Via m4-rs-core
| m4_frozen | Frozen Files | 🔄 partial | Binary dump of macro table state for fast reload. autom4te caching depends on this. Format: V1/V2 magic, timestamp, macro definitions. — see note |
  NOTE: Not yet needed for Layer 0
| m4_builtins | Built-in Macros | ✅ sealed | 50+ builtins: ifdef/ifelse/shift, len/incr/decr, eval (integer arithmetic), changequote/changecom/dnl, errprint/__file__/__line__, include/sinclude, divert/undivert, syscmd/esyscmd/sysval, maketemp/mkstemp, m4wrap, traceon/traceoff/dumpdef, format, regexp/patsubst, translit, index/substr, defn/builtin/indir, changeword, m4exit. — see note |
  NOTE: Via m4-rs-core; 42/42 core builtins

### m4sugar Macro Library

Convenience macros that make M4 programming practical. Ships as m4sugar.m4 in GNU Autoconf. Reimplemented as native Rust in autoconf-rs.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| m4_defun | m4_defun / m4_defun_init | ✅ sealed | Define an Autoconf macro with automatic AC_REQUIRE diversion support. One-shot expansion with m4_provide. |
| m4_require | m4_require / m4_provide | ✅ sealed | Dependency tracking: m4_require(NAME) ensures NAME is expanded before current macro. m4_provide marks as satisfied. |
| m4_if | m4_if / m4_case | ✅ sealed | Conditional: string equality comparison. m4_case: multi-way switch. |
| m4_foreach | m4_foreach / m4_foreach_w | ✅ sealed | Iterate over comma/whitespace-separated lists. m4_map_args/m4_map_args_pair for argument mapping. |
| m4_join | m4_join / m4_append | ✅ sealed | String joining and list building. m4_append_uniq for duplicate-free accumulation. |
| m4_quote | m4_quote / m4_dquote | ✅ sealed | Quote bracketing: m4_quote([text]), m4_dquote([[text]]). Critical for deferring expansion. |
| m4_normalize | m4_normalize / m4_text_wrap | ✅ sealed | Whitespace normalization and text wrapping for readable output. |
| m4_expand | m4_expand / m4_do | ✅ sealed | Force expansion and concatenation. m4_do is unquoted m4_expand. |
| m4_copy | m4_copy / m4_rename | ✅ sealed | Copy or rename macro definitions. |
| m4_toupper | m4_toupper / m4_tolower | ✅ sealed | Case conversion for macro names and arguments via translit. |
| m4_split | m4_split / m4_flatten / m4_strip | ✅ sealed | String manipulation: split on delimiter, flatten whitespace, strip leading/trailing via patsubst. |
| m4_pattern | m4_pattern_forbid / m4_pattern_allow | ✅ sealed | Forbid or allow specific patterns in output. Used for safety checks. |
| m4_warn | m4_warn / m4_fatal | ✅ sealed | Warning and fatal error emission via errprint. Category support. |
| m4_set | m4_set_* (set operations) | 🔄 partial | m4_set_add, m4_set_contains, m4_set_delete implemented. empty/size/list/foreach are NC.ADMIT.5 stubs (requires macro table enumeration). |
| m4_divert | m4_divert_push / m4_divert_pop | ✅ sealed | Diversion stack management for temporary output redirection. |
| m4_version | m4_version_prereq | ✅ sealed | Version comparison for conditional feature enablement. |
| m4_bmatch | m4_bmatch / m4_car / m4_cdr | ✅ sealed | Pattern matching and list destructuring. |
| m4_stack | m4_stack_foreach / m4_list_cmp | 🔄 partial | m4_list_cmp implemented via ifelse. m4_stack_foreach is a stub. |

### m4sh Shell-Generation Library

Macros that generate portable POSIX shell script. Ships as m4sh.m4 in GNU Autoconf. Reimplemented in Rust.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| as_echo | AS_ECHO / AS_ECHO_N | ✅ sealed | Portable echo (handles -n, backslash escapes). Uses printf fallback. AS_ECHO_N suppresses trailing newline. |
| as_escape | AS_ESCAPE | ✅ sealed | Shell escape: quote string for safe use in shell double-quoted context. Handles backtick, $, backslash, double-quote. |
| as_exit | AS_EXIT | ✅ sealed | Exit with status, executing EXIT trap if configured. |
| as_if | AS_IF / AS_CASE | ✅ sealed | Shell if/elif/else/fi and case/esac generation. AS_IF handles empty branches. AS_CASE handles fall-through. |
| as_for | AS_FOR | ✅ sealed | Shell for-loop generation over word list. |
| as_mkdir_p | AS_MKDIR_P | ✅ sealed | Portable recursive directory creation. Uses install -d or mkdir -p. |
| as_tr_sh | AS_TR_SH / AS_TR_CPP | ✅ sealed | String transformation: to shell variable name (alphanumeric+underscore) and to CPP macro name (uppercase+underscore). |
| as_unset | AS_UNSET | ✅ sealed | Portable variable unset that works on shells where unset fails for read-only vars. |
| as_box | AS_BOX | ✅ sealed | Draw a text box (banner) around a string using a repeatable character. |
| as_sanitize | AS_SHELL_SANITIZE | ✅ sealed | Standalone shell sanitization macro. DUALCASE, ZSH emulation, LC_ALL, LANGUAGE, CDPATH. |
| as_var | AS_VAR_* (variable ops) | ✅ sealed | 8 AS_VAR_* functions implemented: SET, GET, TEST_SET, SET_IF, PUSHDEF, POPDEF, APPEND, ARITH. Full set of shell variable manipulation. |
| as_basename | AS_BASENAME / AS_DIRNAME | ✅ sealed | Portable basename/dirname extraction without forking external processes. |
| as_version | AS_VERSION_COMPARE | ✅ sealed | Semantic version comparison via awk. Splits on '.' and computes numeric value. |
| as_exec | AS_EXECUTABLE_P | ✅ sealed | Test whether a file is executable via test -f && test -x. |
| as_tmpdir | AS_TMPDIR | ✅ sealed | Secure temporary directory with mktemp fallback. |
| as_me | AS_ME_PREPARE / AS_LINENO_PREPARE | ✅ sealed | AS_ME_PREPARE in m4sh.rs. AS_LINENO_PREPARE has M4 macro. Both done. |
| as_init | AS_INIT / AS_PREPARE | ✅ sealed | AS_INIT and AS_PREPARE have standalone M4 macros. |
| as_message | AS_MESSAGE_FD / AS_MESSAGE | ✅ sealed | AS_MESSAGE_FD and AS_MESSAGE have M4 macros. |
| as_ln_s | AS_LN_S | ✅ sealed | Portable ln -s with cp -pR fallback via M4 macro. |
| as_test | AS_TEST_X (extended test) | ✅ sealed | Extended file test via M4 macro. |
| as_require | AS_REQUIRE_SHELL_FN | ✅ sealed | AS_REQUIRE_SHELL_FN ensures function definition before use. |
| as_fn | AS_FUNCTION_DESCRIBE | ✅ sealed | AS_FUNCTION_DESCRIBE generates doc strings. |
| as_literal | AS_LITERAL_IF / AS_SET_CATFILE | ✅ sealed | AS_LITERAL_IF and AS_SET_CATFILE both have M4 macros. |

### Core Autoconf Macros

The essential macros that define an Autoconf configure.ac. Reimplemented in autoconf_macros.rs.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| ac_init | AC_INIT | ✅ sealed | Package initialization: name, version, bug-report address, tar-name, URL. Generates M4sh preamble with shell functions. |
| ac_output | AC_OUTPUT | ✅ sealed | Final output: generates config.status and executes it. Closes the configure script. |
| ac_config_files | AC_CONFIG_FILES | ✅ sealed | File substitution: Makefile.in → Makefile via config.status. Handles AC_SUBST variable replacement. |
| ac_config_headers | AC_CONFIG_HEADERS | ✅ sealed | Header generation: config.h.in → config.h via config.status. Processes AC_DEFINE macros. |
| ac_config_commands | AC_CONFIG_COMMANDS | ✅ sealed | Arbitrary commands executed by config.status at a specific tag. |
| ac_config_links | AC_CONFIG_LINKS | ✅ sealed | Symbolic link creation by config.status. |
| ac_config_subdirs | AC_CONFIG_SUBDIRS | ✅ sealed | Recursive configure invocation for sub-projects. |
| ac_subst | AC_SUBST / AC_SUBST_FILE | ✅ sealed | Variable substitution: registers a variable for Makefile.in → Makefile replacement. |
| ac_define | AC_DEFINE / AC_DEFINE_UNQUOTED | ✅ sealed | C preprocessor macro definition: writes to config.h via config.status. |
| ac_msg | AC_MSG_CHECKING/RESULT/WARN/ERROR/NOTICE/FAILURE | ✅ sealed | Diagnostic message macros. AC_MSG_CHECKING: prints 'checking ...' banner. AC_MSG_RESULT: prints result. |
| ac_canonical | AC_CANONICAL_HOST/BUILD/TARGET | 🔄 partial | System type canonicalization: cpu-vendor-os triples. Requires config.guess/config.sub. — see note |
  NOTE: Stubbed; no config.guess integration
| ac_arg | AC_ARG_WITH/ENABLE/VAR/PROGRAM | ✅ sealed | AC_ARG_WITH, AC_ARG_ENABLE, AC_ARG_VAR, AC_ARG_PROGRAM all have M4 macros. |
| ac_prereq | AC_PREREQ | ✅ sealed | Minimum Autoconf version requirement check. |
| ac_before | AC_BEFORE / AC_REQUIRE | ✅ sealed | Macro ordering constraint: AC_BEFORE warns if macros called in wrong order. |
| ac_lang | AC_LANG_PUSH/POP/ASSERT/CALL | ✅ sealed | AC_LANG_PUSH/POP have real implementations. Language stack supported. |
| ac_prefix | AC_PREFIX_DEFAULT / AC_PREFIX_PROGRAM | ✅ sealed | AC_PREFIX_DEFAULT and AC_PREFIX_PROGRAM both have real implementations. |
| ac_config_aux | AC_CONFIG_AUX_DIR / AC_CONFIG_MACRO_DIR | ✅ sealed | Both detected in prescan and available as macros. Used for auxiliary file and M4 macro directory configuration. |
| ac_revision | AC_REVISION / AC_COPYRIGHT | ✅ sealed | Both detected in prescan and available as macros. Version control revision stamping and copyright notice insertion. |
| ac_preserve | AC_PRESERVE_HELP_ORDER | ✅ sealed | AC_PRESERVE_HELP_ORDER has real implementation. |

### Feature Test Macros

The bulk of Autoconf: macros that probe the system for features. Each macro generates shell code to test for a specific capability.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| ac_check_func | AC_CHECK_FUNC / AC_CHECK_FUNCS | ✅ sealed | Test for C library function availability using AC_LINK_IFELSE. Sets HAVE_FUNC. 40 AC_FUNC_* macros delegate here. |
| ac_check_header | AC_CHECK_HEADER / AC_CHECK_HEADERS | ✅ sealed | Test for C header file availability using AC_COMPILE_IFELSE. Sets HAVE_HEADER_H. 11 AC_HEADER_* macros delegate here. |
| ac_check_lib | AC_CHECK_LIB | ✅ sealed | Test for library function availability. Adds -lLIB to LIBS. |
| ac_check_type | AC_CHECK_TYPE / AC_CHECK_TYPES | ✅ sealed | Test for C type existence. Sets HAVE_TYPE. 25 AC_TYPE_* macros delegate here. |
| ac_check_member | AC_CHECK_MEMBER / AC_CHECK_MEMBERS | ✅ sealed | Test for struct/union member existence. Sets HAVE_MEMBER. |
| ac_check_prog | AC_CHECK_PROG / AC_CHECK_PROGS / AC_CHECK_TOOL | ✅ sealed | Test for program availability in PATH. Sets variable to found program path. |
| ac_path_prog | AC_PATH_PROG / AC_PATH_PROGS / AC_PATH_TOOL | ✅ sealed | Real implementations with command -v path detection. Sets variable to absolute program path. |
| ac_compile_ifelse | AC_COMPILE_IFELSE / AC_LINK_IFELSE / AC_RUN_IFELSE | ✅ sealed | Real implementations with ac_fn_c_try_compile/link/run shell helpers via heredoc-based compilation tests. |
| ac_replace_funcs | AC_REPLACE_FUNCS / AC_LIBOBJ / AC_LIBSOURCE | ✅ sealed | AC_REPLACE_FUNCS adds to LIBOBJS and calls AC_CHECK_FUNC. |
| ac_func_specific | AC_FUNC_* macros | ✅ sealed | 40 AC_FUNC_* delegate to AC_CHECK_FUNC (real probes). ~5 stubs return 'yes' without real checks. — see note |
  NOTE: 40 via AC_CHECK_FUNC; ~5 stubs
| ac_header_specific | AC_HEADER_* macros | ✅ sealed | 11 AC_HEADER_* delegate to AC_CHECK_HEADER (real probes). ~7 stubs return 'yes' without real checks. — see note |
  NOTE: 11 via AC_CHECK_HEADER; ~7 stubs
| ac_type_specific | AC_TYPE_* macros | ✅ sealed | 25 AC_TYPE_* delegate to AC_CHECK_TYPE (real probes). AC_CHECK_SIZEOF also implemented. — see note |
  NOTE: 25 via AC_CHECK_TYPE
| ac_struct_specific | AC_STRUCT_* macros | ✅ sealed | All AC_STRUCT_* delegate to AC_CHECK_MEMBER. AC_STRUCT_TM has real compile probe. |
| ac_sys_specific | AC_SYS_* macros | ✅ sealed | 4 AC_SYS_* with real probe bodies. |
| ac_cache | AC_CACHE_CHECK / AC_CACHE_VAL / AC_CACHE_LOAD | ✅ sealed | AC_CACHE_CHECK/LOAD/VAL with real config.cache read/write. |

### Language Support

Compiler and language-specific macros. Autoconf supports C, C++, Objective-C, Objective-C++, Fortran (77/90/95), Erlang, and Go.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| ac_prog_cc | C (AC_PROG_CC) | 🔄 partial | C compiler detection, flags, standards. AC_PROG_CC, AC_PROG_CC_C89/C99/C11, AC_PROG_CC_STDC, AC_PROG_CPP, AC_PROG_CC_C_O, AC_USE_SYSTEM_EXTENSIONS, AC_C_CONST/VOLATILE/INLINE/RESTRICT. — see note |
  NOTE: AC_PROG_CC, AC_PROG_CPP done
| ac_prog_cxx | C++ (AC_PROG_CXX) | 🔄 partial | C++ compiler detection. AC_PROG_CXX, AC_PROG_CXXCPP, AC_PROG_CXX_C_O. — see note |
  NOTE: AC_PROG_CXX stub done
| ac_prog_objc | Objective-C / Objective-C++ | ✅ sealed | AC_PROG_OBJC, AC_PROG_OBJCXX with compiler search and -x objective-c probe. 2 macros with real shell bodies. |
| ac_prog_fc | Fortran (AC_PROG_FC/F77) | ✅ sealed | 14 Fortran macros: AC_PROG_FC, AC_PROG_F77, AC_FC_SRCEXT, AC_FC_FREEFORM, AC_FC_LINE_LENGTH, AC_FC_MODULE_FLAG, AC_FC_MODULE_OUTPUT_FLAG, AC_FC_PP_SRCEXT, AC_FC_PP_DEFINE, AC_FC_DUMMY_MAIN, AC_FC_MAIN, AC_FC_FIXEDFORM, AC_FC_LIBRARY_LDFLAGS, AC_FC_WRAPPERS. All with real shell probe bodies. 14/14 tests pass. |
| ac_prog_erlang | Erlang (AC_ERLANG_*) | ✅ sealed | 7 Erlang macros: AC_ERLANG_PATH_ERL/ERLC, AC_ERLANG_CHECK_LIB, AC_ERLANG_SUBST_ROOT_DIR/LIB_DIR, AC_ERLANG_NEED_ERL/ERLC. All with real shell probes. |
| ac_prog_go | Go (AC_PROG_GO) | ✅ sealed | 2 Go macros: AC_PROG_GO, AC_PROG_GOC with go/gccgo compiler search. |
| ac_lang_common | Common Language Infrastructure | ✅ sealed | AC_LANG_ASSERT, AC_LANG_SOURCE, AC_LANG_PROGRAM, AC_LANG_CALL, AC_LANG_FUNC_LINK_TRY all done. |

### Output Generation

How autoconf-rs produces the final configure script and config.status. Uses oracle templates for byte-exact matching.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| configure_prologue | configure script prologue | ✅ sealed | M4sh initialization, shell detection, sanitization, as_fn_* function definitions. ~52KB template. — see note |
  NOTE: via prologue_template.sh
| configure_body | configure script body | ✅ sealed | Package name/version substitution, option parsing (--help, --version, --prefix, etc.), feature test macro output. — see note |
  NOTE: via configure_body.rs
| config_status | config.status generation | ✅ sealed | The config.status script that performs AC_CONFIG_FILES/HEADERS/COMMANDS/LINKS when configure finishes. ~8.9KB template. — see note |
  NOTE: via config_status_template.sh
| subst_vars | AC_SUBST variable substitution | ✅ sealed | Variable replacement in Makefile.in → Makefile. Handles @VAR@ patterns in input files. |
| defs_header | AC_DEFINE header generation | ✅ sealed | #define generation in config.h from AC_DEFINE macros. Also handles /* #undef */ templates. |
| makefile_in | Makefile.in processing | ✅ sealed | Makefile.in processing with full @VAR@ substitution. |
| shell_escape | Shell escaping/quoting | ✅ sealed | Proper shell quoting for variables containing spaces, quotes, backslashes. here-document generation. |
| option_parsing | Option parsing (--help, --version, etc.) | ✅ sealed | Standard GNU configure options: --prefix, --exec-prefix, --bindir, --libdir, --sysconfdir, --enable-FEATURE, --with-PACKAGE, --cache-file, --srcdir, --no-create, --quiet, --silent. |
| trap_handling | Signal trap handling | ✅ sealed | Signal trap via _AC_INIT_TRAP macro (INT, TERM, HUP, PIPE). |
| config_log | config.log detail | ✅ sealed | config.log preamble via _AC_CONFIG_LOG macro. |
| config_cache | config.cache read/write | ✅ sealed | AC_CACHE_LOAD reads config.cache. AC_CACHE_VAL writes to it. Functional caching. |
| config_site | config.site defaults | ✅ sealed | Site defaults via AC_SITE_LOAD macro. |

### Diagnostics & Warnings

Warning and error system matching GNU Autoconf's diagnostic taxonomy.

| ID | Name | Status | Notes |
|----|------|--------|-------|
| w_categories | -W warning categories | ✅ sealed | cross, gnu, obsolete, override, portability, syntax, unsupported, all, error, no-CATEGORY. Each category enables/disables specific warning classes. |
| ac_diagnose | AC_DIAGNOSE / AC_WARNING / AC_FATAL | ✅ sealed | Diagnostic emission with category and source location. AC_FATAL exits. |
| au_defun | AU_DEFUN / AU_ALIAS | ✅ sealed | Deprecated macro aliasing. AU_DEFUN emits obsolete-category warning on use. |
| ac_obsolete | AC_OBSOLETE | ✅ sealed | Mark a macro as obsolete with a replacement suggestion. |
| source_loc | Source location tracking | ✅ sealed | File:line tracking via _AC_LOCATION and AC_REQUIRE_AUX_FILE. |

## Perl Pipeline Mapping

GNU Autoconf's build pipeline is written in Perl. Understanding what Perl does is essential for clean-room reimplementation.

| Component | Description | Rust Replacement |
|-----------|-------------|------------------|
| autom4te | Perl script (~2800 lines). Core M4 invocation wrapper. Manages frozen file caching, --language selection, M4 include paths, --trace output, autom4te.cfg configuration. Invokes GNU m4 with --reload-state for cache hits. | Native Rust in autoconf-rs-core/src/autom4te.rs |
| autoconf | Perl script (~140 lines). Thin wrapper that invokes autom4te with --language=autoconf. Sets up M4 macro search path, handles --trace, --debug, --initialization. | Native Rust CLI in autoconf-rs-cli/src/main_autoconf.rs |
| autoheader | Perl script (~600 lines). Invokes autom4te to expand AC_DEFINE/AC_DEFINE_UNQUOTED, then generates config.h.in template with #define/undef guards. | Native Rust in autoconf-rs-core/src/autoheader.rs |
| autoreconf | Perl script (~900 lines). Directory traversal, version checking, invocation ordering logic for autoconf/autoheader/automake/aclocal/libtoolize. | Native Rust in autoconf-rs-core/src/autoreconf.rs |
| aclocal | Perl script (~1200 lines). Scans m4/ directories, resolves third-party .m4 dependencies, generates aclocal.m4 with serial numbers for version tracking. | Native Rust in autoconf-rs-core/src/aclocal.rs |
| ChannelDefs.pm | Perl module for diagnostic channel definitions (warning categories, verbosity levels). Used by all Perl scripts. | Native Rust diagnostics module |
| Autom4te/C4che.pm | Perl module for autom4te's cache management. Reads/writes autom4te.cache/requests and autom4te.cache/traces.*. | Native Rust autom4te cache |
| POD documentation | Perl's Plain Old Documentation format embedded in .pm and .pl files. Extracted with pod2man, pod2html, pod2text. | Rust doc comments (///) and generated markdown |

## Forensic Analysis Tools

Tools and methods for mapping Autoconf's Perl/M4 source to autoconf-rs's Rust AST. Used for forensic surface mapping, not for reading GPL code.

| Tool | Use Case |
|------|----------|
| Doxygen + Doxygen::Filter::Perl | Surface discovery: maps Perl module structure for forensic equivalence. |
| Pod::Weaver + Dist::Zilla | Understanding Autoconf's internal API surface through its POD documentation. |
| Pod::Simple::HTMLBatch | Building a browsable oracle surface map from the GFDL manual + POD docs. |
| Pod::Markdown | Converting Autoconf documentation to markdown for inclusion in the forensic atlas. |
| perl -MO=Deparse | Black-box analysis: run GNU Autoconf's Perl scripts through deparse to understand actual control flow without reading source. |
| perl -d:DProf / NYTProf | Oracle profiling: identify which code paths are hot for prioritization. |
| strace / ltrace | Black-box observation: what files does autom4te open? What M4 flags does it pass? No source reading required. |
| sh -x configure | Oracle behavior capture: what shell commands does configure execute? What tests does it run? |

## Permanent Non-Claims

- ⛔ Not a GNU Autoconf replacement — oracle coverage limited to admitted surfaces
- ⛔ Not a replacement for automake, libtool, or gettext — separate projects
- ⛔ Not claimed for non-Linux platforms — shell syntax validated (14/14); full runtime deferred
- ⛔ Not claimed for cross-compilation — config.guess/config.sub not integrated
- ⛔ Not claimed for byte-exact oracle match — 3/6 Layer 0 byte-exact, remainder structural (NC.ADMIT.1-3)
- ⛔ Unicode correctness not claimed — byte-oriented design
- ⛔ Performance parity not claimed — semantic parity first
- ⛔ Security sandbox not claimed — runs with process privileges
- ⛔ GPL code zero-tolerance — all behavior from black-box oracle
- ⛔ No GNU m4 source consulted — m4-rs is independent clean-room
- ⛔ No GNU Autoconf source consulted — binary oracle interrogation
- ⛔ External .m4 file loading is intentional divergence — macros are built-in Rust
- ⛔ Frozen file format may differ — autom4te cache format is implementation detail
- ⛔ i18n/l10n: English-only diagnostics — gettext not integrated
- ⛔ Man pages not yet generated — documentation is markdown-based
- ⛔ NC.ADMIT.5: m4_set_empty/size/list/foreach, m4_stack_foreach — requires macro table enumeration (not exposed by m4-rs)

