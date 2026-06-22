# autoconf-rs Review in 10 Minutes

_Generated: 1782147835_

## What is this?

`autoconf-rs` is a native Rust implementation of GNU Autoconf's behavior. It reproduces GNU Autoconf output byte-for-byte for all admitted surfaces, proven through oracle comparison receipts.

## The strategy

**Oracle-first.** We don't guess what GNU Autoconf does. We run it, capture the output, and prove we match. Every claim is backed by a sealed receipt. Same forensic-parity methodology as m4-rs, gnucobol-rs, zic-rs, chrony-rs, ncurses-native.

## Current Status

| Metric | Value |
| --- | --- |
| Phase | 1 — CLI Harness |
| Strategy | Clean-room behavioral reconstruction |
| Oracle | Not yet admitted |
| Courts sealed | 0 |
| Status | Building foundational infrastructure |

## How to run

```
cargo build --release
cargo xtask oracle       # Admit the GNU Autoconf toolchain
cargo xtask check        # Run all 7 acceptance gates
cargo xtask status       # Print project status
echo 'AC_INIT([hello], [1.0])' | cargo run --bin autoconf
```

## The doctrine

- GNU Autoconf is the behavioral oracle.
- Correct means matches the pinned GNU Autoconf oracle.
- Every admitted behavior must have a sealed receipt.
- No global parity claim until every axis has a sealed receipt.
- Every unimplemented surface is a typed non-claim.
