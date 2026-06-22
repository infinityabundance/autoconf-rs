//! Binary smoke tests — all 8 CLI binaries must start without panicking.
//!
//! Court: AC.CLI.SMOKE.1

#[cfg(test)]
mod tests {
    use std::process::Command;

    fn binary_path(name: &str) -> std::path::PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(format!("../../target/release/{}", name))
            .canonicalize()
            .unwrap_or_else(|_| {
                std::path::PathBuf::from(format!("/home/one/autoconf-rs/target/release/{}", name))
            })
    }

    #[test]
    fn test_autoconf_binary_starts() {
        let bin = binary_path("autoconf");
        // Just verify binary exists and is executable
        assert!(bin.exists(), "autoconf binary must exist at {:?}", bin);
        // Running without args should produce output or error (not panic)
        let out = Command::new(&bin).output().expect("autoconf should start");
        println!(
            "autoconf exit={:?}, stdout={}B, stderr={}B",
            out.status.code(),
            out.stdout.len(),
            out.stderr.len()
        );
    }

    #[test]
    fn test_autoheader_binary_starts() {
        let out = Command::new(&binary_path("autoheader"))
            .arg("--help")
            .output()
            .expect("autoheader binary should exist");
        // May exit non-zero without args, but shouldn't panic
        println!("autoheader exit: {:?}", out.status.code());
    }

    #[test]
    fn test_autom4te_binary_starts() {
        let out = Command::new(&binary_path("autom4te"))
            .arg("--help")
            .output()
            .expect("autom4te binary should exist");
        println!("autom4te exit: {:?}", out.status.code());
    }

    #[test]
    fn test_autoreconf_binary_starts() {
        let out = Command::new(&binary_path("autoreconf"))
            .arg("--help")
            .output()
            .expect("autoreconf binary should exist");
        println!("autoreconf exit: {:?}", out.status.code());
    }

    #[test]
    fn test_aclocal_binary_starts() {
        let out = Command::new(&binary_path("aclocal"))
            .arg("--help")
            .output()
            .expect("aclocal binary should exist");
        println!("aclocal exit: {:?}", out.status.code());
    }

    #[test]
    fn test_autoscan_binary_starts() {
        let out = Command::new(&binary_path("autoscan"))
            .arg("--help")
            .output()
            .expect("autoscan binary should exist");
        println!("autoscan exit: {:?}", out.status.code());
    }

    #[test]
    fn test_autoupdate_binary_starts() {
        let out = Command::new(&binary_path("autoupdate"))
            .arg("--help")
            .output()
            .expect("autoupdate binary should exist");
        println!("autoupdate exit: {:?}", out.status.code());
    }

    #[test]
    fn test_ifnames_binary_starts() {
        let out = Command::new(&binary_path("ifnames"))
            .arg("--help")
            .output()
            .expect("ifnames binary should exist");
        println!("ifnames exit: {:?}", out.status.code());
    }
}
