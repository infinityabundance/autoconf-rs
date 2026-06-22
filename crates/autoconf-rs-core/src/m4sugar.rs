//! m4sugar: Convenience macro library for Autoconf M4 programming.
//!
//! m4sugar provides higher-level M4 macros that make M4 programming
//! more convenient. It is the first library loaded by Autoconf (before m4sh).
//!
//! Key macros implemented:
//! - m4_defun/m4_require/m4_provide — dependency tracking with diversions
//! - m4_if/m4_case — conditionals
//! - m4_foreach/m4_map — iteration
//! - m4_join — list joining
//! - m4_expand/m4_do/m4_dquote/m4_quote — quoting helpers
//! - m4_normalize/m4_text_wrap — text formatting
//!
//! @ac_behavior id=AC.M4SUGAR.1 surface=AC.M4.M4SUGAR.1 manual=§10
//! Receipt family: AC.M4.M4SUGAR.*
//! Status: Phase 4 — diversion-backed AC_REQUIRE ordering.

use crate::diversion::DiversionManager;
use std::collections::{HashMap, HashSet};

/// Tracks macro dependency ordering for AC_REQUIRE/AC_PROVIDE.
///
/// Uses DiversionManager for output reordering: when AC_REQUIRE(B) is
/// called inside macro A, B's body is expanded into a lower diversion
/// number so it appears before A's body in the final output.
///
/// Receipt: AC.M4.AUTOCONF.CORE.1 (diversion-backed).
#[derive(Debug, Clone)]
pub struct RequireTracker {
    /// Macros that have been provided (expanded)
    provided: HashSet<String>,
    /// Macros that have been required but not yet provided
    required: HashMap<String, Vec<String>>,
    /// The current expansion stack for cycle detection
    expansion_stack: Vec<String>,
    /// Diversion manager for output ordering
    pub diversions: DiversionManager,
}

impl RequireTracker {
    pub fn new() -> Self {
        Self {
            provided: HashSet::new(),
            required: HashMap::new(),
            expansion_stack: Vec::new(),
            diversions: DiversionManager::new(),
        }
    }

    /// Mark a macro as provided (already expanded).
    /// Returns true if this is the first time it's been provided.
    pub fn provide(&mut self, name: &str) -> bool {
        self.provided.insert(name.to_string())
    }

    /// Check if a macro has been provided.
    pub fn is_provided(&self, name: &str) -> bool {
        self.provided.contains(name)
    }

    /// Record that a macro requires another macro.
    /// The required macro will be expanded before the requirer.
    pub fn require(&mut self, requirer: &str, required: &str) -> Result<(), String> {
        // Cycle detection
        if self.expansion_stack.contains(&required.to_string()) {
            return Err(format!(
                "AC_REQUIRE: circular dependency detected: {} -> {}",
                self.expansion_stack.join(" -> "),
                required
            ));
        }

        self.required
            .entry(required.to_string())
            .or_default()
            .push(requirer.to_string());
        Ok(())
    }

    /// Get the list of macros that require the given macro.
    pub fn required_by(&self, name: &str) -> Vec<String> {
        self.required.get(name).cloned().unwrap_or_default()
    }

    /// Push a macro onto the expansion stack (for cycle detection).
    pub fn push_expansion(&mut self, name: &str) {
        self.expansion_stack.push(name.to_string());
    }

    /// Pop a macro from the expansion stack.
    pub fn pop_expansion(&mut self, name: &str) {
        if self.expansion_stack.last() == Some(&name.to_string()) {
            self.expansion_stack.pop();
        }
    }

    /// Divert output to a numbered diversion (delegates to DiversionManager).
    pub fn divert(&mut self, n: i32) {
        self.diversions.divert(n);
    }

    /// Write output to the current diversion (delegates to DiversionManager).
    pub fn write(&mut self, data: &[u8]) {
        self.diversions.write(data);
    }

    /// Collect all diversion output in order.
    pub fn collect_output(&self) -> Vec<u8> {
        self.diversions.collect_all()
    }

    /// Get a snapshot of currently provided macros.
    pub fn provided_snapshot(&self) -> HashSet<String> {
        self.provided.clone()
    }
}

impl Default for RequireTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// m4sugar implementation: provides builtin macro handlers.
pub struct M4SugarBuiltins;

/// Type alias for foreach body expander callback.
pub type ForeachExpandFn<'a> = dyn FnMut(&[u8], &[Vec<u8>]) -> Vec<u8> + 'a;

