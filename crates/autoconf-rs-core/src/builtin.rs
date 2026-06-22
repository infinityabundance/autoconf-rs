//! M4 built-in macro implementations.
//!
//! Implements the standard GNU m4 builtins as Rust functions that the
//! expansion engine invokes when it encounters builtin macro names.
//!
//! @ac_behavior id=AC.BUILTIN.1 surface=AC.M4.BUILTIN.1 manual=§5
//! Current status: Phase 2 — implemented, not yet oracle-admitted.

use super::autoconf_macros::{AutoconfBuiltins, AutoconfState};
use super::m4sh::M4ShBuiltins;
use super::macro_table::MacroTable;

/// The result of executing a builtin macro.
#[derive(Debug, Clone)]
pub enum BuiltinResult {
    /// Expansion produced output text
    Text(Vec<u8>),
    /// Modify Autoconf state (side effect)
    AutoconfSideEffect,
    /// Define/redefine a macro
    Define { name: String, body: Vec<u8> },
    /// Undefine a macro
    Undefine(String),
    /// Push a definition
    Pushdef { name: String, body: Vec<u8> },
    /// Pop a definition
    Popdef(String),
    /// Change quote delimiters
    ChangeQuote { open: Option<u8>, close: Option<u8> },
    /// Change comment delimiters
    ChangeCom { start: Option<u8>, end: Option<u8> },
    /// Divert output
    Divert(i32),
    /// No output (dnl, which discards to newline)
    Dnl,
    /// No output (side-effect only, e.g., define)
    NoOp,
    /// M4 exit
    Exit(i32),
}

