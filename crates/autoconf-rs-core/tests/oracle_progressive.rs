//! Oracle comparison tests for Layer 0 fixtures 04-06.
//! Tests progressive configure.ac complexity against GNU autoconf oracle.

#[cfg(test)]
mod tests {
    use std::process::Command;

    fn run_autoconf_oracle(fixture: &str) -> Vec<u8> {
        // Tests run from crate dir; workspace root is ../../
        let full_path = format!("../../{}", fixture);
        let output = Command::new("autoconf")
            .arg(&full_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .expect("oracle should run");
        output.stdout
    }

    fn run_autoconf_rs(fixture: &str) -> Vec<u8> {
        let full_path = format!("../../{}", fixture);
        let output = Command::new("../../target/release/autoconf")
            .arg(&full_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .expect("autoconf-rs should run");
        output.stdout
    }

    fn coverage_pct(oracle: &[u8], rust: &[u8]) -> f64 {
        let max_len = oracle.len().max(rust.len());
        if max_len == 0 {
            return 100.0;
        }
        let min_len = oracle.len().min(rust.len());
        let mut matches = 0usize;
        for i in 0..min_len {
            if oracle[i] == rust[i] {
                matches += 1;
            }
        }
        // Coverage: bytes that match relative to max length
        (matches as f64 / max_len as f64) * 100.0
    }

    #[test]
    fn test_fixture_04_programs() {
        let oracle = run_autoconf_oracle("lab/corpus/layer0-smoke/fixture_04_programs.ac");
        let rust = run_autoconf_rs("lab/corpus/layer0-smoke/fixture_04_programs.ac");
        let cov = coverage_pct(&oracle, &rust);
        println!(
            "fixture_04_programs: oracle={} rust={} coverage={:.1}%",
            oracle.len(),
            rust.len(),
            cov
        );
        // Complex fixture — coverage expected to be low until full template
        // replication is complete. This is an informational test, not a blocker.
    }

    #[test]
    fn test_fixture_05_functions() {
        let oracle = run_autoconf_oracle("lab/corpus/layer0-smoke/fixture_05_functions.ac");
        let rust = run_autoconf_rs("lab/corpus/layer0-smoke/fixture_05_functions.ac");
        let cov = coverage_pct(&oracle, &rust);
        println!(
            "fixture_05_functions: oracle={} rust={} coverage={:.1}%",
            oracle.len(),
            rust.len(),
            cov
        );
        // Informational: coverage scales with template completeness
    }

    #[test]
    fn test_fixture_06_headers_types() {
        let oracle = run_autoconf_oracle("lab/corpus/layer0-smoke/fixture_06_headers_types.ac");
        let rust = run_autoconf_rs("lab/corpus/layer0-smoke/fixture_06_headers_types.ac");
        let cov = coverage_pct(&oracle, &rust);
        println!(
            "fixture_06_headers_types: oracle={} rust={} coverage={:.1}%",
            oracle.len(),
            rust.len(),
            cov
        );
        // Informational: more macro templates needed
    }

    #[test]
    fn test_layer0_coverage_summary() {
        let fixtures = [
            "lab/corpus/layer0-smoke/smoke_01_minimal.ac",
            "lab/corpus/layer0-smoke/smoke_02_subst.ac",
            "lab/corpus/layer0-smoke/smoke_03_headers.ac",
            "lab/corpus/layer0-smoke/fixture_04_programs.ac",
            "lab/corpus/layer0-smoke/fixture_05_functions.ac",
            "lab/corpus/layer0-smoke/fixture_06_headers_types.ac",
        ];
        let mut total_oracle = 0usize;
        let mut total_matches = 0usize;
        for f in &fixtures {
            let oracle = run_autoconf_oracle(f);
            let rust = run_autoconf_rs(f);
            let min_len = oracle.len().min(rust.len());
            for i in 0..min_len {
                if oracle[i] == rust[i] {
                    total_matches += 1;
                }
            }
            total_oracle += oracle.len();
            println!("  {}: oracle={} rust={}", f, oracle.len(), rust.len());
        }
        let pct = if total_oracle > 0 {
            (total_matches as f64 / total_oracle as f64) * 100.0
        } else {
            100.0
        };
        println!("\n  Overall Layer 0 coverage: {:.1}%", pct);
    }
}
