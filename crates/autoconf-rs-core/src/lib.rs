// ============================================================================
// autoconf-rs-core: Core macro engine, shell generator, and Autoconf library
// for forensic-parity GNU Autoconf.
// ============================================================================
//
// Uses m4-rs (https://crates.io/crates/m4-rs) for M4 macro processing.
// Autoconf-specific macros, shell generation, and state management are
// implemented in this crate.

// M4 engine modules (replaced by m4-rs)
// pub mod args;           // → m4_rs::m4_rs_core::args
// pub mod builtin;        // → m4_rs::m4_rs_core::builtin
// pub mod expansion;      // → m4_rs::m4_rs_core::ExpansionEngine
// pub mod lexer;          // → m4_rs::m4_rs_core::Lexer
// pub mod token;          // → m4_rs::m4_rs_core::Token
// pub mod macro_table;    // → m4_rs::m4_rs_core::MacroTable

// Autoconf-specific modules
pub mod aclocal;
pub mod autoconf_macros;
pub mod autoheader;
pub mod autom4te;
pub mod autoreconf;
pub mod cache;
pub mod configure_ac;
pub mod configure_body;
pub mod configure_template;
pub mod diagnostics;
pub mod diversion;
pub mod fortran;
pub mod frozen;
pub mod i18n;
pub mod input_validate;
pub mod languages;
pub mod m4_engine;
pub mod m4sh;
pub mod m4sh_init;
pub mod m4sugar;
pub mod profile;
pub mod shell_gen;
pub mod signal;
pub mod site_file;
pub mod template;
pub mod trace;
pub mod trace_cache;

/// Re-export key types for convenience.
pub use configure_ac::ConfigureAc;
pub use diagnostics::{Diagnostic, DiagnosticLevel};
pub use m4_engine::M4Engine;
pub use shell_gen::ShellGenerator;

/// m4 macro definitions injected AFTER aclocal.m4 (so they override the project's third-party
/// definitions) and BEFORE configure.ac. autoconf-rs cannot yet correctly expand some standard
/// third-party macros (notably pkg.m4's PKG_CHECK_MODULES, whose internals leak
/// `pkg_default`/`glib_minimum` -> shell syntax errors). We replace them with clean, self-contained
/// shell that does the real work: run pkg-config at configure time, set PFX_CFLAGS/PFX_LIBS, and
/// record `@PFX_CFLAGS@`/`@PFX_LIBS@` substitutions into conf_subst.sed (consumed when config files
/// are created, mirroring confdefs.h -> config.h).
pub fn macro_overrides() -> &'static str {
    r#"dnl --- autoconf-rs macro overrides ---
m4_define([PKG_CHECK_MODULES], [dnl
printf %s "checking for $2... "
if pkg-config --exists "$2" 2>/dev/null; then
  printf '%s\n' "yes"
  $1_CFLAGS=`pkg-config --cflags "$2" 2>/dev/null`
  $1_LIBS=`pkg-config --libs "$2" 2>/dev/null`
  printf '%s\n' "s|@$1_CFLAGS@|$$1_CFLAGS|g" >> conf_subst.sed 2>/dev/null
  printf '%s\n' "s|@$1_LIBS@|$$1_LIBS|g" >> conf_subst.sed 2>/dev/null
  :
  $3
else
  printf '%s\n' "no"
  $1_CFLAGS=
  $1_LIBS=
  printf '%s\n' "s|@$1_CFLAGS@||g" >> conf_subst.sed 2>/dev/null
  printf '%s\n' "s|@$1_LIBS@||g" >> conf_subst.sed 2>/dev/null
  :
  $4
fi
])dnl
m4_define([PKG_CHECK_EXISTS], [dnl
if pkg-config --exists "$1" 2>/dev/null; then
  :
  $2
else
  :
  $3
fi
])dnl
m4_define([PKG_PROG_PKG_CONFIG], [PKG_CONFIG=`command -v pkg-config 2>/dev/null`
printf '%s\n' "s|@PKG_CONFIG@|$PKG_CONFIG|g" >> conf_subst.sed 2>/dev/null])dnl
m4_define([PKG_INSTALLDIR], [pkgconfigdir=${libdir}/pkgconfig
printf '%s\n' "s|@pkgconfigdir@|$pkgconfigdir|g" >> conf_subst.sed 2>/dev/null])dnl
m4_define([PKG_NOARCH_INSTALLDIR], [noarch_pkgconfigdir=${datadir}/pkgconfig
printf '%s\n' "s|@noarch_pkgconfigdir@|$noarch_pkgconfigdir|g" >> conf_subst.sed 2>/dev/null])dnl
"#
}
