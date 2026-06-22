//! Diagnostics system matching Autoconf warning/error taxonomy.
//!
//! Implements the full GNU Autoconf diagnostics model:
//! - -W categories: cross, gnu, obsolete, override, portability, syntax,
//!   unsupported, all, error, no-CATEGORY
//! - Source location tracking (file:line) across include files
//! - AC_DIAGNOSE, AC_WARNING, AC_FATAL macros
//! - AU_DEFUN deprecation warnings
//! - AC_OBSOLETE support
//! - Exit code mapping: 0=success, 1=warnings issued, 2=errors/fatal
//!
//! Receipt family: AC.DIAG.*
//! Court: AC.DIAG.1 — sealed (full diagnostics taxonomy)
//! Status: Phase 5 — complete with location tracking, categories, exit codes.

use std::collections::HashSet;

/// Warning categories matching GNU Autoconf's -W flags.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WarningCategory {
    Cross,
    Gnu,
    Obsolete,
    Override,
    Portability,
    Syntax,
    Unsupported,
    All,
    Error,
    NoCategory(String),
}

impl WarningCategory {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cross" => Some(Self::Cross),
            "gnu" => Some(Self::Gnu),
            "obsolete" => Some(Self::Obsolete),
            "override" => Some(Self::Override),
            "portability" => Some(Self::Portability),
            "syntax" => Some(Self::Syntax),
            "unsupported" => Some(Self::Unsupported),
            "all" => Some(Self::All),
            "error" => Some(Self::Error),
            other => other
                .strip_prefix("no-")
                .map(|cat| Self::NoCategory(cat.to_string())),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Cross => "cross",
            Self::Gnu => "gnu",
            Self::Obsolete => "obsolete",
            Self::Override => "override",
            Self::Portability => "portability",
            Self::Syntax => "syntax",
            Self::Unsupported => "unsupported",
            Self::All => "all",
            Self::Error => "error",
            Self::NoCategory(c) => c, // Returns the category without "no-"
        }
    }
}

/// Diagnostic severity levels matching Autoconf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// AC_DIAGNOSE / AC_WARNING level (non-fatal)
    Warning,
    /// AC_MSG_ERROR level (fatal, exits configure)
    Error,
    /// AC_FATAL level (immediate abort)
    Fatal,
}

/// Source location for diagnostics.
#[derive(Debug, Clone, Default)]
pub struct SourceLocation {
    pub file: Option<String>,
    pub line: Option<usize>,
    /// Include stack for tracking __file__/__line__ across M4 includes
    pub include_stack: Vec<(String, usize)>,
}

impl SourceLocation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn at(file: &str, line: usize) -> Self {
        Self {
            file: Some(file.to_string()),
            line: Some(line),
            include_stack: Vec::new(),
        }
    }
}

/// A diagnostic message emitted during Autoconf processing.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub category: Option<WarningCategory>,
    pub message: String,
    pub location: SourceLocation,
}

/// Full diagnostics manager with category filtering and exit code tracking.
#[derive(Debug)]
pub struct DiagnosticManager {
    /// Active warning categories (from -W flags)
    active_categories: HashSet<WarningCategory>,
    /// Suppressed categories (from -W no-CATEGORY)
    suppressed_categories: HashSet<String>,
    /// Error-as-warning flag (-W error makes warnings into errors)
    errors_from_warnings: bool,
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Counts by level
    warning_count: usize,
    error_count: usize,
    fatal_count: usize,
    /// Current source location for tracking
    current_location: SourceLocation,
}

impl DiagnosticManager {
    /// Create a new diagnostic manager with default settings.
    /// By default: gnu, portability, and syntax warnings are enabled.
    pub fn new() -> Self {
        let mut active = HashSet::new();
        active.insert(WarningCategory::Gnu);
        active.insert(WarningCategory::Portability);
        active.insert(WarningCategory::Syntax);

        Self {
            active_categories: active,
            suppressed_categories: HashSet::new(),
            errors_from_warnings: false,
            diagnostics: Vec::new(),
            warning_count: 0,
            error_count: 0,
            fatal_count: 0,
            current_location: SourceLocation::new(),
        }
    }

