//! Input validation and security sandbox — NC.PERM.2 resolution.
//!
//! Provides configure.ac path validation, macro name sanitization,
//! and basic input size limits to prevent denial-of-service and
//! path traversal attacks through malicious configure.ac files.
//!
//! Court: NC.PERM.2 RESOLUTION

use std::path::Path;

/// Maximum allowed configure.ac file size (100 MB).
pub const MAX_CONFIGURE_AC_SIZE: usize = 100 * 1024 * 1024;

/// Maximum allowed macro nesting depth.
pub const MAX_MACRO_NESTING: usize = 100;

/// Validate a configure.ac input for basic safety.
/// Returns Ok(()) if safe, Err(message) if rejected.
pub fn validate_configure_ac(input: &str) -> Result<(), String> {
    // Size check
    if input.len() > MAX_CONFIGURE_AC_SIZE {
        return Err(format!(
            "configure.ac exceeds maximum size ({} bytes > {} max)",
            input.len(),
            MAX_CONFIGURE_AC_SIZE
        ));
    }

    // NUL byte check
    if input.contains('\0') {
        return Err("configure.ac contains NUL bytes (not allowed)".into());
    }

    // Line length sanity check (no 1MB single lines)
    for (i, line) in input.lines().enumerate() {
        if line.len() > 1_000_000 {
            return Err(format!(
                "configure.ac line {} exceeds maximum line length ({} bytes)",
                i + 1,
                line.len()
            ));
        }
    }

    Ok(())
}

/// Validate a file path for safe access (no directory traversal).
pub fn validate_path(path: &str) -> Result<(), String> {
    let p = Path::new(path);

    // Reject empty paths
    if path.is_empty() {
        return Err("empty path not allowed".into());
    }

    // Reject absolute paths that escape the project directory
    if p.is_absolute() {
        return Err(format!("absolute path not allowed: {}", path));
    }

    // Check for path traversal components
    for component in p.components() {
        use std::path::Component;
        match component {
            Component::ParentDir => {
                return Err(format!("path traversal detected: {}", path));
            }
            Component::RootDir => {
                return Err(format!("root path not allowed: {}", path));
            }
            _ => {}
        }
    }

    Ok(())
}

/// Validate a macro name (alphanumeric + underscore only).
pub fn validate_macro_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("empty macro name".into());
    }

    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!("macro name contains invalid characters: {}", name));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_normal_input() {
        assert!(validate_configure_ac("AC_INIT([t],[1.0])\nAC_OUTPUT\n").is_ok());
    }

    #[test]
    fn test_validate_empty_input() {
        assert!(validate_configure_ac("").is_ok());
    }

    #[test]
    fn test_validate_nul_byte() {
        assert!(validate_configure_ac("AC_INIT\0([t])").is_err());
    }

    #[test]
    fn test_validate_path_normal() {
        assert!(validate_path("src/main.c").is_ok());
    }

    #[test]
    fn test_validate_path_traversal() {
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("foo/../../bar").is_err());
    }

    #[test]
    fn test_validate_path_absolute() {
        assert!(validate_path("/etc/passwd").is_err());
    }

    #[test]
    fn test_validate_path_empty() {
        assert!(validate_path("").is_err());
    }

    #[test]
    fn test_validate_macro_name_valid() {
        assert!(validate_macro_name("AC_INIT").is_ok());
        assert!(validate_macro_name("HAVE_FOO").is_ok());
        assert!(validate_macro_name("my_macro_1").is_ok());
    }

    #[test]
    fn test_validate_macro_name_invalid() {
        assert!(validate_macro_name("").is_err());
        assert!(validate_macro_name("bad name").is_err());
        assert!(validate_macro_name("bad;name").is_err());
    }

    #[test]
    fn test_validate_path_subdir() {
        assert!(validate_path("subdir/file.m4").is_ok());
        assert!(validate_path("./configure").is_ok());
    }
}