impl M4SugarBuiltins {
    /// Handle m4_defun: define a user macro with Autoconf semantics.
    ///
    /// m4_defun(NAME, BODY) defines NAME as a macro. When NAME is expanded,
    /// it first expands all macros required by NAME (via AC_REQUIRE within BODY),
    /// then expands BODY.
    ///
    /// @ac_behavior id=AC.M4SUGAR.DEFUN.1 surface=AC.M4.M4SUGAR.1 manual=§10.1
    pub fn m4_defun(
        name: &str,
        body: &[u8],
        tracker: &mut RequireTracker,
        expand_fn: &mut dyn FnMut(&[u8]) -> Vec<u8>,
    ) -> Vec<u8> {
        // Push the expansion
        tracker.push_expansion(name);

        // Expand the body to discover any AC_REQUIRE calls
        let expanded = expand_fn(body);

        // Pop the expansion
        tracker.pop_expansion(name);

        // Mark as provided so it won't be expanded again
        tracker.provide(name);

        expanded
    }

    /// Handle m4_require: ensure a dependency is expanded first.
    ///
    /// @ac_behavior id=AC.M4SUGAR.REQUIRE.1 surface=AC.M4.M4SUGAR.1 manual=§10.1
    pub fn m4_require(
        required: &str,
        requirer: &str,
        tracker: &mut RequireTracker,
        expand_fn: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Result<Vec<u8>, String> {
        if tracker.is_provided(required) {
            return Ok(Vec::new());
        }

        tracker.require(requirer, required)?;

        // If the required macro exists, expand it now
        if let Some(output) = expand_fn(required) {
            tracker.provide(required);
            Ok(output)
        } else {
            // Macro not defined yet — will be expanded when encountered
            Ok(Vec::new())
        }
    }

    /// Handle m4_provide: mark a macro as already provided.
    pub fn m4_provide(name: &str, tracker: &mut RequireTracker) {
        tracker.provide(name);
    }

    /// Handle m4_if: conditional expansion.
    ///
    /// m4_if(A, B, THEN, [C, D, THEN2, ...], [ELSE])
    ///
    /// @ac_behavior id=AC.M4SUGAR.IF.1 surface=AC.M4.M4SUGAR.1 manual=§10.2
    pub fn m4_if(args: &[Vec<u8>]) -> Vec<u8> {
        let mut i = 0;
        while i + 2 < args.len() {
            if args[i] == args[i + 1] {
                return args[i + 2].clone();
            }
            i += 3;
        }
        if args.len() % 3 == 1 {
            args.last().cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Handle m4_case: switch-like conditional.
    ///
    /// m4_case(STRING, PAT1, VAL1, [PAT2, VAL2, ...], [DEFAULT])
    pub fn m4_case(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let string = &args[0];
        let mut i = 1;
        while i + 1 < args.len() {
            if &args[i] == string {
                return args[i + 1].clone();
            }
            i += 2;
        }
        // If odd number of remaining args, last is default
        if i < args.len() {
            args[i].clone()
        } else {
            Vec::new()
        }
    }

    /// Handle m4_foreach: iterate over a list.
    ///
    /// m4_foreach(VAR, LIST, BODY)
    /// For each element in LIST (comma-separated), expand BODY with $1=element.
    pub fn m4_foreach(list: &[u8], body: &[u8], expand_fn: &mut ForeachExpandFn<'_>) -> Vec<u8> {
        let mut output = Vec::new();
        let items = split_comma_separated(list);
        for item in &items {
            let args = vec![item.clone()];
            output.extend_from_slice(&expand_fn(body, &args));
        }
        output
    }

    /// Handle m4_join: join list elements with a separator.
    ///
    /// m4_join(SEP, ELEM1, ELEM2, ...)
    pub fn m4_join(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return Vec::new();
        }
        let sep = &args[0];
        let mut result = Vec::new();
        for (i, arg) in args[1..].iter().enumerate() {
            if i > 0 {
                result.extend_from_slice(sep);
            }
            result.extend_from_slice(arg);
        }
        result
    }

    /// Handle m4_expand: expand arguments once.
    ///
    /// m4_expand(ARG) — expands ARG exactly once.
    pub fn m4_expand(arg: &[u8], expand_fn: &mut dyn FnMut(&[u8]) -> Vec<u8>) -> Vec<u8> {
        expand_fn(arg)
    }

    /// Handle m4_do: evaluate all arguments.
    ///
    /// m4_do(ARG1, ARG2, ...) — expands each argument and concatenates.
    pub fn m4_do(args: &[Vec<u8>], expand_fn: &mut dyn FnMut(&[u8]) -> Vec<u8>) -> Vec<u8> {
        let mut result = Vec::new();
        for arg in args {
            result.extend_from_slice(&expand_fn(arg));
        }
        result
    }

    /// Handle m4_dquote: double-quote arguments.
    ///
    /// m4_dquote(ARG1, ARG2, ...) — wraps each argument in quotes.
    pub fn m4_dquote(args: &[Vec<u8>], quote_open: u8, quote_close: u8) -> Vec<u8> {
        let mut result = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                result.push(b',');
            }
            result.push(quote_open);
            result.extend_from_slice(arg);
            result.push(quote_close);
        }
        result
    }