    /// Enable a warning category (from -W CATEGORY).
    pub fn enable_category(&mut self, category: &str) {
        if let Some(cat) = category.strip_prefix("no-") {
            self.suppressed_categories.insert(cat.to_string());
            if cat == "all" {
                self.active_categories.clear();
            }
        } else if let Some(c) = WarningCategory::parse(category) {
            // Enabling a specific category removes it from suppression and
            // clears the "all" suppression if it was previously set
            let name = c.as_str().to_string();
            self.suppressed_categories.remove(&name);
            self.suppressed_categories.remove("all");
            match &c {
                WarningCategory::All => {
                    self.active_categories.insert(WarningCategory::Cross);
                    self.active_categories.insert(WarningCategory::Gnu);
                    self.active_categories.insert(WarningCategory::Obsolete);
                    self.active_categories.insert(WarningCategory::Override);
                    self.active_categories.insert(WarningCategory::Portability);
                    self.active_categories.insert(WarningCategory::Syntax);
                    self.active_categories.insert(WarningCategory::Unsupported);
                    self.suppressed_categories.remove("all");
                }
                WarningCategory::Error => {
                    self.errors_from_warnings = true;
                }
                _ => {
                    self.active_categories.insert(c);
                }
            }
        }
    }

    /// Check if a diagnostic category is currently active.
    fn is_active(&self, cat: Option<&WarningCategory>) -> bool {
        // If all categories are suppressed, block everything including uncategorized
        if self.suppressed_categories.contains("all") {
            return false;
        }
        match cat {
            Some(WarningCategory::Error) => true, // errors always active
            Some(c) => {
                let name = c.as_str();
                if self.suppressed_categories.contains(name) {
                    return false;
                }
                self.active_categories.contains(c)
                    || self.active_categories.contains(&WarningCategory::All)
            }
            None => true, // uncategorized always active (unless -Wno-all)
        }
    }

    /// Set current source location (called when entering a file or macro).
    pub fn set_location(&mut self, file: &str, line: usize) {
        self.current_location.file = Some(file.to_string());
        self.current_location.line = Some(line);
    }

    /// Push a file onto the include stack (for __file__/__line__ tracking).
    pub fn push_include(&mut self, file: &str, line: usize) {
        self.current_location.include_stack.push((
            self.current_location.file.clone().unwrap_or_default(),
            self.current_location.line.unwrap_or(0),
        ));
        self.current_location.file = Some(file.to_string());
        self.current_location.line = Some(line);
    }

    /// Pop from the include stack.
    pub fn pop_include(&mut self) {
        if let Some((file, line)) = self.current_location.include_stack.pop() {
            self.current_location.file = Some(file);
            self.current_location.line = Some(line);
        }
    }

    /// Emit a diagnostic. Returns true if the diagnostic was accepted
    /// (not suppressed by category filtering).
    pub fn emit(
        &mut self,
        level: DiagnosticLevel,
        category: Option<WarningCategory>,
        message: &str,
    ) -> bool {
        if !self.is_active(category.as_ref()) {
            return false;
        }

        // If -W error is set, upgrade warnings to errors
        let effective_level = if self.errors_from_warnings && level == DiagnosticLevel::Warning {
            DiagnosticLevel::Error
        } else {
            level
        };

        match effective_level {
            DiagnosticLevel::Warning => self.warning_count += 1,
            DiagnosticLevel::Error => self.error_count += 1,
            DiagnosticLevel::Fatal => self.fatal_count += 1,
        }

        let diag = Diagnostic {
            level: effective_level,
            category,
            message: message.to_string(),
            location: self.current_location.clone(),
        };

        // Format and print to stderr
        eprintln!("{}", diag.format());
        self.diagnostics.push(diag);
        true
    }

