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
define([AX_CXX_COMPILE_STDCXX_11], [AX_CXX_COMPILE_STDCXX([11])])dnl
define([AX_CXX_COMPILE_STDCXX_14], [AX_CXX_COMPILE_STDCXX([14])])dnl
define([AX_CXX_COMPILE_STDCXX_17], [AX_CXX_COMPILE_STDCXX([17])])dnl
define([AX_CXX_COMPILE_STDCXX_20], [AX_CXX_COMPILE_STDCXX([20])])dnl
define([LT_INIT], [_acrs_write_libtool])dnl
define([LT_OUTPUT], [_acrs_write_libtool])dnl
define([AC_PROG_LIBTOOL], [_acrs_write_libtool])dnl
define([AM_PROG_LIBTOOL], [_acrs_write_libtool])dnl
define([LT_LANG], [])dnl
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
# Collect the real command, dropping libtool's own options.
cmd=
for a in "$@"; do
  case $a in
    --mode=*|--tag=*|--silent|--quiet|--verbose|--no-silent|--preserve-dup-deps) continue ;;
  esac
  cmd="$cmd $a"
done
eval set -- $cmd

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
  out=; inobjs=; deplibs=; rest=
  while test $# -gt 0; do
    case $1 in
      -o) shift; out=$1 ;;
      -rpath|-version-info|-release|-version-number) shift ;;
      -export-dynamic|-module|-avoid-version|-no-undefined|-shared|-static|-no-install|-export-symbols-regex) ;;
      -export-symbols-regex) shift ;;
      *.lo) o=`echo "$1" | sed 's/\.lo$/.o/'`; d=`dirname "$1"`; b=`basename "$o"`; inobjs="$inobjs $d/.libs/$b" ;;
      *.o) inobjs="$inobjs $1" ;;
      *.la) la=$1; d=`dirname "$la"`; n=`basename "$la" .la`; deplibs="$deplibs -L$d/.libs -l${n#lib}" ;;
      *) rest="$rest $1" ;;
    esac
    shift
  done
  case $out in
  *.la)
    d=`dirname "$out"`; n=`basename "$out" .la`; mkdir -p "$d/.libs"
    ${AR:-ar} cru "$d/.libs/$n.a" $inobjs && ${RANLIB:-ranlib} "$d/.libs/$n.a" 2>/dev/null
    $comp -shared -fPIC -o "$d/.libs/$n.so" $inobjs $deplibs $rest 2>/dev/null || :
    { echo "# $out - libtool library (autoconf-rs)"
      echo "dlname='$n.so'"
      echo "library_names='$n.so'"
      echo "old_library='$n.a'"
      echo "installed=no"
      echo "libdir='/usr/local/lib'"; } > "$out"
    ;;
  *)
    mkdir -p .libs
    $comp -o "$out" $inobjs $deplibs $rest || exit 1
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
