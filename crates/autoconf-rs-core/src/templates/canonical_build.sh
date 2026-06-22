# Determine the build system type. Uses config.guess if available, falls back to host.
# CROSS.020: config.guess integration for build canonical triple.
# In cross-compilation scenarios, build system differs from host system.

if test "x$build_alias" = x; then
  if test -n "$ac_config_guess"; then
    build=`$ac_config_guess`
  else
    build=$host
    build_cpu=$host_cpu
    build_vendor=$host_vendor
    build_os=$host_os
  fi
else
  build=$build_alias
fi
