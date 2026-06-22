# AC.SURVIVAL.TIER1.1 — cross-distro runtime receipts

Runtime test of the **crates.io** install (`cargo install autoconf-rs-cli --version 0.1.0`,
rustup-stable toolchain / cargo 1.96) against the Tier1 + Tier2 package `configure.ac`
fixtures, run inside five QEMU/KVM VMs. The sealed `survival-receipt.json` was marked
"Not runtime-tested"; these receipts are that runtime test.

Each VM booted a stock cloud image, installed the published crate from crates.io,
ran the installed `autoconf` on every fixture, and PUT its receipt back to a host
HTTP server. Survival criterion: output begins with `#! /bin/sh` **and** size > 100
**and** contains `config.status`/`AC_OUTPUT`.

## Result (identical on all 5 distros)

| distro | survived | Tier1 | Tier2 |
|--------|----------|-------|-------|
| Ubuntu 24.04.4 LTS | 22/27 | 19/23 | 3/4 |
| Debian 13 (trixie) | 22/27 | 19/23 | 3/4 |
| Fedora 43 | 22/27 | 19/23 | 3/4 |
| AlmaLinux 9.8 | 22/27 | 19/23 | 3/4 |
| openSUSE Leap 16.0 | 22/27 | 19/23 | 3/4 |

**Same 5 failures on every distro:** `curl`, `openssl`, `sqlite`, `zlib` (Tier1) and
`stress_02_nested` (Tier2).

**Root cause:** those 5 produce substantial output with a `config.status`/`AC_OUTPUT`
stage, but the generated script does **not** begin with `#! /bin/sh` — autoconf-rs
echoes a leading comment line from the `.ac` (e.g. `# configure.ac — curl (Tier 2)`)
before the shebang, so the shebang is not line 1.

**Vs. the sealed claim:** `survival-receipt.json` claims 18/18 Tier1 + 4/4 Tier2 survive;
runtime is **19/23 Tier1 + 3/4 Tier2**. The "survive" claim is not reproduced at runtime.

## Status: FIXED — re-verified across all 5 distros (27/27)

Fixed in `M4Engine::process` (`ensure_shebang_first`, commit `3f28caa`): the generated
`configure` now always begins with `#! /bin/sh`, dropping leading `.ac` comment lines
(matching the GNU Autoconf 2.73 oracle).

The harvest was re-run on all 5 distros against the **patched** crate (installed from
the GitHub source pinned to the fix commit, since the patch is not yet on crates.io).
Post-fix receipts are in [`post-fix/`](post-fix/):

| distro | survived | Tier1 | Tier2 | verdict |
|--------|----------|-------|-------|---------|
| Ubuntu 24.04.4 | 27/27 | 23/23 | 4/4 | PASS |
| Debian 13 | 27/27 | 23/23 | 4/4 | PASS |
| Fedora 43 | 27/27 | 23/23 | 4/4 | PASS |
| AlmaLinux 9.8 | 27/27 | 23/23 | 4/4 | PASS |
| openSUSE Leap 16.0 | 27/27 | 23/23 | 4/4 | PASS |

All previously-failing fixtures (`curl`, `openssl`, `sqlite`, `zlib`,
`stress_02_nested`) now survive on every distro. The top-level receipts in this
directory remain as the **pre-fix** evidence that found the defect.

## Files

- `ubuntu.json`, `debian.json`, `fedora.json`, `almalinux.json`, `opensuse.json` — per-VM receipts
- `aggregate.json` — host-built cross-distro summary
