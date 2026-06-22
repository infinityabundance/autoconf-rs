//! autoconf-rs — forensic-parity GNU Autoconf reimplementation.
//! Clean-room behavioral reconstruction. No GPL code included.

fn main() {
    println!("autoconf-rs {}", env!("CARGO_PKG_VERSION"));
    println!("Forensic-parity GNU Autoconf reimplementation");
    println!("Clean-room behavioral reconstruction — no GPL code");
    println!();
    println!("Components:");
    println!("  autoconf   — Generate configure scripts from configure.ac");
    println!("  autoheader — Generate config.h.in from configure.ac");
    println!("  autom4te   — Caching M4 wrapper");
    println!("  autoreconf — Orchestrate autotools chain");
    println!("  aclocal    — Generate aclocal.m4");
    println!("  autoscan   — Scan sources for configure.ac hints");
    println!("  autoupdate — Update outdated Autoconf macros");
    println!("  ifnames    — Extract #if names from C sources");
    println!();
    println!("Usage: cargo run --bin <component> [options]");
    println!();
    println!("IMPORTANT: autoconf-rs is NOT a GNU Autoconf replacement.");
    println!("See STATUS.md for current claim status.");
    println!("See docs/negative-capabilities.md for explicit non-claims.");
}
