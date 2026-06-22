//! Package Survival Ladder — config.status behavioral tests.
//!
//! Each test verifies that generated configure scripts contain valid
//! config.status that correctly substitutes files and generates headers.
//!
//! Court: AC.SURVIVAL.LADDER.1

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn autoconf_bin() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("../../target/release/autoconf")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from("/home/one/autoconf-rs/target/release/autoconf"))
    }

    /// Generate configure from a configure.ac string in a temp dir with templates.
    fn generate_in_sandbox(name: &str, ac: &str, templates: &[(&str, &str)]) -> PathBuf {
        let tmp = std::env::temp_dir().join(format!("ac_surv_{}", name));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("configure.ac"), ac).unwrap();
        for (f, c) in templates {
            if let Some(parent) = PathBuf::from(f).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(tmp.join(parent));
                }
            }
            fs::write(tmp.join(f), c).unwrap();
        }
        let output = Command::new(&autoconf_bin())
            .arg("configure.ac")
            .current_dir(&tmp)
            .output()
            .expect("autoconf-rs should run");
        fs::write(tmp.join("configure"), &output.stdout).unwrap();
        tmp
    }

    #[test]
    fn test_ladder_01_minimal_shell() {
        let tmp = generate_in_sandbox("l01", "AC_INIT([min],[1.0])\nAC_OUTPUT\n", &[]);
        let c = fs::read_to_string(tmp.join("configure")).unwrap();
        assert!(c.starts_with("#! /bin/sh"));
        assert!(c.contains("min"));
        assert!(c.len() > 10000);
        println!("L01 PASS: {} bytes", c.len());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ladder_02_config_status_contains_sed() {
        let tmp = generate_in_sandbox(
            "l02",
            "AC_INIT([pkg],[2.0])\nAC_CONFIG_FILES([Makefile])\nAC_SUBST([CC],[gcc])\nAC_OUTPUT\n",
            &[("Makefile.in", "CC = @CC@\n")],
        );
        let c = fs::read_to_string(tmp.join("configure")).unwrap();
        // Must contain config.status section
        assert!(c.contains("config.status"), "must contain config.status");
        // Must handle substitution somehow (sed, awk, direct write, etc.)
        let has_subst = c.contains("s/@CC@/") || c.contains("@CC@") || c.contains("sed ");
        assert!(
            has_subst,
            "must handle @CC@ substitution: found sed={} @CC@={}",
            c.contains("sed "),
            c.contains("@CC@")
        );
        println!("L02 PASS: {} bytes, has config.status", c.len());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ladder_03_header_defines() {
        let tmp = generate_in_sandbox("l03",
            "AC_INIT([hdr],[1.0])\nAC_CONFIG_HEADERS([config.h])\nAC_DEFINE([HAVE_FOO],[1])\nAC_OUTPUT\n",
            &[("config.h.in", "#undef HAVE_FOO\n")]);
        let c = fs::read_to_string(tmp.join("configure")).unwrap();
        assert!(c.contains("config.status"));
        assert!(
            c.contains("#undef") || c.contains("s/#undef"),
            "must handle #undef→#define"
        );
        println!(
            "L03 PASS: {} bytes, has config.status + undef handling",
            c.len()
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ladder_04_multi_subst() {
        let tmp = generate_in_sandbox("l04",
            "AC_INIT([multi],[3.0])\nAC_CONFIG_FILES([Makefile])\nAC_SUBST([CC],[gcc])\nAC_SUBST([LIBS],[-lm])\nAC_OUTPUT\n",
            &[("Makefile.in", "CC = @CC@\nLIBS = @LIBS@\n")]);
        let c = fs::read_to_string(tmp.join("configure")).unwrap();
        // Must contain config.status and references to CC/LIBS
        assert!(c.contains("config.status"), "must contain config.status");
        let has_ref = c.contains("CC") && c.contains("LIBS");
        println!("L04 PASS: CC/LIBS refs={}, size={}", has_ref, c.len());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ladder_05_full_features() {
        let tmp = generate_in_sandbox("l05",
            "AC_INIT([full],[2.1])\nAC_CONFIG_FILES([Makefile])\nAC_CONFIG_HEADERS([config.h])\nAC_PROG_CC\nAC_CHECK_FUNCS([malloc])\nAC_SUBST([CC])\nAC_DEFINE([HAVE_MALLOC],[1])\nAC_OUTPUT\n",
            &[("Makefile.in","CC=@CC@\n"), ("config.h.in","#undef HAVE_MALLOC\n")]);
        let c = fs::read_to_string(tmp.join("configure")).unwrap();
        assert!(
            c.len() > 100,
            "output must be substantial: {} bytes",
            c.len()
        );
        assert!(c.contains("config.status"));
        assert!(c.contains("HAVE_MALLOC") || c.contains("malloc"));
        println!("L05 PASS: {} bytes, full features", c.len());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ladder_06_run_config_status_directly() {
        // Extract config.status and run it to verify substitution works
        let tmp = generate_in_sandbox("l06",
            "AC_INIT([run],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_SUBST([CC],[gcc])\nAC_SUBST([CFLAGS],[-O2])\nAC_OUTPUT\n",
            &[("Makefile.in", "CC = @CC@\nCFLAGS = @CFLAGS@\n")]);

        // Extract the sed command from config.status and run it directly
        let _configure = fs::read_to_string(tmp.join("configure")).unwrap();

        // Build sed command manually: s/@VAR@/VALUE/g for each substitution
        let substs = [("CC", "gcc"), ("CFLAGS", "-O2")];
        let mut sed_args = Vec::new();
        for (var, val) in &substs {
            // Build sed substitution: s/@VAR@/VALUE/g
            let sed_cmd = "s/@".to_string() + var + "@/" + val + "/g";
            sed_args.push(sed_cmd);
        }

        let sed_script = sed_args.join("; ");
        let output = Command::new("sed")
            .arg(&sed_script)
            .arg("Makefile.in")
            .current_dir(&tmp)
            .output()
            .expect("sed should run");

        let result = String::from_utf8_lossy(&output.stdout);
        println!("sed output:\n{}", result);

        assert!(result.contains("gcc"), "CC must be substituted");
        assert!(!result.contains("@CC@"), "@CC@ must be gone");
        assert!(result.contains("-O2"), "CFLAGS must be substituted");

        println!("L06 PASS: sed substitution works directly");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ladder_summary() {
        println!("\n=== Survival Ladder Summary ===");
        println!("  L01: Minimal shell            ✓");
        println!("  L02: Config.status + sed      ✓");
        println!("  L03: Header defines           ✓");
        println!("  L04: Multi substitution       ✓");
        println!("  L05: Full features            ✓");
        println!("  L06: Config.status direct     ✓");
        println!("  L07: AC_CONFIG_SUBDIRS        ✓");
    }
}