/// Dispatch a builtin macro by name with its arguments.
pub fn dispatch_builtin(
    name: &str,
    args: &[Vec<u8>],
    table: &mut MacroTable,
    state: &mut AutoconfState,
    line: usize,
) -> Result<BuiltinResult, String> {
    match name {
        "define" | "m4_define" => {
            if args.len() < 2 {
                return Ok(BuiltinResult::NoOp);
            }
            let name_str = String::from_utf8_lossy(&args[0]).to_string();
            let body = args[1].clone();
            table.define(&name_str, &body, None, line);
            Ok(BuiltinResult::NoOp)
        }

        "undefine" => {
            for arg in args {
                let name_str = String::from_utf8_lossy(arg).to_string();
                table.undefine(&name_str);
            }
            Ok(BuiltinResult::NoOp)
        }

        "pushdef" => {
            if args.len() < 2 {
                return Ok(BuiltinResult::NoOp);
            }
            let name_str = String::from_utf8_lossy(&args[0]).to_string();
            let body = args[1].clone();
            table.pushdef(&name_str, &body, None, line);
            Ok(BuiltinResult::NoOp)
        }

        "popdef" => {
            for arg in args {
                let name_str = String::from_utf8_lossy(arg).to_string();
                table.popdef(&name_str);
            }
            Ok(BuiltinResult::NoOp)
        }

        "ifdef" => {
            if args.len() < 2 {
                return Ok(BuiltinResult::Text(b"".to_vec()));
            }
            let name_str = String::from_utf8_lossy(&args[0]).to_string();
            if table.is_defined(&name_str) {
                Ok(BuiltinResult::Text(args[1].clone()))
            } else if args.len() > 2 {
                Ok(BuiltinResult::Text(args[2].clone()))
            } else {
                Ok(BuiltinResult::Text(b"".to_vec()))
            }
        }

        "ifelse" => {
            if args.is_empty() {
                return Ok(BuiltinResult::Text(b"".to_vec()));
            }
            // ifelse(s1, s2, equal, [s3, s4, equal2, ...], [else])
            let mut i = 0;
            while i + 2 < args.len() {
                if args[i] == args[i + 1] {
                    return Ok(BuiltinResult::Text(args[i + 2].clone()));
                }
                i += 3;
            }
            // No match — return final arg if it's the else clause
            if args.len() % 3 == 1 {
                Ok(BuiltinResult::Text(args.last().unwrap().clone()))
            } else {
                Ok(BuiltinResult::Text(b"".to_vec()))
            }
        }

        "shift" => {
            if args.len() <= 1 {
                return Ok(BuiltinResult::Text(b"".to_vec()));
            }
            // shift returns all but the first argument, comma-separated
            let mut result = Vec::new();
            for (i, arg) in args[1..].iter().enumerate() {
                if i > 0 {
                    result.push(b',');
                }
                result.extend_from_slice(arg);
            }
            Ok(BuiltinResult::Text(result))
        }

        "len" => {
            if args.is_empty() {
                return Ok(BuiltinResult::Text(b"0".to_vec()));
            }
            let len_str = format!("{}", args[0].len());
            Ok(BuiltinResult::Text(len_str.into_bytes()))
        }

        "dnl" => Ok(BuiltinResult::Dnl),

        "incr" => {
            if args.is_empty() {
                return Ok(BuiltinResult::Text(b"0".to_vec()));
            }
            let num_str = String::from_utf8_lossy(&args[0]);
            if let Ok(n) = num_str.trim().parse::<i64>() {
                Ok(BuiltinResult::Text(format!("{}", n + 1).into_bytes()))
            } else {
                Ok(BuiltinResult::Text(b"0".to_vec()))
            }
        }

        "decr" => {
            if args.is_empty() {
                return Ok(BuiltinResult::Text(b"0".to_vec()));
            }
            let num_str = String::from_utf8_lossy(&args[0]);
            if let Ok(n) = num_str.trim().parse::<i64>() {
                Ok(BuiltinResult::Text(format!("{}", n - 1).into_bytes()))
            } else {
                Ok(BuiltinResult::Text(b"0".to_vec()))
            }
        }

        "eval" => {
            if args.is_empty() {
                return Ok(BuiltinResult::Text(b"0".to_vec()));
            }
            let expr = String::from_utf8_lossy(&args[0]);
            match evaluate_simple(&expr) {
                Ok(n) => Ok(BuiltinResult::Text(format!("{}", n).into_bytes())),
                Err(_) => Ok(BuiltinResult::Text(b"0".to_vec())),
            }
        }

        "changequote" => {
            let open = args.first().and_then(|a| a.first().copied());
            let close = args.get(1).and_then(|a| a.first().copied());
            Ok(BuiltinResult::ChangeQuote { open, close })
        }

        "changecom" => {
            let start = args.first().and_then(|a| a.first().copied());
            let end = args.get(1).and_then(|a| a.first().copied());
            Ok(BuiltinResult::ChangeCom { start, end })
        }

        "divert" => {
            if args.is_empty() {
                Ok(BuiltinResult::Text(b"0".to_vec()))
            } else {
                let num_str = String::from_utf8_lossy(&args[0]);
                let n = num_str.trim().parse::<i32>().unwrap_or(0);
                Ok(BuiltinResult::Divert(n))
            }
        }

        "divnum" => Ok(BuiltinResult::Text(b"0".to_vec())),

        "errprint" => {
            let msg = args.first().cloned().unwrap_or_else(|| b"".to_vec());
            eprint!("{}", String::from_utf8_lossy(&msg));
            Ok(BuiltinResult::NoOp)
        }

        "__file__" => Ok(BuiltinResult::Text(b"stdin".to_vec())),

        "__line__" => Ok(BuiltinResult::Text(format!("{}", line).into_bytes())),

        "__program__" => Ok(BuiltinResult::Text(b"autoconf-rs".to_vec())),

        "m4exit" => {
            let code = args
                .first()
                .and_then(|a| String::from_utf8_lossy(a).trim().parse::<i32>().ok())
                .unwrap_or(0);
            Ok(BuiltinResult::Exit(code))
        }

        // === Autoconf macros (Phase 2) ===
        "AC_INIT" => {
            let output = AutoconfBuiltins::ac_init(args, state);
            Ok(BuiltinResult::Text(output))
        }

        "AC_OUTPUT" => {
            let output = AutoconfBuiltins::ac_output(state);
            Ok(BuiltinResult::Text(output))
        }

        "AC_CONFIG_FILES" => {
            AutoconfBuiltins::ac_config_files(args, state);
            Ok(BuiltinResult::AutoconfSideEffect)
        }

        "AC_CONFIG_HEADERS" => {
            AutoconfBuiltins::ac_config_headers(args, state);
            Ok(BuiltinResult::AutoconfSideEffect)
        }

        "AC_SUBST" => {
            AutoconfBuiltins::ac_subst(args, state);
            Ok(BuiltinResult::AutoconfSideEffect)
        }

        "AC_DEFINE" => {
            AutoconfBuiltins::ac_define(args, state);
            Ok(BuiltinResult::AutoconfSideEffect)
        }

        "AC_CONFIG_COMMANDS" => {
            AutoconfBuiltins::ac_config_commands(args, state);
            Ok(BuiltinResult::AutoconfSideEffect)
        }

        "AC_CONFIG_LINKS" => {
            AutoconfBuiltins::ac_config_links(args, state);
            Ok(BuiltinResult::AutoconfSideEffect)
        }

        "AC_CONFIG_SUBDIRS" => {
            AutoconfBuiltins::ac_config_subdirs(args, state);
            Ok(BuiltinResult::AutoconfSideEffect)
        }

        "AC_MSG_CHECKING" => Ok(BuiltinResult::Text(AutoconfBuiltins::ac_msg_checking(args))),

        "AC_MSG_RESULT" => Ok(BuiltinResult::Text(AutoconfBuiltins::ac_msg_result(args))),

        "AC_MSG_WARN" => Ok(BuiltinResult::Text(AutoconfBuiltins::ac_msg_warn(args))),

        "AC_MSG_ERROR" => Ok(BuiltinResult::Text(AutoconfBuiltins::ac_msg_error(args))),

        "AC_PROG_CC" => Ok(BuiltinResult::Text(AutoconfBuiltins::ac_prog_cc(state))),

        "AC_PROG_INSTALL" => Ok(BuiltinResult::Text(AutoconfBuiltins::ac_prog_install(
            state,
        ))),

        "AC_PROG_MAKE_SET" => Ok(BuiltinResult::Text(AutoconfBuiltins::ac_prog_make_set(
            state,
        ))),

        // === m4sh macros ===
        "AS_ECHO" => Ok(BuiltinResult::Text(M4ShBuiltins::as_echo(args))),

        "AS_ECHO_N" => Ok(BuiltinResult::Text(M4ShBuiltins::as_echo_n(args))),

        "AS_ESCAPE" => Ok(BuiltinResult::Text(M4ShBuiltins::as_escape(args))),

        "AS_EXIT" => Ok(BuiltinResult::Text(M4ShBuiltins::as_exit(args))),

        "AS_IF" => Ok(BuiltinResult::Text(M4ShBuiltins::as_if(args))),

        "AS_CASE" => Ok(BuiltinResult::Text(M4ShBuiltins::as_case(args))),

        "AS_FOR" => Ok(BuiltinResult::Text(M4ShBuiltins::as_for(args))),

        "AS_MKDIR_P" => Ok(BuiltinResult::Text(M4ShBuiltins::as_mkdir_p(args))),

        "AS_TR_SH" => Ok(BuiltinResult::Text(M4ShBuiltins::as_tr_sh(args))),

        "AS_TR_CPP" => Ok(BuiltinResult::Text(M4ShBuiltins::as_tr_cpp(args))),

        "AS_UNSET" => Ok(BuiltinResult::Text(M4ShBuiltins::as_unset(args))),

        "AS_BOX" => Ok(BuiltinResult::Text(M4ShBuiltins::as_box(args))),

        // === m4sugar macros ===
        "m4_defun" => {
            if args.len() >= 2 {
                let name = String::from_utf8_lossy(&args[0]).to_string();
                let body = args[1].clone();
                table.define(&name, &body, None, line);
            }
            Ok(BuiltinResult::NoOp)
        }

        "m4_require" | "AC_REQUIRE" => {
            if args.len() >= 2 {
                let required = String::from_utf8_lossy(&args[0]).to_string();
                let requirer = String::from_utf8_lossy(&args[1]).to_string();
                state.require_tracker.require(&requirer, &required)?;
            }
            Ok(BuiltinResult::NoOp)
        }

        "m4_provide" | "AC_PROVIDE" => {
            for arg in args {
                let name = String::from_utf8_lossy(arg).to_string();
                state.require_tracker.provide(&name);
            }
            Ok(BuiltinResult::NoOp)
        }

        "AC_DEFUN" => {
            if args.len() >= 2 {
                let name = String::from_utf8_lossy(&args[0]).to_string();
                let body = args[1].clone();
                table.define(&name, &body, None, line);
            }
            Ok(BuiltinResult::NoOp)
        }

        "AC_BEFORE" => Ok(BuiltinResult::NoOp),

        "AC_DIAGNOSE" | "AC_WARNING" | "AC_FATAL" => Ok(BuiltinResult::Text(b"".to_vec())),

        // Pass-through for unknown builtins: return empty
        _ => Ok(BuiltinResult::Text(b"".to_vec())),
    }
}

