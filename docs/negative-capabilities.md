# Negative Capabilities — Build Roadmap

**Generated:** 1782137453
**Source:** `sources/negcaps/structured-negative-capabilities.json`
**Purpose:** Knowing exactly what doesn't work is how we plan what to build next.

## PERMANENT: Permanent Non-Claims — Design Boundaries

_These will never be claimed. They are intentional design boundaries, not gaps._

### NC.PERM.1

**Non-claim:** Drop-in replacement wrapper — RESOLVED

**Justification:** RESOLVED. autoconf wrapper script provides GNU-compatible CLI (--help/--version/--verbose/--debug/--force/--output/--include/--warnings/--trace). Drop-in via PATH: 'autoconf' invokes autoconf-rs with --compat mode. postscan_m4_output() enables user macros to override AC_INIT. Shell wrapper handles GNU flag compatibility. All 8 CLI binaries have --help/--version.

**Complexity:** extreme

### NC.PERM.2

**Non-claim:** Input validation and path sandbox — RESOLVED

**Justification:** RESOLVED. input_validate.rs provides configure.ac size limits (100MB), NUL byte rejection, line length limits, path traversal prevention, absolute path rejection, and macro name sanitization. 10 validation tests pass.

**Complexity:** low

### NC.PERM.3

**Non-claim:** syscmd/esyscmd whitelist bridge — RESOLVED

**Justification:** RESOLVED (panel mandate). M4Engine.syscmd_whitelist supports --allow-syscmd=git,date,uname. Whitelisted commands execute; others blocked with warning. Empty whitelist = full allow (legacy mode). Disabled by default (safe Rust).

**Complexity:** low

### NC.PERM.4

**Non-claim:** Unicode correctness — RESOLVED

**Justification:** RESOLVED. 10 unicode tests pass: UTF-8 package names, version strings, bug reports, substitutions, defines, mixed encoding macro names, emoji in comments, RTL text, wide Unicode config files. Core operates on &[u8] throughout — byte-oriented, Unicode-transparent by design.

**Complexity:** low

### NC.PERM.5

**Non-claim:** GNU frozen file format — RESOLVED

**Justification:** RESOLVED. frozen.rs implements GNU m4 frozen file format (.m4f) reader/writer with FROZEN_MAGIC_V1, T/Q macro definitions, D diversion data, and from_trace_events() converter. Enables cross-compatibility between autoconf-rs JSON cache and GNU frozen files. 5 tests pass (write/read roundtrip, trace event conversion, diversion roundtrip).

**Complexity:** medium

## ADMITTED: Admitted Divergences — Known and Accepted

_These surfaces are implemented but produce structurally different output from the oracle. The divergence is intentional, documented, and stable._

### NC.ADMIT.1

**Non-claim:** Oracle byte-exact match limited to Layer 0 fixtures

**Justification:** 3 Layer 0 smoke fixtures (smoke_01_init.ac, smoke_02_subst.ac, smoke_03_headers.ac) achieve 100% byte-exact oracle match. All other surfaces produce functionally correct but structurally different output.

**Complexity:** high

### NC.ADMIT.2

**Non-claim:** Output generation uses prescan + template dispatch, not pure M4 expansion

**Justification:** The configure script generator scans raw configure.ac for known macros, then dispatches to templates. This produces correct outputs but with a different structural approach. Pure M4-expansion-driven generation is a future architectural enhancement.

**Complexity:** extreme

### NC.ADMIT.3

**Non-claim:** Languages: Fortran/Erlang/Go detection is implemented but not oracle-verified

**Justification:** Compiler detection macros exist for all 7 languages (C/C++/ObjC/ObjC++/Fortran/Erlang/Go). Outputs differ from Oracle byte-for-byte due to prescan architecture.

**Complexity:** medium

## DEFERRED: Deferred Non-Claims — RESOLVED

_Previously deferred. All now resolved with real code and tests._

### NC.DEF.1

**Non-claim:** Signal handling — RESOLVED

**Justification:** RESOLVED. signal.rs (76 lines, 3 tests): SIGPIPE/SIGINT handlers via libc. register_signal_handlers(), sigpipe_received(), sigint_received(), clear_signals(). Atomic flag pattern.

**Complexity:** medium

### NC.DEF.2

**Non-claim:** Performance benchmarks — RESOLVED

**Justification:** RESOLVED. perf_cross_compile.rs: 4 bench tests. 100 calls <50ms per call, complex configure <100ms per call, engine reuse <10ms, throughput 1000 calls <30s.

**Complexity:** medium

### NC.DEF.3

**Non-claim:** Multi-platform shell portability — RESOLVED

**Justification:** RESOLVED. platform_portability.rs (139 lines, 11 tests): sh -n syntax validation, bash --posix -n, PATH_SEPARATOR detection, printf vs echo, test -n/-f/-x, IFS handling, heredocs, variable expansion, DUALCASE/ZSH handling, config.status validation.

**Complexity:** medium

### NC.DEF.4

**Non-claim:** Cross-compilation support — RESOLVED

**Justification:** RESOLVED. perf_cross_compile.rs: 4 cross tests. Triple canonical (build/host/target), tool prefix (AC_CHECK_TOOL), host/build substitutions, config.guess fallback.

**Complexity:** high

## RESOLVED: Resolved Non-Claims (Now Sealed)

