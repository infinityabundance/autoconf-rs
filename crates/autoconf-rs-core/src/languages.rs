//! Language support modules for Autoconf.
//!
//! Provides compiler detection for non-C languages:
//! - Objective-C: AC_PROG_OBJC, AC_PROG_OBJCXX
//! - Erlang: AC_ERLANG_PATH_ERL, AC_ERLANG_PATH_ERLC
//! - Go: AC_PROG_GO
//!
//! These macros search standard compiler paths and set
//! language-specific variables for use in Makefiles.
//!
//! Court: AC.LANG.ALL
//! Status: Phase 5 — all language detection complete.

use m4_rs::m4_rs_core::MacroTable;

/// Register Objective-C compiler detection macros.
pub fn register_objc_macros(table: &mut MacroTable) {
    // AC_PROG_OBJC — find an Objective-C compiler
    table.define(
        b"AC_PROG_OBJC",
        b"# Check for Objective-C compiler\n\
          printf %s \"checking for Objective-C compiler... \"\n\
          ac_ct_OBJC=${OBJC}\n\
          if test -n \"$OBJC\"; then\n\
            printf '%s\\n' \"$OBJC\"\n\
          else\n\
            for ac_prog in gcc cc clang; do\n\
              if echo 'int main(){return 0;}' | $ac_prog -x objective-c -c -o /dev/null - 2>/dev/null; then\n\
                OBJC=$ac_prog\n\
                break\n\
              fi\n\
            done\n\
            printf '%s\\n' \"${OBJC-not found}\"\n\
          fi\n",
    );

    // AC_PROG_OBJCXX — find an Objective-C++ compiler
    table.define(
        b"AC_PROG_OBJCXX",
        b"# Check for Objective-C++ compiler\n\
          printf %s \"checking for Objective-C++ compiler... \"\n\
          ac_ct_OBJCXX=${OBJCXX}\n\
          if test -n \"$OBJCXX\"; then\n\
            printf '%s\\n' \"$OBJCXX\"\n\
          else\n\
            for ac_prog in g++ c++ clang++; do\n\
              if echo 'int main(){return 0;}' | $ac_prog -x objective-c++ -c -o /dev/null - 2>/dev/null; then\n\
                OBJCXX=$ac_prog\n\
                break\n\
              fi\n\
            done\n\
            printf '%s\\n' \"${OBJCXX-not found}\"\n\
          fi\n",
    );
}

/// Register Erlang compiler/interpreter detection macros.
pub fn register_erlang_macros(table: &mut MacroTable) {
    // AC_ERLANG_PATH_ERL — find Erlang interpreter
    table.define(
        b"AC_ERLANG_PATH_ERL",
        b"# Check for Erlang interpreter\n\
          printf %s \"checking for erl... \"\n\
          for ac_prog in erl; do\n\
            if command -v \"$ac_prog\" >/dev/null 2>&1; then\n\
              ERL=$ac_prog\n\
              printf '%s\\n' \"$ERL\"\n\
              break\n\
            fi\n\
          done\n",
    );

    // AC_ERLANG_PATH_ERLC — find Erlang compiler
    table.define(
        b"AC_ERLANG_PATH_ERLC",
        b"# Check for Erlang compiler\n\
          printf %s \"checking for erlc... \"\n\
          for ac_prog in erlc; do\n\
            if command -v \"$ac_prog\" >/dev/null 2>&1; then\n\
              ERLC=$ac_prog\n\
              printf '%s\\n' \"$ERLC\"\n\
              break\n\
            fi\n\
          done\n",
    );

    // AC_ERLANG_CHECK_LIB — check for Erlang library
    table.define(
        b"AC_ERLANG_CHECK_LIB",
        b"# Check for Erlang library: $1\n\
          printf %s \"checking for Erlang library $1... \"\n\
          if erl -noshell -eval 'code:lib_dir($1), halt().' 2>/dev/null; then\n\
            printf '%s\\n' \"yes\"\n\
          else\n\
            printf '%s\\n' \"no\"\n\
          fi\n",
    );

    table.define(
        b"AC_ERLANG_SUBST_ROOT_DIR",
        b"# Erlang root directory\nprintf %s \"checking for Erlang root directory... \"\nerl_root=`erl -noshell -eval 'io:format(\"~s\", [code:root_dir()]), halt().'`\nERLANG_ROOT_DIR=${ERLANG_ROOT_DIR-$erl_root}\nprintf '%s\\n' \"$ERLANG_ROOT_DIR\"\n",
    );

    table.define(
        b"AC_ERLANG_SUBST_LIB_DIR",
        b"# Erlang library directory\nERLANG_LIB_DIR=${ERLANG_LIB_DIR-$ERLANG_ROOT_DIR/lib}\nprintf '%s\\n' \"$ERLANG_LIB_DIR\"\n",
    );

    table.define(
        b"AC_ERLANG_NEED_ERL",
        b"# Verify Erlang is available\nif command -v erl >/dev/null 2>&1; then\n  printf '%s\\n' \"checking for erl... yes\"\nelse\n  printf '%s\\n' \"configure: error: Erlang not found\" >&2\n  exit 1\nfi\n",
    );

    table.define(
        b"AC_ERLANG_NEED_ERLC",
        b"# Verify Erlang compiler is available\nif command -v erlc >/dev/null 2>&1; then\n  printf '%s\\n' \"checking for erlc... yes\"\nelse\n  printf '%s\\n' \"configure: error: Erlang compiler not found\" >&2\n  exit 1\nfi\n",
    );
}

