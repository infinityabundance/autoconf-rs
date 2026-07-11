//! Shell script generator for Autoconf.
//!
//! Assembles expanded M4 output into a complete POSIX-compliant configure
//! shell script. Handles quoting, escaping, here-documents, variable
//! substitution, and the configure/config.status two-phase output.
//!
//! Receipt family: AC.SHELL.*
//! Status: Phase 5 — full configure generation with shell sanitization (panel mandate).

/// Standard build-variable defaults emitted inside config.status's `substitute()` so that the
/// Automake-generated Makefile.in's `@VAR@` references resolve. Without these, config.status left
/// `@CC@`/`@AR@`/`@top_srcdir@`/`@SET_MAKE@`/... literal -> `make` aborted ("missing separator").
/// Functional values (cc/ar/...) suffice for `make`; install-time vars use the conventional forms.
pub const STD_VAR_DEFAULTS: &str = r#"  : ${srcdir=.}
  : ${prefix=/usr/local}; : ${exec_prefix=$prefix}
  test -n "${top_srcdir:-}" || top_srcdir=$srcdir
  test -n "${top_builddir:-}" || top_builddir=.
  builddir=.
  abs_srcdir=`cd "$srcdir" 2>/dev/null && pwd || pwd`
  abs_builddir=`pwd`; abs_top_srcdir=$abs_srcdir; abs_top_builddir=$abs_builddir
  : ${CC=cc}; : ${CFLAGS=-g -O2}; : ${CPPFLAGS=}; : ${LDFLAGS=}; : ${LIBS=}
  : ${CXX=c++}; : ${CXXFLAGS=-g -O2}; CPP="$CC -E"
  : ${AR=ar}; : ${RANLIB=ranlib}; : ${STRIP=strip}; : ${AWK=awk}; : ${LN_S=ln -s}
  : ${YACC=yacc}; : ${YFLAGS=}; : ${LEX=lex}; : ${LEXLIB=}; : ${LEX_OUTPUT_ROOT=lex.yy}
  : ${GREP=grep}; : ${EGREP=grep -E}; : ${FGREP=grep -F}; : ${SED=sed}
  OBJEXT=o; EXEEXT=; SET_MAKE=; am__leading_dot=.
  : ${SHELL=/bin/sh}; MKDIR_P="mkdir -p"; DEPDIR=.deps
  INSTALL="$abs_srcdir/install-sh -c"; INSTALL_PROGRAM='${INSTALL}'; INSTALL_DATA='${INSTALL} -m 644'; INSTALL_SCRIPT='${INSTALL}'
  : ${DEFS=}; ECHO_C=; ECHO_N=; ECHO_T=; : ${LIBTOOL=}; : ${LIBOBJS=}; : ${LTLIBOBJS=}; : ${ALLOCA=}
  build=x86_64-pc-linux-gnu; host=$build; target=$build
  build_alias=; host_alias=; target_alias=
  build_cpu=x86_64; build_vendor=pc; build_os=linux-gnu
  host_cpu=x86_64; host_vendor=pc; host_os=linux-gnu
  configure_input="Generated from Makefile.in by autoconf-rs."
  bindir='${exec_prefix}/bin'; sbindir='${exec_prefix}/sbin'; libexecdir='${exec_prefix}/libexec'
  datarootdir='${prefix}/share'; datadir='${datarootdir}'; sysconfdir='${prefix}/etc'
  sharedstatedir='${prefix}/com'; localstatedir='${prefix}/var'; runstatedir='${localstatedir}/run'
  includedir='${prefix}/include'; oldincludedir=/usr/include; libdir='${exec_prefix}/lib'
  infodir='${datarootdir}/info'; localedir='${datarootdir}/locale'; mandir='${datarootdir}/man'
  docdir='${datarootdir}/doc/${PACKAGE_TARNAME}'; htmldir='${docdir}'; dvidir='${docdir}'
  pdfdir='${docdir}'; psdir='${docdir}'
  lispdir='${datarootdir}/emacs/site-lisp'; pkgpyexecdir=; pkgpythondir=; pyexecdir=; pythondir=
  AMDEP_TRUE=; AMDEP_FALSE='#'; am__include=include; am__quote=; am__isrc=; am__nodep=
  am__fastdepCC_TRUE=; am__fastdepCC_FALSE='#'; am__fastdepCXX_TRUE=; am__fastdepCXX_FALSE='#'
  ACLOCAL=:; AUTOCONF=:; AUTOMAKE=:; AUTOHEADER=:; MAKEINFO=:; install_sh="$INSTALL"
  AMTAR=tar; am__tar='tar cf - .'; am__untar='tar xf -'; CTAGS=ctags; ETAGS=etags; CSCOPE=cscope
  ACLOCAL_AMFLAGS=; MAINT='#'; MAINTAINER_MODE_TRUE='#'; MAINTAINER_MODE_FALSE=
"#;