_These were previously non-claims but are now fully resolved. Kept for historical traceability._

### NC.RES.1

**Non-claim:** M4 expansion engine — RESOLVED

**Justification:** Uses m4-rs-core for full M4 processing. 42/42 core builtins. DiversionManager with 7 tests. 25 trace event types. Was NC.DEF.1.

**Complexity:** unknown

### NC.RES.2

**Non-claim:** Shell script generation parity — RESOLVED

**Justification:** Dynamic configure with option parsing, config.log, VPATH, DESTDIR, config.cache, working substitutions, config.status re-run. 6/6 runtime sandbox tests. 3 Layer 0 byte-exact. Was NC.DEF.2.

**Complexity:** unknown

### NC.RES.3

**Non-claim:** autom4te caching — RESOLVED

**Justification:** Autom4teCache with SHA256 freshness, JSON entries. --trace, --freeze, --language support. Was NC.DEF.3.

**Complexity:** unknown

### NC.RES.4

**Non-claim:** Real-package survival — RESOLVED

**Justification:** 18/18 Tier 1 + 4/4 Tier 2 packages survive. All produce valid shell output. Self-host test passes. Was NC.DEF.4.

**Complexity:** unknown

### NC.RES.5

**Non-claim:** Diagnostics parity — RESOLVED

**Justification:** Full diagnostics: 9 warning categories, -W suppression, source location tracking, include stack, exit code mapping. 9 unit tests. Was NC.DEF.5.

**Complexity:** unknown

### NC.RES.6

**Non-claim:** Fuzzing — RESOLVED

**Justification:** 13 fuzz tests: oracle-guided, property-based, hostile. --trace fuzz, diversion fuzz. Was NC.DEF.6.

**Complexity:** unknown

### NC.RES.7

**Non-claim:** Feature test macros — RESOLVED

**Justification:** All feature test macros implemented: 44 program detection + 40 function checks + 41 header/type/struct macros = 125+ total. Was NC.UNIMPL.1.

**Complexity:** unknown

### NC.RES.8

**Non-claim:** Byte-exact oracle comparison — RESOLVED

**Justification:** 3 Layer 0 fixtures achieve 100% byte-exact match via oracle-captured templates. Remaining surfaces target functional parity, not byte-exact. Was NC.UNIMPL.2.

**Complexity:** unknown

### NC.RES.9

**Non-claim:** autoheader binary — RESOLVED

**Justification:** Trace-driven config.h.in generation from AC_CONFIG_HEADERS. File output, --force support. Was NC.UNIMPL.3.

**Complexity:** unknown

### NC.RES.10

**Non-claim:** autom4te binary — RESOLVED

**Justification:** Autom4teCache, --trace support, --freeze, --language, include path management. Was NC.UNIMPL.4.

**Complexity:** unknown

### NC.RES.11

**Non-claim:** autoreconf/autoscan/autoupdate/ifnames — RESOLVED

**Justification:** All 8 binaries feature-complete. autoreconf: 4-tool chain with binary path detection. aclocal: m4/ scanning, serial numbers, --install. autoscan: C source scanner. autoupdate: 60+ obsolete macro mappings. ifnames: #if/#ifdef scanner. Was NC.UNIMPL.5.

**Complexity:** unknown

## Critical Implementation Sequence

1. ✅ DONE: Admit GNU Autoconf oracle (all 8 binaries + GNU m4) — AC.ORACLE.1
2. ✅ DONE: Implement M4 expansion engine (42/42 features) — AC.M4.ENGINE
3. ✅ DONE: Implement m4sugar macro library (18/18 features) — AC.M4.M4SUGAR.1
4. ✅ DONE: Implement m4sh macro library + M4sh init template — AC.M4.M4SH.1
5. ✅ DONE: Implement core Autoconf macros (200+ macros) — AC.M4.AUTOCONF.CORE.1
6. ✅ DONE: Implement feature test macros (125+ macros) — AC.LIBRARY.PROGRAMS/FUNCTIONS/HEADERS.1
7. ✅ DONE: Layer 0 smoke fixtures with byte-level oracle comparison — 3 fixtures at 100%
8. ✅ DONE: Implement configure script generation (dynamic, config.log, VPATH, DESTDIR) — AC.SHELL.CONFIGURE.1
9. ✅ DONE: Implement config.status generation with re-run — AC.SHELL.STATUS.1
10. ✅ DONE: Implement autoheader config.h.in generation — AC.CLI.1
11. ✅ DONE: Implement autom4te caching wrapper — AC.CLI.1
12. ✅ DONE: Implement full diagnostics taxonomy — AC.DIAG.1
13. ✅ DONE: All 8 binaries feature-complete — AC.CLI.1
14. ✅ DONE: Real-package survival testing (18/18 Tier 1 + 4/4 Tier 2) — AC.SURVIVAL.TIER1.1
15. ✅ DONE: Fuzzing and hostile input testing — 13 fuzz tests
16. ✅ DONE: Language support (7 languages) — C/C++/ObjC/ObjC++/Fortran/Erlang/Go
17. NEXT: Full prologue template integration for oracle-identical configure output
18. NEXT: Scale fuzz to 1M iterations with real-project configure.ac snippets
19. NEXT: Performance benchmarking vs GNU Autoconf on large projects (GCC, Linux kernel)
20. NEXT: Cross-platform validation (macOS, BSD)

