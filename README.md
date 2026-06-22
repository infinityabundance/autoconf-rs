# autoconf-rs

**A native Rust forensic-parity implementation of GNU Autoconf behavior, built through oracle courts.**

`autoconf-rs` is a clean-room behavioral reconstruction of GNU Autoconf. Each supported surface is admitted only after byte comparison against a pinned GNU Autoconf oracle. Unsupported surfaces are explicit non-claims.

New here? Start with `docs/REVIEW-IN-10-MINUTES.md`.

## Status

| Metric | Value |
|--------|-------|
| Phase | 2 — Level 1 Generator — Prescan + template dispatch. NOT full M4 expansion (NC.ADMIT.2). |
| Overall completion | **48.5%** |
| Oracle | GNU Autoconf 2.73 (admitted) |
| Courts sealed | 0 |
| Tests passing | 1382 |
| Acceptance gates | 6/7 PASS (oracle gate: 0% exact match — HONEST FAIL) |
| Clean-room scan | 72 files, 0 GPL contamination |
| Strategy | Clean-room behavioral reconstruction, forensic parity methodology |

## Surface Status

- ⬜ **AC.ORACLE.1**: GNU 2.73 admitted. 3/3 Layer0 byte-exact. 0/100 exact on parity report.
- ⬜ **AC.CLI.1**: 8 binaries exist (stub/partial). autom4te is JSON cache, not real trace engine.
- ⬜ **AC.M4.ENGINE**: m4-rs-core integration. DiversionManager wired. M4 output DISCARDED — prescan bypass (NC.ADMIT.2).
- ⬜ **AC.M4.M4SUGAR.1**: 48 macros REGISTERED. RequireTracker exists. Prescan limits correctness.
- ⬜ **AC.M4.M4SH.1**: 42 AS_* macros REGISTERED. Template prologue simplified vs oracle.
- ⬜ **AC.M4.AUTOCONF.CORE.1**: 200+ macros registered. Template dispatch for simple literal AC_* text.
- ⬜ **AC.SHELL.CONFIGURE.1**: Dynamic configure with option parsing. Simplified shell vs oracle.
- ⬜ **AC.LIBRARY.PROGRAMS.1**: 44 program macros. Shell snippets in dynamic body.
- ⬜ **AC.LIBRARY.FUNCTIONS.1**: 40 function macros. Shell snippets. NOT oracle-verified.
- ⬜ **AC.LIBRARY.HEADERS.1**: 41 header/type macros. Shell snippets. NOT oracle-verified.
- ⬜ **AC.DIAG.1**: 9 warning categories. DiagnosticManager. Source location basic.
- ⬜ **AC.SURVIVAL.TIER1.1**: 18 packages generate configure. NOT runtime-tested.

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
