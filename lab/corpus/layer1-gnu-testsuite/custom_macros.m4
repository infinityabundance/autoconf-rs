dnl custom_macros.m4 — User-defined M4 macros loaded via m4_include
dnl CROSS.002: Runtime .m4 file loading support
dnl These macros would normally be defined in a project's m4/ directory

dnl Define project-specific substitution
AC_SUBST([PROJECT_VERSION_MAJOR], [1])
AC_SUBST([PROJECT_VERSION_MINOR], [0])
AC_SUBST([PROJECT_VERSION_PATCH], [0])

dnl Define project-specific checks
AC_DEFINE([ENABLE_DEBUG], [0], [Enable debug mode])
AC_DEFINE([ENABLE_LOGGING], [1], [Enable logging])