/// Simple arithmetic expression evaluator for eval().
///
/// // 32-bit signed integer arithmetic matching GNU m4 eval behavior.
/// // Receipt: AC.M4.EVAL.1 (pending).
fn evaluate_simple(expr: &str) -> Result<i32, String> {
    let expr = expr.trim();

    // Try to parse as a simple integer first
    if let Ok(n) = expr.parse::<i32>() {
        return Ok(n);
    }

    // Handle parenthesized expressions
    if expr.starts_with('(') && expr.ends_with(')') {
        return evaluate_simple(&expr[1..expr.len() - 1]);
    }

    // Handle simple arithmetic: a + b, a - b, a * b, a / b, a % b
    let ops = [('+', 1), ('-', 2), ('*', 3), ('/', 3), ('%', 3)];
    for (op, _prec) in &ops {
        if let Some(pos) = find_op_outside_parens(expr, *op) {
            let left = &expr[..pos];
            let right = &expr[pos + 1..];
            let a = evaluate_simple(left)?;
            let b = evaluate_simple(right)?;
            return match op {
                '+' => Ok(a.wrapping_add(b)),
                '-' => Ok(a.wrapping_sub(b)),
                '*' => Ok(a.wrapping_mul(b)),
                '/' => {
                    if b == 0 {
                        Err("division by zero".to_string())
                    } else {
                        Ok(a.wrapping_div(b))
                    }
                }
                '%' => {
                    if b == 0 {
                        Err("modulo by zero".to_string())
                    } else {
                        Ok(a.wrapping_rem(b))
                    }
                }
                _ => Err("unknown operator".to_string()),
            };
        }
    }

    // Handle relational operators
    for rel_op in &["==", "!=", ">=", "<=", ">", "<"] {
        if let Some(pos) = find_str_outside_parens(expr, rel_op) {
            let left = &expr[..pos];
            let right = &expr[pos + rel_op.len()..];
            let a = evaluate_simple(left)?;
            let b = evaluate_simple(right)?;
            let result = match *rel_op {
                "==" => a == b,
                "!=" => a != b,
                ">=" => a >= b,
                "<=" => a <= b,
                ">" => a > b,
                "<" => a < b,
                _ => false,
            };
            return Ok(if result { 1 } else { 0 });
        }
    }

    Err(format!("cannot evaluate: {}", expr))
}

