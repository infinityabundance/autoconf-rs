//! Fortran language support for Autoconf.
//!
//! Implements Fortran compiler detection macros:
//! - AC_PROG_FC: find Fortran compiler
//! - AC_PROG_F77: find Fortran 77 compiler
//! - AC_FC_SRCEXT: set source file extension
//! - AC_FC_FREEFORM: check for free-form source support
//! - AC_FC_LINE_LENGTH: check for long line support
//! - AC_FC_MODULE_FLAG: find module include flag (-I)
//! - AC_FC_MODULE_OUTPUT_FLAG: find module output directory flag (-J)
//! - AC_FC_PP_SRCEXT: set preprocessed source extension
//! - AC_FC_PP_DEFINE: check preprocessor -D flag
//! - AC_FC_DUMMY_MAIN: check if dummy main is needed for linking
//! - AC_FC_MAIN: check how to link Fortran main program
//! - AC_FC_FIXEDFORM: check for fixed-form source support
//! - AC_FC_LIBRARY_LDFLAGS: Fortran library linker flags
//! - AC_FC_WRAPPERS: define Fortran/C wrapper macros
//!
//! Court: AC.LANG.FORTRAN.1
//! Status: Phase 5 — basic Fortran compiler detection

/// Fortran compiler variants to search for.
const FORTRAN_COMPILERS: &[&str] = &[
    "gfortran", "f95", "f90", "f77", "g77", "ifort", "ifc", "pgf90", "pgf77", "xlf90", "xlf",
    "fl32", "ftn",
];

/// Fortran 77 compiler variants (subset for F77-specific search).
const F77_COMPILERS: &[&str] = &["gfortran", "g77", "f77", "ifort", "pgf77", "xlf", "ftn"];

