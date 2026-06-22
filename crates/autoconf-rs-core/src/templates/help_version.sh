# Report the --help message.
#
if test "$ac_init_help" = "long"; then
  # Omit some internal or obsolete options to make the list less imposing.
  # This message is too long to be a string in the A/UX 3.1 sh.
  cat <<_ACEOF
'configure' configures {NAME} {VERSION} to adapt to many kinds of systems.

Usage: $0 [OPTION]... [VAR=VALUE]...

To assign environment variables (e.g., CC, CFLAGS...), specify them as
VAR=VALUE.  See below for descriptions of some of the useful variables.

Defaults for the options are specified in brackets.

Configuration:
  -h, --help              display this help and exit
      --help=short        display options specific to this package
      --help=recursive    display the short help of all the included packages
  -V, --version           display version information and exit
  -q, --quiet, --silent   do not print 'checking ...' messages
      --cache-file=FILE   cache test results in FILE [disabled]
  -C, --config-cache      alias for '--cache-file=config.cache'
  -n, --no-create         do not create output files
      --srcdir=DIR        find the sources in DIR [configure dir or '..']

Installation directories:
  --prefix=PREFIX         install architecture-independent files in PREFIX
                          [$ac_default_prefix]
  --exec-prefix=EPREFIX   install architecture-dependent files in EPREFIX
                          [PREFIX]

By default, 'make install' will install all the files in
'$ac_default_prefix/bin', '$ac_default_prefix/lib' etc.  You can specify
an installation prefix other than '$ac_default_prefix' using '--prefix',
for instance '--prefix=\$HOME'.

For better control, use the options below.

Fine tuning of the installation directories:
  --bindir=DIR            user executables [EPREFIX/bin]
  --sbindir=DIR           system admin executables [EPREFIX/sbin]
  --libexecdir=DIR        program executables [EPREFIX/libexec]
  --sysconfdir=DIR        read-only single-machine data [PREFIX/etc]
  --sharedstatedir=DIR    modifiable architecture-independent data [PREFIX/com]
  --localstatedir=DIR     modifiable single-machine data [PREFIX/var]
  --runstatedir=DIR       modifiable per-process data [LOCALSTATEDIR/run]
  --libdir=DIR            object code libraries [EPREFIX/lib]
  --includedir=DIR        C header files [PREFIX/include]
  --oldincludedir=DIR     C header files for non-gcc [/usr/include]
  --datarootdir=DIR       read-only arch.-independent data root [PREFIX/share]
  --datadir=DIR           read-only architecture-independent data [DATAROOTDIR]
  --infodir=DIR           info documentation [DATAROOTDIR/info]
  --localedir=DIR         locale-dependent data [DATAROOTDIR/locale]
  --mandir=DIR            man documentation [DATAROOTDIR/man]
  --docdir=DIR            documentation root [DATAROOTDIR/doc/{NAME}]
  --htmldir=DIR           html documentation [DOCDIR]
  --dvidir=DIR            dvi documentation [DOCDIR]
  --pdfdir=DIR            pdf documentation [DOCDIR]
  --psdir=DIR             ps documentation [DOCDIR]
_ACEOF

  cat <<\_ACEOF
_ACEOF
fi

if test -n "$ac_init_help"; then
  case $ac_init_help in
     short | recursive ) echo "Configuration of {NAME} {VERSION}:";;
   esac
  cat <<\_ACEOF

Report bugs to <{BUGREPORT}>.
_ACEOF
ac_status=$?
fi

if test "$ac_init_help" = "recursive"; then
  # If there are subdirs, report their specific --help.
  for ac_dir in : $ac_subdirs_all; do test "x$ac_dir" = x: && continue
    test -d "$ac_dir" ||
      { cd "$srcdir" && ac_pwd=`pwd` && srcdir=. && test -d "$ac_dir"; } ||
      continue
    ac_builddir=.

case "$ac_dir" in
.) ac_dir_suffix= ac_top_builddir_sub=. ac_top_build_prefix= ;;
*)
  ac_dir_suffix=/`printf '%s\n' "$ac_dir" | sed 's|^\.[\\/]||'`
  # A ".." for each directory in $ac_dir_suffix.
  ac_top_builddir_sub=`printf '%s\n' "$ac_dir_suffix" | sed 's|/[^\\/]*|/..|g;s|/||'`
  case $ac_top_builddir_sub in
  "") ac_top_builddir_sub=. ac_top_build_prefix= ;;
  *)  ac_top_build_prefix=$ac_top_builddir_sub/ ;;
  esac ;;
esac
ac_abs_top_builddir=$ac_pwd
ac_abs_builddir=$ac_pwd$ac_dir_suffix
# for backward compatibility:
ac_top_builddir=$ac_top_build_prefix

case $srcdir in
  .)  # We are building in place.
    ac_srcdir=.
    ac_top_srcdir=$ac_top_builddir_sub
    ac_abs_top_srcdir=$ac_pwd ;;
  [\\/]* | ?:[\\/]* )  # Absolute name.
    ac_srcdir=$srcdir$ac_dir_suffix;
    ac_top_srcdir=$srcdir
    ac_abs_top_srcdir=$srcdir ;;
  *) # Relative name.
    ac_srcdir=$ac_top_build_prefix$srcdir$ac_dir_suffix
    ac_top_srcdir=$ac_top_build_prefix$srcdir
    ac_abs_top_srcdir=$ac_pwd/$srcdir ;;
esac
ac_abs_srcdir=$ac_abs_top_srcdir$ac_dir_suffix

    cd "$ac_dir" || { ac_status=$?; continue; }
    # Check for configure.gnu first; this name is used for a wrapper for
    # Metaconfig's "Configure" on case-insensitive file systems.
    if test -f "$ac_srcdir/configure.gnu"; then
      echo &&
      $SHELL "$ac_srcdir/configure.gnu" --help=recursive
    elif test -f "$ac_srcdir/configure"; then
      echo &&
      $SHELL "$ac_srcdir/configure" --help=recursive
    else
      printf '%s\n' "$as_me: WARNING: no configuration information is in $ac_dir" >&2
    fi || ac_status=$?
    cd "$ac_pwd" || { ac_status=$?; break; }
  done
fi

test -n "$ac_init_help" && exit $ac_status
if $ac_init_version; then
  cat <<\_ACEOF
{NAME} configure {VERSION}
generated by GNU Autoconf 2.73

Copyright (C) 2026 Free Software Foundation, Inc.
This configure script is free software; the Free Software Foundation
gives unlimited permission to copy, distribute and modify it.
_ACEOF
  exit
fi

## ------------------------ ##
## Autoconf initialization. ##
## ------------------------ ##
ac_configure_args_raw=
for ac_arg
do
  case $ac_arg in
  *\'*)
    ac_arg=`printf '%s\n' "$ac_arg" | sed "s/'/'\\\\\\\\''/g"` ;;
  esac
  as_fn_append ac_configure_args_raw " '$ac_arg'"
done

case $ac_configure_args_raw in
  *$as_nl*)
    ac_safe_unquote= ;;
  *)
    ac_unsafe_z='|&;<>()$`\\"*?[ ''	' # This string ends in space, tab.
    ac_unsafe_a="$ac_unsafe_z#~"
    ac_safe_unquote="s/ '\\([^$ac_unsafe_a][^$ac_unsafe_z]*\\)'/ \\1/g"
    ac_configure_args_raw=`      printf '%s\n' "$ac_configure_args_raw" | sed "$ac_safe_unquote"`;;
esac