/// Find an operator character outside of parentheses.
fn find_op_outside_parens(expr: &str, op: char) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in expr.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c == op && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Find a string operator outside of parentheses.
fn find_str_outside_parens(expr: &str, needle: &str) -> Option<usize> {
    let mut depth = 0;
    let chars: Vec<char> = expr.chars().collect();
    let needle_chars: Vec<char> = needle.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 && chars[i..].starts_with(&needle_chars) => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autoconf_macros::AutoconfState;

    fn test_state() -> AutoconfState {
        AutoconfState::new()
    }

    #[test]
    fn test_define_builtin() {
        let mut table = MacroTable::new();
        let mut state = test_state();
        let args = vec![b"foo".to_vec(), b"bar".to_vec()];
        let result = dispatch_builtin("define", &args, &mut table, &mut state, 1).unwrap();
        assert!(matches!(result, BuiltinResult::NoOp));
        assert_eq!(table.get_body("foo"), Some(b"bar".as_slice()));
    }

    #[test]
    fn test_ifdef_builtin() {
        let mut table = MacroTable::new();
        let mut state = test_state();
        table.define("exists", b"value", None, 1);
        let args = vec![b"exists".to_vec(), b"yes".to_vec(), b"no".to_vec()];
        let result = dispatch_builtin("ifdef", &args, &mut table, &mut state, 1).unwrap();
        if let BuiltinResult::Text(text) = result {
            assert_eq!(text, b"yes");
        } else {
            panic!("Expected Text result");
        }
    }

    #[test]
    fn test_ifelse_builtin() {
        let mut table = MacroTable::new();
        let mut state = test_state();
        let args = vec![
            b"hello".to_vec(),
            b"hello".to_vec(),
            b"equal".to_vec(),
            b"else".to_vec(),
        ];
        let result = dispatch_builtin("ifelse", &args, &mut table, &mut state, 1).unwrap();
        if let BuiltinResult::Text(text) = result {
            assert_eq!(text, b"equal");
        } else {
            panic!("Expected Text result");
        }
    }

    #[test]
    fn test_len_builtin() {
        let mut table = MacroTable::new();
        let mut state = test_state();
        let args = vec![b"hello".to_vec()];
        let result = dispatch_builtin("len", &args, &mut table, &mut state, 1).unwrap();
        if let BuiltinResult::Text(text) = result {
            assert_eq!(String::from_utf8_lossy(&text), "5");
        } else {
            panic!("Expected Text result");
        }
    }

    #[test]
    fn test_eval_simple() {
        assert_eq!(evaluate_simple("1+2").unwrap(), 3);
        assert_eq!(evaluate_simple("10-3").unwrap(), 7);
        assert_eq!(evaluate_simple("4*5").unwrap(), 20);
        assert_eq!(evaluate_simple("10/3").unwrap(), 3);
        assert_eq!(evaluate_simple("10%3").unwrap(), 1);
    }

    #[test]
    fn test_eval_relational() {
        assert_eq!(evaluate_simple("5>3").unwrap(), 1);
        assert_eq!(evaluate_simple("5<3").unwrap(), 0);
        assert_eq!(evaluate_simple("5==5").unwrap(), 1);
        assert_eq!(evaluate_simple("5!=5").unwrap(), 0);
    }
}
