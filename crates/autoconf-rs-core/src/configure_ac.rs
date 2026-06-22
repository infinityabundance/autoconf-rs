//! configure.ac parser and analyzer.
//!
//! Parses Autoconf input files (configure.ac / configure.in) to extract
//! structural information: AC_INIT, AC_OUTPUT, AC_CONFIG_FILES, etc.
//! This is the entry point for all Autoconf processing.
//!
//! Receipt family: AC.PARSE.*
//! Current status: Phase 1 — stub, not yet admitted.

/// Parsed representation of a configure.ac file.
pub struct ConfigureAc {
    /// Raw file contents
    pub raw: String,
    /// Package name from AC_INIT
    pub package_name: Option<String>,
    /// Package version from AC_INIT
    pub package_version: Option<String>,
    /// Bug report address from AC_INIT
    pub bug_report: Option<String>,
}

impl ConfigureAc {
    /// Parse a configure.ac file from its contents.
    pub fn parse(contents: &str) -> Self {
        let mut ac = Self {
            raw: contents.to_string(),
            package_name: None,
            package_version: None,
            bug_report: None,
        };

        // Phase 1 stub: simple regex-based extraction
        // Future: full M4-aware parsing through the M4 engine
        if let Some(init) = ac.extract_ac_init(contents) {
            ac.package_name = init.0;
            ac.package_version = init.1;
            ac.bug_report = init.2;
        }

        ac
    }

    /// Extract AC_INIT arguments using simple text matching (Phase 1 stub).
    fn extract_ac_init(
        &self,
        contents: &str,
    ) -> Option<(Option<String>, Option<String>, Option<String>)> {
        // Simple regex-free extraction for Phase 1
        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("AC_INIT(") || trimmed.starts_with("AC_INIT ") {
                // Extract bracketed arguments
                let args: Vec<String> = Self::extract_bracketed_args(trimmed);
                let name = args.first().cloned();
                let version = args.get(1).cloned();
                let bug = args.get(2).cloned();
                return Some((name, version, bug));
            }
        }
        None
    }

    /// Extract bracketed arguments from an Autoconf macro call (Phase 1 stub).
    fn extract_bracketed_args(s: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut depth = 0;
        let mut current = String::new();
        let mut in_bracket = false;

        for ch in s.chars() {
            match ch {
                '[' if !in_bracket => {
                    in_bracket = true;
                    depth = 1;
                    current.clear();
                }
                '[' if in_bracket => {
                    depth += 1;
                    current.push(ch);
                }
                ']' if in_bracket => {
                    depth -= 1;
                    if depth == 0 {
                        in_bracket = false;
                        args.push(current.clone());
                    } else {
                        current.push(ch);
                    }
                }
                _ if in_bracket => {
                    current.push(ch);
                }
                _ => {}
            }
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let contents = "AC_INIT([hello], [1.0])\nAC_OUTPUT\n";
        let ac = ConfigureAc::parse(contents);
        assert_eq!(ac.package_name, Some("hello".to_string()));
        assert_eq!(ac.package_version, Some("1.0".to_string()));
    }

    #[test]
    fn test_parse_with_bug_report() {
        let contents = "AC_INIT([GNU Hello], [2.12], [bug-hello@gnu.org])\nAC_OUTPUT\n";
        let ac = ConfigureAc::parse(contents);
        assert_eq!(ac.package_name, Some("GNU Hello".to_string()));
        assert_eq!(ac.package_version, Some("2.12".to_string()));
        assert_eq!(ac.bug_report, Some("bug-hello@gnu.org".to_string()));
    }

    #[test]
    fn test_extract_bracketed_args() {
        let args = ConfigureAc::extract_bracketed_args(
            "AC_INIT([hello], [1.0], [http://bugs.example.com])",
        );
        assert_eq!(args, vec!["hello", "1.0", "http://bugs.example.com"]);
    }
}
