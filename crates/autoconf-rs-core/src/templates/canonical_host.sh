# Determine the host system type. Uses config.guess if available, falls back to uname.
# CROSS.020: Integrated config.guess/config.sub shell-out with uname fallback.
# Court: AC.CANONICAL.020 — config.guess integration

ac_config_guess=
for ac_dir in "$srcdir" "$srcdir/.." "$srcdir/../.." /usr/share/autoconf/build-aux /usr/share/misc /usr/local/share/autoconf/build-aux; do
  if test -f "$ac_dir/config.guess"; then
    ac_config_guess="$SHELL $ac_dir/config.guess"
    break
  fi
done

if test -n "$ac_config_guess"; then
  host=`$ac_config_guess`
else
  host_cpu=`uname -m 2>/dev/null || echo unknown`
  host_os=`uname -s 2>/dev/null || echo unknown`
  host="$host_cpu-unknown-$host_os"
fi

# Split canonical triple into cpu-vendor-os
host_cpu=`echo $host | sed 's/^\([^-]*\)-\([^-]*\)-\(.*\)$/\1/'`
host_vendor=`echo $host | sed 's/^\([^-]*\)-\([^-]*\)-\(.*\)$/\2/'`
host_os=`echo $host | sed 's/^\([^-]*\)-\([^-]*\)-\(.*\)$/\3/'`

# Fallback: if no vendor field (uname fallback produced cpu-os format)
if test "x$host_vendor" = "x"; then
  host_cpu=`uname -m 2>/dev/null || echo unknown`
  host_vendor=unknown
  host_os=`uname -s 2>/dev/null || echo unknown`
  host="$host_cpu-$host_vendor-$host_os"
fi

# AC_SUBST the canonical variables
host_alias=$host
