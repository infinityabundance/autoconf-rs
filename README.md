# autoconf-rs

**A native Rust forensic-parity implementation of GNU Autoconf behavior, built through oracle courts.**

`autoconf-rs` is a clean-room behavioral reconstruction of GNU Autoconf. Each supported surface is admitted only after byte comparison against a pinned GNU Autoconf oracle. Unsupported surfaces are explicit non-claims.

New here? Start with `docs/REVIEW-IN-10-MINUTES.md`.

## Status

| Metric | Value |
|--------|-------|
| Phase | 6 — Honest Assessment — 101 sealed, 6 partial, 0 missing. Prescan + template dispatch with --pure-m4 escape hatch. |
| Overall completion | **94.4%** |
| Oracle | GNU Autoconf 2.73 (admitted) |
| Courts sealed | 11 |
| Tests passing | 2288 |
| Acceptance gates | 7/7 PASS |
| Clean-room scan | 72 files, 0 GPL contamination |
| Strategy | Clean-room behavioral reconstruction, forensic parity methodology |

## Surface Status

- ✅ **AC.ORACLE.1**: 4/4 versions, 32/32 binaries. 3/6 Layer0 byte-exact. Oracle fuzz: 0 panics.
- ✅ **AC.CLI.1**: 8/8 binaries build and run. Cache integration. --pure-m4 flag added.
- ✅ **AC.M4.ENGINE**: m4-rs-core integration. DiversionManager, TraceLog. 318 tests.
- ✅ **AC.M4.M4SUGAR.1**: 18 features sealed. RequireTracker, M4SugarBuiltins. 108 tests.
- ✅ **AC.M4.M4SH.1**: 26 AS_* macros sealed. Shell portability. 113 tests.
- ✅ **AC.M4.AUTOCONF.CORE.1**: 45 core macros sealed. AC_INIT..AC_OUTPUT pipeline. 72 tests.
- ✅ **AC.SHELL.CONFIGURE.1**: Prologue, option parsing, config.log/status. 14/14 shell tests pass.
- ✅ **AC.LIBRARY.PROGRAMS.1**: All 7 languages with real compiler detection. Fortran 14, Erlang 7, Go 2, ObjC 2.
- ✅ **AC.LIBRARY.FUNCTIONS.1**: 40 AC_FUNC_* delegate to AC_CHECK_FUNC. ~5 stubs. Structural divergence admitted.
- ✅ **AC.LIBRARY.HEADERS.1**: 11 AC_HEADER_* delegate to AC_CHECK_HEADER. 25 AC_TYPE_* delegate to AC_CHECK_TYPE. ~7 stubs.
- ✅ **AC.DIAG.1**: 30 tests. WarningCategory taxonomy, DiagnosticManager, include stack.
- ⬜ **AC.SURVIVAL.TIER1.1**: 18/18 Tier1 + 4/4 Tier2 survive. Self-host passes. Not runtime-tested.

## Quick Start

```bash
# Build
cargo build --release

# Process a configure.ac
cargo run -p autoconf-rs-cli --bin autoconf -- configure.ac

# Run acceptance gates
cargo xtask check

# Regenerate documents
cargo xtask generate

# Run fuzz harness
cargo xtask fuzz

# View status
cargo xtask status
```

## Key Documents

| Document | Purpose |
|----------|---------|
| STATUS.md | Live current-state authority (generated, freshness-gated) |
| reports/NEEDLE-REPORT.md | Per-surface completion percentages |
| reports/FORENSIC-GAP-ANALYSIS.md | Full C→Rust gap audit |
| docs/forensic-atlas.md | Complete surface atlas with archaeology |
| docs/negative-capabilities.md | Explicit non-claims and build roadmap |
| docs/REVIEW-IN-10-MINUTES.md | Quick overview for new reviewers |

## License

MIT OR Apache-2.0. Zero GPL code.
