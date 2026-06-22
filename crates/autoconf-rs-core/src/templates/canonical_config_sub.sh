# config.sub integration — canonicalize user-supplied --host/--build/--target triples.
# CROSS.020: config.sub canonicalization for cross-compilation support.
# Includes config.sub validation (sun4 test) matching GNU Autoconf behavior.

ac_config_sub=
for ac_dir in "$srcdir" "$srcdir/.." "$srcdir/../.." /usr/share/autoconf/build-aux /usr/share/misc /usr/local/share/autoconf/build-aux; do
  if test -f "$ac_dir/config.sub"; then
    ac_config_sub="$SHELL $ac_dir/config.sub"
    break
  fi
done

# Validate that config.sub actually works (GNU sun4 test)
if test -n "$ac_config_sub"; then
  if ! $ac_config_sub sun4 >/dev/null 2>&1; then
    as_fn_error $? "cannot run $ac_config_sub" "$LINENO" 5
  fi
fi

# Canonicalize host alias through config.sub when user specified --host
if test -n "$ac_config_sub" && test -n "$host_alias"; then
  host=`$ac_config_sub "$host_alias"` 2>/dev/null || host=$host_alias
elif test -z "$host"; then
  host=$host_alias
fi

# Canonicalize build alias similarly
if test -n "$ac_config_sub" && test -n "$build_alias"; then
  build=`$ac_config_sub "$build_alias"` 2>/dev/null || build=$build_alias
elif test -z "$build"; then
  build=$build_alias
fi

# Canonicalize target alias
if test -n "$ac_config_sub" && test -n "$target_alias"; then
  target=`$ac_config_sub "$target_alias"` 2>/dev/null || target=$target_alias
elif test -z "$target"; then
  target=$target_alias
fi

# Set cross-compilation flag when host != build
if test "x$host" != "x$build"; then
  cross_compiling=yes
else
  cross_compiling=no
fi
