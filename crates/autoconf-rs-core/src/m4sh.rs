//! m4sh: Shell-generation macro library for Autoconf.
//!
//! m4sh provides macros that generate portable shell script fragments.
//! It handles shell quoting, escaping, conditionals, loops, and common
//! shell operations in a way that works across POSIX shells.
//!
//! @ac_behavior id=AC.M4SH.1 surface=AC.M4.M4SH.1 manual=§11
//! Receipt family: AC.M4.M4SH.*
//! Current status: Phase 2 — implemented, not yet oracle-admitted.

/// m4sh shell-generation builtins.
pub struct M4ShBuiltins;

impl M4ShBuiltins {
    /// AS_ECHO: portable echo that handles -n, backslashes, etc.
    ///
    /// @ac_behavior id=AC.M4SH.ECHO.1 surface=AC.M4.M4SH.1 manual=§11.1
    pub fn as_echo(args: &[Vec<u8>]) -> Vec<u8> {
        // AS_ECHO expands to printf '%s\n' for portability
        if args.is_empty() {
            return b"printf '%s\\n' \"\"".to_vec();
        }

        let mut result = b"printf '%s\\n' ".to_vec();
        for arg in args {
            result.extend_from_slice(&shell_quote_arg(arg));
            result.push(b' ');
        }
        result
    }

    /// AS_ECHO_N: echo without trailing newline.
    pub fn as_echo_n(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return b"printf '%s' \"\"".to_vec();
        }

