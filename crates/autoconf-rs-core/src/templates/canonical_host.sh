# Determine the host system type. Uses config.guess if available, falls back to a normalized uname.
# CROSS.020: Integrated config.guess/config.sub shell-out with uname fallback.
# Court: AC.CANONICAL.020 — config.guess integration

ac_config_guess=
for ac_dir in "$srcdir" "$srcdir/.." "$srcdir/../.." "$srcdir/config" "$srcdir/build-aux" "$srcdir/aux-build" "$srcdir/scripts" /usr/share/autoconf/build-aux /usr/share/misc /usr/local/share/autoconf/build-aux; do
  if test -f "$ac_dir/config.guess"; then
    ac_config_guess="$SHELL $ac_dir/config.guess"
    break
  fi
done

if test -n "$ac_config_guess"; then
  host=`$ac_config_guess 2>/dev/null`
fi

# uname fallback (or if config.guess produced nothing): build a normalized GNU triple. The OS field
# must be lowercase and GNU-style (`linux-gnu`, not `Linux`) so project `case $host_os in linux*)`
# dispatch matches (postgres template selection, many others).
if test -z "$host"; then
  host_cpu=`uname -m 2>/dev/null || echo unknown`
  _acrs_os=`uname -s 2>/dev/null | tr 'A-Z' 'a-z'`
  test -z "$_acrs_os" && _acrs_os=unknown
  case $_acrs_os in
    linux)   host_os=linux-gnu ;;
    gnu/kfreebsd) host_os=kfreebsd-gnu ;;
    darwin*) host_os=$_acrs_os ;;
    *)       host_os=$_acrs_os ;;
  esac
  host="$host_cpu-pc-$host_os"
fi

# Split canonical triple cpu-vendor-os using POSIX parameter expansion. NB: sed char-classes like
# `[^-]` MUST NOT be used here — this text is an m4 macro body, so `[` / `]` are m4 quote characters
# and get stripped (`[^-]` -> `^-`), which silently broke the regex and left host_os = the whole triple.
host_cpu=${host%%-*}                    # first field
host_os=${host#*-}; host_os=${host_os#*-}   # drop cpu- and vendor- -> os (may contain dashes: linux-gnu)
host_vendor=${host#*-}; host_vendor=${host_vendor%%-*}   # middle field

# Fallback: if no vendor field (cpu-os format), synthesize a full triple with a lowercase OS.
if test "x$host_vendor" = "x" || test "x$host_vendor" = "x$host"; then
  host_cpu=`uname -m 2>/dev/null || echo unknown`
  host_vendor=pc
  _acrs_os=`uname -s 2>/dev/null | tr 'A-Z' 'a-z'`
  test -z "$_acrs_os" && _acrs_os=unknown
  case $_acrs_os in
    linux) host_os=linux-gnu ;;
    *)     host_os=$_acrs_os ;;
  esac
  host="$host_cpu-$host_vendor-$host_os"
fi

# AC_SUBST the canonical variables
host_alias=$host
