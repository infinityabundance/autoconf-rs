//! M4sh initialization -- shell portability preamble.
//!
//! Generates the full configure script prologue: shebang, header,
//! M4sh initialization (shell portability, variable reset, locale,
//! self-location, CONFIG_SHELL re-exec with neutralization,
//! shell capability probing (as_required/as_suggested),
//! shell search loop, echo detection, mkdir_p detection,
//! test -x detection, LINENO setup),
//! M4sh shell functions (as_fn_*), and sanitization.
//!
//! @ac_behavior id=AC.M4SH.INIT.1 surface=AC.M4.M4SH.1 manual=11
//! Phase 3 -- converging toward oracle byte-exact match

pub fn generate_m4sh_init(_pn: &str, _pv: &str) -> Vec<u8> {
    let mut i = Vec::new();

    // Shell portability
    i.extend_from_slice(b"\n# Be more Bourne compatible\n");
    i.extend_from_slice(b"DUALCASE=1; export DUALCASE # for MKS sh\n");
    // Create confdefs.h as the VERY FIRST thing so every compile probe's `cat confdefs.h - <<EOF`
    // works — even AC_CHECK_LIB/AC_COMPILE_IFELSE emitted early (before the standard confdefs init).
    // `cat: confdefs.h: No such file` was the single most common corpus failure (36/138 repos).
    i.extend_from_slice(b"test -f confdefs.h || printf '%s\\n' '/* confdefs.h */' > confdefs.h\n");
    i.extend_from_slice(b"if test ${ZSH_VERSION+y} && (emulate sh) >/dev/null 2>&1\n");
    i.extend_from_slice(b"then :\n  emulate sh\n  NULLCMD=:\n");
    i.extend_from_slice(b"  # Pre-4.2 versions of Zsh do word splitting on ${1+\"$@\"}, which\n");
    i.extend_from_slice(b"  # contradicts POSIX and common usage.  Disable this.\n");
    i.extend_from_slice(b"  alias -g '${1+\"$@\"}'='\"$@\"'\n  setopt NO_GLOB_SUBST\n");
    i.extend_from_slice(b"else case e in #(\n");
    i.extend_from_slice(b"  e) case `(set -o) 2>/dev/null` in #(\n");
    i.extend_from_slice(b"  *posix*) :\n    set -o posix ;; #(\n");
    i.extend_from_slice(b"  *) :\n     ;;\nesac ;;\nesac\nfi\n\n");

    // Reset variables
    i.extend_from_slice(
        b"# Reset variables that may have inherited troublesome values from\n# the environment.\n",
    );
    i.extend_from_slice(
        b"# IFS needs to be set, to space, tab, and newline, in precisely that order.\n",
    );
    i.extend_from_slice(b"as_nl='\n'\nexport as_nl\n");
    i.extend_from_slice(b"IFS=\" \"\"\t$as_nl\"\n\n");
    i.extend_from_slice(b"PS1='$ '\nPS2='> '\nPS4='+ '\n\n");

    // Locale
    i.extend_from_slice(
        b"# Ensure predictable behavior from utilities with locale-dependent output.\n",
    );
    i.extend_from_slice(b"LC_ALL=C\nexport LC_ALL\nLANGUAGE=C\nexport LANGUAGE\n\n");

    // Unset dangerous variables
    i.extend_from_slice(
        b"# We cannot yet rely on \"unset\" to work, but we need these variables\n",
    );
    i.extend_from_slice(b"# to be unset to avoid bugs in old shells (e.g. pre-3.0 UWIN ksh).\n");
    i.extend_from_slice(b"for as_var in BASH_ENV ENV MAIL MAILPATH CDPATH\n");
    i.extend_from_slice(b"do eval test \\${$as_var+y} \\\n");
    i.extend_from_slice(
        b"  && ( (unset $as_var) || exit 1) >/dev/null 2>&1 && unset $as_var || :\n",
    );
    i.extend_from_slice(b"done\n\n");

    // Ensure FDs open
    i.extend_from_slice(b"# Ensure that fds 0, 1, and 2 are open.\n");
    i.extend_from_slice(b"if (exec 3>&0) 2>/dev/null; then :; else exec 0</dev/null; fi\n");
    i.extend_from_slice(b"if (exec 3>&1) 2>/dev/null; then :; else exec 1>/dev/null; fi\n");
    i.extend_from_slice(b"if (exec 3>&2)            ; then :; else exec 2>/dev/null; fi\n\n");

    // PATH_SEPARATOR
    i.extend_from_slice(b"# The user is always right.\n");
    i.extend_from_slice(b"if ${PATH_SEPARATOR+false} :; then\n  PATH_SEPARATOR=:\n");
    i.extend_from_slice(b"  (PATH='/bin;/bin'; FPATH=$PATH; sh -c :) >/dev/null 2>&1 && {\n");
    i.extend_from_slice(b"    (PATH='/bin:/bin'; FPATH=$PATH; sh -c :) >/dev/null 2>&1 ||\n      PATH_SEPARATOR=';'\n  }\nfi\n\n");

    // Self-location
    i.extend_from_slice(
        b"# Find who we are.  Look in the path if we contain no directory separator.\n",
    );
    i.extend_from_slice(b"as_myself=\ncase ${0} in #((\n");
    i.extend_from_slice(b"  *[\\\\/]* ) as_myself=${0} ;;\n");
    i.extend_from_slice(b"  *) as_save_IFS=$IFS; IFS=$PATH_SEPARATOR\n");
    i.extend_from_slice(b"for as_dir in $PATH\ndo\n  IFS=$as_save_IFS\n");
    i.extend_from_slice(b"  case $as_dir in #(((\n    '') as_dir=./ ;;\n    */) ;;\n");
    i.extend_from_slice(b"    *) as_dir=$as_dir/ ;;\n  esac\n");
    i.extend_from_slice(b"    test -r \"$as_dir${0}\" && as_myself=$as_dir${0} && break\n");
    i.extend_from_slice(b"  done\nIFS=$as_save_IFS\n\n     ;;\nesac\n");
    i.extend_from_slice(
        b"# We did not find ourselves, most probably we were run as 'sh COMMAND'\n",
    );
    i.extend_from_slice(b"# in which case we are not to be found in the path.\n");
    i.extend_from_slice(b"if test \"x$as_myself\" = x; then\n  as_myself=${0}\nfi\n");
    i.extend_from_slice(b"if test ! -f \"$as_myself\"; then\n");
    i.extend_from_slice(b"  printf '%s\\n' \"$as_myself: error: cannot find myself\" >&2\n");
    i.extend_from_slice(b"  exit 1\nfi\n\n");

    // === CONFIG_SHELL re-exec with neutralization ===
    generate_shell_reexec(&mut i);

    // === Shell capability probing (as_required / as_suggested) ===
    generate_shell_probe(&mut i);

    // as_echo, as_mkdir_p, as_test_x detection
    generate_tool_detection(&mut i);

    // LINENO setup (simplified — portability for shells without LINENO)
    i.extend_from_slice(b"as_lineno_1=$LINENO as_lineno_1a=$LINENO\n");
    i.extend_from_slice(b"as_lineno_2=$LINENO as_lineno_2a=$LINENO\n");
    i.extend_from_slice(b"test \"x$as_lineno_1\" != \"x$as_lineno_2\" ||\n");
    i.extend_from_slice(b"  test \"x`expr $as_lineno_1 + 1`\" = \"x$as_lineno_2\" || {\n");
    i.extend_from_slice(b"  LINENO=1\n}\n\n");
    i.extend_from_slice(b"as_run='\"\"'\n\n");

    i
}