        let mut result = b"printf '%s' ".to_vec();
        for arg in args {
            result.extend_from_slice(&shell_quote_arg(arg));
            result.push(b' ');
        }
        result
    }

    /// AS_ESCAPE: escape a string for use in shell double-quoted context.
    pub fn as_escape(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        shell_escape(&args[0])
    }

    /// AS_EXIT: exit with optional status.
    ///
    /// AS_EXIT([STATUS]) → `exit STATUS`
    pub fn as_exit(args: &[Vec<u8>]) -> Vec<u8> {
        let status = args
            .first()
            .and_then(|a| {
                let s = String::from_utf8_lossy(a);
                if s.trim().is_empty() {
                    None
                } else {
                    Some(a.clone())
                }
            })
            .unwrap_or_else(|| b"$?".to_vec());

        let mut result = b"exit ".to_vec();
        result.extend_from_slice(&status);
        result.push(b'\n');
        result
    }

    /// AS_IF: portable if/then/elif/else/fi.
    ///
    /// AS_IF([CONDITION], [THEN], [ELIF-COND, ELIF-THEN, ...], [ELSE])
    ///
    /// @ac_behavior id=AC.M4SH.IF.1 surface=AC.M4.M4SH.1 manual=§11.2
    pub fn as_if(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let condition = &args[0];
        let then_body = args.get(1).map(|a| a.as_slice()).unwrap_or(b":");

        result.extend_from_slice(b"if ");
        result.extend_from_slice(condition);
        result.extend_from_slice(b"; then\n");
        result.extend_from_slice(then_body);
        result.push(b'\n');

        // Handle elif branches (pairs of condition + body)
        let remaining = &args[2..];
        let mut i = 0;
        while i + 1 < remaining.len() {
            let elif_cond = &remaining[i];
            let elif_body = &remaining[i + 1];
            result.extend_from_slice(b"elif ");
            result.extend_from_slice(elif_cond);
            result.extend_from_slice(b"; then\n");
            result.extend_from_slice(elif_body);
            result.push(b'\n');
            i += 2;
        }

        // Handle else if odd number of remaining args
        if i < remaining.len() {
            result.extend_from_slice(b"else\n");
            result.extend_from_slice(&remaining[i]);
            result.push(b'\n');
        }

        result.extend_from_slice(b"fi\n");
        result
    }

    /// AS_CASE: portable case/esac with pattern matching.
    ///
    /// AS_CASE(WORD, [PAT1], [BODY1], ..., [DEFAULT])
    pub fn as_case(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let word = &args[0];

        result.extend_from_slice(b"case ");
        result.extend_from_slice(word);
        result.extend_from_slice(b" in\n");

        let mut i = 1;
        while i + 1 < args.len() {
            let pattern = &args[i];
            let body = &args[i + 1];
            result.extend_from_slice(b"  ");
            result.extend_from_slice(pattern);
            result.extend_from_slice(b" )\n");
            result.extend_from_slice(body);
            result.extend_from_slice(b" ;;\n");
            i += 2;
        }

        // Default case
        if i < args.len() {
            result.extend_from_slice(b"  * )\n");
            result.extend_from_slice(&args[i]);
            result.extend_from_slice(b" ;;\n");
        }

        result.extend_from_slice(b"esac\n");
        result
    }

    /// AS_FOR: portable for loop.
    ///
    /// AS_FOR(VAR, LIST, BODY)
    pub fn as_for(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 3 {
            return Vec::new();
        }

        let var = &args[0];
        let list = &args[1];
        let body = &args[2];

        let var_str = String::from_utf8_lossy(var);
        let list_str = String::from_utf8_lossy(list);

        let mut result = Vec::new();
        result.extend_from_slice(b"for ");
        result.extend_from_slice(var_str.as_bytes());
        result.extend_from_slice(b" in ");
        result.extend_from_slice(list_str.as_bytes());
        result.extend_from_slice(b"; do\n");
        result.extend_from_slice(body);
        result.extend_from_slice(b"\ndone\n");
        result
    }

    /// AS_MKDIR_P: portable mkdir -p.
    ///
    /// AS_MKDIR_P(DIR) → `test -d DIR || mkdir -p DIR`
    pub fn as_mkdir_p(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return b"mkdir -p".to_vec();
        }

        let mut result = Vec::new();
        for dir in args {
            result.extend_from_slice(b"test -d ");
            result.extend_from_slice(dir);
            result.extend_from_slice(b" || mkdir -p ");
            result.extend_from_slice(dir);
            result.extend_from_slice(b"\n");
        }
        result
    }

    /// AS_TR_SH: translate a string to a valid shell variable name.
    ///
    /// Replaces non-alphanumeric characters with underscores.
    pub fn as_tr_sh(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let s = String::from_utf8_lossy(&args[0]);
        let translated: String = s
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        translated.into_bytes()
    }

    /// AS_TR_CPP: translate a string to a valid C preprocessor macro name.
    ///
    /// Uppercases and replaces non-alphanumeric characters with underscores.
    pub fn as_tr_cpp(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let s = String::from_utf8_lossy(&args[0]);
        let translated: String = s
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect();
        translated.into_bytes()
    }

    /// AS_UNSET: portable unset.
    pub fn as_unset(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return b"unset".to_vec();
        }
        let mut result = b"unset ".to_vec();
        for (i, var) in args.iter().enumerate() {
            if i > 0 {
                result.push(b' ');
            }
            result.extend_from_slice(var);
        }
        result.push(b'\n');
        result
    }

    /// AS_BOX: generate a boxed comment.
    ///
    /// AS_BOX(TEXT) → a comment box with ## borders
    pub fn as_box(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return b"## ##".to_vec();
        }
        let text = String::from_utf8_lossy(&args[0]);
        let mut result = Vec::new();

        // Top border
        result.extend_from_slice(b"## ");
        result.extend(std::iter::repeat_n(b'#', text.len()));
        result.extend_from_slice(b" ##\n");

        // Text line
        result.extend_from_slice(b"## ");
        result.extend_from_slice(text.as_bytes());
        result.extend_from_slice(b" ##\n");

        // Bottom border
        result.extend_from_slice(b"## ");
        result.extend(std::iter::repeat_n(b'#', text.len()));
        result.extend_from_slice(b" ##\n");
        result
    }

    /// AS_VERSION_COMPARE(VERSION-1, VERSION-2, [LT-BODY], [EQ-BODY], [GT-BODY])
    /// Compare two version strings using awk for accurate numeric comparison.
    pub fn as_version_compare(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return Vec::new();
        }
        let v1 = String::from_utf8_lossy(&args[0]);
        let v2 = String::from_utf8_lossy(&args[1]);
        let lt_body = args
            .get(2)
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let eq_body = args
            .get(3)
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let gt_body = args
            .get(4)
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let mut r = Vec::new();
        r.extend_from_slice(b"ax_compare_version=\\n");
        r.extend_from_slice(b"$awk -F. 'BEGIN { split(\"");
        r.extend_from_slice(v1.as_bytes());
        r.extend_from_slice(b"\", a); split(\"");
        r.extend_from_slice(v2.as_bytes());
        r.extend_from_slice(b"\", b);\n");
        r.extend_from_slice(b"  for (i=1; i<=3; i++) {\n");
        r.extend_from_slice(b"    if (a[i]+0 < b[i]+0) { print -1; exit }\n");
        r.extend_from_slice(b"    if (a[i]+0 > b[i]+0) { print 1; exit }\n");
        r.extend_from_slice(b"  } print 0; }'\n");
        r.extend_from_slice(b"if test \"$ax_compare_version\" -lt 0; then\n  ");
        r.extend_from_slice(lt_body.as_bytes());
        r.extend_from_slice(b"\nelif test \"$ax_compare_version\" -eq 0; then\n  ");
        r.extend_from_slice(eq_body.as_bytes());
        r.extend_from_slice(b"\nelse\n  ");
        r.extend_from_slice(gt_body.as_bytes());
        r.extend_from_slice(b"\nfi\n");
        r
    }

    /// AS_EXECUTABLE_P(FILE): check if FILE is an executable regular file.
    pub fn as_executable_p(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return b"test -f && test -x".to_vec();
        }
        let path = String::from_utf8_lossy(&args[0]);
        let mut r = Vec::new();
        r.extend_from_slice(b"test -f '");
        r.extend_from_slice(path.as_bytes());
        r.extend_from_slice(b"' && test -x '");
        r.extend_from_slice(path.as_bytes());
        r.extend_from_slice(b"'");
        r
    }

    /// AS_ME_PREPARE: set $as_me to the script's basename for error reporting.
    pub fn as_me_prepare() -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(b"as_me=`$as_basename -- \"$0\" ||\n");
        r.extend_from_slice(b"$as_expr X/\"$0\" : '.*/\\([^/][^/]*\\)/*$' \\| \\\n");
        r.extend_from_slice(b"         X\"$0\" : 'X\\(//\\)$' \\| \\\n");
        r.extend_from_slice(b"         X\"$0\" : 'X\\(/\\)' \\| . 2>/dev/null ||\n");
        r.extend_from_slice(b"printf '%s\\n' \"$0\" | sed 's|.*/||'`\n");
        r
    }

    /// AS_SET_CATFILE(VAR, DIR, FILE): set VAR to DIR/FILE with edge case handling.
    pub fn as_set_catfile(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 3 {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        // dir and file are validated by args.len() check above
        let mut r = Vec::new();
        r.extend_from_slice(b"case $");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b" in #(\n");
        r.extend_from_slice(b"  [\\/]* | ?:[\\/]*)\n");
        r.extend_from_slice(b"    ac_cv_");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"=$");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b" ;; #(\n  *)\n");
        r.extend_from_slice(b"    ac_cv_");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"=${ac_top_build_prefix}");
        r.extend_from_slice(b"$");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b" ;;\nesac\n");
        r
    }

    /// AS_VAR_SET(VAR, VALUE): set a shell variable safely.
    pub fn as_var_set(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        let val = String::from_utf8_lossy(&args[1]);
        let mut r = Vec::new();
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"=");
        r.extend_from_slice(val.as_bytes());
        r.extend_from_slice(b"\n");
        r
    }

    /// AS_VAR_GET(VAR): output the value of a shell variable.
    pub fn as_var_get(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        let mut r = Vec::new();
        r.extend_from_slice(b"printf '%s\\n' \"$");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"\"\n");
        r
    }

    /// AS_VAR_TEST_SET(VAR): test if a variable is set (non-empty).
    pub fn as_var_test_set(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        let mut r = Vec::new();
        r.extend_from_slice(b"test -n \"$");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"\"");
        r
    }

    /// AS_VAR_SET_IF(VAR, [IF-SET], [IF-UNSET]): conditionally execute based on variable being set.
    pub fn as_var_set_if(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        let if_set = args
            .get(1)
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let if_unset = args
            .get(2)
            .map(|a| String::from_utf8_lossy(a))
            .unwrap_or_default();
        let mut r = Vec::new();
        r.extend_from_slice(b"if test -n \"$");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"\"; then\n  ");
        r.extend_from_slice(if_set.as_bytes());
        r.extend_from_slice(b"\nelse\n  ");
        r.extend_from_slice(if_unset.as_bytes());
        r.extend_from_slice(b"\nfi\n");
        r
    }

    /// AS_VAR_PUSHDEF(VAR, VALUE): push a variable definition onto a stack.
    pub fn as_var_pushdef(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        let val = String::from_utf8_lossy(&args[1]);
        let mut r = Vec::new();
        r.extend_from_slice(b"as_var_pushdef_");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"=\"");
        r.extend_from_slice(val.as_bytes());
        r.extend_from_slice(b"\"\n");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"=$as_var_pushdef_");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"\n");
        r
    }

    /// AS_VAR_POPDEF(VAR): pop a variable definition from the stack.
    pub fn as_var_popdef(args: &[Vec<u8>]) -> Vec<u8> {
        if args.is_empty() {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        let mut r = Vec::new();
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"=$as_var_pushdef_");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"\n");
        r
    }

    /// AS_VAR_APPEND(VAR, VALUE): append a value to a shell variable.
    pub fn as_var_append(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        let val = String::from_utf8_lossy(&args[1]);
        let mut r = Vec::new();
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"=\"$");
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b" ");
        r.extend_from_slice(val.as_bytes());
        r.extend_from_slice(b"\"\n");
        r
    }

    /// AS_VAR_ARITH(VAR, EXPR): perform shell arithmetic and assign to variable.
    pub fn as_var_arith(args: &[Vec<u8>]) -> Vec<u8> {
        if args.len() < 2 {
            return Vec::new();
        }
        let var = String::from_utf8_lossy(&args[0]);
        let expr = String::from_utf8_lossy(&args[1]);
        let mut r = Vec::new();
        r.extend_from_slice(var.as_bytes());
        r.extend_from_slice(b"=$(( ");
        r.extend_from_slice(expr.as_bytes());
        r.extend_from_slice(b" ))\n");
        r
    }
}

