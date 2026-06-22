//! Byte-level lexer for M4 input.
//!
//! Converts raw input bytes into a stream of tokens. The lexer handles:
//! - Quote delimiter recognition (default `` ` `` and `'`)
//! - Comment delimiter recognition (default `#` and newline)
//! - Macro name recognition
//! - Parenthesis and comma recognition for argument lists
//! - Changequote and changecom (same-pass re-lexing)
//!
//! @ac_behavior id=AC.LEX.1 surface=AC.M4.LEX.1 manual=§3.1
//! // Quote characters MUST be recognized as single bytes, not characters.
//! // GNU m4 is eight-bit-clean. Using char-based matching would fail on
//! // non-UTF-8 input. Receipt: AC.M4.LEX.1 (pending).
//!
//! Current status: Phase 2 — implemented, not yet oracle-admitted.

use super::token::{Token, TokenKind};

/// Quote state tracked by the lexer.
#[derive(Debug, Clone)]
pub struct QuoteState {
    /// Opening quote character (byte)
    pub open: u8,
    /// Closing quote character (byte)
    pub close: u8,
    /// Whether we're currently inside a quoted string
    pub in_quote: bool,
    /// Nesting depth of quotes
    pub depth: usize,
}

impl Default for QuoteState {
    fn default() -> Self {
        Self {
            open: b'`',
            close: b'\'',
            in_quote: false,
            depth: 0,
        }
    }
}

/// Comment state tracked by the lexer.
#[derive(Debug, Clone)]
pub struct CommentState {
    /// Start-of-comment character (byte)
    pub start: u8,
    /// End-of-comment character (byte) — newline for line comments
    pub end: u8,
    /// Whether we're currently inside a comment
    pub in_comment: bool,
}

impl Default for CommentState {
    fn default() -> Self {
        Self {
            start: b'#',
            end: b'\n',
            in_comment: false,
        }
    }
}

/// The M4 lexer — converts bytes to tokens.
///
/// // Why a push-based design: The expansion engine needs to feed rescan
/// // output back through the lexer. A push-based design lets us push
/// // expanded text onto the input and continue lexing seamlessly.
/// // Receipt: AC.M4.EXPAND.1 (pending).
pub struct Lexer {
    /// Current quote delimiters
    pub quote: QuoteState,
    /// Current comment delimiters
    pub comment: CommentState,
    /// Current line number
    pub line: usize,
    /// Source file name
    pub file: Option<String>,
    /// Pending re-lex flag (set when changequote happens mid-stream)
    pub needs_relex: bool,
}

impl Lexer {
    /// Create a new lexer with default quote/comment settings.
    pub fn new() -> Self {
        Self {
            quote: QuoteState::default(),
            comment: CommentState::default(),
            line: 1,
            file: None,
            needs_relex: false,
        }
    }

    /// Tokenize an entire input string.
    pub fn tokenize(&mut self, input: &[u8]) -> Vec<Token> {
        let mut tokens = Vec::new();
        let bytes = input.to_vec();
        let len = bytes.len();
        let mut pos = 0;

        while pos < len {
            let b = bytes[pos];

            // Handle comments: skip everything until end-of-comment
            if b == self.comment.start && !self.quote.in_quote {
                self.comment.in_comment = true;
                pos += 1;
                // Consume comment body
                while pos < len && bytes[pos] != self.comment.end {
                    if bytes[pos] == b'\n' {
                        self.line += 1;
                    }
                    pos += 1;
                }
                if pos < len && bytes[pos] == self.comment.end {
                    if bytes[pos] == b'\n' {
                        self.line += 1;
                    }
                    pos += 1;
                }
                self.comment.in_comment = false;
                continue;
            }

            // Handle newlines (track line numbers, produce text token)
            if b == b'\n' {
                // If we were in a comment, newline ends it
                if self.comment.in_comment {
                    self.comment.in_comment = false;
                    self.line += 1;
                    pos += 1;
                    continue;
                }
                tokens.push(Token::new(
                    TokenKind::Text,
                    vec![b'\n'],
                    self.file.clone(),
                    self.line,
                ));
                self.line += 1;
                pos += 1;
                continue;
            }

            // Handle quoted strings
            if b == self.quote.open && !self.quote.in_quote {
                // Entering a quoted string
                self.quote.in_quote = true;
                self.quote.depth = 1;
                tokens.push(Token::new(
                    TokenKind::QuoteOpen,
                    vec![b],
                    self.file.clone(),
                    self.line,
                ));
                pos += 1;

                // Collect quoted content until matching close quote
                let mut quoted = Vec::new();
                let start_line = self.line;
                while pos < len && self.quote.depth > 0 {
                    let cb = bytes[pos];
                    if cb == self.quote.open {
                        self.quote.depth += 1;
                        quoted.push(cb);
                    } else if cb == self.quote.close {
                        self.quote.depth -= 1;
                        if self.quote.depth == 0 {
                            // End of quoted string
                            break;
                        }
                        quoted.push(cb);
                    } else {
                        if cb == b'\n' {
                            self.line += 1;
                        }
                        quoted.push(cb);
                    }
                    pos += 1;
                }

                // Emit quoted content as a single text token
                if !quoted.is_empty() {
                    tokens.push(Token::new(
                        TokenKind::Text,
                        quoted,
                        self.file.clone(),
                        start_line,
                    ));
                }

                // Emit close quote
                if pos < len && bytes[pos] == self.quote.close {
                    tokens.push(Token::new(
                        TokenKind::QuoteClose,
                        vec![bytes[pos]],
                        self.file.clone(),
                        self.line,
                    ));
                    pos += 1;
                }
                self.quote.in_quote = false;
                self.quote.depth = 0;
                continue;
            }

            // Handle parentheses (macro argument delimiters)
            if b == b'(' && !self.quote.in_quote {
                tokens.push(Token::new(
                    TokenKind::ParenOpen,
                    vec![b'('],
                    self.file.clone(),
                    self.line,
                ));
                pos += 1;
                continue;
            }

            if b == b')' && !self.quote.in_quote {
                tokens.push(Token::new(
                    TokenKind::ParenClose,
                    vec![b')'],
                    self.file.clone(),
                    self.line,
                ));
                pos += 1;
                continue;
            }

            // Handle commas (argument separators)
            if b == b',' && !self.quote.in_quote {
                tokens.push(Token::new(
                    TokenKind::Comma,
                    vec![b','],
                    self.file.clone(),
                    self.line,
                ));
                pos += 1;
                continue;
            }

            // Handle whitespace (space, tab)
            if b == b' ' || b == b'\t' {
                // Collect whitespace run
                let mut ws = vec![b];
                pos += 1;
                while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
                    ws.push(bytes[pos]);
                    pos += 1;
                }
                tokens.push(Token::new(
                    TokenKind::Text,
                    ws,
                    self.file.clone(),
                    self.line,
                ));
                continue;
            }

            // Handle macro names and other text
            // A macro name starts with a letter or underscore
            if (b.is_ascii_alphabetic() || b == b'_') && !self.quote.in_quote {
                let mut name = vec![b];
                pos += 1;
                while pos < len && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                    name.push(bytes[pos]);
                    pos += 1;
                }

                // Always emit as Name — the expansion engine decides whether
                // to expand or pass through
                tokens.push(Token::new(
                    TokenKind::Name,
                    name,
                    self.file.clone(),
                    self.line,
                ));
                continue;
            }

