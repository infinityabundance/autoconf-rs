//! M4 expansion engine — the core processing loop.
//!
//! Takes a token stream, looks up macro names, collects arguments,
//! expands macros, handles $n substitution, and rescans output.
//!
//! @ac_behavior id=AC.EXPAND.1 surface=AC.M4.EXPAND.1 manual=§5.3
//! // Rescanning is the key M4 behavior: macro expansion output is fed back
//! // through the lexer and expansion engine. This enables macro composition.
//! // Without rescannning, macros could not generate other macro calls.
//! // Receipt: AC.M4.EXPAND.1 (pending).
//!
//! Current status: Phase 2 — implemented, not yet oracle-admitted.

use super::args;
use super::autoconf_macros::AutoconfState;
use super::builtin::{dispatch_builtin, BuiltinResult};
use super::lexer::Lexer;
use super::macro_table::MacroTable;
use super::token::{Token, TokenKind};

/// The M4 expansion engine.
pub struct ExpansionEngine {
    /// The macro symbol table
    pub table: MacroTable,
    /// The lexer for tokenizing input and rescan output
    pub lexer: Lexer,
    /// Autoconf processing state
    pub ac_state: AutoconfState,
    /// Maximum recursion depth for expansion
    pub max_recursion: usize,
    /// Maximum number of expansion steps (prevents infinite loops)
    pub max_steps: usize,
}

impl ExpansionEngine {
    pub fn new() -> Self {
        Self {
            table: MacroTable::new(),
            lexer: Lexer::new(),
            ac_state: AutoconfState::new(),
            max_recursion: 1024,
            max_steps: 100_000,
        }
    }

    /// Process input through the M4 engine and return expanded output.
    ///
    /// This is the main entry point for M4 processing. It tokenizes,
    /// expands, and collects output in a loop until all input is consumed.
    pub fn expand(&mut self, input: &[u8]) -> Vec<u8> {
        let tokens = self.lexer.tokenize(input);
        self.expand_tokens(&tokens, 0)
    }