    /// AC_DIAGNOSE(CATEGORY, MESSAGE) — emit a categorized warning.
    pub fn ac_diagnose(&mut self, category: &str, message: &str) -> bool {
        let cat = WarningCategory::parse(category);
        self.emit(DiagnosticLevel::Warning, cat, message)
    }

    /// AC_WARNING(MESSAGE) — emit an uncategorized warning.
    pub fn ac_warning(&mut self, message: &str) -> bool {
        self.emit(DiagnosticLevel::Warning, None, message)
    }

    /// AC_FATAL(MESSAGE) — emit a fatal error.
    pub fn ac_fatal(&mut self, message: &str) -> ! {
        self.emit(DiagnosticLevel::Fatal, None, message);
        // In Autoconf, AC_FATAL aborts immediately
        eprintln!("autoconf: fatal error, aborting");
        std::process::exit(2);
    }

    /// AC_OBSOLETE(MACRO, REPLACEMENT) — emit deprecation warning.
    pub fn ac_obsolete(&mut self, macro_name: &str, replacement: &str) -> bool {
        let msg = if replacement.is_empty() {
            format!("The macro `{}` is obsolete.", macro_name)
        } else {
            format!(
                "The macro `{}` is obsolete; use `{}` instead.",
                macro_name, replacement
            )
        };
        self.emit(
            DiagnosticLevel::Warning,
            Some(WarningCategory::Obsolete),
            &msg,
        )
    }

    /// AU_DEFUN deprecation warning — emit when a deprecated macro is used.
    pub fn au_defun_warning(&mut self, old_name: &str, new_name: Option<&str>) -> bool {
        let msg = if let Some(new) = new_name {
            format!(
                "The macro `{}` is obsolete; use `{}` instead.",
                old_name, new
            )
        } else {
            format!("The macro `{}` is obsolete.", old_name)
        };
        self.emit(
            DiagnosticLevel::Warning,
            Some(WarningCategory::Obsolete),
            &msg,
        )
    }

    /// Get the exit code based on diagnostics emitted.
    /// 0 = no diagnostics, 1 = warnings only, 2 = errors or fatals.
    pub fn exit_code(&self) -> i32 {
        if self.fatal_count > 0 || self.error_count > 0 {
            2
        } else if self.warning_count > 0 {
            1
        } else {
            0
        }
    }

    /// Get total warning count.
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    /// Get total error count.
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Get all collected diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Format a summary line.
    pub fn summary(&self) -> String {
        format!(
            "{} warning(s), {} error(s), {} fatal(s)",
            self.warning_count, self.error_count, self.fatal_count
        )
    }
}

impl Diagnostic {
    /// Create a diagnostic at the current source location.
    pub fn at_location(
        level: DiagnosticLevel,
        category: Option<WarningCategory>,
        message: &str,
        location: &SourceLocation,
    ) -> Self {
        Self {
            level,
            category,
            message: message.to_string(),
            location: location.clone(),
        }
    }

    /// Format the diagnostic for display.
    pub fn format(&self) -> String {
        let prefix = match self.level {
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Fatal => "fatal error",
        };

        let category_str = self
            .category
            .as_ref()
            .map(|c| format!(" [{}]", c.as_str()))
            .unwrap_or_default();

        match (&self.location.file, &self.location.line) {
            (Some(f), Some(l)) => {
                format!("{}:{}: {}{}: {}", f, l, prefix, category_str, self.message)
            }
            (Some(f), None) => format!("{}: {}{}: {}", f, prefix, category_str, self.message),
            _ => format!("autoconf: {}{}: {}", prefix, category_str, self.message),
        }
    }
}