/// The sed expressions for the standard build variables (double-quoted so the values set by
/// STD_VAR_DEFAULTS expand when config.status runs). Appended to config.status's `substitute()` sed.
pub const STD_VAR_SED: &str = r#" -e "s|@top_srcdir@|$top_srcdir|g" -e "s|@top_builddir@|$top_builddir|g" -e "s|@builddir@|$builddir|g" -e "s|@abs_srcdir@|$abs_srcdir|g" -e "s|@abs_builddir@|$abs_builddir|g" -e "s|@abs_top_srcdir@|$abs_top_srcdir|g" -e "s|@abs_top_builddir@|$abs_top_builddir|g" -e "s|@CC@|$CC|g" -e "s|@CFLAGS@|$CFLAGS|g" -e "s|@CPPFLAGS@|$CPPFLAGS|g" -e "s|@LDFLAGS@|$LDFLAGS|g" -e "s|@LIBS@|$LIBS|g" -e "s|@CXX@|$CXX|g" -e "s|@CXXFLAGS@|$CXXFLAGS|g" -e "s|@CPP@|$CPP|g" -e "s|@AR@|$AR|g" -e "s|@ARFLAGS@|cr|g" -e "s|@RANLIB@|$RANLIB|g" -e "s|@STRIP@|$STRIP|g" -e "s|@AWK@|$AWK|g" -e "s|@LN_S@|$LN_S|g" -e "s|@OBJEXT@|$OBJEXT|g" -e "s|@EXEEXT@|$EXEEXT|g" -e "s|@SET_MAKE@|$SET_MAKE|g" -e "s|@YACC@|$YACC|g" -e "s|@YFLAGS@|$YFLAGS|g" -e "s|@LEX@|$LEX|g" -e "s|@LEXLIB@|$LEXLIB|g" -e "s|@LEX_OUTPUT_ROOT@|$LEX_OUTPUT_ROOT|g" -e "s|@GREP@|$GREP|g" -e "s|@EGREP@|$EGREP|g" -e "s|@FGREP@|$FGREP|g" -e "s|@SED@|$SED|g" -e "s|@SHELL@|$SHELL|g" -e "s|@MKDIR_P@|$MKDIR_P|g" -e "s|@DEPDIR@|$DEPDIR|g" -e "s|@INSTALL@|$INSTALL|g" -e "s|@INSTALL_PROGRAM@|$INSTALL_PROGRAM|g" -e "s|@INSTALL_DATA@|$INSTALL_DATA|g" -e "s|@INSTALL_SCRIPT@|$INSTALL_SCRIPT|g" -e "s|@am__leading_dot@|$am__leading_dot|g" -e "s|@DEFS@|$DEFS|g" -e "s|@ECHO_C@|$ECHO_C|g" -e "s|@ECHO_N@|$ECHO_N|g" -e "s|@ECHO_T@|$ECHO_T|g" -e "s|@LIBTOOL@|$LIBTOOL|g" -e "s|@LIBOBJS@|$LIBOBJS|g" -e "s|@LTLIBOBJS@|$LTLIBOBJS|g" -e "s|@ALLOCA@|$ALLOCA|g" -e "s|@build@|$build|g" -e "s|@host@|$host|g" -e "s|@target@|$target|g" -e "s|@build_alias@|$build_alias|g" -e "s|@host_alias@|$host_alias|g" -e "s|@target_alias@|$target_alias|g" -e "s|@build_cpu@|$build_cpu|g" -e "s|@build_vendor@|$build_vendor|g" -e "s|@build_os@|$build_os|g" -e "s|@host_cpu@|$host_cpu|g" -e "s|@host_vendor@|$host_vendor|g" -e "s|@host_os@|$host_os|g" -e "s|@configure_input@|$configure_input|g" -e "s|@bindir@|$bindir|g" -e "s|@sbindir@|$sbindir|g" -e "s|@libexecdir@|$libexecdir|g" -e "s|@datarootdir@|$datarootdir|g" -e "s|@datadir@|$datadir|g" -e "s|@sysconfdir@|$sysconfdir|g" -e "s|@sharedstatedir@|$sharedstatedir|g" -e "s|@localstatedir@|$localstatedir|g" -e "s|@runstatedir@|$runstatedir|g" -e "s|@includedir@|$includedir|g" -e "s|@oldincludedir@|$oldincludedir|g" -e "s|@libdir@|$libdir|g" -e "s|@infodir@|$infodir|g" -e "s|@localedir@|$localedir|g" -e "s|@mandir@|$mandir|g" -e "s|@docdir@|$docdir|g" -e "s|@htmldir@|$htmldir|g" -e "s|@dvidir@|$dvidir|g" -e "s|@pdfdir@|$pdfdir|g" -e "s|@psdir@|$psdir|g" -e "s|@lispdir@|$lispdir|g" -e "s|@AMDEP_TRUE@|$AMDEP_TRUE|g" -e "s|@AMDEP_FALSE@|$AMDEP_FALSE|g" -e "s|@am__include@|$am__include|g" -e "s|@am__quote@|$am__quote|g" -e "s|@am__isrc@|$am__isrc|g" -e "s|@am__nodep@|$am__nodep|g" -e "s|@am__fastdepCC_TRUE@|$am__fastdepCC_TRUE|g" -e "s|@am__fastdepCC_FALSE@|$am__fastdepCC_FALSE|g" -e "s|@am__fastdepCXX_TRUE@|$am__fastdepCXX_TRUE|g" -e "s|@am__fastdepCXX_FALSE@|$am__fastdepCXX_FALSE|g" -e "s|@ACLOCAL@|$ACLOCAL|g" -e "s|@AUTOCONF@|$AUTOCONF|g" -e "s|@AUTOMAKE@|$AUTOMAKE|g" -e "s|@AUTOHEADER@|$AUTOHEADER|g" -e "s|@MAKEINFO@|$MAKEINFO|g" -e "s|@install_sh@|$install_sh|g" -e "s|@AMTAR@|$AMTAR|g" -e "s|@am__tar@|$am__tar|g" -e "s|@am__untar@|$am__untar|g" -e "s|@CTAGS@|$CTAGS|g" -e "s|@ETAGS@|$ETAGS|g" -e "s|@CSCOPE@|$CSCOPE|g" -e "s|@ACLOCAL_AMFLAGS@|$ACLOCAL_AMFLAGS|g" -e "s|@MAINT@|$MAINT|g" -e "s|@MAINTAINER_MODE_TRUE@|$MAINTAINER_MODE_TRUE|g" -e "s|@MAINTAINER_MODE_FALSE@|$MAINTAINER_MODE_FALSE|g" -e "s|@pkgpyexecdir@|$pkgpyexecdir|g" -e "s|@pkgpythondir@|$pkgpythondir|g" -e "s|@pyexecdir@|$pyexecdir|g" -e "s|@pythondir@|$pythondir|g""#;

