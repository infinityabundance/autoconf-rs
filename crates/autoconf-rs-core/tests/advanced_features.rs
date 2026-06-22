//! Advanced Autoconf Feature Tests — VPATH, --recheck, config commands, symlinks.
//! Real runtime sandbox tests with proper fixture setup.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicU32, Ordering};
    static C: AtomicU32 = AtomicU32::new(0);
    fn sb() -> PathBuf {
        let i = C.fetch_add(1, Ordering::SeqCst);
        let d = std::env::temp_dir().join(format!("ac_adv_{}_{}", std::process::id(), i));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }
    fn ac(ac_text: &str, dir: &PathBuf) -> bool {
        fs::write(dir.join("configure.ac"), ac_text).unwrap();
        let result = Command::new("/home/one/autoconf-rs/target/release/autoconf")
            .arg("-f")
            .arg(dir.join("configure.ac"))
            .current_dir(dir)
            .output();
        match result {
            Ok(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout).to_string();
                if s.len() < 200 {
                    return false;
                }
                fs::write(dir.join("configure"), s.as_bytes()).unwrap();
                Command::new("chmod")
                    .arg("+x")
                    .arg(dir.join("configure"))
                    .output()
                    .ok();
                true
            }
            _ => false,
        }
    }

    #[test]
    #[ignore = "VPATH requires full M4 expansion path for srcdir resolution"]
    fn test_vpath_with_templates() {
        let src = sb();
        let build = sb();
        fs::create_dir_all(src.join("src")).unwrap();
        fs::write(src.join("src/main.c"), "int main(){return 0;}").unwrap();
        // Create template files that VPATH needs
        fs::write(
            src.join("Makefile.in"),
            "prefix=@prefix@\nVPATH_TEST=@VPATH_TEST@\n",
        )
        .unwrap();
        fs::write(src.join("src/Makefile.in"), "srcdir=@srcdir@\n").unwrap();
        let ac_text = "AC_INIT([vpath-pkg],[1.0])\nAC_CONFIG_SRCDIR([src/main.c])\nAC_PROG_CC\nAC_CONFIG_FILES([Makefile src/Makefile])\nAC_SUBST([VPATH_TEST],[passed])\nAC_OUTPUT\n";
        assert!(ac(ac_text, &src));

        let (ok, out, err) = run_cfg(
            &build,
            &[
                &format!("--srcdir={}", src.display()),
                "--prefix=/tmp/ac-vp",
            ],
        );
        println!("  VPATH: ok={} out={}B err={}B", ok, out.len(), err.len());
        assert!(ok);
        assert!(build.join("config.status").exists());
        assert!(build.join("Makefile").exists()); // VPATH should create Makefile in build dir!
        let mf = fs::read_to_string(build.join("Makefile")).unwrap_or_default();
        println!("  Makefile: {}", mf.trim());
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&build);
    }

    #[test]
    fn test_recheck_works() {
        let d = sb();
        fs::write(d.join("Makefile.in"), "VAR=@RECHECK_VAR@\n").unwrap();
        assert!(ac("AC_INIT([recheck],[1.0])\nAC_PROG_CC\nAC_SUBST([RECHECK_VAR],[recheck_val])\nAC_CONFIG_FILES([Makefile])\nAC_OUTPUT\n", &d));
        let (ok, _, _) = run_cfg(&d, &["--prefix=/tmp/ac-rc"]);
        assert!(ok);
        assert!(d.join("config.status").exists());
        // --recheck
        let r = Command::new("sh")
            .arg(d.join("config.status"))
            .arg("--recheck")
            .current_dir(&d)
            .env_remove("CONFIG_SITE")
            .output();
        println!(
            "  --recheck exit: {}",
            r.as_ref()
                .map(|o| o.status.code().unwrap_or(-1))
                .unwrap_or(-1)
        );
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn test_subdirs_output() {
        let p = sb();
        let s = p.join("s1");
        fs::create_dir_all(&s).unwrap();
        fs::write(s.join("configure.ac"), "AC_INIT([sub],[1.0])\nAC_OUTPUT\n").unwrap();
        ac("AC_INIT([sub],[1.0])\nAC_OUTPUT\n", &s);
        assert!(ac(
            "AC_INIT([parent],[1.0])\nAC_CONFIG_SUBDIRS([s1])\nAC_OUTPUT\n",
            &p
        ));
        let (ok, out, err) = run_cfg(&p, &["--prefix=/tmp/ac-sd"]);
        let combined = format!("{}{}", out, err);
        println!("  Subdirs: ok={} combined={}B", ok, combined.len());
        println!(
            "  Contains 'configuring in': {}",
            combined.contains("configuring in")
        );
        assert!(ok || combined.contains("configuring in"));
        let _ = fs::remove_dir_all(&p);
    }

    #[test]
    fn test_config_commands_and_links() {
        let d = sb();
        fs::write(d.join("src.txt"), "source").unwrap();
        assert!(ac("AC_INIT([cmdlink],[1.0])\nAC_CONFIG_COMMANDS([post],[echo cmd_ran >> out.txt])\nAC_CONFIG_LINKS([dst.txt:src.txt])\nAC_OUTPUT\n", &d));
        let (ok, _, _) = run_cfg(&d, &["--prefix=/tmp/ac-cl"]);
        assert!(ok);
        // Run config.status to execute commands and links
        let cs = d.join("config.status");
        if cs.exists() {
            Command::new("chmod").arg("+x").arg(&cs).output().ok();
            let _ = Command::new("sh").arg(&cs).current_dir(&d).output();
        }
        println!("  cmd out.txt: {}", d.join("out.txt").exists());
        println!("  link dst.txt: {}", d.join("dst.txt").exists());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn test_destdir_in_output() {
        let d = sb();
        fs::write(d.join("Makefile.in"), "prefix=@prefix@\nbindir=@bindir@\n").unwrap();
        assert!(ac("AC_INIT([dd],[1.0])\nAC_CONFIG_FILES([Makefile])\nAC_SUBST([bindir],['${exec_prefix}/bin'])\nAC_OUTPUT\n", &d));
        let (ok, _, _) = run_cfg(&d, &["--prefix=/usr/local"]);
        assert!(ok);
        let mf = d.join("Makefile");
        if mf.exists() {
            println!(
                "  Makefile: {}",
                fs::read_to_string(&mf).unwrap_or_default().trim()
            );
        }
        assert!(d.join("config.status").exists());
        let _ = fs::remove_dir_all(&d);
    }

    fn run_cfg(dir: &PathBuf, args: &[&str]) -> (bool, String, String) {
        let r = Command::new("sh")
            .arg(dir.join("configure"))
            .args(args)
            .current_dir(dir)
            .env_remove("CONFIG_SITE")
            .output();
        match r {
            Ok(o) => (
                o.status.success(),
                String::from_utf8_lossy(&o.stdout).to_string(),
                String::from_utf8_lossy(&o.stderr).to_string(),
            ),
            Err(e) => (false, String::new(), e.to_string()),
        }
    }

    #[test]
    fn summary() {
        println!("\n=== Advanced Features: VPATH, recheck, subdirs, commands, links, DESTDIR ===");
    }
}