            // Any other byte: emit as text
            tokens.push(Token::new(
                TokenKind::Text,
                vec![b],
                self.file.clone(),
                self.line,
            ));
            pos += 1;
        }

        tokens.push(Token::eof());
        tokens
    }

    /// Change the quote delimiters (changequote).
    ///
    /// @ac_behavior id=AC.QUOTE.CHANGE.1 surface=AC.M4.QUOTE.1 manual=§5.5
    /// // Same-pass re-lex: when changequote is called during expansion,
    /// // the token stream being produced must reflect the new delimiters
    /// // immediately. Receipt: AC.M4.QUOTE.1 (pending).
    pub fn changequote(&mut self, open: Option<u8>, close: Option<u8>) {
        self.quote.open = open.unwrap_or(b'`');
        self.quote.close = close.unwrap_or(b'\'');
    }

    /// Change the comment delimiters (changecom).
    pub fn changecom(&mut self, start: Option<u8>, end: Option<u8>) {
        self.comment.start = start.unwrap_or(b'#');
        self.comment.end = end.unwrap_or(b'\n');
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_text() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"hello world\n");
        // "hello"=Name, " "=Text, "world"=Name, "\n"=Text, Eof
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].kind, TokenKind::Name);
        assert_eq!(tokens[0].as_str(), "hello");
    }

    #[test]
    fn test_tokenize_macro_name() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"define(");
        assert_eq!(tokens[0].kind, TokenKind::Name);
        assert_eq!(tokens[0].as_str(), "define");
        assert_eq!(tokens[1].kind, TokenKind::ParenOpen);
    }

    #[test]
    fn test_tokenize_quoted_string() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"`hello world'");
        assert_eq!(tokens[0].kind, TokenKind::QuoteOpen);
        assert_eq!(tokens[1].kind, TokenKind::Text);
        assert_eq!(tokens[1].as_str(), "hello world");
        assert_eq!(tokens[2].kind, TokenKind::QuoteClose);
    }

    #[test]
    fn test_tokenize_nested_quotes() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"`outer `inner' outer'");
        // Should handle nested quotes properly
        assert_eq!(tokens[0].kind, TokenKind::QuoteOpen);
        // The inner content includes the nested quote chars
        let inner_text: String = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Text)
            .map(|t| t.as_str())
            .collect();
        assert!(inner_text.contains("inner"));
    }

    #[test]
    fn test_tokenize_comment() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"hello # this is a comment\nworld\n");
        // hello=Name, " "=Text, comment consumed, newline, world=Name, newline
        let all_text: String = tokens.iter().map(|t| t.as_str()).collect();
        // Comment should be excluded
        assert!(!all_text.contains("comment"));
        assert!(all_text.contains("hello"));
        assert!(all_text.contains("world"));
    }

    #[test]
    fn test_tokenize_arguments() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"define(`foo', `bar')");
        // define ( Name
        // ( ( ParenOpen
        // `foo' ( QuoteOpen Text QuoteClose
        // , ( Comma
        // `bar' ( QuoteOpen Text QuoteClose
        // ) ( ParenClose
        assert_eq!(tokens[0].kind, TokenKind::Name);
        assert_eq!(tokens[0].as_str(), "define");
        assert_eq!(tokens[1].kind, TokenKind::ParenOpen);
    }
}
