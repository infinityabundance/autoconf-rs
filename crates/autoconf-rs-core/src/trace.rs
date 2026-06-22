//! Autoconf Trace Event System
//!
//! Trace events are the semantic spine of autoconf-rs. Instead of prescan
//! (string-matching AC_* in raw configure.ac text), macro expansion emits
//! structured trace events. The configure generator, autoheader, and autom4te
//! consume these events — never raw text.
//!
//! This is the architectural shift the panel mandated:
//!   configure.ac → M4 expansion → trace events → configure IR → output
//!
//! Court: AC.TRACE.1
//! Panel directive: "Make M4 expansion the source of truth."

use std::fmt;

/// A span in the source configure.ac (or included file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(file: &str, line: usize, column: usize) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Actions to take when a check succeeds or fails.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Actions {
    pub if_true: Vec<String>,
    pub if_false: Vec<String>,
}

/// An Autoconf trace event emitted during M4 expansion.
///
/// This is the panel's exact specification (not compilable doc-test):
///
/// ```text
/// enum AutoconfEvent {
///     Init { package: String, version: String },
///     Subst { name: String, value: Option<String>, origin: Span },
///     Define { name: String, value: Option<String>, description: Option<String>, origin: Span },
///     ...
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoconfEvent {
    /// AC_INIT was expanded
    Init {
        package: String,
        version: String,
        bug_report: Option<String>,
        tarname: Option<String>,
        origin: Span,
    },
    /// AC_SUBST was expanded
    Subst {
        name: String,
        value: Option<String>,
        origin: Span,
    },
    /// AC_DEFINE was expanded
    Define {
        name: String,
        value: Option<String>,
        description: Option<String>,
        origin: Span,
    },
    /// AC_CONFIG_FILES was expanded
    ConfigFile {
        output: String,
        inputs: Vec<String>,
        origin: Span,
    },
    /// AC_CONFIG_HEADERS was expanded
    ConfigHeader {
        output: String,
        templates: Vec<String>,
        origin: Span,
    },
    /// AC_CONFIG_COMMANDS was expanded
    ConfigCommand {
        tag: String,
        commands: String,
        origin: Span,
    },
    /// AC_CONFIG_LINKS was expanded
    ConfigLink {
        dest: String,
        source: String,
        origin: Span,
    },
    /// AC_CONFIG_SUBDIRS was expanded
    ConfigSubdir { directory: String, origin: Span },
    /// AC_CHECK_HEADER was expanded
    CheckHeader {
        header: String,
        actions: Actions,
        origin: Span,
    },
    /// AC_CHECK_FUNC was expanded
    CheckFunc {
        function: String,
        actions: Actions,
        origin: Span,
    },
    /// AC_CHECK_LIB was expanded
    CheckLib {
        library: String,
        function: String,
        actions: Actions,
        origin: Span,
    },
    /// AC_CHECK_TYPE was expanded
    CheckType {
        type_name: String,
        actions: Actions,
        origin: Span,
    },
    /// AC_CHECK_PROG was expanded
    CheckProg {
        variable: String,
        programs: Vec<String>,
        actions: Actions,
        origin: Span,
    },
    /// AC_MSG_CHECKING / AC_MSG_RESULT / AC_MSG_WARN / AC_MSG_ERROR
    Message {
        kind: MessageKind,
        text: String,
        origin: Span,
    },
    /// AC_REQUIRE was triggered
    Require { macro_name: String, origin: Span },
    /// AC_PROVIDE marks a macro as satisfied
    Provide { macro_name: String, origin: Span },
    /// AC_CANONICAL_HOST/BUILD/TARGET
    Canonical { kind: CanonicalKind, origin: Span },
    /// AC_ARG_ENABLE / AC_ARG_WITH / AC_ARG_VAR
    Argument {
        kind: ArgumentKind,
        name: String,
        help: Option<String>,
        origin: Span,
    },
    /// AC_LANG_PUSH / AC_LANG_POP
    Language {
        kind: LanguageAction,
        language: String,
        origin: Span,
    },
    /// AC_PREREQ check
    Prereq { version: String, origin: Span },
    /// Generic trace for any other macro (autom4te --trace compatibility)
    Trace {
        macro_name: String,
        args: Vec<String>,
        file: String,
        line: usize,
    },
    /// Diagnostic emission (AC_DIAGNOSE, AC_WARNING, AC_FATAL)
    Diagnostic {
        category: String,
        message: String,
        origin: Span,
    },
    /// AC_OUTPUT was reached — finalize
    Output { origin: Span },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageKind {
    Checking,
    Result,
    Warn,
    Error,
    Notice,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalKind {
    Host,
    Build,
    Target,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentKind {
    With,
    Enable,
    Var,
    Program,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageAction {
    Push,
    Pop,
}

/// A trace event log — the output of M4 expansion consumed by downstream tools.
#[derive(Debug, Clone, Default)]
pub struct TraceLog {
    pub events: Vec<AutoconfEvent>,
}

impl TraceLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: AutoconfEvent) {
        self.events.push(event);
    }

    /// Emit autom4te-compatible trace lines for --trace output.
    /// GNU format: `m4trace:file:line: -1- MACRO(args)`
    pub fn emit_autom4te_traces(&self) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|e| {
                let (macro_name, args, file, line) = match e {
                    AutoconfEvent::Init {
                        package,
                        version,
                        bug_report,
                        tarname,
                        origin,
                    } => {
                        let _ = (bug_report, tarname); // trace records package:version only
                        let a = vec![format!("[{}]", package), format!("[{}]", version)];
                        ("AC_INIT", a, origin.file.as_str(), origin.line)
                    }
                    AutoconfEvent::Subst {
                        name,
                        value,
                        origin,
                    } => {
                        let mut a = vec![format!("[{}]", name)];
                        if let Some(v) = value {
                            a.push(format!("[{}]", v));
                        }
                        ("AC_SUBST", a, origin.file.as_str(), origin.line)
                    }
                    AutoconfEvent::Define {
                        name,
                        value,
                        description,
                        origin,
                    } => {
                        let mut a = vec![format!("[{}]", name)];
                        if let Some(v) = value {
                            a.push(format!("[{}]", v));
                        }
                        if let Some(d) = description {
                            a.push(format!("[{}]", d));
                        }
                        ("AC_DEFINE", a, origin.file.as_str(), origin.line)
                    }
                    AutoconfEvent::ConfigFile { output, origin, .. } => (
                        "AC_CONFIG_FILES",
                        vec![format!("[{}]", output)],
                        origin.file.as_str(),
                        origin.line,
                    ),
                    AutoconfEvent::ConfigHeader { output, origin, .. } => (
                        "AC_CONFIG_HEADERS",
                        vec![format!("[{}]", output)],
                        origin.file.as_str(),
                        origin.line,
                    ),
                    AutoconfEvent::CheckFunc {
                        function, origin, ..
                    } => (
                        "AC_CHECK_FUNC",
                        vec![format!("[{}]", function)],
                        origin.file.as_str(),
                        origin.line,
                    ),
                    AutoconfEvent::CheckHeader { header, origin, .. } => (
                        "AC_CHECK_HEADER",
                        vec![format!("[{}]", header)],
                        origin.file.as_str(),
                        origin.line,
                    ),
                    AutoconfEvent::CheckLib {
                        library,
                        function,
                        origin,
                        ..
                    } => (
                        "AC_CHECK_LIB",
                        vec![format!("[{}]", library), format!("[{}]", function)],
                        origin.file.as_str(),
                        origin.line,
                    ),
                    AutoconfEvent::CheckType {
                        type_name, origin, ..
                    } => (
                        "AC_CHECK_TYPE",
                        vec![format!("[{}]", type_name)],
                        origin.file.as_str(),
                        origin.line,
                    ),
                    AutoconfEvent::Trace {
                        macro_name,
                        args,
                        file,
                        line,
                    } => (
                        macro_name.as_str(),
                        args.iter().map(|a| format!("[{}]", a)).collect(),
                        file.as_str(),
                        *line,
                    ),
                    AutoconfEvent::Message { kind, text, origin } => {
                        let name = match kind {
                            MessageKind::Checking => "AC_MSG_CHECKING",
                            MessageKind::Result => "AC_MSG_RESULT",
                            MessageKind::Warn => "AC_MSG_WARN",
                            MessageKind::Error => "AC_MSG_ERROR",
                            MessageKind::Notice => "AC_MSG_NOTICE",
                            MessageKind::Failure => "AC_MSG_FAILURE",
                        };
                        (
                            name,
                            vec![format!("[{}]", text)],
                            origin.file.as_str(),
                            origin.line,
                        )
                    }
                    AutoconfEvent::Require { macro_name, origin } => (
                        "AC_REQUIRE",
                        vec![format!("[{}]", macro_name)],
                        origin.file.as_str(),
                        origin.line,
                    ),
                    AutoconfEvent::Canonical { kind, origin } => {
                        let name = match kind {
                            CanonicalKind::Host => "AC_CANONICAL_HOST",
                            CanonicalKind::Build => "AC_CANONICAL_BUILD",
                            CanonicalKind::Target => "AC_CANONICAL_TARGET",
                        };
                        (name, vec![], origin.file.as_str(), origin.line)
                    }
                    AutoconfEvent::Argument {
                        kind, name, origin, ..
                    } => {
                        let macro_name = match kind {
                            ArgumentKind::With => "AC_ARG_WITH",
                            ArgumentKind::Enable => "AC_ARG_ENABLE",
                            ArgumentKind::Var => "AC_ARG_VAR",
                            ArgumentKind::Program => "AC_ARG_PROGRAM",
                        };
                        (
                            macro_name,
                            vec![format!("[{}]", name)],
                            origin.file.as_str(),
                            origin.line,
                        )
                    }
                    AutoconfEvent::Prereq { version, origin } => (
                        "AC_PREREQ",
                        vec![format!("[{}]", version)],
                        origin.file.as_str(),
                        origin.line,
                    ),
                    _ => return None,
                };
                // Colon-delimited trace: file:line:MACRO:arg:arg... (args carry no surrounding [ ]).
                let stripped: Vec<String> = args
                    .iter()
                    .map(|a| a.trim_start_matches('[').trim_end_matches(']').to_string())
                    .collect();
                let args_str = if stripped.is_empty() {
                    String::new()
                } else {
                    format!(":{}", stripped.join(":"))
                };
                Some(format!("{}:{}:{}{}", file, line, macro_name, args_str))
            })
            .collect()
    }

    /// Get structured trace data for format-string rendering.
    /// Returns (file, line, macro_name, args_vec) tuples.
    pub fn structured_traces(&self) -> Vec<(String, usize, String, Vec<String>)> {
        self.events
            .iter()
            .filter_map(|e| {
                let (macro_name, args, origin) = match e {
                    AutoconfEvent::Init {
                        package,
                        version,
                        bug_report,
                        tarname,
                        origin,
                    } => {
                        let mut a = vec![package.clone(), version.clone()];
                        if let Some(b) = bug_report {
                            a.push(b.clone());
                        }
                        if let Some(t) = tarname {
                            a.push(t.clone());
                        }
                        ("AC_INIT".to_string(), a, origin.clone())
                    }
                    AutoconfEvent::Subst {
                        name,
                        value,
                        origin,
                    } => {
                        let mut a = vec![name.clone()];
                        if let Some(v) = value {
                            a.push(v.clone());
                        }
                        ("AC_SUBST".to_string(), a, origin.clone())
                    }
                    AutoconfEvent::Define {
                        name,
                        value,
                        description,
                        origin,
                    } => {
                        let mut a = vec![name.clone()];
                        if let Some(v) = value {
                            a.push(v.clone());
                        }
                        if let Some(d) = description {
                            a.push(d.clone());
                        }
                        ("AC_DEFINE".to_string(), a, origin.clone())
                    }
                    AutoconfEvent::ConfigFile {
                        output,
                        inputs: _,
                        origin,
                    } => (
                        "AC_CONFIG_FILES".to_string(),
                        vec![output.clone()],
                        origin.clone(),
                    ),
                    AutoconfEvent::ConfigHeader { output, origin, .. } => (
                        "AC_CONFIG_HEADERS".to_string(),
                        vec![output.clone()],
                        origin.clone(),
                    ),
                    AutoconfEvent::CheckFunc {
                        function, origin, ..
                    } => (
                        "AC_CHECK_FUNC".to_string(),
                        vec![function.clone()],
                        origin.clone(),
                    ),
                    AutoconfEvent::CheckHeader { header, origin, .. } => (
                        "AC_CHECK_HEADER".to_string(),
                        vec![header.clone()],
                        origin.clone(),
                    ),
                    AutoconfEvent::CheckLib {
                        library,
                        function,
                        origin,
                        ..
                    } => (
                        "AC_CHECK_LIB".to_string(),
                        vec![library.clone(), function.clone()],
                        origin.clone(),
                    ),
                    AutoconfEvent::CheckType {
                        type_name, origin, ..
                    } => (
                        "AC_CHECK_TYPE".to_string(),
                        vec![type_name.clone()],
                        origin.clone(),
                    ),
                    AutoconfEvent::Trace {
                        macro_name,
                        args,
                        file,
                        line,
                    } => (macro_name.clone(), args.clone(), Span::new(file, *line, 1)),
                    AutoconfEvent::Require { macro_name, origin } => (
                        "AC_REQUIRE".to_string(),
                        vec![macro_name.clone()],
                        origin.clone(),
                    ),
                    AutoconfEvent::Provide { macro_name, origin } => (
                        "AC_PROVIDE".to_string(),
                        vec![macro_name.clone()],
                        origin.clone(),
                    ),
                    AutoconfEvent::Canonical { kind, origin } => {
                        let name = match kind {
                            CanonicalKind::Host => "AC_CANONICAL_HOST",
                            CanonicalKind::Build => "AC_CANONICAL_BUILD",
                            CanonicalKind::Target => "AC_CANONICAL_TARGET",
                        };
                        (name.to_string(), vec![], origin.clone())
                    }
                    AutoconfEvent::Prereq { version, origin } => (
                        "AC_PREREQ".to_string(),
                        vec![version.clone()],
                        origin.clone(),
                    ),
                    _ => return None,
                };
                Some((origin.file, origin.line, macro_name, args))
            })
            .collect()
    }

    /// Format a single trace using a GNU-style format string.
    /// Format specifiers: $f (file), $l (line), $n (macro name), $1..$N (args)
    pub fn format_trace(
        file: &str,
        line: usize,
        macro_name: &str,
        args: &[String],
        format: &str,
    ) -> String {
        let mut result = format.to_string();
        result = result.replace("$f", file);
        result = result.replace("$l", &line.to_string());
        result = result.replace("$n", macro_name);
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("${}", i + 1), arg);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_log_basic() {
        let mut log = TraceLog::new();
        log.push(AutoconfEvent::Init {
            package: "test".into(),
            version: "1.0".into(),
            bug_report: None,
            tarname: None,
            origin: Span::new("configure.ac", 1, 1),
        });
        log.push(AutoconfEvent::Subst {
            name: "CC".into(),
            value: Some("gcc".into()),
            origin: Span::new("configure.ac", 5, 1),
        });
        log.push(AutoconfEvent::Output {
            origin: Span::new("configure.ac", 10, 1),
        });

        assert_eq!(log.events.len(), 3);

        let traces = log.emit_autom4te_traces();
        assert_eq!(traces.len(), 2);
        assert!(traces[0].contains("AC_INIT"));
        assert!(traces[1].contains("AC_SUBST"));
    }
}
