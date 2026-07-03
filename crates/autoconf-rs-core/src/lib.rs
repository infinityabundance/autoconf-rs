// ============================================================================
// autoconf-rs-core: Core macro engine, shell generator, and Autoconf library
// for forensic-parity GNU Autoconf.
// ============================================================================
//
// Uses m4-rs (https://crates.io/crates/m4-rs) for M4 macro processing.
// Autoconf-specific macros, shell generation, and state management are
// implemented in this crate.

// M4 engine modules (replaced by m4-rs)
// pub mod args;           // → m4_rs::m4_rs_core::args
// pub mod builtin;        // → m4_rs::m4_rs_core::builtin
// pub mod expansion;      // → m4_rs::m4_rs_core::ExpansionEngine
// pub mod lexer;          // → m4_rs::m4_rs_core::Lexer
// pub mod token;          // → m4_rs::m4_rs_core::Token
// pub mod macro_table;    // → m4_rs::m4_rs_core::MacroTable

// Autoconf-specific modules
pub mod aclocal;
pub mod autoconf_macros;
pub mod autoheader;
pub mod autom4te;
pub mod autoreconf;
pub mod cache;
pub mod configure_ac;
pub mod configure_body;
pub mod configure_template;
pub mod diagnostics;
pub mod diversion;
pub mod fortran;
pub mod frozen;
pub mod i18n;
pub mod input_validate;
pub mod languages;
pub mod m4_engine;
pub mod m4sh;
pub mod m4sh_init;
pub mod m4sugar;
pub mod profile;
pub mod shell_gen;
pub mod signal;
pub mod site_file;
pub mod template;
pub mod trace;
pub mod trace_cache;

/// Re-export key types for convenience.
pub use configure_ac::ConfigureAc;
pub use diagnostics::{Diagnostic, DiagnosticLevel};
pub use m4_engine::M4Engine;
pub use shell_gen::ShellGenerator;

/// m4 macro definitions injected AFTER aclocal.m4 (so they override the project's third-party
/// definitions) and BEFORE configure.ac. autoconf-rs cannot yet correctly expand some standard
/// third-party macros (notably pkg.m4's PKG_CHECK_MODULES, whose internals leak
/// `pkg_default`/`glib_minimum` -> shell syntax errors). We replace them with clean, self-contained
/// shell that does the real work: run pkg-config at configure time, set PFX_CFLAGS/PFX_LIBS, and
/// record `@PFX_CFLAGS@`/`@PFX_LIBS@` substitutions into conf_subst.sed (consumed when config files
/// are created, mirroring confdefs.h -> config.h).
pub fn macro_overrides() -> &'static str {
    r#"dnl --- autoconf-rs macro overrides ---
