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
