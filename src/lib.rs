//! `autoconf-rs` — forensic-parity GNU Autoconf reimplementation.
//! The workspace crate: it ships the `autoconf-rs` CLI binary and re-exports
//! the member crates so `cargo install autoconf-rs` gives the tool and
//! `cargo add autoconf-rs` gives the engine.
pub use autoconf_casefile_rs;
pub use autoconf_oracle_rs;
pub use autoconf_rs_cli;
pub use autoconf_rs_core;