dnl NB: do NOT use shell `eval` here — m4's own `eval` builtin would consume it (eval "x" -> 0).
dnl We just set + export $1_CFLAGS/$1_LIBS; the generic @VAR@ pass in config-file creation substitutes
dnl them (using the EXPORTED runtime values), so no m4-level value plumbing is needed.
define([PKG_CHECK_MODULES], [dnl
printf %s "checking for $2... "
if pkg-config --exists "$2" 2>/dev/null; then
  printf '%s\n' "yes"
  $1_CFLAGS=`pkg-config --cflags "$2" 2>/dev/null`
  $1_LIBS=`pkg-config --libs "$2" 2>/dev/null`
  export $1_CFLAGS $1_LIBS
  :
  $3
else
  printf '%s\n' "no"
  $1_CFLAGS=
  $1_LIBS=
  export $1_CFLAGS $1_LIBS
  :
  $4
fi
dnl Substitute @PFX_CFLAGS@/@PFX_LIBS@ in generated files (else they leak -> `ld: cannot find
dnl @RELAY_LIBS@`). Inline (NOT AC_SUBST([$1_LIBS]) — the prescan would capture the literal `$1_LIBS`
dnl from this definition into state.substitutions -> a broken `s|@$1_LIBS@|${$1_LIBS}|g`). Shell
dnl indirection reads the runtime value; m4 expands $1->PFX at the call site.
eval "_acrs_pc=\${$1_CFLAGS}"; printf '%s\n' "s|@$1_CFLAGS@|${_acrs_pc}|g" >> conf_subst.sed 2>/dev/null
eval "_acrs_pl=\${$1_LIBS}"; printf '%s\n' "s|@$1_LIBS@|${_acrs_pl}|g" >> conf_subst.sed 2>/dev/null
])dnl
define([PKG_CHECK_EXISTS], [dnl
if pkg-config --exists "$1" 2>/dev/null; then
  :
  $2
else
  :
  $3
fi
])dnl
define([PKG_PROG_PKG_CONFIG], [PKG_CONFIG=`command -v pkg-config 2>/dev/null`
printf '%s\n' "s|@PKG_CONFIG@|$PKG_CONFIG|g" >> conf_subst.sed 2>/dev/null])dnl
define([PKG_INSTALLDIR], [pkgconfigdir=${libdir}/pkgconfig
printf '%s\n' "s|@pkgconfigdir@|$pkgconfigdir|g" >> conf_subst.sed 2>/dev/null])dnl
define([PKG_NOARCH_INSTALLDIR], [noarch_pkgconfigdir=${datadir}/pkgconfig
printf '%s\n' "s|@noarch_pkgconfigdir@|$noarch_pkgconfigdir|g" >> conf_subst.sed 2>/dev/null])dnl
define([AX_CXX_COMPILE_STDCXX], [dnl
printf %s "checking for C++$1 support... "
ax_cxx_std_ok=no
for ax_sw in -std=c++$1 -std=gnu++$1 -std=c++0x; do
  printf '%s\n' "int main(){return 0;}" > conftest.cpp
  if ${CXX:-c++} $ax_sw -c conftest.cpp -o conftest.o >/dev/null 2>&1; then
    CXX="${CXX:-c++} $ax_sw"; CXXFLAGS="$CXXFLAGS $ax_sw"; ax_cxx_std_ok=yes; HAVE_CXX$1=1; break
  fi
done
rm -f conftest.cpp conftest.o
printf '%s\n' "$ax_cxx_std_ok"
printf '%s\n' "s|@CXX@|$CXX|g" >> conf_subst.sed 2>/dev/null
printf '%s\n' "s|@CXXFLAGS@|$CXXFLAGS|g" >> conf_subst.sed 2>/dev/null
:
])dnl
dnl AX_PTHREAD(ACT-IF-FOUND, ACT-IF-NOT): the vendored autoconf-archive macro's multi-flag link loop
dnl dies in our generated configure (top failure root, 9 repos: "pthreads work with -mt... no"). Replace
dnl with a clean probe: try -pthread / -pthreads / -lpthread (flag AFTER source so it works for both
dnl compile-flags and link-libs), set PTHREAD_CC/CFLAGS/LIBS, AC_SUBST them, run the action.
define([AX_PTHREAD], [dnl
printf %s "checking for the pthreads flag... "
ax_pthread_ok=no
PTHREAD_CC="${CC:-cc}"
PTHREAD_CFLAGS=
PTHREAD_LIBS=
printf '%s\n' "int main(void){return 0;}" > conftest.c
for ax_pthread_flag in -pthread -pthreads -lpthread; do
  if ${CC:-cc} conftest.c $ax_pthread_flag -o conftest.pthr >/dev/null 2>&1; then
    case $ax_pthread_flag in
      -l*) PTHREAD_LIBS="$ax_pthread_flag" ;;
      *) PTHREAD_CFLAGS="$ax_pthread_flag"; PTHREAD_LIBS="$ax_pthread_flag" ;;
    esac
    ax_pthread_ok=yes
    break
  fi
done
rm -f conftest.c conftest.pthr
printf '%s\n' "${PTHREAD_CFLAGS:-${PTHREAD_LIBS:-none}}"
export PTHREAD_CC PTHREAD_CFLAGS PTHREAD_LIBS
printf '%s\n' "s|@PTHREAD_CC@|$PTHREAD_CC|g" >> conf_subst.sed 2>/dev/null
printf '%s\n' "s|@PTHREAD_CFLAGS@|$PTHREAD_CFLAGS|g" >> conf_subst.sed 2>/dev/null
printf '%s\n' "s|@PTHREAD_LIBS@|$PTHREAD_LIBS|g" >> conf_subst.sed 2>/dev/null
if test "x$ax_pthread_ok" = xyes; then
  :
  $1
else
  :
  $2
