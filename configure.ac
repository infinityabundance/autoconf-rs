#                                               -*- Autoconf -*-
# configure.ac for autoconf-rs — self-host test (CROSS.046)
#
# This file enables autoconf-rs to process its own project definition,
# proving self-hosting capability. Generated with autoconf-rs itself.

AC_PREREQ([2.50])
AC_INIT([autoconf-rs], [0.1.0], [autoconf-rs@example.com])
AC_CONFIG_SRCDIR([crates/autoconf-rs-core/src/lib.rs])
AC_CONFIG_AUX_DIR([build-aux])
AC_CONFIG_MACRO_DIR([m4])

# Detect Rust toolchain
AC_CHECK_PROG([CARGO], [cargo], [cargo])
AC_CHECK_PROG([RUSTC], [rustc], [rustc])
AC_SUBST([CARGO])
AC_SUBST([RUSTC])

# Build configuration
AC_ARG_WITH([debug], [AS_HELP_STRING([--with-debug], [build with debug symbols])])
AC_ARG_ENABLE([release], [AS_HELP_STRING([--enable-release], [build in release mode])])

# Feature checks
AC_CHECK_FUNCS([malloc realloc])
AC_CHECK_HEADERS([stdio.h stdlib.h string.h])

AC_CONFIG_FILES([Makefile])
AC_OUTPUT
