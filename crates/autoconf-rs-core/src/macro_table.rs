//! Macro symbol table for the M4 macro processor.
//!
//! Maintains a mapping from macro names to their definitions.
//! Supports: define, undefine, pushdef, popdef, defn, builtin, indir.
//!
//! // Why a stack-based design: M4 macros can be temporarily overridden
//! // with pushdef/popdef. Each name maps to a stack of definitions.
//! // The top of the stack is the active definition. Receipt: AC.M4.DEFINE.1 (pending).
//!
//! @ac_behavior id=AC.MACRO.TABLE.1 surface=AC.M4.DEFINE.1 manual=§5.1
//! Current status: Phase 2 — implemented, not yet oracle-admitted.

use std::collections::HashMap;

/// A single macro definition.
#[derive(Debug, Clone)]
pub struct MacroDef {
    /// The macro body text (unexpanded)
    pub body: Vec<u8>,
    /// Whether this is a builtin (cannot be redefined without -P)
    pub is_builtin: bool,
    /// The source file where this was defined
    pub file: Option<String>,
    /// The line where this was defined
    pub line: usize,
}

impl MacroDef {
    pub fn new(body: Vec<u8>, is_builtin: bool) -> Self {
        Self {
            body,
            is_builtin,
            file: None,
            line: 0,
        }
    }

    pub fn with_location(
        body: Vec<u8>,
        is_builtin: bool,
        file: Option<String>,
        line: usize,
    ) -> Self {
        Self {
            body,
            is_builtin,
            file,
            line,
        }
    }
}

/// The macro symbol table.
///
/// Each macro name maps to a stack (Vec) of definitions. The last element
/// is the active definition. pushdef adds to the stack, popdef removes
/// the top, define replaces the entire stack.
pub struct MacroTable {
    /// Name → stack of definitions (last = active)
    table: HashMap<String, Vec<MacroDef>>,
    /// Whether we're in prefix-builtins mode (-P flag)
    prefix_builtins: bool,
}

impl MacroTable {
    pub fn new() -> Self {
        let mut table = Self {
            table: HashMap::new(),
            prefix_builtins: false,
        };
        table.register_builtins();
        table
    }

    /// Register all standard GNU m4 builtins.
    fn register_builtins(&mut self) {
        let builtins: &[(&str, &str)] = &[
            ("define", ""),
            ("undefine", ""),
            ("defn", ""),
            ("pushdef", ""),
            ("popdef", ""),
            ("indir", ""),
            ("builtin", ""),
            ("ifdef", ""),
            ("ifelse", ""),
            ("shift", ""),
            ("forloop", ""),
            ("foreach", ""),
            ("len", ""),
            ("index", ""),
            ("substr", ""),
            ("translit", ""),
            ("patsubst", ""),
            ("regexp", ""),
            ("format", ""),
            ("incr", ""),
            ("decr", ""),
            ("eval", ""),
            ("changequote", ""),
            ("changecom", ""),
            ("changeword", ""),
            ("m4wrap", ""),
            ("include", ""),
            ("sinclude", ""),
            ("divert", ""),
            ("undivert", ""),
            ("divnum", ""),
            ("errprint", ""),
            ("traceon", ""),
            ("traceoff", ""),
            ("dumpdef", ""),
            ("debugmode", ""),
            ("debugfile", ""),
            ("dnl", ""),
            ("syscmd", ""),
            ("esyscmd", ""),
            ("sysval", ""),
            ("maketemp", ""),
            ("mkstemp", ""),
            ("__file__", ""),
            ("__line__", ""),
            ("__program__", ""),
            ("m4exit", ""),
            // m4sugar builtins
            ("m4_define", ""),
            ("m4_defun", ""),
            ("m4_require", ""),
            ("m4_provide", ""),
            ("m4_if", ""),
            ("m4_case", ""),
            ("m4_foreach", ""),
            ("m4_map", ""),
            ("m4_join", ""),
            ("m4_expand", ""),
            ("m4_do", ""),
            ("m4_dquote", ""),
            ("m4_quote", ""),
            ("m4_normalize", ""),
            ("m4_text_wrap", ""),
            // m4sh builtins
            ("AS_ECHO", ""),
            ("AS_ESCAPE", ""),
            ("AS_EXIT", ""),
            ("AS_IF", ""),
            ("AS_CASE", ""),
            ("AS_FOR", ""),
            ("AS_MKDIR_P", ""),
            ("AS_TR_SH", ""),
            ("AS_TR_CPP", ""),
            // Autoconf builtins
            ("AC_INIT", ""),
            ("AC_OUTPUT", ""),
            ("AC_CONFIG_FILES", ""),
            ("AC_CONFIG_HEADERS", ""),
            ("AC_CONFIG_COMMANDS", ""),
            ("AC_CONFIG_LINKS", ""),
            ("AC_CONFIG_SUBDIRS", ""),
            ("AC_SUBST", ""),
            ("AC_DEFINE", ""),
            ("AC_DEFUN", ""),
            ("AC_REQUIRE", ""),
            ("AC_PROVIDE", ""),
            ("AC_BEFORE", ""),
            ("AC_DIAGNOSE", ""),
            ("AC_WARNING", ""),
            ("AC_FATAL", ""),
        ];

        for (name, _body) in builtins {
            let def = MacroDef::new(Vec::new(), true);
            self.table.entry(name.to_string()).or_default().push(def);
        }
    }