fi
])dnl
dnl C-feature probes: on a modern compiler const/inline/volatile/restrict always work, so the macro's
dnl only job (AC_DEFINE the keyword to empty/__inline__ on ancient compilers) is a no-op. The vendored /
dnl aclocal definitions of these leak their explanatory comment text after a `fi` (the
dnl leaked-text-after-conditional root). Override with a clean probe-line + cache var so they win over
dnl aclocal and stop leaking. (The engine also defines them, but engine defines lose to aclocal.m4.)
define([AC_C_CONST], [printf '%s\n' "checking for working const... yes"
ac_cv_c_const=yes])dnl
define([AC_C_INLINE], [printf '%s\n' "checking for inline... inline"
ac_cv_c_inline=inline])dnl
define([AC_C_VOLATILE], [ac_cv_c_volatile=yes])dnl
define([AC_C_RESTRICT], [printf '%s\n' "checking for C/C++ restrict keyword... restrict"
ac_cv_c_restrict=restrict])dnl
dnl _AM_DEPENDENCIES (automake dep-tracking internals) leaks its language arg (OBJC/CXX) raw; the
dnl dep-mode is handled by our generated Makefile machinery, so a no-op here is correct.
define([_AM_DEPENDENCIES], [])dnl
dnl AC_USE_SYSTEM_EXTENSIONS / AC_GNU_SOURCE: enable the platform "system extensions" by AC_DEFINE-ing
dnl the feature-test macros into config.h (BEFORE any system header is included). Was a no-op stub, so
dnl GNU-only symbols (CLONE_*, UIO_MAXIOV, __THROW, asprintf, …) were "undeclared" -> compile errors on a
dnl big class of Linux projects. _GNU_SOURCE is the load-bearing one on glibc; the rest are harmless on Linux.
define([AC_USE_SYSTEM_EXTENSIONS], [AC_DEFINE([_GNU_SOURCE], [1], [Enable GNU/system extensions])
AC_DEFINE([__EXTENSIONS__], [1], [Enable general extensions on Solaris.])
AC_DEFINE([_ALL_SOURCE], [1], [Enable extensions on AIX, Interix.])
AC_DEFINE([_POSIX_PTHREAD_SEMANTICS], [1], [Enable POSIX pthread semantics on Solaris.])
AC_DEFINE([_DARWIN_C_SOURCE], [1], [Enable extensions on macOS.])
AC_DEFINE([_TANDEM_SOURCE], [1], [Enable extensions on HP NonStop.])])dnl
define([AC_GNU_SOURCE], [AC_DEFINE([_GNU_SOURCE], [1], [Enable GNU extensions])])dnl
define([AX_CXX_COMPILE_STDCXX_11], [AX_CXX_COMPILE_STDCXX([11])])dnl
define([AX_CXX_COMPILE_STDCXX_14], [AX_CXX_COMPILE_STDCXX([14])])dnl
define([AX_CXX_COMPILE_STDCXX_17], [AX_CXX_COMPILE_STDCXX([17])])dnl
define([AX_CXX_COMPILE_STDCXX_20], [AX_CXX_COMPILE_STDCXX([20])])dnl
dnl AX_APPEND_FLAG(FLAG, [FLAGS-VAR=CFLAGS]): append FLAG to the shell var named by FLAGS-VAR iff
dnl not already present. The vendored autoconf-archive macro uses AS_VAR_PUSHDEF([FLAGS],
dnl [m4_default([$2],[CFLAGS])]) — our engine stores that pushdef value UNEXPANDED and doesn't
dnl re-expand it on use, so `$FLAGS` leaked as the literal `m4_default([$2],[CFLAGS])` sanitized to
dnl `m4_default__C_FLAGS__` (wolfssl -> `-Wunused-variable: command not found`). Override with a clean
dnl form: m4_default resolves the var name at m4-time (works standalone), no AS_VAR_PUSHDEF needed.
define([AX_APPEND_FLAG], [dnl
case " ${m4_default([$2],[CFLAGS])} " in
  *" $1 "*) ;;
  *) m4_default([$2],[CFLAGS])="${m4_default([$2],[CFLAGS]):+${m4_default([$2],[CFLAGS])} }$1" ;;
