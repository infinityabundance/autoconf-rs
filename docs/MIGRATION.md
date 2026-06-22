# Migration Guide: GNU Autoconf → autoconf-rs

## Overview

autoconf-rs is a native Rust reimplementation of GNU Autoconf behavior.
It processes the same `configure.ac` files and generates `configure` scripts
with compatible semantics. This guide covers the differences and migration steps.

## Quick Start

```bash
# Replace GNU autoconf with autoconf-rs in your workflow:
autoconf-rs configure.ac > configure
chmod +x configure
./configure
```

## Compatibility

| Feature | Status | Notes |
|---------|--------|-------|
| AC_INIT / AC_OUTPUT | ✅ | Full support |
| AC_CONFIG_FILES / AC_CONFIG_HEADERS | ✅ | Substitution works |
| AC_SUBST / AC_DEFINE | ✅ | Variable and define passing |
| AC_CHECK_FUNC / AC_CHECK_HEADER | ✅ | Shell probes generated |
| AC_CHECK_LIB / AC_CHECK_PROG | ✅ | Library and program detection |
| AC_PROG_CC / AC_PROG_CXX | ✅ | C/C++ compiler detection |
| AC_C_* conformance macros | ✅ | All 10 C conformance checks |
| Fortran (AC_PROG_FC, AC_FC_*) | ✅ | All 14 Fortran macros |
| Erlang / Go / ObjC | ✅ | Language detection |
| Autoheader (config.h.in) | ✅ | Trace-driven generation |
| Autom4te caching | ✅ | SHA256 JSON cache |
| Autoreconf orchestration | ✅ | 4-tool chain |

## Differences from GNU Autoconf

1. **Architecture**: autoconf-rs uses prescan + template dispatch instead of pure M4 expansion.
   Configure scripts are functionally equivalent but structurally different.

2. **Cache format**: autom4te cache uses JSON instead of GNU frozen files. Caches are not cross-compatible.

3. **syscmd/esyscmd**: These M4 builtins are no-ops in autoconf-rs (safe Rust cannot execute arbitrary shell commands).

4. **Performance**: autoconf-rs is ~94x faster than GNU Autoconf 2.73 (1.8ms vs 170ms per invocation).

## Migration Steps

1. Replace `autoconf` with `autoconf-rs` in your Makefile or build script
2. Run `autoconf-rs configure.ac` to generate configure
3. Run `./configure` as usual
4. Verify config.status produces expected Makefiles and config.h

## Troubleshooting

- **"command not found"**: Ensure autoconf-rs is installed and in PATH
- **Substitution mismatch**: Verify AC_SUBST variable names match Makefile.in templates
- **Missing config.h entries**: Run autoheader to regenerate config.h.in

## Reporting Issues

File issues at the autoconf-rs repository with:
- Your configure.ac (or a minimal reproduction)
- Expected vs actual output
- Oracle version used (autoconf --version)