    /// Expand a token stream, handling macros and collecting output.
    ///
    /// When `rescanning` is true (depth > 0), bare macro names not followed
    /// by `(` are treated as plain text rather than expanded. This prevents
    /// shell keywords (eval, exit, test, etc.) in generated configure output
    /// from being corrupted by M4 builtin dispatch during rescan.
    fn expand_tokens(&mut self, tokens: &[Token], depth: usize) -> Vec<u8> {
        if depth > self.max_recursion {
            return b"<recursion limit exceeded>".to_vec();
        }

        let rescanning = depth > 0;
        let mut output = Vec::new();
        let mut i = 0;
        let mut steps = 0;

        while i < tokens.len() {
            steps += 1;
            if steps > self.max_steps {
                output.extend_from_slice(b"<expansion limit exceeded>");
                break;
            }

            let token = &tokens[i];

            match token.kind {
                TokenKind::Eof => break,

                TokenKind::Name => {
                    let name = token.as_str();
                    let name_str = name.trim().to_string();

                    // Check if this name is a defined macro AND is followed by '('
                    // During rescan, bare names pass through as text to avoid
                    // corrupting shell keywords in configure output.
                    let has_paren = i + 1 < tokens.len() && tokens[i + 1].is_paren_open();
                    let should_expand =
                        self.table.is_defined(&name_str) && (!rescanning || has_paren);

                    if should_expand {
                        // Check if followed by '(' for argument collection
                        let (arg_texts, mut skip_count) = if has_paren {
                            let rest = &tokens[i + 2..]; // skip '(' and the name
                            let (args_tokens, consumed) = args::collect_args(rest);
                            let texts: Vec<Vec<u8>> =
                                args_tokens.iter().map(|arg| args::arg_text(arg)).collect();
                            (texts, 2 + consumed) // skip the name token and '('
                        } else {
                            // Macro called without arguments (initial input only)
                            (vec![], 1) // just consume the name token
                        };

                        match self.expand_macro(&name_str, &arg_texts, depth) {
                            Ok(expanded) => {
                                // Check if dnl was just processed (empty result from Dnl builtin)
                                // If so, skip the next newline token
                                let just_dnl = expanded.is_empty()
                                    && self.table.is_defined("dnl")
                                    && name_str == "dnl";

                                if !expanded.is_empty() {
                                    let rescanned = self.expand(&expanded);
                                    output.extend_from_slice(&rescanned);
                                }

                                if just_dnl {
                                    // dnl: discard all text up to and including the next newline.
                                    // In GNU m4, dnl at top level (no args) consumes everything
                                    // to the next newline, or to EOF if no newline follows.
                                    // @ac_behavior id=AC.DNL.1 surface=AC.M4.BUILTIN.1 manual=§5.4
                                    let mut consumed = 0;
                                    let next_idx = i + skip_count;
                                    while next_idx + consumed < tokens.len() {
                                        let t = &tokens[next_idx + consumed];
                                        match t.kind {
                                            TokenKind::Eof => break,
                                            _ => {
                                                if t.text == b"\n" {
                                                    consumed += 1; // consume the newline too
                                                    break;
                                                }
                                                consumed += 1;
                                            }
                                        }
                                    }
                                    skip_count += consumed;
                                }
                            }
                            Err(_e) => {
                                // Error expanding — skip
                            }
                        }
                        i += skip_count;
                    } else {
                        // Undefined name — copy as text
                        output.extend_from_slice(&token.text);
                        i += 1;
                    }
                }

                TokenKind::QuoteOpen => {
                    // Quoted text is passed through with quotes
                    output.push(self.lexer.quote.open);
                    i += 1;

                    // Collect quoted content
                    let mut quote_depth: usize = 1;
                    while i < tokens.len() && quote_depth > 0 {
                        let qt = &tokens[i];
                        match qt.kind {
                            TokenKind::QuoteOpen => {
                                quote_depth += 1;
                                output.push(self.lexer.quote.open);
                            }
                            TokenKind::QuoteClose => {
                                quote_depth -= 1;
                                if quote_depth > 0 {
                                    output.push(self.lexer.quote.close);
                                }
                            }
                            TokenKind::Eof => break,
                            _ => {
                                output.extend_from_slice(&qt.text);
                            }
                        }
                        i += 1;
                    }
                    // Emit closing quote
                    output.push(self.lexer.quote.close);
                }

                TokenKind::ParenOpen
                | TokenKind::ParenClose
                | TokenKind::Comma
                | TokenKind::QuoteClose => {
                    // Unmatched syntactic tokens pass through as text
                    output.extend_from_slice(&token.text);
                    i += 1;
                }

                TokenKind::Text => {
                    // Regular text passes through
                    output.extend_from_slice(&token.text);
                    i += 1;
                }
            }
        }

        output
    }

    /// Expand a single macro invocation.
    fn expand_macro(
        &mut self,
        name: &str,
        args: &[Vec<u8>],
        _depth: usize,
    ) -> Result<Vec<u8>, String> {
        let def = self.table.lookup(name);
        let is_builtin = def.map(|d| d.is_builtin).unwrap_or(false);

        if is_builtin {
            // Dispatch to Rust builtin handler
            let result = dispatch_builtin(
                name,
                args,
                &mut self.table,
                &mut self.ac_state,
                self.lexer.line,
            )?;

            match result {
                BuiltinResult::Text(text) => Ok(text),
                BuiltinResult::AutoconfSideEffect => Ok(Vec::new()),
                BuiltinResult::Define { .. } => Ok(Vec::new()),
                BuiltinResult::Undefine(_) => Ok(Vec::new()),
                BuiltinResult::Pushdef { .. } => Ok(Vec::new()),
                BuiltinResult::Popdef(_) => Ok(Vec::new()),
                BuiltinResult::ChangeQuote { open, close } => {
                    self.lexer.changequote(open, close);
                    Ok(Vec::new())
                }
                BuiltinResult::ChangeCom { start, end } => {
                    self.lexer.changecom(start, end);
                    Ok(Vec::new())
                }
                BuiltinResult::Divert(_n) => {
                    // Phase 2 stub: diversion system not yet implemented
                    Ok(Vec::new())
                }
                BuiltinResult::Dnl => {
                    // dnl: skip the next newline token in the expansion engine
                    Ok(Vec::new())
                }
                BuiltinResult::NoOp => Ok(Vec::new()),
                BuiltinResult::Exit(_code) => {
                    // Phase 2: exit is handled by caller
                    Ok(Vec::new())
                }
            }
        } else if let Some(body) = self.table.get_body(name) {
            // User-defined macro: substitute $n placeholders and return body
            let body = body.to_vec();
            Ok(substitute_placeholders(&body, args))
        } else {
            // Undefined: return name and arguments verbatim
            let mut result = name.as_bytes().to_vec();
            result.push(b'(');
            for (j, arg) in args.iter().enumerate() {
                if j > 0 {
                    result.push(b',');
                }
                result.extend_from_slice(arg);
            }
            result.push(b')');
            Ok(result)
        }
    }
}