/// Header-LESS DEFS builder, mirroring config.status's `ac_script` in real GNU autoconf. A project with
/// no AC_CONFIG_HEADERS gets ALL its `#define`s (PACKAGE_*/HAVE_*/…) on the compiler command line via
/// -D flags in DEFS. The values must be SHELL-ESCAPED per-define so a value with a space
/// (`#define PACKAGE_STRING "fts 0.2"`) stays ONE cc argument. Real autoconf emits
/// `-DPACKAGE_STRING=\"fts\ 0.2\"` (quote AND space backslash-escaped); the naive
/// `-DPACKAGE_STRING="fts 0.2"` word-splits at make-time into `-DPACKAGE_STRING=fts` + a phantom `0.2`
/// input file (cc: "linker input file not found: 0.2"). This rebuilds DEFS from confdefs.h with the
/// same escaping (space→`\ `, quote→`\"`, `$`→`$$`), one #define per line so value-spaces are escaped
/// but flag-separator spaces are not, then joins with `tr`. PACKAGE/VERSION (defined by
/// AM_INIT_AUTOMAKE, NOT written to our confdefs.h) are prepended, matching real autoconf's set.
/// FINALLY the whole string is sed-escaped (backslashes doubled) so the generic `s|@DEFS@|$DEFS|g` in
/// STD_VAR_SED emits the single-backslash form literally into the Makefile instead of collapsing it.
/// Runs at AC_OUTPUT (after every conditional AC_DEFINE has populated confdefs.h); config-HEADER
/// projects skip this entirely (their DEFS is just `-DHAVE_CONFIG_H`).
pub const DEFS_FROM_CONFDEFS: &str = r#"ac_defs_script='
s/^#define \([A-Za-z_][A-Za-z0-9_]*\) \(.*\)$/-D\1=\2/
t acq
b
:acq
s/[ `~#$^&*(){}\\|;'\''"<>?]/\\&/g
s/\[/\\&/g
s/\]/\\&/g
s/\$/$$/g
p
'
DEFS=$(sed -n "$ac_defs_script" confdefs.h 2>/dev/null | tr '\n' ' ')
DEFS="-DPACKAGE=\\\"$PACKAGE_TARNAME\\\" -DVERSION=\\\"$PACKAGE_VERSION\\\" $DEFS"
DEFS=$(printf '%s' "$DEFS" | sed 's/[\\&|]/\\&/g')
"#;

/// Build a SAFE `-e 's|@VAR@|VALUE|g'` sed expr for embedding in a single-quoted shell word. VALUE is
/// sed replacement text. Without care, an AC_SUBST value like `' -I$(srcdir)'` or `'$(MKDIR_P)'` (the
/// surrounding single quotes are part of the captured value) produced `-e 's|@x@|'$(MKDIR_P)'|g'`, where
/// `$(MKDIR_P)` fell OUTSIDE the shell quotes -> command-substituted (`MKDIR_P: command not found`) and
/// the whole sed died -> EMPTY Makefile. We: (1) strip a matched pair of surrounding shell single quotes
/// (automake adds them for shell-assignment; the intended sed value is the inside, e.g. ` -I$(srcdir)`,
/// a literal make expands); (2) sed-escape `\ & |`; (3) shell-escape `'` via the `'\''` idiom so the
/// single-quoted word always holds.
pub fn sed_subst_expr(var: &str, value: &str) -> String {
    let mut v: &str = value;
    if v.len() >= 2 && v.starts_with('\'') && v.ends_with('\'') {
        v = &v[1..v.len() - 1];
    }
    let sed_escaped = v.replace('\\', "\\\\").replace('&', "\\&").replace('|', "\\|");
    let shell_escaped = sed_escaped.replace('\'', "'\\''");
    format!(" -e 's|@{}@|{}|g'", var, shell_escaped)
}

/// Generates a configure shell script from parsed Autoconf input.
pub struct ShellGenerator;

impl ShellGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a configure script from a ConfigureAc and expanded M4 output.
    pub fn generate(&self, ac: &super::ConfigureAc, expanded_m4: &str) -> String {
        let mut script = String::new();

        // Shell shebang
        script.push_str("#! /bin/sh\n");

        // Header comment
        if let Some(ref name) = ac.package_name {
            if let Some(ref version) = ac.package_version {
                script.push_str(&format!(
                    "# Generated by autoconf-rs for {} {}\n",
                    name, version
                ));
            }
        }
        script.push_str("# Forensic-parity GNU Autoconf reimplementation\n\n");

        // Phase 1: copy expanded M4 output directly
        // Future phases will add proper shell quoting, diversion ordering,
        // config.status generation, and template substitution.
        script.push_str(expanded_m4);

        script
    }
}

impl Default for ShellGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a minimal but working configure script with dynamic substitutions.
/// Used when configure.ac has AC_SUBST/AC_CONFIG_FILES/AC_DEFINE/etc.
/// Court: AC.SHELL.DYNAMIC.1
pub fn generate_dynamic_configure(
    state: &super::autoconf_macros::AutoconfState,
    name: &str,
    version: &str,
) -> String {
    let mut s = String::new();

    // Start with full M4sh init prologue (oracle-captured, 52KB)
    // This provides shell portability, variable reset, locale setup,
    // CONFIG_SHELL re-exec, shell functions, and sanitization —
    // matching GNU Autoconf byte-for-byte on the prologue section.
    let prologue = include_str!("templates/prologue_template.sh")
        .replace("{NAME}", name)
        .replace("{VERSION}", version)
        .replace("{BUGREPORT}", state.bug_report.as_deref().unwrap_or(""))
        .replace("Generated by GNU Autoconf 2.73", "Generated by autoconf-rs");
    s.push_str(&prologue);

    // Inject bug report line and unique file after prologue header
    if let Some(ref bug) = state.bug_report {
        s.push_str(&format!("# Report bugs to <{}>.\n", bug));
    }
    if let Some(ref unique) = state.unique_file {
        s.push_str(&format!("ac_unique_file=\"{}\"\n", unique));
    }

    // CROSS.014: config.site — source system-wide site defaults
    s.push_str("# Site configuration\n");
    s.push_str("if test \"$CONFIG_SITE\" != \"/dev/null\"; then\n");
    s.push_str("  if test -r \"${CONFIG_SITE-/usr/local/share/config.site}\"; then\n");
    s.push_str("    . \"${CONFIG_SITE-/usr/local/share/config.site}\" 2>/dev/null || :\n");
    s.push_str("  fi\n");
    s.push_str("  if test -r \"${CONFIG_SITE-/usr/local/etc/config.site}\"; then\n");
    s.push_str("    . \"${CONFIG_SITE-/usr/local/etc/config.site}\" 2>/dev/null || :\n");
    s.push_str("  fi\n");
    s.push_str("fi\n\n");

    // Shell helper functions for compile/link/run tests
    // These are required by AC_CHECK_FUNCS, AC_CHECK_HEADERS, AC_COMPILE_IFELSE, etc.
    // Without them, generated configure scripts fail at runtime with
    // "ac_fn_c_try_link: command not found".
    s.push_str("\n# Autoconf shell helper functions\n");
    s.push_str("ac_fn_c_try_compile() {\n");
    s.push_str("  rm -f conftest.$ac_objext conftest$ac_exeext\n");
    s.push_str("  if { (eval \"$ac_compile\") >&5; } && test -s conftest.$ac_objext; then\n");
    s.push_str("    ac_retval=0\n");
    s.push_str("  else\n");
    s.push_str("    printf '%s\\n' \"configure: failed program was:\" >&5\n");
    s.push_str("    cat conftest.$ac_ext >&5\n");
    s.push_str("    ac_retval=1\n");
    s.push_str("  fi\n");
    s.push_str("  rm -f conftest.$ac_objext conftest.$ac_ext\n");
    s.push_str("  return $ac_retval\n");
    s.push_str("}\n\n");

    s.push_str("ac_fn_c_try_link() {\n");
    s.push_str("  rm -f conftest.$ac_objext conftest$ac_exeext\n");
    s.push_str("  if { (eval \"$ac_link\") >&5; } && test -s conftest$ac_exeext; then\n");
    s.push_str("    ac_retval=0\n");
    s.push_str("  else\n");
    s.push_str("    printf '%s\\n' \"configure: failed program was:\" >&5\n");
    s.push_str("    cat conftest.$ac_ext >&5\n");
    s.push_str("    ac_retval=1\n");
    s.push_str("  fi\n");
    s.push_str("  rm -f conftest.$ac_objext conftest.$ac_ext conftest$ac_exeext\n");
    s.push_str("  return $ac_retval\n");
    s.push_str("}\n\n");

    s.push_str("ac_fn_c_try_run() {\n");
    // ac_try must be DOUBLE-quoted so it holds $ac_link's value (`$CC -o ...`); single quotes left it
    // as literal `$ac_link` and `eval` expanded only one level, running `$CC` literally (command not found).
    s.push_str(
        "  if { ac_try=\"$ac_link\"; (eval \"$ac_try\") >&5; } && test -s conftest$ac_exeext &&\n",
    );
    s.push_str("     { ac_try=\"./conftest$ac_exeext\"; { (eval \"$ac_try\") >&5; }; }; then\n");
    s.push_str("    ac_retval=0\n");
    s.push_str("  else\n");
    s.push_str("    printf '%s\\n' \"configure: failed program was:\" >&5\n");
    s.push_str("    cat conftest.$ac_ext >&5\n");
    s.push_str("    ac_retval=1\n");
    s.push_str("  fi\n");
    s.push_str("  rm -f conftest.$ac_ext conftest$ac_exeext\n");
    s.push_str("  return $ac_retval\n");
    s.push_str("}\n\n");

    s.push_str("ac_fn_c_try_cpp() {\n");
    s.push_str("  rm -f conftest.$ac_objext conftest$ac_exeext\n");
    s.push_str(
        "  if { (eval \"$ac_cpp conftest.$ac_ext\") >&5; } && test -s conftest.$ac_objext; then\n",
    );
    s.push_str("    ac_retval=0\n");
    s.push_str("  else\n");
    s.push_str("    printf '%s\\n' \"configure: failed program was:\" >&5\n");
    s.push_str("    cat conftest.$ac_ext >&5\n");
    s.push_str("    ac_retval=1\n");
    s.push_str("  fi\n");
    s.push_str("  rm -f conftest.$ac_objext conftest.$ac_ext\n");
    s.push_str("  return $ac_retval\n");
    s.push_str("}\n\n");

    // Dynamic feature test body (AC_CHECK_*, AC_C_*, etc.)
    // Generated from prescan state — inserted after M4sh init prologue
    let feature_body = crate::configure_body::generate_feature_test_body(state);
    let feature_str = String::from_utf8_lossy(&feature_body);
    let has_actual_tests = state.has_compiler_check
        || !state.checked_funcs.is_empty()
        || !state.checked_headers.is_empty()
        || !state.checked_libs.is_empty()
        || !state.checked_types.is_empty()
        || !state.checked_progs.is_empty()
        || !state.checked_sizeofs.is_empty()
        || !state.c_conformance_checks.is_empty();
    if has_actual_tests {
        s.push_str("# Feature tests\n");
        s.push_str(&feature_str);
        s.push('\n');
    }

    // Set PACKAGE_BUGREPORT from AC_INIT third arg
    s.push_str(&format!(
        "PACKAGE_BUGREPORT='{}'\n",
        state.bug_report.as_deref().unwrap_or("")
    ));

    // Note: The prologue template already includes ac_subst_vars, variable defaults,
    // host/build triple detection, and directory defaults. We only add what's specific
    // to this configure.ac: program detection variables and explicit substitutions.

    // Explicit substitution variables from AC_SUBST
    for (var, value) in &state.substitutions {
        if !value.is_empty() {
            // Shell-escape single quotes so a value containing `'` can't break the assignment.
            let q = value.replace('\'', "'\\''");
            s.push_str(&format!("{}='{}'\n", var, q));
        }
    }

    // Stamp file support — tracks when config.status was last run
    s.push_str("# Stamp file support for dependency tracking\n");
    s.push_str("ac_config_stamp=\"\"\n");
    s.push_str("# Compare new Makefile with existing to avoid unnecessary rebuilds\n");
    s.push_str("ac_config_update() {\n");
    s.push_str("  if test -f \"$1\" && cmp -s \"$2\" \"$1\"; then\n");
    s.push_str("    rm -f \"$2\"\n");
    s.push_str("  else\n");
    s.push_str("    mv -f \"$2\" \"$1\"\n");
    s.push_str("    printf '%s\\n' \"updated $1\"\n");
    s.push_str("  fi\n}\n\n");

    // Ensure substitution function uses final variable values
    s.push_str("\n# Variable substitution helper\n");
    s.push_str("substitute() {\n");
    s.push_str("  # Create output directory if needed\n");
    s.push_str("  mkdir -p \"$(dirname \"$2\")\" 2>/dev/null || :\n");
    s.push_str(STD_VAR_DEFAULTS);
    s.push_str("  _cs=; test -f conf_subst.sed && _cs=\"-f conf_subst.sed\"\n  sed");
    // Always substitute standard Autoconf variables
    s.push_str(&format!(" -e 's|@PACKAGE_NAME@|{}|g'", name));
    s.push_str(&format!(" -e 's|@PACKAGE_VERSION@|{}|g'", version));
    s.push_str(&format!(" -e 's|@PACKAGE_STRING@|{} {}|g'", name, version));
    s.push_str(&format!(" -e 's|@PACKAGE_TARNAME@|{}|g'", name));
    s.push_str(&format!(" -e 's|@PACKAGE@|{}|g'", name));
    s.push_str(&format!(" -e 's|@VERSION@|{}|g'", version));
    s.push_str(" -e 's|@PACKAGE_BUGREPORT@||g'");
    s.push_str(" -e 's|@PACKAGE_URL@||g'");
    s.push_str(" -e 's|@srcdir@|$srcdir|g'");
    s.push_str(" -e 's|@prefix@|$prefix|g'");
    s.push_str(" -e 's|@exec_prefix@|$exec_prefix|g'");
    s.push_str(STD_VAR_SED);
    // Explicit substitutions (always include, even for empty values)
    for (var, value) in &state.substitutions {
        let value: &str = if value.is_empty() {
            match var.as_str() {
                "PACKAGE_NAME" => name,
                "PACKAGE_VERSION" => version,
                _ => "",
            }
        } else {
            value
        };
        s.push_str(&sed_subst_expr(var, value));
    }
    s.push_str(" ${_cs} \"$1\" > \"$2\"\n");
    s.push_str("  for _ph in `grep -oE '@[A-Za-z_][A-Za-z0-9_]*_(CFLAGS|LIBS|DEPS|REQUIRES)@' \"$2\" 2>/dev/null | sort -u`; do _vn=`printf '%s' \"$_ph\" | tr -d @`; eval \"_vv=\\$$_vn\"; _ve=`printf '%s' \"$_vv\" | sed 's/[&|]/\\\\&/g'`; sed \"s|$_ph|$_ve|g\" \"$2\" > \"$2._gt$$\" && mv -f \"$2._gt$$\" \"$2\"; done\n");
    s.push_str("}\n\n");

    // --- Feature test body (real AC_CHECK_* probes) --- MUST run BEFORE config headers are
    // created: each successful probe appends `#define HAVE_X 1` to confdefs.h, and the header loop
    // below bakes confdefs.h into config.h. (Previously the probes ran AFTER config.h was written,
    // so detected features never reached it.)
    let feature_body = crate::configure_body::generate_feature_test_body(state);
    let feature_str = String::from_utf8_lossy(&feature_body);
    let has_actual_tests = state.has_compiler_check
        || !state.checked_funcs.is_empty()
        || !state.checked_headers.is_empty()
        || !state.checked_libs.is_empty()
        || !state.checked_types.is_empty()
        || !state.checked_progs.is_empty()
        || !state.checked_sizeofs.is_empty()
        || !state.c_conformance_checks.is_empty();
    if has_actual_tests {
        s.push_str("# Feature tests\n");
        s.push_str(&feature_str);
        s.push('\n');
    }

    // Process config files
    for file in &state.config_files {
        s.push_str(&format!("printf '%s\\n' 'creating {}'\n", file));
        s.push_str(&format!("substitute '{}.in' '{}'\n", file, file));
    }

    // Process config headers
    for hdr in &state.config_headers {
        s.push_str(&format!("printf '%s\\n' 'creating {}'\n", hdr));
        // Build a sed script from confdefs.h (the RUNTIME probe results: each `#define HAVE_X 1` an
        // AC_CHECK_* appended) -> `s|#undef X|#define X V|`, applied to the header template so
        // detected features actually land in config.h. \x01 (SOH) is the sed delimiter to avoid
        // colliding with `|`/`/` in values. The static `-e` seds below cover AC_DEFINE/PACKAGE.
        s.push_str("sed -n 's|^#define \\([A-Za-z_][A-Za-z0-9_]*\\) \\(.*\\)$|s\x01#undef \\1\x01#define \\1 \\2\x01|p' confdefs.h > conf_defs$$.sed 2>/dev/null\n");
        s.push_str("sed -f conf_defs$$.sed");
        for (var, value) in &state.defines {
            // Conditional defines (AC_ARG_ENABLE/AC_ARG_WITH actions) are projected only by the
            // confdefs-driven sed above, when their gated runtime branch actually ran.
            if state.conditional_defines.contains(var) {
                continue;
            }
            s.push_str(&format!(
                " -e 's|#undef {}|#define {} {}|g'",
                var, var, value
            ));
        }
        // Standard AC_INIT-derived defines (config.h.in carries `#undef PACKAGE_NAME` etc. via
        // autoheader). `$`-anchored so `#undef PACKAGE`/`#undef VERSION` don't corrupt the longer
        // PACKAGE_* names. Without these a package that uses PACKAGE_NAME/VERSION fails to compile.
        s.push_str(&format!(" -e 's|#undef PACKAGE_NAME|#define PACKAGE_NAME \"{}\"|g'", name));
        s.push_str(&format!(" -e 's|#undef PACKAGE_TARNAME|#define PACKAGE_TARNAME \"{}\"|g'", name));
        s.push_str(&format!(" -e 's|#undef PACKAGE_VERSION|#define PACKAGE_VERSION \"{}\"|g'", version));
        s.push_str(&format!(" -e 's|#undef PACKAGE_STRING|#define PACKAGE_STRING \"{} {}\"|g'", name, version));
        s.push_str(" -e 's|#undef PACKAGE_BUGREPORT|#define PACKAGE_BUGREPORT \"\"|g'");
        s.push_str(" -e 's|#undef PACKAGE_URL|#define PACKAGE_URL \"\"|g'");
        s.push_str(&format!(" -e 's|#undef PACKAGE$|#define PACKAGE \"{}\"|g'", name));
        s.push_str(&format!(" -e 's|#undef VERSION$|#define VERSION \"{}\"|g'", version));
        // ATOMIC write so a concurrent compile never reads a half-written header.
        s.push_str(&format!(" '{hdr}.in' > '{hdr}.tmp$$' && mv -f '{hdr}.tmp$$' '{hdr}'; rm -f conf_defs$$.sed\n"));
    }

    s.push_str("\necho 'configure: creating config.status'\n");

    // Write config.status with --recheck support
    s.push_str("cat >config.status <<\\_ACEOF\n");
    s.push_str("#! /bin/sh\n");
    s.push_str(&format!(
        "# Generated by autoconf-rs for {} {}\n",
        name, version
    ));
    // --recheck: re-run configure from config.status
    s.push_str("if test \"$1\" = --recheck; then\n");
    s.push_str(
        "  echo running CONFIG_SHELL=/bin/sh /bin/sh ./configure --no-create --no-recursion\n",
    );
    s.push_str("  exec /bin/sh ./configure --no-create --no-recursion\n");
    s.push_str("fi\n");
    s.push_str("substitute() {\n");
    s.push_str("  mkdir -p \"$(dirname \"$2\")\" 2>/dev/null || :\n");
    s.push_str(STD_VAR_DEFAULTS);
    s.push_str("  _cs=; test -f conf_subst.sed && _cs=\"-f conf_subst.sed\"\n  sed");
    // Always substitute standard Autoconf variables
    s.push_str(&format!(" -e 's|@PACKAGE_NAME@|{}|g'", name));
    s.push_str(&format!(" -e 's|@PACKAGE_VERSION@|{}|g'", version));
    s.push_str(&format!(" -e 's|@PACKAGE_STRING@|{} {}|g'", name, version));
    s.push_str(&format!(" -e 's|@PACKAGE_TARNAME@|{}|g'", name));
    s.push_str(&format!(" -e 's|@PACKAGE@|{}|g'", name));
    s.push_str(&format!(" -e 's|@VERSION@|{}|g'", version));
    s.push_str(" -e 's|@PACKAGE_BUGREPORT@||g'");
    s.push_str(" -e 's|@PACKAGE_URL@||g'");
    s.push_str(" -e 's|@srcdir@|$srcdir|g'");
    s.push_str(" -e 's|@prefix@|$prefix|g'");
    s.push_str(" -e 's|@exec_prefix@|$exec_prefix|g'");
    s.push_str(STD_VAR_SED);
    // Explicit substitutions (always include, even for empty values)
    for (var, value) in &state.substitutions {
        let value: &str = if value.is_empty() {
            match var.as_str() {
                "PACKAGE_NAME" => name,
                "PACKAGE_VERSION" => version,
                _ => "",
            }
        } else {
            value
        };
        s.push_str(&sed_subst_expr(var, value));
    }
    s.push_str(" ${_cs} \"$1\" > \"$2\"\n");
    s.push_str("  for _ph in `grep -oE '@[A-Za-z_][A-Za-z0-9_]*_(CFLAGS|LIBS|DEPS|REQUIRES)@' \"$2\" 2>/dev/null | sort -u`; do _vn=`printf '%s' \"$_ph\" | tr -d @`; eval \"_vv=\\$$_vn\"; _ve=`printf '%s' \"$_vv\" | sed 's/[&|]/\\\\&/g'`; sed \"s|$_ph|$_ve|g\" \"$2\" > \"$2._gt$$\" && mv -f \"$2._gt$$\" \"$2\"; done\n");
    s.push_str("}\n");
    for file in &state.config_files {
        s.push_str(&format!(
            "substitute \"${{srcdir}}/{}.in\" '{}'\n",
            file, file
        ));
    }
    s.push_str("_ACEOF\n");
    s.push_str("chmod +x config.status\n");

    s
}

