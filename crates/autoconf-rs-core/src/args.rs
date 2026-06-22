//! Argument collection for M4 macro invocations.
//!
//! When a macro name is followed by `(`, the expansion engine collects
//! comma-separated arguments until the matching `)`. Arguments may contain
//! nested parentheses, quoted strings, and expanded macro calls.
//!
//! @ac_behavior id=AC.ARGS.1 surface=AC.M4.ARGS.1 manual=§5.2
//! // $1 inside a quoted string is NOT expanded during argument collection.
//! // GNU m4 defers $n expansion to the rescan phase (manual §5.2).
//! // This path is guarded by AC.M4.ARGS.1; changing it to expand $1 eagerly
//! // would break the Autoconf AC_DEFUN pattern (receipt AC.M4.AUTOCONF.CORE.1 pending).
//!
//! Current status: Phase 2 — implemented, not yet oracle-admitted.

use super::token::{Token, TokenKind};

/// Collect comma-separated arguments from a token stream.
///
/// Assumes the first token is ParenOpen (already consumed by caller).
/// Reads tokens until matching ParenClose, splitting on Comma tokens
/// that are at nesting depth 0.
///
/// Returns (arguments, number of tokens consumed including closing paren).
pub fn collect_args(tokens: &[Token]) -> (Vec<Vec<Token>>, usize) {
    let mut args: Vec<Vec<Token>> = Vec::new();
    let mut current: Vec<Token> = Vec::new();
    let mut depth: usize = 0;

    // Start after the opening paren
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        if token.is_eof() {
            break;
        }

        match token.kind {
            TokenKind::ParenOpen => {
                depth += 1;
                current.push(token.clone());
            }
            TokenKind::ParenClose => {
                if depth == 0 {
                    // This is the closing paren for our argument list
                    // Push the last argument (if non-empty)
                    args.push(current);
                    return (args, i + 1);
                }
                depth -= 1;
                current.push(token.clone());
            }
            TokenKind::Comma => {
                if depth == 0 {
                    // Top-level comma: split arguments
                    args.push(current);
                    current = Vec::new();
                } else {
                    // Nested comma: part of current argument
                    current.push(token.clone());
                }
            }
            TokenKind::QuoteOpen => {
                current.push(token.clone());
                // Collect everything until matching QuoteClose
                i += 1;
                let mut quote_depth: usize = 1;
                while i < tokens.len() && quote_depth > 0 {
                    let qt = &tokens[i];
                    match qt.kind {
                        TokenKind::QuoteOpen => {
                            quote_depth += 1;
                            current.push(qt.clone());
                        }
                        TokenKind::QuoteClose => {
                            quote_depth -= 1;
                            current.push(qt.clone());
                        }
                        _ => {
                            current.push(qt.clone());
                        }
                    }
                    i += 1;
                }
                // Back up one because the loop will increment
                i = i.saturating_sub(1);
            }
            _ => {
                current.push(token.clone());
            }
        }
        i += 1;
    }

    // If we exit the loop without finding closing paren, push what we have
    if !current.is_empty() || !args.is_empty() {
        args.push(current);
    }
    (args, i)
}

/// Extract argument text (stripping quotes) from a token slice.
///
/// This is the core of argument processing: quoted strings have their
/// quote delimiters stripped, and the inner content is returned as-is.
/// Text tokens are concatenated directly.
pub fn arg_text(tokens: &[Token]) -> Vec<u8> {
    let mut result = Vec::new();

    for token in tokens {
        match token.kind {
            TokenKind::QuoteOpen | TokenKind::QuoteClose => {
                // Quote delimiters are stripped from output
            }
            _ => {
                result.extend_from_slice(&token.text);
            }
        }
    }

    // Trim leading whitespace (GNU m4 behavior)
    let trimmed = trim_leading_ws(&result);
    trimmed.to_vec()
}

/// Trim leading whitespace from a byte slice.
fn trim_leading_ws(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < bytes.len() && (bytes[start] == b' ' || bytes[start] == b'\t') {
        start += 1;
    }
    &bytes[start..]
}

#[cfg(test)]
mod tests {
    use super::super::token::Token;
    use super::*;

    fn make_tokens(spec: &[(&str, TokenKind)]) -> Vec<Token> {
        spec.iter()
            .map(|(text, kind)| Token::new(kind.clone(), text.as_bytes().to_vec(), None, 1))
            .collect()
    }

    #[test]
    fn test_collect_single_arg() {
        let tokens = make_tokens(&[("arg1", TokenKind::Text), (")", TokenKind::ParenClose)]);
        let (args, consumed) = collect_args(&tokens);
        assert_eq!(args.len(), 1);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_collect_multiple_args() {
        let tokens = make_tokens(&[
            ("foo", TokenKind::Text),
            (",", TokenKind::Comma),
            ("bar", TokenKind::Text),
            (")", TokenKind::ParenClose),
        ]);
        let (args, consumed) = collect_args(&tokens);
        assert_eq!(args.len(), 2);
        assert_eq!(consumed, 4);
    }

    #[test]
    fn test_collect_nested_parens() {
        let tokens = make_tokens(&[
            ("outer", TokenKind::Text),
            ("(", TokenKind::ParenOpen),
            ("inner", TokenKind::Text),
            (")", TokenKind::ParenClose),
            (",", TokenKind::Comma),
            ("second", TokenKind::Text),
            (")", TokenKind::ParenClose),
        ]);
        let (args, _consumed) = collect_args(&tokens);
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn test_arg_text_strips_quotes() {
        let tokens = make_tokens(&[
            ("`", TokenKind::QuoteOpen),
            ("hello", TokenKind::Text),
            ("'", TokenKind::QuoteClose),
        ]);
        let text = arg_text(&tokens);
        assert_eq!(String::from_utf8_lossy(&text), "hello");
    }

    #[test]
    fn test_arg_text_trims_leading_ws() {
        let tokens = make_tokens(&[
            (" ", TokenKind::Text),
            ("  ", TokenKind::Text),
            ("hello", TokenKind::Text),
        ]);
        let text = arg_text(&tokens);
        assert_eq!(String::from_utf8_lossy(&text), "hello");
    }
}
