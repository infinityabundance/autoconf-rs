//! autoconf-rs-cli shared library.
//!
//! Provides the CLI argument parsing harness used by all 8 Autoconf binaries.
//! Each binary is a thin wrapper that delegates to autoconf-rs-core.

use std::io::Read;
use std::path::PathBuf;

/// Common result type for CLI operations.
pub type CliResult = Result<(), String>;

/// Read all of stdin into a String.
pub fn read_stdin() -> Result<String, String> {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| format!("error reading stdin: {}", e))?;
    Ok(buffer)
}

/// Read a file into a String, or return stdin contents if path is "-".
pub fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        read_stdin()
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("error reading {}: {}", path, e))
    }
}

/// Common CLI options shared across Autoconf tools.
#[derive(Debug, Default)]
pub struct CommonOpts {
    pub include_dirs: Vec<PathBuf>,
    pub prepend_include_dirs: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub force: bool,
    pub debug: bool,
    pub verbose: bool,
    pub output_file: Option<PathBuf>,
}