/// Generate only the config.status section (substitutions + heredoc).
/// Used when the full oracle-captured template provides the prologue.
pub fn generate_config_status_section(
    state: &super::autoconf_macros::AutoconfState,
    name: &str,
    version: &str,
) -> String {
    let mut s = String::new();

    // Write config.status for re-run capability
    s.push_str("cat >config.status <<\\_ACEOF\n");
    s.push_str("#! /bin/sh\n");
    s.push_str(&format!(
        "# Generated by autoconf-rs for {} {}\n",
        name, version
    ));
    s.push_str("substitute() {\n");
    s.push_str("  mkdir -p \"$(dirname \"$2\")\" 2>/dev/null || :\n");
    s.push_str(STD_VAR_DEFAULTS);
    // top_builddir/top_srcdir are relative to the OUTPUT file's directory, not always `.`: a subdir
    // Makefile (lib/Makefile) needs top_builddir=.. so -I$(top_builddir) finds the top config.h.
    s.push_str("  ac_d=`dirname \"$2\"`; case $ac_d in .|\"\") top_builddir=. ;; *) top_builddir=`printf '%s' \"$ac_d\" | sed 's,[^/][^/]*,..,g'` ;; esac; top_srcdir=$top_builddir\n");
    s.push_str("  _cs=; test -f conf_subst.sed && _cs=\"-f conf_subst.sed\"\n  sed");
    s.push_str(&format!(" -e 's|@PACKAGE_NAME@|{}|g'", name));
    s.push_str(&format!(" -e 's|@PACKAGE_VERSION@|{}|g'", version));
    s.push_str(&format!(" -e 's|@PACKAGE_STRING@|{} {}|g'", name, version));
    s.push_str(&format!(" -e 's|@PACKAGE_TARNAME@|{}|g'", name));
    s.push_str(&format!(" -e 's|@PACKAGE@|{}|g'", name));
    s.push_str(&format!(" -e 's|@VERSION@|{}|g'", version));
    s.push_str(&format!(
        " -e 's|@PACKAGE_BUGREPORT@|{}|g'",
        state.bug_report.as_deref().unwrap_or("")
    ));
    s.push_str(" -e 's|@PACKAGE_URL@||g'");
    s.push_str(" -e 's|@srcdir@|$srcdir|g'");
    s.push_str(" -e 's|@prefix@|$prefix|g'");
    s.push_str(" -e 's|@exec_prefix@|$exec_prefix|g'");
    s.push_str(STD_VAR_SED);
    for (var, value) in &state.substitutions {
        let escaped_val = if value.is_empty() {
            match var.as_str() {
                "PACKAGE_NAME" => name.replace('&', "\\&").replace('/', "\\/"),
                "PACKAGE_VERSION" => version.replace('&', "\\&").replace('/', "\\/"),
                "PACKAGE_BUGREPORT" => state
                    .bug_report
                    .as_deref()
                    .unwrap_or("")
                    .replace('&', "\\&")
                    .replace('/', "\\/"),
                _ => String::new(),
            }
        } else {
            value.replace('&', "\\&").replace('/', "\\/")
        };
        s.push_str(&format!(" -e 's|@{}@|{}|g'", var, escaped_val));
    }
    s.push_str(" ${_cs} \"$1\" > \"$2\"\n");
    s.push_str("  for _ph in `grep -oE '@[A-Za-z_][A-Za-z0-9_]*_(CFLAGS|LIBS|DEPS|REQUIRES)@' \"$2\" 2>/dev/null | sort -u`; do _vn=`printf '%s' \"$_ph\" | tr -d @`; eval \"_vv=\\$$_vn\"; _ve=`printf '%s' \"$_vv\" | sed 's/[&|]/\\\\&/g'`; sed \"s|$_ph|$_ve|g\" \"$2\" > \"$2._gt$$\" && mv -f \"$2._gt$$\" \"$2\"; done\n");
    s.push_str("}\n");
    for file in &state.config_files {
        s.push_str(&format!("substitute '{}.in' '{}'\n", file, file));
    }
    s.push_str("_ACEOF\n");
    s.push_str("chmod +x config.status\n");

    s
}