/// CONFIG_SHELL re-exec: if CONFIG_SHELL is set, re-execute under that shell.
/// Includes environment neutralization and option preservation.
fn generate_shell_reexec(i: &mut Vec<u8>) {
    i.extend_from_slice(b"# Use a proper internal environment variable to ensure we don't fall\n");
    i.extend_from_slice(b"  # into an infinite loop, continuously re-executing ourselves.\n");
    i.extend_from_slice(
        b"  if test x\"${_as_can_reexec}\" != xno && test \"x$CONFIG_SHELL\" != x; then\n",
    );
    i.extend_from_slice(b"    _as_can_reexec=no; export _as_can_reexec;\n");
    i.extend_from_slice(b"    # We cannot yet assume a decent shell, so we have to provide a\n");
    i.extend_from_slice(b"# neutralization value for shells without unset; and this also\n");
    i.extend_from_slice(b"# works around shells that cannot unset nonexistent variables.\n");
    i.extend_from_slice(b"# Preserve -v and -x to the replacement shell.\n");
    i.extend_from_slice(b"BASH_ENV=/dev/null\n");
    i.extend_from_slice(b"ENV=/dev/null\n");
    i.extend_from_slice(b"(unset BASH_ENV) >/dev/null 2>&1 && unset BASH_ENV ENV\n");
    i.extend_from_slice(b"case $- in # ((((\n");
    i.extend_from_slice(b"  *v*x* | *x*v* ) as_opts=-vx ;;\n");
    i.extend_from_slice(b"  *v* ) as_opts=-v ;;\n");
    i.extend_from_slice(b"  *x* ) as_opts=-x ;;\n");
    i.extend_from_slice(b"  * ) as_opts= ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"case ${#} in # ((\n");
    i.extend_from_slice(b"  0) exec $CONFIG_SHELL $as_opts \"$as_myself\" ;;\n");
    i.extend_from_slice(b"  *) exec $CONFIG_SHELL $as_opts \"$as_myself\" \"$@\" ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"# Admittedly, this is quite paranoid, since all the known shells bail\n");
    i.extend_from_slice(b"# out after a failed 'exec'.\n");
    i.extend_from_slice(b"printf '%s\\n' \"${0}: could not re-execute with $CONFIG_SHELL\" >&2\n");
    i.extend_from_slice(b"exit 255\n");
    i.extend_from_slice(b"  fi\n");
    i.extend_from_slice(b"  # We don't want this to propagate to other subprocesses.\n");
    i.extend_from_slice(b"          { _as_can_reexec=; unset _as_can_reexec;}\n");
}

/// Shell capability probing: test if the current shell supports functions,
/// return, positional parameter saving, command substitution nesting, and
/// test -x. If not, search for a better shell.
///
/// This is the as_bourne_compatible / as_required / as_suggested mechanism
/// from GNU Autoconf's M4sh initialization. It defines shell function
/// wrappers for testing and uses them to verify the shell is capable enough
/// to run the configure script.
fn generate_shell_probe(i: &mut Vec<u8>) {
    i.extend_from_slice(b"if test \"x$CONFIG_SHELL\" = x; then\n");
    i.extend_from_slice(
        b"  as_bourne_compatible=\"if test \\${ZSH_VERSION+y} && (emulate sh) >/dev/null 2>&1\n",
    );
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"  emulate sh\n");
    i.extend_from_slice(b"  NULLCMD=:\n");
    i.extend_from_slice(
        b"  # Pre-4.2 versions of Zsh do word splitting on \\${1+\\\"\\$@\\\"}, which\n",
    );
    i.extend_from_slice(b"  # contradicts POSIX and common usage.  Disable this.\n");
    i.extend_from_slice(b"  alias -g '\\${1+\\\"\\$@\\\"}'='\\\"\\$@\\\"'\n");
    i.extend_from_slice(b"  setopt NO_GLOB_SUBST\n");
    i.extend_from_slice(b"else case e in #(\n");
    i.extend_from_slice(b"  e) case \\`(set -o) 2>/dev/null\\` in #(\n");
    i.extend_from_slice(b"  *posix*) :\n");
    i.extend_from_slice(b"    set -o posix ;; #(\n");
    i.extend_from_slice(b"  *) :\n");
    i.extend_from_slice(b"     ;;\n");
    i.extend_from_slice(b"esac ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"fi\n");
    i.extend_from_slice(b"\"\n");

    // as_required: mandatory shell features the configure script needs
    i.extend_from_slice(b"  as_required=\"as_fn_return () { (exit \\$1); }\n");
    i.extend_from_slice(b"as_fn_success () { as_fn_return 0; }\n");
    i.extend_from_slice(b"as_fn_failure () { as_fn_return 1; }\n");
    i.extend_from_slice(b"as_fn_ret_success () { return 0; }\n");
    i.extend_from_slice(b"as_fn_ret_failure () { return 1; }\n");
    i.extend_from_slice(b"\n");
    i.extend_from_slice(b"exitcode=0\n");
    i.extend_from_slice(b"as_fn_success || { exitcode=1; echo as_fn_success failed.; }\n");
    i.extend_from_slice(b"as_fn_failure && { exitcode=1; echo as_fn_failure succeeded.; }\n");
    i.extend_from_slice(b"as_fn_ret_success || { exitcode=1; echo as_fn_ret_success failed.; }\n");
    i.extend_from_slice(
        b"as_fn_ret_failure && { exitcode=1; echo as_fn_ret_failure succeeded.; }\n",
    );
    i.extend_from_slice(b"if ( set x; as_fn_ret_success y && test x = \\\"\\$1\\\" )\n");
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"\n");
    i.extend_from_slice(b"else case e in #(\n");
    i.extend_from_slice(b"  e) exitcode=1; echo positional parameters were not saved. ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"fi\n");
    i.extend_from_slice(b"test x\\$exitcode = x0 || exit 1\n");
    i.extend_from_slice(b"blah=\\$(echo \\$(echo blah))\n");
    i.extend_from_slice(b"test x\\\"\\$blah\\\" = xblah || exit 1\n");
    i.extend_from_slice(b"test -x / || exit 1\"\n");

    // as_suggested: desirable shell features (LINENO support)
    i.extend_from_slice(b"  as_suggested=\"  as_lineno_1=\";as_suggested=$as_suggested$LINENO;as_suggested=$as_suggested\" as_lineno_1a=\\$LINENO\n");
    i.extend_from_slice(b"  as_lineno_2=\";as_suggested=$as_suggested$LINENO;as_suggested=$as_suggested\" as_lineno_2a=\\$LINENO\n");
    i.extend_from_slice(b"  eval 'test \\\"x\\$as_lineno_1'\\$as_run'\\\" != \\\"x\\$as_lineno_2'\\$as_run'\\\" &&\n");
    i.extend_from_slice(b"  test \\\"x\\`expr \\$as_lineno_1'\\$as_run' + 1\\`\\\" = \\\"x\\$as_lineno_2'\\$as_run'\\\"' || exit 1\"\n");

    // Test required capabilities
    i.extend_from_slice(b"  if (eval \"$as_required\") 2>/dev/null\n");
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"  as_have_required=yes\n");
    i.extend_from_slice(b"else case e in #(\n");
    i.extend_from_slice(b"  e) as_have_required=no ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"fi\n");

    // Test suggested capabilities; if they fail, search for a better shell
    i.extend_from_slice(
        b"  if test x$as_have_required = xyes && (eval \"$as_suggested\") 2>/dev/null\n",
    );
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"\n");
    i.extend_from_slice(b"else case e in #(\n");
    i.extend_from_slice(b"  e) as_save_IFS=$IFS; IFS=$PATH_SEPARATOR\n");
    i.extend_from_slice(b"as_found=false\n");
    i.extend_from_slice(b"for as_dir in /bin$PATH_SEPARATOR/usr/bin$PATH_SEPARATOR$PATH\n");
    i.extend_from_slice(b"do\n");
    i.extend_from_slice(b"  IFS=$as_save_IFS\n");
    i.extend_from_slice(b"  case $as_dir in #(((\n");
    i.extend_from_slice(b"    '') as_dir=./ ;;\n");
    i.extend_from_slice(b"    */) ;;\n");
    i.extend_from_slice(b"    *) as_dir=$as_dir/ ;;\n");
    i.extend_from_slice(b"  esac\n");
    i.extend_from_slice(b"  as_found=:\n");
    i.extend_from_slice(b"  case $as_dir in #(\n");
    i.extend_from_slice(b"\t /*)\n");
    i.extend_from_slice(b"\t   for as_base in sh bash ksh sh5; do\n");
    i.extend_from_slice(b"\t     # Try only shells that exist, to save several forks.\n");
    i.extend_from_slice(b"\t     as_shell=$as_dir$as_base\n");
    i.extend_from_slice(b"\t     if { test -f \"$as_shell\" || test -f \"$as_shell.exe\"; } &&\n");
    i.extend_from_slice(b"\t\t    as_run=a \"$as_shell\" -c \"$as_bourne_compatible\"\"$as_required\" 2>/dev/null\n");
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"  CONFIG_SHELL=$as_shell as_have_required=yes\n");
    i.extend_from_slice(b"\t\t   if as_run=a \"$as_shell\" -c \"$as_bourne_compatible\"\"$as_suggested\" 2>/dev/null\n");
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"  break 2\n");
    i.extend_from_slice(b"fi\n");
    i.extend_from_slice(b"fi\n");
    i.extend_from_slice(b"\t   done;;\n");
    i.extend_from_slice(b"       esac\n");
    i.extend_from_slice(b"  as_found=false\n");
    i.extend_from_slice(b"done\n");
    i.extend_from_slice(b"IFS=$as_save_IFS\n");
    i.extend_from_slice(b"if $as_found\n");
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"\n");
    i.extend_from_slice(b"else case e in #(\n");
    i.extend_from_slice(b"  e) if { test -f \"$SHELL\" || test -f \"$SHELL.exe\"; } &&\n");
    i.extend_from_slice(
        b"\t      as_run=a \"$SHELL\" -c \"$as_bourne_compatible\"\"$as_required\" 2>/dev/null\n",
    );
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"  CONFIG_SHELL=$SHELL as_have_required=yes\n");
    i.extend_from_slice(b"fi ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"fi\n");
    i.extend_from_slice(b"\n");
    i.extend_from_slice(b"\n");

    // If we found a better shell, re-exec under it
    i.extend_from_slice(b"      if test \"x$CONFIG_SHELL\" != x\n");
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(b"  export CONFIG_SHELL\n");
    i.extend_from_slice(
        b"             # We cannot yet assume a decent shell, so we have to provide a\n",
    );
    i.extend_from_slice(b"# neutralization value for shells without unset; and this also\n");
    i.extend_from_slice(b"# works around shells that cannot unset nonexistent variables.\n");
    i.extend_from_slice(b"# Preserve -v and -x to the replacement shell.\n");
    i.extend_from_slice(b"BASH_ENV=/dev/null\n");
    i.extend_from_slice(b"ENV=/dev/null\n");
    i.extend_from_slice(b"(unset BASH_ENV) >/dev/null 2>&1 && unset BASH_ENV ENV\n");
    i.extend_from_slice(b"case $- in # ((((\n");
    i.extend_from_slice(b"  *v*x* | *x*v* ) as_opts=-vx ;;\n");
    i.extend_from_slice(b"  *v* ) as_opts=-v ;;\n");
    i.extend_from_slice(b"  *x* ) as_opts=-x ;;\n");
    i.extend_from_slice(b"  * ) as_opts= ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"case ${#} in # ((\n");
    i.extend_from_slice(b"  0) exec $CONFIG_SHELL $as_opts \"$as_myself\" ;;\n");
    i.extend_from_slice(b"  *) exec $CONFIG_SHELL $as_opts \"$as_myself\" \"$@\" ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"# Admittedly, this is quite paranoid, since all the known shells bail\n");
    i.extend_from_slice(b"# out after a failed 'exec'.\n");
    i.extend_from_slice(b"printf '%s\\n' \"${0}: could not re-execute with $CONFIG_SHELL\" >&2\n");
    i.extend_from_slice(b"exit 255\n");
    i.extend_from_slice(b"fi\n");
    i.extend_from_slice(b"\n");

    // If no suitable shell found, print error and exit
    i.extend_from_slice(b"    if test x$as_have_required = xno\n");
    i.extend_from_slice(b"then :\n");
    i.extend_from_slice(
        b"  printf '%s\\n' \"${0}: This script requires a shell more modern than all\"\n",
    );
    i.extend_from_slice(b"  printf '%s\\n' \"${0}: the shells that I found on your system.\"\n");
    i.extend_from_slice(b"  if test ${ZSH_VERSION+y} ; then\n");
    i.extend_from_slice(
        b"    printf '%s\\n' \"${0}: In particular, zsh $ZSH_VERSION has bugs and should\"\n",
    );
    i.extend_from_slice(b"    printf '%s\\n' \"${0}: be upgraded to zsh 4.3.4 or later.\"\n");
    i.extend_from_slice(b"  else\n");
    i.extend_from_slice(
        b"    printf '%s\\n' \"${0}: Please tell bug-autoconf@gnu.org about your system,\n",
    );
    i.extend_from_slice(b"${0}: including any error possibly output before this\n");
    i.extend_from_slice(b"${0}: message. Then install a modern shell, or manually run\n");
    i.extend_from_slice(b"${0}: the script under such a shell if you do have one.\"\n");
    i.extend_from_slice(b"  fi\n");
    i.extend_from_slice(b"  exit 1\n");
    i.extend_from_slice(b"fi ;;\n");
    i.extend_from_slice(b"esac\n");
    i.extend_from_slice(b"fi\n");
    i.extend_from_slice(b"fi\n");

    // Export SHELL and clean up
    i.extend_from_slice(b"SHELL=${CONFIG_SHELL-/bin/sh}\n");
    i.extend_from_slice(b"export SHELL\n");
    i.extend_from_slice(
        b"# Unset more variables known to interfere with behavior of common tools.\n",
    );
    i.extend_from_slice(b"CLICOLOR_FORCE= GREP_OPTIONS=\n");
    i.extend_from_slice(b"unset CLICOLOR_FORCE GREP_OPTIONS\n\n");
    i.extend_from_slice(b"## --------------------- ##\n");
    i.extend_from_slice(b"## M4sh Shell Functions. ##\n");
    i.extend_from_slice(b"## --------------------- ##\n");
}

/// Tool detection: as_echo, as_mkdir_p, as_test_x, as_executable_p
fn generate_tool_detection(i: &mut Vec<u8>) {
    // as_echo detection
    i.extend_from_slice(b"# Find a good echo command.\n");
    i.extend_from_slice(b"if (test \"X`printf %s '\\043'`\" = \"X#\") 2>/dev/null; then :\n");
    i.extend_from_slice(b"  as_echo='printf %s\\n'\n  as_echo_n='printf %s'\nelse :\n");
    i.extend_from_slice(
        b"  if test \"X`(/usr/ucb/echo -n -n '\\043') 2>/dev/null`\" = \"X-n #\"; then :\n",
    );
    i.extend_from_slice(b"    as_echo_body='eval /usr/ucb/echo -n \"$1$as_nl\"'\n");
    i.extend_from_slice(b"    as_echo_n='/usr/ucb/echo -n'\n  else :\n");
    i.extend_from_slice(b"    as_echo_body='eval expr \"X$1\" : \"X\\(.*\\)\"'\n");
    i.extend_from_slice(b"    as_echo_n_body='eval\n      arg=$1;\n      case $arg in #(\n");
    i.extend_from_slice(b"      *\"$as_nl\"*)\n        expr \"X$arg\" : \"X\\(.*\\)$as_nl\";\n");
    i.extend_from_slice(b"        arg=`expr \"X$arg\" : \".*$as_nl\\(.*\\)\"`;;\n      esac;\n");
    i.extend_from_slice(b"      expr \"X$arg\" : \"X\\(.*\\)\" | tr -d \"$as_nl\"\n    '\n");
    i.extend_from_slice(
        b"    export as_echo_n_body\n    as_echo_n='sh -c $as_echo_n_body as_echo'\n",
    );
    i.extend_from_slice(
        b"  fi\n  export as_echo_body\n  as_echo='sh -c $as_echo_body as_echo'\nfi\n\n",
    );

    // as_mkdir_p
    i.extend_from_slice(b"# Find a good mkdir -p.\n");
    i.extend_from_slice(b"if test -d ./-p 2>/dev/null; then :\n  as_mkdir_p='mkdir -p --'\n");
    i.extend_from_slice(
        b"else :\n  if test -d ./--version 2>/dev/null; then :\n    as_mkdir_p='mkdir -p --'\n",
    );
    i.extend_from_slice(b"  else :\n    as_mkdir_p='mkdir -p'\n  fi\nfi\n\n");

    // as_test_x
    i.extend_from_slice(b"# Find a good test -x.\n");
    i.extend_from_slice(b"if test -x / >/dev/null 2>&1; then :\n  as_test_x='test -x'\n");
    i.extend_from_slice(
        b"else :\n  if ls -dL / >/dev/null 2>&1; then :\n    as_test_x='test -x'\n",
    );
    i.extend_from_slice(b"  else :\n    as_test_x=:\n  fi\nfi\n\n");
    i.extend_from_slice(b"as_executable_p=$as_test_x\n\n");
}

/// M4sh shell functions section
pub fn generate_m4sh_functions() -> Vec<u8> {
    let mut f = Vec::new();

    // as_fn_unset
    f.extend_from_slice(b"# as_fn_unset VAR\n# ---------------\n# Portably unset VAR.\n");
    f.extend_from_slice(b"as_fn_unset ()\n{\n  { eval ${1}=; unset ${1};}\n}\n");
    f.extend_from_slice(b"as_unset=as_fn_unset\n\n");

    // as_fn_set_status
    f.extend_from_slice(b"# as_fn_set_status STATUS\n# -----------------------\n");
    f.extend_from_slice(b"# Set $? to STATUS, without forking.\n");
    f.extend_from_slice(b"as_fn_set_status ()\n{\n  return ${1}\n}\n\n");

    // as_fn_exit
    f.extend_from_slice(b"# as_fn_exit STATUS\n# -----------------\n");
    f.extend_from_slice(
        b"# Exit the shell with STATUS, even in a \"trap 0\" or \"set -e\" context.\n",
    );
    f.extend_from_slice(b"as_fn_exit ()\n{\n  set +e\n  as_fn_set_status ${1}\n  exit ${1}\n}\n\n");

    // as_fn_mkdir_p
    f.extend_from_slice(b"# as_fn_mkdir_p\n# -------------\n");
    f.extend_from_slice(b"# Create \"$as_dir\" as a directory, including parents if necessary.\n");
    f.extend_from_slice(b"as_fn_mkdir_p ()\n{\n");
    f.extend_from_slice(b"  case $as_dir in #(\n  -*) as_dir=./$as_dir;;\n  esac\n");
    f.extend_from_slice(b"  test -d \"$as_dir\" || eval $as_mkdir_p || {\n");
    f.extend_from_slice(b"    as_dirs=\n    while :; do\n");
    f.extend_from_slice(b"      case $as_dir in #(\n");
    f.extend_from_slice(
        b"      *\\'*) as_qdir=`printf '%s\\n' \"$as_dir\" | sed \"s/'/'\\\\\\\\''/g\"`;; #'(\n",
    );
    f.extend_from_slice(b"      *) as_qdir=$as_dir;;\n      esac\n");
    f.extend_from_slice(b"      as_dirs=\"'$as_qdir' $as_dirs\"\n");
    f.extend_from_slice(b"      as_dir=`$as_dirname -- \"$as_dir\" ||\n");
    f.extend_from_slice(b"$as_expr X\"$as_dir\" : 'X\\(.*[^/]\\)//*[^/][^/]*/*$' \\| \\\n");
    f.extend_from_slice(b"         X\"$as_dir\" : 'X\\(//\\)[^/]' \\| \\\n");
    f.extend_from_slice(b"         X\"$as_dir\" : 'X\\(//\\)$' \\| \\\n");
    f.extend_from_slice(b"         X\"$as_dir\" : 'X\\(/\\)' \\| . 2>/dev/null ||\n");
    f.extend_from_slice(b"printf '%s\\n' \"$as_dir\" |\n");
    f.extend_from_slice(b"    sed '/^X\\(.*[^/]\\)\\/\\/*[^/][^/]*\\/*$/{\n            s//\\1/\n            q\n          }\n");
    f.extend_from_slice(
        b"          /^X\\(\\/\\/\\)[^/].*/{\n            s//\\1/\n            q\n          }\n",
    );
    f.extend_from_slice(
        b"          /^X\\(\\/\\/\\)$/{\n            s//\\1/\n            q\n          }\n",
    );
    f.extend_from_slice(
        b"          /^X\\(\\/\\).*/{\n            s//\\1/\n            q\n          }\n",
    );
    f.extend_from_slice(b"          s/.*/./; q'`\n");
    f.extend_from_slice(b"      test -d \"$as_dir\" && break\n    done\n");
    f.extend_from_slice(b"    test -z \"$as_dirs\" || eval \"mkdir $as_dirs\"\n");
    f.extend_from_slice(
        b"  } || test -d \"$as_dir\" || as_fn_error $? \"cannot create directory $as_dir\"\n",
    );
    f.extend_from_slice(b"}\n\n");

    // as_fn_executable_p
    f.extend_from_slice(b"# as_fn_executable_p FILE\n# -----------------------\n");
    f.extend_from_slice(b"# Test if FILE is an executable regular file.\n");
    f.extend_from_slice(b"as_fn_executable_p ()\n{\n  test -f \"${1}\" && test -x \"${1}\"\n}\n\n");

    // as_fn_append
    f.extend_from_slice(b"# as_fn_append VAR VALUE\n# ----------------------\n");
    f.extend_from_slice(b"# Append the text in VALUE to the end of the definition in VAR.\n");
    f.extend_from_slice(b"as_fn_append ()\n{\n  eval $1=\\$$1\\$2\n}\n\n");

    // as_fn_arith
    f.extend_from_slice(b"# as_fn_arith ARG...\n# ------------------\n");
    f.extend_from_slice(b"# Perform arithmetic evaluation on the ARGs.\n");
    f.extend_from_slice(b"as_fn_arith ()\n{\n  as_val=`expr \"${@}\" 2>/dev/null`\n  test $? -eq 0 || as_val=0\n}\n\n");

    // as_fn_error
    f.extend_from_slice(
        b"# as_fn_error STATUS ERROR [LINENO LOG_FD]\n# ----------------------------------------\n",
    );
    f.extend_from_slice(
        b"# Output \"`basename ${0}`: error: ERROR\" to stderr and exit with STATUS.\n",
    );
    f.extend_from_slice(
        b"as_fn_error ()\n{\n  as_status=${1}; test $as_status -eq 0 && as_status=1\n",
    );
    f.extend_from_slice(b"  if test \"${4}\"; then\n    as_lineno=${as_lineno-\"${3}\"} as_lineno_stack=as_lineno_stack=$as_lineno_stack\n");
    f.extend_from_slice(
        b"    printf '%s\\n' \"$as_me:${as_lineno-$LINENO}: error: ${2}\" >&${4}\n  fi\n",
    );
    f.extend_from_slice(
        b"  printf '%s\\n' \"$as_me: error: ${2}\" >&2\n  as_fn_exit $as_status\n}\n\n",
    );

    // as_dirname / as_basename. The probe MUST actually exercise `dirname`: the previous test
    // (`test -r "./chmod"`) checked for an unrelated file in the cwd, so it almost always failed ->
    // as_dirname=false -> the fragile inline-sed dirname fallback, which INFINITE-LOOPS in
    // as_fn_mkdir_p when it can't shorten the path (hung configure on essentially every project).
    f.extend_from_slice(b"if (dirname -- / || dirname /) >/dev/null 2>&1; then :\n");
    f.extend_from_slice(b"  as_dirname=dirname\nelse\n  as_dirname=false\nfi\n\n");

    // as_basename — portable basename with fallbacks
    f.extend_from_slice(b"as_basename=basename\n");
    f.extend_from_slice(b"if test -x /usr/bin/basename; then\n");
    f.extend_from_slice(b"  as_basename=/usr/bin/basename\n");
    f.extend_from_slice(b"fi\n\n");
    f.extend_from_slice(b"as_me=`$as_basename -- \"$0\" ||\n");
    f.extend_from_slice(b"$as_expr X/\"$0\" : '.*/\\([^/][^/]*\\)/*$' \\| \\\n");
    f.extend_from_slice(b"         X\"$0\" : 'X\\(//\\)$' \\| \\\n");
    f.extend_from_slice(b"         X\"$0\" : 'X\\(/\\)' \\| . 2>/dev/null ||\n");
    f.extend_from_slice(b"printf '%s\\n' \"$0\" | sed 's,.*/,,'`\n\n");

    // Character range tables (avoid depending on locale ranges)
    f.extend_from_slice(b"# Avoid depending upon Character Ranges.\n");
    f.extend_from_slice(b"as_cr_letters='abcdefghijklmnopqrstuvwxyz'\n");
    f.extend_from_slice(b"as_cr_LETTERS='ABCDEFGHIJKLMNOPQRSTUVWXYZ'\n");
    f.extend_from_slice(b"as_cr_Letters=$as_cr_letters$as_cr_LETTERS\n");
    f.extend_from_slice(b"as_cr_digits='0123456789'\n");
    f.extend_from_slice(b"as_cr_alnum=$as_cr_Letters$as_cr_digits\n\n");

    f
}

pub fn generate_configure_prologue(
    package_name: &str,
    package_version: &str,
    bug: Option<&str>,
) -> Vec<u8> {
    let mut h = Vec::new();
    h.extend_from_slice(b"#! /bin/sh\n");
    h.extend_from_slice(b"# Guess values for system-dependent variables and create Makefiles.\n");
    h.extend_from_slice(
        format!(
            "# Generated by autoconf-rs for {} {}.\n",
            package_name, package_version
        )
        .as_bytes(),
    );
    h.extend_from_slice(b"#\n#\n");
    h.extend_from_slice(
        b"# Copyright (C) 1992-1996, 1998-2017, 2020-2026 Free Software Foundation,\n",
    );
    h.extend_from_slice(b"# Inc.\n#\n#\n");
    h.extend_from_slice(
        b"# This configure script is free software; the Free Software Foundation\n",
    );
    h.extend_from_slice(b"# gives unlimited permission to copy, distribute and modify it.\n");
    h.extend_from_slice(
        b"## -------------------- ##\n## M4sh Initialization. ##\n## -------------------- ##\n",
    );
    h.extend_from_slice(&generate_m4sh_init(package_name, package_version));
    h.extend_from_slice(&generate_m4sh_functions());
    // _acrs_write_libtool: writes the native libtool wrapper + sets/substitutes LIBTOOL. Called by
    // our LT_INIT/LT_OUTPUT/AC_PROG_LIBTOOL overrides. Defined here (prologue) so it exists before
    // any LT_INIT call in the configure.ac body.
    h.extend_from_slice(b"_acrs_write_libtool () {\ncat > ./libtool <<'_ACRS_LTEOF'\n");
    h.extend_from_slice(crate::libtool_script().as_bytes());
    h.extend_from_slice(b"_ACRS_LTEOF\nchmod +x ./libtool\nLIBTOOL=\"${CONFIG_SHELL:-/bin/sh} `pwd`/libtool\"\nexport LIBTOOL\ntest -f conf_subst.sed || : > conf_subst.sed\nprintf '%s\\n' \"s|@LIBTOOL@|$LIBTOOL|g\" >> conf_subst.sed\n}\n");
    // Create the runtime AC_SUBST sink ONCE, here in the prologue (before the configure.ac body), so
    // macros called early in the body (LT_INIT -> _acrs_write_libtool, PKG_CHECK_MODULES) can append
    // to it without a later `: > conf_subst.sed` truncating their entries.
    h.extend_from_slice(b": > conf_subst.sed\n");
    // Create confdefs.h ONCE here in the prologue too. The compile probes do `cat confdefs.h - ...`;
    // a project AC_COMPILE_IFELSE/AC_CHECK in the configure.ac body runs BEFORE the generated
    // feature-test section, so confdefs.h must already exist — otherwise `cat: confdefs.h: No such
    // file` (the single most common corpus failure, 36/138 repos) corrupts every early probe.
    h.extend_from_slice(b"test -f confdefs.h || printf '%s\\n' '/* confdefs.h */' > confdefs.h\n");
    h.extend_from_slice(b"# Sanitize environment\n");
    h.extend_from_slice(b"LC_ALL=C\nexport LC_ALL\nLANGUAGE=C\nexport LANGUAGE\n\nCDPATH=\n\n");
    // Identity of this package (set near the top, as GNU Autoconf does). These shell vars carry the
    // AC_INIT arguments through the rest of configure; PACKAGE_BUGREPORT is the third AC_INIT arg.
    let bug = bug.unwrap_or("");
    h.extend_from_slice(
        format!(
            "# Identity of this package.\n\
             PACKAGE_NAME='{name}'\n\
             PACKAGE_TARNAME='{name}'\n\
             PACKAGE_VERSION='{ver}'\n\
             PACKAGE_STRING='{name} {ver}'\n\
             PACKAGE_BUGREPORT='{bug}'\n\
             PACKAGE_URL=''\n\n",
            name = package_name,
            ver = package_version,
            bug = bug
        )
        .as_bytes(),
    );
    // Open the message/log file descriptors used throughout configure: fd 5 -> config.log, fd 6 -> a copy
    // of stdout (so `>&5` / `>&6` redirections do not fail with "Bad file descriptor").
    h.extend_from_slice(b"exec 5>>config.log\nexec 6>&1\n\n");
    // C try-compile/link/run helpers. Defined in the prologue so they exist before ANY feature test calls
    // them, whatever generation path produced the checks (otherwise: "ac_fn_c_try_compile: command not
    // found"). They reference $ac_compile/$ac_link/$ac_ext at call time, which the body sets up.
    h.extend_from_slice(b"ac_ext=c\nac_objext=o\nac_exeext=\n");
    // Set the compile/link/cpp command strings HERE, in the prologue — not in the footer. They are
    // eval'd at each check, resolving $CC/$CFLAGS/$LIBS then, so defining the strings early is safe
    // and REQUIRED: otherwise the first AC_CHECK_LIB/FUNC/HEADER (emitted before the footer) runs
    // `eval ''` -> empty link command -> every compile/link test spuriously fails ("math library
    // required" etc. when the lib is actually present). The footer re-sets these identically.
    h.extend_from_slice(b"ac_cpp='$CPP $CPPFLAGS'\n");
    h.extend_from_slice(b"ac_compile='$CC -c $CFLAGS $CPPFLAGS conftest.$ac_ext >&5'\n");
    h.extend_from_slice(b"ac_link='$CC -o conftest$ac_exeext $CFLAGS $CPPFLAGS $LDFLAGS conftest.$ac_ext $LIBS >&5'\n");
    h.extend_from_slice(b"ac_fn_c_try_compile () {\n  rm -f conftest.$ac_objext conftest$ac_exeext\n  if { (eval \"$ac_compile\") 2>&5; } && test -s conftest.$ac_objext; then ac_retval=0; else printf '%s\\n' \"configure: failed program was:\" >&5; cat conftest.$ac_ext >&5 2>/dev/null; ac_retval=1; fi\n  rm -f conftest.$ac_objext conftest.$ac_ext\n  return $ac_retval\n}\n");
    h.extend_from_slice(b"ac_fn_c_try_link () {\n  rm -f conftest.$ac_objext conftest$ac_exeext\n  if { (eval \"$ac_link\") 2>&5; } && test -s conftest$ac_exeext; then ac_retval=0; else printf '%s\\n' \"configure: failed program was:\" >&5; cat conftest.$ac_ext >&5 2>/dev/null; ac_retval=1; fi\n  rm -f conftest.$ac_objext conftest.$ac_ext conftest$ac_exeext\n  return $ac_retval\n}\n");
    h.extend_from_slice(b"ac_fn_c_try_run () {\n  if { ac_try='$ac_link'; (eval \"$ac_try\") 2>&5; } && test -s conftest$ac_exeext && { ac_try='./conftest$ac_exeext'; (eval \"$ac_try\") 2>&5; }; then ac_retval=0; else printf '%s\\n' \"configure: failed program was:\" >&5; cat conftest.$ac_ext >&5 2>/dev/null; ac_retval=1; fi\n  rm -f conftest.$ac_ext conftest$ac_exeext\n  return $ac_retval\n}\n");
    h.extend_from_slice(b"ac_fn_c_try_cpp () {\n  if { (eval \"$ac_cpp conftest.$ac_ext\") 2>&5; }; then ac_retval=0; else ac_retval=1; fi\n  return $ac_retval\n}\n\n");
    // Exit/signal trap: clean up conftest debris on EXIT (and on INT/TERM), preserving the exit status.
    // ac_exit_trap MUST disarm the EXIT(0) trap before calling `exit`, otherwise `exit $ac_status`
    // re-triggers the EXIT trap -> infinite recursion that HANGS configure (observed on nearly every
    // real-world project). `trap - 0 1 2 13 15` disarms it so the final exit terminates the shell.
    h.extend_from_slice(b"ac_clean_files=\nac_exit_trap () { ac_status=$?; trap - 0 1 2 13 15; rm -f conftest* conf$$* core 2>/dev/null; exit $ac_status; }\ntrap 'ac_exit_trap' 0  # 0 = EXIT\ntrap 'ac_status=1; ac_exit_trap' 1 2 13 15\n\n");
    // Standard m4sh command-line option parsing (--prefix, --bindir, --enable/--with, env-var
    // capture, srcdir handling) and the --help/--version reports. These are static boilerplate in
    // every Autoconf configure; emitting them here makes the dynamic script handle options exactly
    // like the oracle instead of silently ignoring them.
    let opts = include_str!("templates/option_parsing.sh")
        .replace("{NAME}", package_name)
        .replace("{VERSION}", package_version)
        .replace("{BUGREPORT}", bug);
    h.extend_from_slice(opts.as_bytes());
    let help = include_str!("templates/help_version.sh")
        .replace("{NAME}", package_name)
        .replace("{VERSION}", package_version)
        .replace("{BUGREPORT}", bug);
    h.extend_from_slice(help.as_bytes());
    h.extend_from_slice(b"\n");
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_m4sh_init() {
        let init = generate_m4sh_init("t", "1.0");
        assert!(
            init.len() > 4000,
            "M4sh init too short: {} bytes",
            init.len()
        );
        let s = String::from_utf8_lossy(&init);
        assert!(s.contains("as_myself"), "missing self-location");
        assert!(s.contains("CONFIG_SHELL"), "missing re-exec");
        assert!(s.contains("as_have_required"), "missing shell probe");
        assert!(s.contains("as_bourne_compatible"), "missing bourne compat");
    }

    #[test]
    fn test_generate_prologue() {
        let p = generate_configure_prologue("hello", "1.0", None);
        assert!(p.len() > 8000, "Prologue too short: {} bytes", p.len());
        let s = String::from_utf8_lossy(&p);
        assert!(s.contains("as_fn_unset"), "missing unset function");
        assert!(s.contains("as_fn_mkdir_p"), "missing mkdir_p function");
        assert!(s.contains("as_fn_error"), "missing error function");
        assert!(s.contains("as_have_required"), "missing shell probe");
    }
}