impl Default for DiagnosticManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warning_categories() {
        assert_eq!(
            WarningCategory::parse("cross"),
            Some(WarningCategory::Cross)
        );
        assert_eq!(
            WarningCategory::parse("obsolete"),
            Some(WarningCategory::Obsolete)
        );
        assert_eq!(WarningCategory::parse("all"), Some(WarningCategory::All));
        assert_eq!(
            WarningCategory::parse("no-portability"),
            Some(WarningCategory::NoCategory("portability".to_string()))
        );
        assert_eq!(WarningCategory::parse("invalid"), None);
    }

    #[test]
    fn test_diagnostic_manager_basic() {
        let mut dm = DiagnosticManager::new();
        assert!(dm.ac_warning("test warning"));
        assert_eq!(dm.warning_count(), 1);
        assert_eq!(dm.error_count(), 0);
        assert_eq!(dm.exit_code(), 1);
    }

    #[test]
    fn test_category_filtering() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("no-all"); // Suppress all

        // GNU warnings should be suppressed
        let emitted = dm.ac_diagnose("gnu", "should be suppressed");
        assert!(!emitted);
        assert_eq!(dm.warning_count(), 0);
    }

    #[test]
    fn test_obsolete_warning() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("obsolete");
        dm.set_location("configure.ac", 42);

        assert!(dm.ac_obsolete("AC_OLD_MACRO", "AC_NEW_MACRO"));
        assert_eq!(dm.warning_count(), 1);
        assert!(dm.diagnostics()[0].message.contains("AC_NEW_MACRO"));
    }

    #[test]
    fn test_exit_codes() {
        let mut dm = DiagnosticManager::new();
        assert_eq!(dm.exit_code(), 0);

        dm.ac_warning("warn");
        assert_eq!(dm.exit_code(), 1);

        let mut dm2 = DiagnosticManager::new();
        dm2.emit(DiagnosticLevel::Error, None, "err");
        assert_eq!(dm2.exit_code(), 2);
    }

    #[test]
    fn test_source_location_tracking() {
        let mut dm = DiagnosticManager::new();
        dm.set_location("configure.ac", 100);
        dm.ac_warning("test");

        let diag = &dm.diagnostics()[0];
        assert_eq!(diag.location.file.as_deref(), Some("configure.ac"));
        assert_eq!(diag.location.line, Some(100));
    }

    #[test]
    fn test_include_stack() {
        let mut dm = DiagnosticManager::new();
        dm.set_location("configure.ac", 10);
        dm.push_include("m4/macros.m4", 5);
        dm.ac_warning("from include");

        let diag = &dm.diagnostics()[0];
        assert_eq!(diag.location.file.as_deref(), Some("m4/macros.m4"));
        assert_eq!(diag.location.line, Some(5));

        dm.pop_include();
        dm.ac_warning("back to main");
        let diag2 = &dm.diagnostics()[1];
        assert_eq!(diag2.location.file.as_deref(), Some("configure.ac"));
    }

    #[test]
    fn test_error_as_warning() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("error"); // -W error makes warnings errors

        dm.ac_warning("upgraded to error");
        assert_eq!(dm.error_count(), 1);
        assert_eq!(dm.warning_count(), 0);
        assert_eq!(dm.exit_code(), 2);
    }

    #[test]
    fn test_full_taxonomy() {
        let mut dm = DiagnosticManager::new();
        dm.enable_category("all");

        // Emit one of each category
        dm.ac_diagnose("cross", "cross compilation");
        dm.ac_diagnose("gnu", "GNU extension");
        dm.ac_diagnose("obsolete", "obsolete macro");
        dm.ac_diagnose("override", "overridden variable");
        dm.ac_diagnose("portability", "non-portable construct");
        dm.ac_diagnose("syntax", "syntax issue");
        dm.ac_diagnose("unsupported", "unsupported feature");

        assert_eq!(dm.warning_count(), 7);
        println!("Diagnostics taxonomy complete: {}", dm.summary());
    }
}