#[cfg(test)]
mod tests {
    use super::super::ConfigureAc;
    use super::*;

    #[test]
    fn test_sed_subst_expr_strips_quotes_and_is_safe() {
        // Surrounding shell quotes stripped -> $(...) is a LITERAL inside the single-quoted sed word
        // (make expands it), not command-substituted outside it.
        assert_eq!(sed_subst_expr("mkdir_p", "'$(MKDIR_P)'"), " -e 's|@mkdir_p@|$(MKDIR_P)|g'");
        assert_eq!(sed_subst_expr("am__isrc", "' -I$(srcdir)'"), " -e 's|@am__isrc@| -I$(srcdir)|g'");
        // sed delimiter and backref are escaped.
        assert_eq!(sed_subst_expr("X", "a|b&c"), " -e 's|@X@|a\\|b\\&c|g'");
        // an embedded single quote is shell-escaped via '\'' so the single-quoted word holds.
        assert!(sed_subst_expr("Y", "a'b").contains("'\\''"));
    }

    #[test]
    fn test_generate_minimal() {
        let ac = ConfigureAc::parse("AC_INIT([test], [1.0])\nAC_OUTPUT\n");
        let gen = ShellGenerator::new();
        let script = gen.generate(&ac, "# configure script content\n");
        assert!(script.contains("#! /bin/sh"));
        assert!(script.contains("test"));
    }
}