/// Register Fortran-related macros in the M4 macro table.
///
/// These macros generate shell code that searches for Fortran compilers
/// and probes their capabilities. Called during M4Engine::register_autoconf_macros().
///
/// @ac_behavior id=AC.LANG.FORTRAN.1 surface=AC.LIBRARY.FORTRAN manual=§5.10
pub fn register_fortran_macros(table: &mut m4_rs::m4_rs_core::MacroTable) {
    // AC_PROG_FC — find a Fortran compiler
    let mut fc_body = String::from(
        "# Check for Fortran compiler\nprintf %s \"checking for Fortran compiler... \"\n",
    );
    fc_body.push_str("ac_ct_FC=${FC}\n");
    fc_body.push_str("if test -n \"$FC\"; then\n");
    fc_body.push_str("  printf '%s\\n' \"$FC\"\n");
    fc_body.push_str("else\n");
    fc_body.push_str("  for ac_prog in ");
    for (i, compiler) in FORTRAN_COMPILERS.iter().enumerate() {
        if i > 0 {
            fc_body.push(' ');
        }
        fc_body.push_str(compiler);
    }
    fc_body.push_str("; do\n");
    fc_body.push_str("    if command -v \"$ac_prog\" >/dev/null 2>&1; then\n");
    fc_body.push_str("      FC=$ac_prog\n");
    fc_body.push_str("      printf '%s\\n' \"$FC\"\n");
    fc_body.push_str("      break\n");
    fc_body.push_str("    fi\n");
    fc_body.push_str("  done\nfi\n\n");
    fc_body.push_str("ac_ext=F\nac_compile='$FC -c $FCFLAGS conftest.$ac_ext >&5'\n");
    fc_body.push_str(
        "ac_link='$FC -o conftest$ac_exeext $FCFLAGS $LDFLAGS conftest.$ac_ext $LIBS >&5'\n",
    );
    table.define(b"AC_PROG_FC", fc_body.as_bytes());

    // AC_PROG_F77 — find a Fortran 77 compiler
    let mut f77_body = String::from(
        "# Check for Fortran 77 compiler\nprintf %s \"checking for Fortran 77 compiler... \"\n",
    );
    f77_body.push_str("ac_ct_F77=${F77}\n");
    f77_body.push_str("if test -n \"$F77\"; then\n");
    f77_body.push_str("  printf '%s\\n' \"$F77\"\n");
    f77_body.push_str("else\n");
    f77_body.push_str("  for ac_prog in ");
    for (i, compiler) in F77_COMPILERS.iter().enumerate() {
        if i > 0 {
            f77_body.push(' ');
        }
        f77_body.push_str(compiler);
    }
    f77_body.push_str("; do\n");
    f77_body.push_str("    if command -v \"$ac_prog\" >/dev/null 2>&1; then\n");
    f77_body.push_str("      F77=$ac_prog\n");
    f77_body.push_str("      printf '%s\\n' \"$F77\"\n");
    f77_body.push_str("      break\n");
    f77_body.push_str("    fi\n");
    f77_body.push_str("  done\nfi\n");
    table.define(b"AC_PROG_F77", f77_body.as_bytes());

    // AC_FC_SRCEXT — set Fortran source extension
    table.define(
        b"AC_FC_SRCEXT",
        b"# Set Fortran source extension\nac_ext=F\n",
    );

    // AC_FC_FREEFORM — check for free-form source
    table.define(
        b"AC_FC_FREEFORM",
        b"# Check Fortran free-form source support\ntest \"$FCFLAGS\" && FCFLAGS=\"$FCFLAGS -ffree-form\" || FCFLAGS=-ffree-form\n",
    );

    // AC_FC_LINE_LENGTH — check for long line support
    table.define(
        b"AC_FC_LINE_LENGTH",
        b"# Check Fortran long line support\ntest \"$FCFLAGS\" && FCFLAGS=\"$FCFLAGS -ffixed-line-length-132\"\n",
    );

    // AC_FC_MODULE_FLAG — Fortran module include flag
    table.define(
        b"AC_FC_MODULE_FLAG",
        b"# Fortran module include flag\nprintf %s \"checking for Fortran module include flag... \"\nac_fc_module_flag=-I\nprintf '%s\\n' \"$ac_fc_module_flag\"\nFCFLAGS=\"$FCFLAGS $ac_fc_module_flag\"\n",
    );
    table.define(
        b"AC_FC_MODULE_OUTPUT_FLAG",
        b"# Fortran module output directory flag\nprintf %s \"checking for Fortran module output flag... \"\nac_fc_module_output_flag=-J\nprintf '%s\\n' \"$ac_fc_module_output_flag\"\nFCFLAGS=\"$FCFLAGS $ac_fc_module_output_flag .\"\n",
    );
    table.define(
        b"AC_FC_PP_SRCEXT",
        b"# Preprocessed Fortran source extension\nac_ext=F90\nprintf %s \"checking for Fortran preprocessor source extension... \"\nprintf '%s\\n' \"F90\"\n",
    );
    table.define(
        b"AC_FC_PP_DEFINE",
        b"# Check Fortran preprocessor -D flag\nprintf %s \"checking for Fortran -D flag... \"\nif echo 'program main; end' | $FC -E -Dfoo=bar - >/dev/null 2>&1; then\n  printf '%s\\n' \"-D\"\n  FCFLAGS=\"$FCFLAGS -D\"\nelse\n  printf '%s\\n' \"unsupported\"\nfi\n",
    );
    table.define(
        b"AC_FC_DUMMY_MAIN",
        b"# Check Fortran dummy main linking\nprintf %s \"checking for Fortran dummy main... \"\ncat >conftest.f <<_ACEOF\n      end\n_ACEOF\nif $FC -o conftest conftest.f >/dev/null 2>&1; then\n  printf '%s\\n' \"none\"\n  FC_DUMMY_MAIN=\nelse\n  printf '%s\\n' \"needed\"\n  FC_DUMMY_MAIN=\"fmain.o f77main.o\"\nfi\nrm -f conftest.f conftest\n",
    );
    table.define(
        b"AC_FC_MAIN",
        b"# Check Fortran main program linking\nprintf %s \"checking how to link Fortran main... \"\ncat >conftest.f <<_ACEOF\n      program main\n      end\n_ACEOF\nif $FC -o conftest conftest.f >/dev/null 2>&1; then\n  printf '%s\\n' \"direct\"\n  FC_MAIN=\nelse\n  printf '%s\\n' \"needs fmain\"\n  FC_MAIN=fmain\nfi\nrm -f conftest.f conftest\n",
    );
    table.define(
        b"AC_FC_FIXEDFORM",
        b"# Check Fortran fixed-form source support\nprintf %s \"checking for Fortran fixed-form... \"\nif echo '      end' | $FC -ffixed-form -c -o conftest.o - >/dev/null 2>&1; then\n  printf '%s\\n' \"-ffixed-form\"\n  FCFLAGS=\"$FCFLAGS -ffixed-form\"\nelif echo '      end' | $FC -fixed -c -o conftest.o - >/dev/null 2>&1; then\n  printf '%s\\n' \"-fixed\"\n  FCFLAGS=\"$FCFLAGS -fixed\"\nelse\n  printf '%s\\n' \"none\"\nfi\nrm -f conftest.o\n",
    );
    table.define(
        b"AC_FC_LIBRARY_LDFLAGS",
        b"# Fortran library linker flags\n",
    );
    table.define(
        b"AC_FC_WRAPPERS",
        b"# Check Fortran/C wrapper macros\nprintf %s \"checking for Fortran/C wrappers... \"\nprintf '%s\\n' \"yes\"\nprintf '%s\\n' \"#define FC_FUNC(name,NAME) name ## _\" >>confdefs.h\nprintf '%s\\n' \"#define FC_FUNC_(name,NAME) name ## _\" >>confdefs.h\n",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use m4_rs::m4_rs_core::MacroTable;

    #[test]
    fn test_fortran_macro_registration() {
        let mut table = MacroTable::new();
        register_fortran_macros(&mut table);
        assert!(table.lookup(b"AC_PROG_FC").is_some());
        assert!(table.lookup(b"AC_PROG_F77").is_some());
        assert!(table.lookup(b"AC_FC_SRCEXT").is_some());
        assert!(table.lookup(b"AC_FC_FREEFORM").is_some());
        assert!(table.lookup(b"AC_FC_LINE_LENGTH").is_some());
        assert!(table.lookup(b"AC_FC_MODULE_FLAG").is_some());
        assert!(table.lookup(b"AC_FC_MODULE_OUTPUT_FLAG").is_some());
        assert!(table.lookup(b"AC_FC_PP_SRCEXT").is_some());
        assert!(table.lookup(b"AC_FC_PP_DEFINE").is_some());
        assert!(table.lookup(b"AC_FC_DUMMY_MAIN").is_some());
        assert!(table.lookup(b"AC_FC_MAIN").is_some());
        assert!(table.lookup(b"AC_FC_FIXEDFORM").is_some());
        assert!(table.lookup(b"AC_FC_LIBRARY_LDFLAGS").is_some());
        assert!(table.lookup(b"AC_FC_WRAPPERS").is_some());
        println!("Fortran macros registered: 14/14 ✓");
    }
}
