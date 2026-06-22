# Oracle Version Pinning

## Purpose

autoconf-rs uses **forensic parity** methodology: correctness means "matches the pinned GNU Autoconf oracle for the admitted surface," not "matches the manual." This document explains how to pin oracle versions and re-run oracle admission when the system autoconf changes.

## Quick Start

```bash
# Admit the current system GNU Autoconf as the oracle
cargo xtask oracle

# Check oracle profile status
cargo xtask status
```

## How Oracle Pinning Works

### 1. Oracle Profile (`reports/oracle-profile.json`)

The oracle profile captures the identity of the GNU Autoconf binary used for parity comparisons:

- **Binary path**: Absolute path to the autoconf binary (e.g., `/usr/bin/autoconf`)
- **Binary SHA256**: Cryptographic hash of the binary
- **Version output**: Exact output of `autoconf --version`
- **Admitted features**: What the oracle supports (tested on first admission)
- **Platform identity**: OS, architecture, kernel version, libc version
- **Locale**: `LC_ALL=C` is used for all comparisons
- **Shell**: `/bin/sh` is used for pipe/command-substitution tests

### 2. Version Drift Detection

```bash
# Check if the system's autoconf has changed since last admission
cargo xtask oracle --check
```

If the system autoconf binary has changed (e.g., a system upgrade), all prior receipts become **stale**. You must:

1. Re-admit the new oracle: `cargo xtask oracle`
2. Re-run all courts: `cargo xtask compare`
3. Review receipt diffs for any behavioral changes

### 3. Multi-Version Oracle Matrix

For comprehensive testing, you can test against multiple Autoconf versions:

```bash
# Admit specific version
cargo xtask oracle --binary /usr/bin/autoconf-2.71 --profile gnu_autoconf_2_71

# Compare against all admitted oracles
cargo xtask compare --all-oracles

# Generate cross-version drift report
cargo xtask compare --drift-report
```

### 4. Cross-Version Stability

GNU Autoconf has maintained strong backward compatibility across versions. However, some behavioral differences exist:

| Between | Category | Impact |
|---------|----------|--------|
| 2.69 → 2.71 | M4sh init prologue | Shell detection expanded |
| 2.71 → 2.73 | Deprecated macros | Additional `AU_DEFUN` warnings |
| All | Template whitespace | Minor formatting differences |
| All | Timestamps | Generated configure has build timestamp |

autoconf-rs operates on **behavioral parity**, not byte-identical template output. The critical test is whether generated `configure` scripts produce identical results when executed.

## Acceptance Gate

The `cargo xtask check` gate verifies:

1. **Oracle profile exists** at `reports/oracle-profile.json`
2. **Oracle profile is valid** (has all required fields)
3. **Oracle binary is accessible** at the recorded path

Gate 5 (oracle profile) will **FAIL** if:
- No oracle profile exists
- The oracle binary is not found at the recorded path
- The profile is missing required fields

To fix:
```bash
# Regenerate oracle profile
cargo xtask oracle

# Or manually specify the oracle path
AUTOCONF=/path/to/autoconf cargo xtask oracle
```

## CI/CD Integration

For CI pipelines, pin the oracle version explicitly:

```yaml
# .github/workflows/test.yml
- name: Install GNU Autoconf oracle
  run: |
    sudo apt-get install -y autoconf=2.73-1
    echo "AUTOCONF_VERSION=$(autoconf --version | head -1)" >> $GITHUB_ENV

- name: Admit oracle
  run: cargo xtask oracle

- name: Run parity tests
  run: cargo xtask compare
```

## Receipt Versioning

Each receipt records:
- The oracle profile it was generated against (`receipt.oracle.profile`)
- The autoconf-rs version (`receipt.rust.crate_version`)
- The git commit (`receipt.rust.git_commit`)

When either changes, receipts become stale and must be regenerated.

## Archival Policy

Old receipts are never automatically deleted. When a new oracle is admitted:

1. Old receipts are moved to `reports/archive/v{old_version}/`
2. A migration note is written to `reports/archive/CHANGELOG.md`
3. Fresh receipts are generated against the new oracle

This preserves the forensic trail and allows historical comparisons.

## Cross-Reference

- **Oracle admission**: `cargo xtask oracle` (runs `xtask/src/main.rs::run_oracle_admission`)
- **Parity comparison**: `cargo xtask compare` (runs `xtask/src/compare.rs`)
- **Cross-version matrix**: `crates/autoconf-oracle-rs/src/lib.rs::CrossVersionMatrix` (AC.ORACLE.1 Feature 6)
- **Oracle regression**: `crates/autoconf-rs-core/tests/oracle_regression.rs` (AC.ORACLE.1 Feature 7)
- **Layer 0 comparison**: `crates/autoconf-rs-core/tests/oracle_compare.rs` (AC.ORACLE.1 Feature 2)
- **Receipt schema**: Defined in `autoconf-casefile-rs/src/lib.rs`
- **Forensic atlas**: `docs/FORENSIC-ATLAS.md`