esac])dnl
dnl AX_APPEND_COMPILE_FLAGS(FLAGS, [VAR=CFLAGS], [EXTRA-FLAGS], [INPUT]): for each FLAG, compile-check
dnl it (with EXTRA-FLAGS) and AX_APPEND_FLAG it to VAR on success. m4_foreach_w iterates the flags at
dnl m4-time so AX_CHECK_COMPILE_FLAG gets a literal flag (proper cache var) and our clean AX_APPEND_FLAG
dnl does the append. Sidesteps the same AS_VAR_PUSHDEF wall the vendored version hits.
define([AX_APPEND_COMPILE_FLAGS], [dnl
m4_foreach_w([_acrs_cf], [$1], [AX_CHECK_COMPILE_FLAG(_acrs_cf, [AX_APPEND_FLAG([_acrs_cf], [$2])], [], [$3], [$4])
])])dnl
define([AX_APPEND_LINK_FLAGS], [dnl
m4_foreach_w([_acrs_lf], [$1], [AX_CHECK_LINK_FLAG(_acrs_lf, [AX_APPEND_FLAG([_acrs_lf], [m4_default([$2],[LDFLAGS])])], [], [$3], [$4])
])])dnl
define([LT_INIT], [_acrs_write_libtool])dnl
define([LT_OUTPUT], [_acrs_write_libtool])dnl
define([AC_PROG_LIBTOOL], [_acrs_write_libtool])dnl
define([AM_PROG_LIBTOOL], [_acrs_write_libtool])dnl
define([LT_LANG], [])dnl
dnl LT_LIB_M: find the math library (-lm) into $LIBM and AC_SUBST it. Was undefined -> leaked
dnl `LT_LIB_M: command not found` (wolfssl). Link-probe cos(); AC_SUBST via the runtime sed sink.
define([LT_LIB_M], [AC_CHECK_LIB([m], [cos], [LIBM=-lm], [LIBM=])
eval "_acrs_sv=${LIBM}"; printf '%s\n' "s|@LIBM@|${_acrs_sv}|g" >> conf_subst.sed 2>/dev/null])dnl
define([LTOPTIONS_VERSION], [])dnl
dnl No-result macros that aclocal.m4's m4sugar/automake definitions otherwise leak as command-not-found
dnl (the engine no-output defaults lose to aclocal.m4; these overrides win, spliced after it).
define([AM_PROG_AS], [])dnl
define([AC_OPENMP], [])dnl
define([AC_ISC_POSIX], [])dnl
define([AM_PROG_VALAC], [])dnl
define([AC_SYS_INTERPRETER], [])dnl
define([AM_PROG_AR], [])dnl
define([AM_SILENT_RULES], [])dnl
dnl AH_* are autoheader (config.h.in) directives; they must emit nothing into configure. Override the
dnl m4sugar definitions that aclocal.m4 may pull in (they leak `m4_define([_ah_top], ...)`).
define([AH_TOP], [])dnl
define([AH_BOTTOM], [])dnl
define([AH_VERBATIM], [])dnl
define([AH_TEMPLATE], [])dnl
"#
}

/// A minimal but functional `libtool` wrapper script (autoconf-rs native, GNU-free). Emitted by
/// configure when the project uses LT_INIT/AC_PROG_LIBTOOL. Handles the common automake toolchain:
///   --mode=compile CC ... -c src -o obj.lo   -> .libs/obj.o (PIC) + obj.o + obj.lo descriptor
///   --mode=link    CC ... -o lib.la objs     -> .libs/lib.a (static) + .libs/lib.so (shared) + lib.la
///   --mode=link    CC ... -o prog objs *.la  -> resolves .lo/.la and links the program
///   --mode=install / --mode=clean            -> best-effort copy / rm
pub fn libtool_script() -> &'static str {
    r##"#! /bin/sh
# Minimal libtool wrapper generated by autoconf-rs (GNU-free).
mode=
for a in "$@"; do case $a in --mode=*) mode=${a#--mode=} ;; esac; done
# Drop libtool's own options from the positional params, preserving every other arg EXACTLY.
# The old `cmd="$cmd $a"` + `eval set -- $cmd` re-split and re-evaluated the args, stripping the
# literal quotes from `-DFOO=\"/path\"` defines -> gcc saw `-DFOO=/path` -> `<command-line>: expected
# expression before '/' token` (a systemic make-stage failure for libtool projects with path defines,
# e.g. pup-volume-monitor's -DPUP_VM_SCRIPTS_DIR). Rotate kept args to the end so quoting survives.
_lt_n=$#
while test $_lt_n -gt 0; do
  _lt_a=$1; shift; _lt_n=$((_lt_n-1))
  case $_lt_a in
    --mode=*|--tag=*|--silent|--quiet|--verbose|--no-silent|--preserve-dup-deps) continue ;;
  esac
  set -- "$@" "$_lt_a"