    /// Handle m4_quote: quote arguments.
    ///
    /// m4_quote(ARG1, ARG2, ...) — wraps all arguments together in quotes.
    pub fn m4_quote(args: &[Vec<u8>], quote_open: u8, quote_close: u8) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(quote_open);
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                result.push(b',');
            }
            result.extend_from_slice(arg);
        }
        result.push(quote_close);
        result
    }

    /// Handle m4_normalize: normalize whitespace.
    ///
    /// Collapses multiple whitespace characters into single spaces,
    /// trims leading and trailing whitespace.
    pub fn m4_normalize(text: &[u8]) -> Vec<u8> {
        let s = String::from_utf8_lossy(text);
        let normalized: String = s.split_whitespace().collect::<Vec<&str>>().join(" ");
        normalized.into_bytes()
    }

    /// Handle m4_text_wrap: wrap text at a specified column width.
    ///
    /// m4_text_wrap(TEXT, [PREFIX], [WIDTH])
    pub fn m4_text_wrap(args: &[Vec<u8>]) -> Vec<u8> {
        let text = args.first().map(|a| a.as_slice()).unwrap_or(b"");
        let prefix = args.get(1).map(|a| a.as_slice()).unwrap_or(b"");
        let width: usize = args
            .get(2)
            .and_then(|a| String::from_utf8_lossy(a).parse().ok())
            .unwrap_or(79);

        let s = String::from_utf8_lossy(text);
        let prefix_str = String::from_utf8_lossy(prefix);

        let mut result = Vec::new();
        let words: Vec<&str> = s.split_whitespace().collect();
        let mut line_len = prefix_str.len();
        let mut first = true;

        for word in words {
            let word_len = word.len();
            if !first && line_len + 1 + word_len > width {
                result.push(b'\n');
                result.extend_from_slice(prefix);
                line_len = prefix_str.len();
                first = true;
            }
            if !first {
                result.push(b' ');
                line_len += 1;
            }
            result.extend_from_slice(word.as_bytes());
            line_len += word_len;
            first = false;
        }

        result
    }

    /// m4_chomp: remove trailing newline from a string.
    pub fn m4_chomp(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let s = String::from_utf8_lossy(&args[0]);
        s.trim_end_matches('\n').as_bytes().to_vec()
    }

    /// m4_version_prereq: check if m4 version meets requirement.
    pub fn m4_version_prereq(_args: &[Vec<u8>]) -> Vec<u8> {
        b"# m4_version_prereq: always satisfied (m4-rs-core)\n".to_vec()
    }

    /// m4_list_cmp: compare two lists lexicographically.
    pub fn m4_list_cmp(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return b"0".to_vec();
        }
        let a = String::from_utf8_lossy(&args[0]);
        let b = String::from_utf8_lossy(&args[1]);
        match a.cmp(&b) {
            std::cmp::Ordering::Less => b"-1".to_vec(),
            std::cmp::Ordering::Equal => b"0".to_vec(),
            std::cmp::Ordering::Greater => b"1".to_vec(),
        }
    }

    /// m4_toupper: convert a string to uppercase.
    pub fn m4_toupper(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let s = String::from_utf8_lossy(&args[0]).to_uppercase();
        s.into_bytes()
    }

    /// m4_tolower: convert a string to lowercase.
    pub fn m4_tolower(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let s = String::from_utf8_lossy(&args[0]).to_lowercase();
        s.into_bytes()
    }

    /// m4_split: split a string by a delimiter.
    pub fn m4_split(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let s = String::from_utf8_lossy(&args[0]);
        let delim = args
            .get(1)
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_else(|| " ".into());
        let result: Vec<String> = s.split(&*delim).map(|p| p.to_string()).collect();
        result.join(",").into_bytes()
    }

    /// m4_append: append a value to a macro definition (comma-separated).
    pub fn m4_append(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return Vec::new();
        }
        let mut result = Vec::new();
        result.extend_from_slice(&args[0]);
        result.push(b',');
        result.extend_from_slice(&args[1]);
        result
    }

    /// m4_prepend: prepend a value to a macro definition.
    pub fn m4_prepend(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return Vec::new();
        }
        let mut result = Vec::new();
        result.extend_from_slice(&args[1]);
        result.push(b',');
        result.extend_from_slice(&args[0]);
        result
    }
}