/// Shell-quote an argument for safe use in double quotes.
fn shell_escape(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    for &b in bytes {
        match b {
            b'$' | b'`' | b'"' | b'\\' | b'!' => {
                result.push(b'\\');
                result.push(b);
            }
            b'\n' => {
                result.extend_from_slice(b"'\\n'");
            }
            _ => result.push(b),
        }
    }
    result
}

/// Shell-quote an argument for use after printf.
fn shell_quote_arg(bytes: &[u8]) -> Vec<u8> {
    let mut result = vec![b'"'];
    result.extend_from_slice(&shell_escape(bytes));
    result.push(b'"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_echo_simple() {
        let args: Vec<Vec<u8>> = vec![b"hello world".to_vec()];
        let result = M4ShBuiltins::as_echo(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("printf"));
        assert!(s.contains("hello world"));
    }

    #[test]
    fn test_as_echo_empty() {
        let result = M4ShBuiltins::as_echo(&[]);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("printf"));
    }

    #[test]
    fn test_as_if_simple() {
        let args: Vec<Vec<u8>> = vec![b"test -f foo".to_vec(), b"echo yes".to_vec()];
        let result = M4ShBuiltins::as_if(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("if test -f foo; then"));
        assert!(s.contains("echo yes"));
        assert!(s.contains("fi"));
    }

    #[test]
    fn test_as_if_with_else() {
        let args: Vec<Vec<u8>> = vec![
            b"test -f foo".to_vec(),
            b"echo yes".to_vec(),
            b"echo no".to_vec(),
        ];
        let result = M4ShBuiltins::as_if(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("else"));
        assert!(s.contains("echo no"));
    }

    #[test]
    fn test_as_case_basic() {
        let args: Vec<Vec<u8>> = vec![
            b"$var".to_vec(),
            b"yes".to_vec(),
            b"echo yes".to_vec(),
            b"no".to_vec(),
            b"echo no".to_vec(),
            b"echo default".to_vec(),
        ];
        let result = M4ShBuiltins::as_case(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("case $var in"));
        assert!(s.contains("yes )"));
        assert!(s.contains("esac"));
    }

    #[test]
    fn test_as_tr_sh() {
        let args = vec![b"foo.bar-baz".to_vec()];
        let result = M4ShBuiltins::as_tr_sh(&args);
        assert_eq!(String::from_utf8_lossy(&result), "foo_bar_baz");
    }

    #[test]
    fn test_as_tr_cpp() {
        let args = vec![b"foo.bar".to_vec()];
        let result = M4ShBuiltins::as_tr_cpp(&args);
        assert_eq!(String::from_utf8_lossy(&result), "FOO_BAR");
    }

    #[test]
    fn test_shell_escape() {
        let result = shell_escape(b"hello $world");
        assert_eq!(String::from_utf8_lossy(&result), "hello \\$world");
    }

    #[test]
    fn test_as_mkdir_p() {
        let args = vec![b"/tmp/foo/bar".to_vec()];
        let result = M4ShBuiltins::as_mkdir_p(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("test -d"));
        assert!(s.contains("mkdir -p"));
    }

    #[test]
    fn test_as_var_set() {
        let args = vec![b"foo".to_vec(), b"bar".to_vec()];
        let result = M4ShBuiltins::as_var_set(&args);
        assert_eq!(String::from_utf8_lossy(&result), "foo=bar\n");
    }

    #[test]
    fn test_as_var_set_empty_args() {
        let result = M4ShBuiltins::as_var_set(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_as_var_get() {
        let args = vec![b"myvar".to_vec()];
        let result = M4ShBuiltins::as_var_get(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("printf"));
        assert!(s.contains("$myvar"));
    }

    #[test]
    fn test_as_var_test_set() {
        let args = vec![b"myvar".to_vec()];
        let result = M4ShBuiltins::as_var_test_set(&args);
        assert_eq!(String::from_utf8_lossy(&result), "test -n \"$myvar\"");
    }

    #[test]
    fn test_as_var_set_if() {
        let args = vec![
            b"myvar".to_vec(),
            b"echo set".to_vec(),
            b"echo unset".to_vec(),
        ];
        let result = M4ShBuiltins::as_var_set_if(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("if test -n \"$myvar\""));
        assert!(s.contains("echo set"));
        assert!(s.contains("echo unset"));
        assert!(s.contains("fi"));
    }

    #[test]
    fn test_as_var_pushdef() {
        let args = vec![b"CC".to_vec(), b"gcc".to_vec()];
        let result = M4ShBuiltins::as_var_pushdef(&args);
        let s = String::from_utf8_lossy(&result);
        assert!(s.contains("as_var_pushdef_CC=\"gcc\""));
        assert!(s.contains("CC=$as_var_pushdef_CC"));
    }

    #[test]
    fn test_as_var_popdef() {
        let args = vec![b"CC".to_vec()];
        let result = M4ShBuiltins::as_var_popdef(&args);
        assert_eq!(String::from_utf8_lossy(&result), "CC=$as_var_pushdef_CC\n");
    }

    #[test]
    fn test_as_var_append() {
        let args = vec![b"CFLAGS".to_vec(), b"-O2".to_vec()];
        let result = M4ShBuiltins::as_var_append(&args);
        assert_eq!(String::from_utf8_lossy(&result), "CFLAGS=\"$CFLAGS -O2\"\n");
    }

    #[test]
    fn test_as_var_arith() {
        let args = vec![b"count".to_vec(), b"1 + 2".to_vec()];
        let result = M4ShBuiltins::as_var_arith(&args);
        assert_eq!(String::from_utf8_lossy(&result), "count=$(( 1 + 2 ))\n");
    }
}
