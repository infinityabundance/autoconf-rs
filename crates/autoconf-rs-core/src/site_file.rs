//! Site file handling for config.site / site-lisp (stub).
//!
//! Current status: Phase 1 — stub.

#[derive(Default)]
pub struct SiteFile;

impl SiteFile {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_file_new() {
        let sf = SiteFile::new();
        // SiteFile is a stub — verify it constructs without panic
    }

    #[test]
    fn test_site_file_default() {
        let sf = SiteFile::default();
        // Default impl works
    }
}