/// Register Go compiler detection macros.
pub fn register_go_macros(table: &mut MacroTable) {
    // AC_PROG_GO — find a Go compiler
    table.define(
        b"AC_PROG_GO",
        b"# Check for Go compiler\n\
          printf %s \"checking for Go compiler... \"\n\
          ac_ct_GO=${GO}\n\
          if test -n \"$GO\"; then\n\
            printf '%s\\n' \"$GO\"\n\
          else\n\
            for ac_prog in go; do\n\
              if command -v \"$ac_prog\" >/dev/null 2>&1; then\n\
                GO=$ac_prog\n\
                GOCMD=$ac_prog\n\
                printf '%s\\n' \"$GO\"\n\
                break\n\
              fi\n\
            done\n\
          fi\n\
          GOROOT=${GOROOT-`$GO env GOROOT 2>/dev/null`}\n\
          GOPATH=${GOPATH-`$GO env GOPATH 2>/dev/null`}\n",
    );

    // AC_PROG_GOC — find the Go compiler (multi-compiler probe: go, gccgo)
    table.define(
        b"AC_PROG_GOC",
        b"# Check for Go compiler\nprintf %s \"checking for Go compiler... \"\n\
          for ac_prog in go gccgo; do\n\
            if command -v \"$ac_prog\" >/dev/null 2>&1; then\n\
              GOC=$ac_prog\n\
              printf '%s\\n' \"$GOC\"\n\
              break\n\
            fi\n\
          done\n",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use m4_rs::m4_rs_core::MacroTable;

    #[test]
    fn test_objc_macros_registered() {
        let mut table = MacroTable::new();
        register_objc_macros(&mut table);
        assert!(table.lookup(b"AC_PROG_OBJC").is_some());
        assert!(table.lookup(b"AC_PROG_OBJCXX").is_some());
    }

    #[test]
    fn test_erlang_macros_registered() {
        let mut table = MacroTable::new();
        register_erlang_macros(&mut table);
        assert!(table.lookup(b"AC_ERLANG_PATH_ERL").is_some());
        assert!(table.lookup(b"AC_ERLANG_PATH_ERLC").is_some());
        assert!(table.lookup(b"AC_ERLANG_CHECK_LIB").is_some());
        assert!(table.lookup(b"AC_ERLANG_SUBST_ROOT_DIR").is_some());
        assert!(table.lookup(b"AC_ERLANG_SUBST_LIB_DIR").is_some());
        assert!(table.lookup(b"AC_ERLANG_NEED_ERL").is_some());
        assert!(table.lookup(b"AC_ERLANG_NEED_ERLC").is_some());
    }

    #[test]
    fn test_go_macros_registered() {
        let mut table = MacroTable::new();
        register_go_macros(&mut table);
        assert!(table.lookup(b"AC_PROG_GO").is_some());
        assert!(table.lookup(b"AC_PROG_GOC").is_some());
    }
}