/// Substitute $1, $2, ..., $@, $*, $# placeholders in macro body text.
///
/// @ac_behavior id=AC.SUBST.1 surface=AC.M4.EXPAND.1 manual=§5.2
/// // $0 expands to the macro name itself. $1-$9 expand to positional
/// // arguments. $# expands to the argument count. $@ and $* expand to
/// // all arguments with different quoting semantics. Receipt: AC.M4.EXPAND.1 (pending).
fn substitute_placeholders(body: &[u8], args: &[Vec<u8>]) -> Vec<u8> {
    let mut result = Vec::new();
    let len = body.len();
    let mut i = 0;

    while i < len {
        if body[i] == b'$' && i + 1 < len {
            match body[i + 1] {
                b'0'..=b'9' => {
                    let digit = (body[i + 1] - b'0') as usize;
                    i += 2;
                    if digit == 0 {
                        // $0 expands to the macro name (available in context)
                        // Phase 2: not tracked here — expander handles this
                    } else if digit <= args.len() {
                        result.extend_from_slice(&args[digit - 1]);
                    }
                    continue;
                }
                b'#' => {
                    i += 2;
                    result.extend_from_slice(format!("{}", args.len()).as_bytes());
                    continue;
                }
                b'@' => {
                    i += 2;
                    for (j, arg) in args.iter().enumerate() {
                        if j > 0 {
                            result.push(b',');
                        }
                        result.extend_from_slice(arg);
                    }
                    continue;
                }
                b'*' => {
                    i += 2;
                    for (j, arg) in args.iter().enumerate() {
                        if j > 0 {
                            result.push(b',');
                        }
                        result.extend_from_slice(arg);
                    }
                    continue;
                }
                _ => {
                    result.push(body[i]);
                    i += 1;
                    continue;
                }
            }
        }
        result.push(body[i]);
        i += 1;
    }

    result
}

impl Default for ExpansionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_simple_text() {
        let mut engine = ExpansionEngine::new();
        let output = engine.expand(b"hello world\n");
        assert_eq!(String::from_utf8_lossy(&output), "hello world\n");
    }

    #[test]
    fn test_expand_define_and_use() {
        let mut engine = ExpansionEngine::new();
        // Define a macro and use it
        let input = b"define(`foo', `bar')dnl\nfoo\n";
        let output = engine.expand(input);
        let text = String::from_utf8_lossy(&output);
        // After define, foo should expand to bar
        assert!(text.contains("bar"), "output was: {:?}", text);
    }

    #[test]
    fn test_expand_ifelse() {
        let mut engine = ExpansionEngine::new();
        let input = b"ifelse(`a', `a', `yes', `no')\n";
        let output = engine.expand(input);
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("yes"));
    }

    #[test]
    fn test_expand_len() {
        let mut engine = ExpansionEngine::new();
        let input = b"len(`hello')\n";
        let output = engine.expand(input);
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("5"));
    }

    #[test]
    fn test_substitute_placeholders() {
        let body = b"$1 and $2";
        let args = vec![b"hello".to_vec(), b"world".to_vec()];
        let result = substitute_placeholders(body, &args);
        assert_eq!(String::from_utf8_lossy(&result), "hello and world");
    }

    #[test]
    fn test_substitute_dollar_at() {
        let body = b"$@";
        let args = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        let result = substitute_placeholders(body, &args);
        assert_eq!(String::from_utf8_lossy(&result), "a,b,c");
    }

    #[test]
    fn test_substitute_dollar_hash() {
        let body = b"$#";
        let args = vec![b"a".to_vec(), b"b".to_vec()];
        let result = substitute_placeholders(body, &args);
        assert_eq!(String::from_utf8_lossy(&result), "2");
    }
}