    /// Define or redefine a macro (define).
    ///
    /// // define replaces the entire stack for this name.
    /// // Receipt: AC.M4.DEFINE.1 (pending).
    pub fn define(&mut self, name: &str, body: &[u8], file: Option<String>, line: usize) {
        if self.prefix_builtins {
            // In prefix mode, only m4_ prefixed names can be defined
            if !name.starts_with("m4_") {
                return;
            }
        }

        let def = MacroDef::with_location(body.to_vec(), false, file, line);
        self.table.insert(name.to_string(), vec![def]);
    }

    /// Push a new definition onto the stack (pushdef).
    pub fn pushdef(&mut self, name: &str, body: &[u8], file: Option<String>, line: usize) {
        let def = MacroDef::with_location(body.to_vec(), false, file, line);
        self.table.entry(name.to_string()).or_default().push(def);
    }

    /// Remove the topmost definition (popdef).
    pub fn popdef(&mut self, name: &str) {
        if let Some(stack) = self.table.get_mut(name) {
            stack.pop();
            if stack.is_empty() {
                self.table.remove(name);
            }
        }
    }

    /// Remove all definitions for a name (undefine).
    pub fn undefine(&mut self, name: &str) {
        self.table.remove(name);
    }

    /// Look up a macro by name. Returns the active definition if found.
    pub fn lookup(&self, name: &str) -> Option<&MacroDef> {
        self.table.get(name).and_then(|stack| stack.last())
    }

    /// Check if a macro is defined.
    pub fn is_defined(&self, name: &str) -> bool {
        self.table.contains_key(name) && !self.table[name].is_empty()
    }

    /// Get the body of the named macro's active definition.
    pub fn get_body(&self, name: &str) -> Option<&[u8]> {
        self.lookup(name).map(|d| d.body.as_slice())
    }

    /// Get the definition of a macro for defn (returns body of topmost).
    pub fn defn(&self, name: &str) -> Option<Vec<u8>> {
        self.lookup(name).map(|d| d.body.clone())
    }

    /// Set prefix-builtins mode (-P flag).
    pub fn set_prefix_builtins(&mut self, enabled: bool) {
        self.prefix_builtins = enabled;
    }
}

impl Default for MacroTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_lookup() {
        let mut table = MacroTable::new();
        table.define("foo", b"bar", None, 1);
        assert!(table.is_defined("foo"));
        assert_eq!(table.get_body("foo"), Some(b"bar".as_slice()));
    }

    #[test]
    fn test_pushdef_popdef() {
        let mut table = MacroTable::new();
        table.pushdef("x", b"first", None, 1);
        table.pushdef("x", b"second", None, 2);
        assert_eq!(table.get_body("x"), Some(b"second".as_slice()));
        table.popdef("x");
        assert_eq!(table.get_body("x"), Some(b"first".as_slice()));
        table.popdef("x");
        assert!(!table.is_defined("x"));
    }

    #[test]
    fn test_undefine() {
        let mut table = MacroTable::new();
        table.define("foo", b"bar", None, 1);
        assert!(table.is_defined("foo"));
        table.undefine("foo");
        assert!(!table.is_defined("foo"));
    }

    #[test]
    fn test_builtins_registered() {
        let table = MacroTable::new();
        assert!(table.is_defined("define"));
        assert!(table.is_defined("ifelse"));
        assert!(table.is_defined("eval"));
        // Autoconf builtins
        assert!(table.is_defined("AC_INIT"));
        assert!(table.is_defined("AC_DEFUN"));
    }
}