done

case $mode in
compile)
  comp=$1; shift
  flags=; src=; lo=
  while test $# -gt 0; do
    case $1 in
      -o) shift; lo=$1 ;;
      -c) ;;
      *) flags="$flags $1"; case $1 in *.c|*.cc|*.cpp|*.cxx|*.C|*.s|*.S) src=$1 ;; esac ;;
    esac
    shift
  done
  test -n "$lo" || lo=`echo "$src" | sed 's/\.[^.]*$/.lo/'`
  obj=`echo "$lo" | sed 's/\.lo$/.o/'`
  dir=`dirname "$lo"`; base=`basename "$obj"`
  mkdir -p "$dir/.libs"
  $comp $flags -fPIC -DPIC -c -o "$dir/.libs/$base" || exit 1
  $comp $flags -c -o "$obj" 2>/dev/null || cp "$dir/.libs/$base" "$obj"
  { echo "# $lo - libtool object (autoconf-rs)"
    echo "pic_object='.libs/$base'"
    echo "non_pic_object='$base'"; } > "$lo"
  exit 0
  ;;
link)
  comp=$1; shift
  out=; inobjs=; deplibs=; rest=; rpaths=
  while test $# -gt 0; do
    case $1 in
      -o) shift; out=$1 ;;
      -rpath|-version-info|-release|-version-number) shift ;;
      -export-dynamic|-module|-avoid-version|-no-undefined|-shared|-static|-no-install|-export-symbols-regex) ;;
      -export-symbols-regex) shift ;;
      *.lo) o=`echo "$1" | sed 's/\.lo$/.o/'`; d=`dirname "$1"`; b=`basename "$o"`; inobjs="$inobjs $d/.libs/$b" ;;
      *.o) inobjs="$inobjs $1" ;;
      # A `-L<dir>/.libs` search path points at an uninstalled libtool library's build dir (e.g. a
      # project links its own lib via `-L./.libs -lfoo` in *_LDFLAGS rather than the .la). Keep the
      # -L and ALSO rpath its absolute path so the program finds libfoo.so at run time (make check).
      -L*.libs|-L*.libs/) rest="$rest $1"; _ldir=${1#-L}; _lab=`cd "$_ldir" 2>/dev/null && pwd`; test -n "$_lab" && rpaths="$rpaths -Wl,-rpath,$_lab" ;;
      # A .la dependency: link `-L<dir>/.libs -l<name>`, AND bake an absolute -rpath to that build
      # `.libs` so the resulting program/library finds the uninstalled .so at RUN time (GNU libtool
      # uses a wrapper script for this; we rpath the build dir instead). Without it, running from the
      # build tree — e.g. `make check` — died: "libfoo.so: cannot open shared object file".
      *.la) la=$1; d=`dirname "$la"`; n=`basename "$la" .la`; deplibs="$deplibs -L$d/.libs -l${n#lib}"; _labs=`cd "$d" 2>/dev/null && pwd`; test -n "$_labs" && rpaths="$rpaths -Wl,-rpath,$_labs/.libs" ;;
      *) rest="$rest $1" ;;
    esac
    shift
  done
  case $out in
  *.la)
    d=`dirname "$out"`; n=`basename "$out" .la`; mkdir -p "$d/.libs"
    ${AR:-ar} cru "$d/.libs/$n.a" $inobjs && ${RANLIB:-ranlib} "$d/.libs/$n.a" 2>/dev/null
    $comp -shared -fPIC -o "$d/.libs/$n.so" $inobjs $deplibs $rpaths $rest 2>/dev/null || :
    { echo "# $out - libtool library (autoconf-rs)"
      echo "dlname='$n.so'"
      echo "library_names='$n.so'"
      echo "old_library='$n.a'"
      echo "installed=no"
      echo "libdir='/usr/local/lib'"; } > "$out"
    ;;
  *)
    mkdir -p .libs
    $comp -o "$out" $inobjs $deplibs $rpaths $rest || exit 1
    ;;
  esac
  exit 0
  ;;
install)
  shift 2>/dev/null
  cp $@ 2>/dev/null || :
  exit 0
  ;;
clean|uninstall)
  exit 0
  ;;
*)
  eval "$cmd"
  ;;
esac
"##
}
