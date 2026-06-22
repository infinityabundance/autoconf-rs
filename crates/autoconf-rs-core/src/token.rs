//! Token types for the M4 macro processor.
//!
//! M4 is a token-oriented macro processor. The lexer converts raw bytes
//! into a stream of these tokens, and the expansion engine consumes them.
//!
//! @ac_behavior id=AC.TOKEN.1 surface=AC.M4.LEX.1 manual=§3.1
//! Receipt family: AC.M4.LEX.*
//! Current status: Phase 2 — implemented, not yet oracle-admitted.

/// The kinds of tokens the M4 lexer can produce.
///
/// M4 recognizes these token types from the input stream:
/// - Text: literal characters passed through to output
/// - Name: a word that might be a macro name
/// - ParenOpen/ParenClose: macro argument delimiters
/// - Comma: argument separator
/// - QuoteOpen/QuoteClose: quoted string delimiters
/// - Eof: end of input
///
/// // What would break if changed: The expansion engine uses these token
/// // kinds to decide whether to look up a name as a macro, collect arguments,
/// // or copy text through. Changing the token kind enum would break all
/// // downstream processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Literal text characters that are not macro names or special chars
    Text,
    /// A word that could be a macro name (alphabetic + `_` prefix)
    Name,
    /// `(` — opens a macro argument list
    ParenOpen,
    /// `)` — closes a macro argument list
    ParenClose,
    /// `,` — separates macro arguments
    Comma,
    /// Opening quote delimiter (default `` ` ``)
    QuoteOpen,
    /// Closing quote delimiter (default `'`)
    QuoteClose,
    /// End of input marker
    Eof,
}

/// A single token produced by the lexer.
///
/// Each token carries its kind, the byte content, and source location
/// for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// What kind of token this is
    pub kind: TokenKind,
    /// The raw bytes of this token
    pub text: Vec<u8>,
    /// Source file (for diagnostics)
    pub file: Option<String>,
    /// Line number (1-based) where this token starts
    pub line: usize,
}

impl Token {
    /// Create a new token.
    pub fn new(kind: TokenKind, text: Vec<u8>, file: Option<String>, line: usize) -> Self {
        Self {
            kind,
            text,
            file,
            line,
        }
    }

    /// Create an EOF token.
    pub fn eof() -> Self {
        Self {
            kind: TokenKind::Eof,
            text: vec![],
            file: None,
            line: 0,
        }
    }

    /// Get the text as a string (lossy UTF-8 conversion).
    pub fn as_str(&self) -> String {
        String::from_utf8_lossy(&self.text).to_string()
    }

    /// Check if this is an EOF token.
    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }

    /// Check if this token is a name (potential macro invocation).
    pub fn is_name(&self) -> bool {
        self.kind == TokenKind::Name
    }

    /// Check if this is a ParenOpen token (starts argument collection).
    pub fn is_paren_open(&self) -> bool {
        self.kind == TokenKind::ParenOpen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_new() {
        let t = Token::new(TokenKind::Name, b"define".to_vec(), None, 1);
        assert_eq!(t.kind, TokenKind::Name);
        assert_eq!(t.as_str(), "define");
        assert_eq!(t.line, 1);
    }

    #[test]
    fn test_token_eof() {
        let t = Token::eof();
        assert!(t.is_eof());
        assert!(!t.is_name());
    }
}