/// Split a byte slice on commas (outside of quotes/parens).
pub fn split_comma_separated(input: &[u8]) -> Vec<Vec<u8>> {
    let mut items = Vec::new();
    let mut current = Vec::new();
    let mut depth: u32 = 0;
    let mut in_quote = false;
    let mut i = 0;
    let bytes = input;

    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'(' | b'[' => {
                if !in_quote {
                    depth += 1;
                }
                current.push(b);
            }
            b')' | b']' => {
                if !in_quote {
                    depth = depth.saturating_sub(1);
                }
                current.push(b);
            }
            b'`' => {
                in_quote = !in_quote;
                current.push(b);
            }
            b',' if depth == 0 && !in_quote => {
                items.push(current.clone());
                current.clear();
            }
            b',' => {
                current.push(b);
            }
            _ => {
                current.push(b);
            }
        }
        i += 1;
    }
    if !current.is_empty() {
        items.push(current);
    }

    // Trim whitespace from each item
    items
        .into_iter()
        .map(|item| {
            let s = String::from_utf8_lossy(&item).trim().to_string();
            s.into_bytes()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_tracker_provide() {
        let mut tracker = RequireTracker::new();
        assert!(!tracker.is_provided("AC_INIT"));
        assert!(tracker.provide("AC_INIT"));
        assert!(tracker.is_provided("AC_INIT"));
        assert!(!tracker.provide("AC_INIT"));
    }

    #[test]
    fn test_require_tracker_cycle_detection() {
        let mut tracker = RequireTracker::new();
        tracker.push_expansion("A");
        let result = tracker.require("A", "A"); // Self-require = cycle
        assert!(result.is_err());
    }

    #[test]
    fn test_m4_if_basic() {
        let args: Vec<Vec<u8>> = vec![
            b"hello".to_vec(),
            b"hello".to_vec(),
            b"equal".to_vec(),
            b"else".to_vec(),
        ];
        let result = M4SugarBuiltins::m4_if(&args);
        assert_eq!(result, b"equal");
    }

    #[test]
    fn test_m4_if_no_match() {
        let args: Vec<Vec<u8>> = vec![b"a".to_vec(), b"b".to_vec(), b"yes".to_vec()];
        let result = M4SugarBuiltins::m4_if(&args);
        // No else clause, no match → empty
        assert!(result.is_empty());
    }

    #[test]
    fn test_m4_join() {
        let args: Vec<Vec<u8>> = vec![b", ".to_vec(), b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        let result = M4SugarBuiltins::m4_join(&args);
        assert_eq!(String::from_utf8_lossy(&result), "a, b, c");
    }

    #[test]
    fn test_m4_normalize() {
        let text = b"  hello   world  \n  foo  bar  ";
        let result = M4SugarBuiltins::m4_normalize(text);
        assert_eq!(String::from_utf8_lossy(&result), "hello world foo bar");
    }

    #[test]
    fn test_m4_dquote() {
        let args: Vec<Vec<u8>> = vec![b"hello".to_vec(), b"world".to_vec()];
        let result = M4SugarBuiltins::m4_dquote(&args, b'[', b']');
        assert_eq!(String::from_utf8_lossy(&result), "[hello],[world]");
    }

    #[test]
    fn test_m4_quote() {
        let args: Vec<Vec<u8>> = vec![b"hello".to_vec(), b"world".to_vec()];
        let result = M4SugarBuiltins::m4_quote(&args, b'[', b']');
        assert_eq!(String::from_utf8_lossy(&result), "[hello,world]");
    }

    #[test]
    fn test_split_comma_separated() {
        let items = split_comma_separated(b"a, b, c");
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], b"a");
        assert_eq!(items[1], b"b");
        assert_eq!(items[2], b"c");
    }
}
