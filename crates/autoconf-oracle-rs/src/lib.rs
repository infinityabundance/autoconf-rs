// autoconf-oracle-rs: GNU Autoconf oracle admission
//
// Locates the system GNU Autoconf binaries (autoconf, autoheader, autom4te,
// autoreconf, aclocal, autoscan, autoupdate, ifnames), captures their
// identity fingerprints, runs smoke tests, and emits an oracle profile
// that all subsequent parity courts reference.
//
// Clean-room design: we interrogate the Autoconf binaries as black-box oracles.
// No implementation code is consulted — only binary output is captured.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

/// An admitted oracle — a specific set of Autoconf binaries with known identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleProfile {
    /// Human label, e.g. "gnu_autoconf_2_72"
    pub kind: String,
    /// Path to the primary autoconf executable
    pub path: String,
    /// Raw output of `autoconf --version`
    pub version_output: String,
    /// SHA-256 of the autoconf executable binary
    pub sha256: String,
    /// Platform triple (e.g. "x86_64-unknown-linux-gnu")
    pub platform: String,
    /// Locale used for admission (e.g. "C")
    pub locale: String,
    /// Shell used for pipe tests (e.g. "/bin/sh")
    pub shell: String,
    /// OS release info
    pub os_release: String,
    /// Feature flags detected
    pub features: OracleFeatures,
    /// Timestamp of admission
    pub admitted_at: String,
    /// Registry of receipts admitted against this oracle
    pub receipt_registry: Vec<String>,
    /// Profile of the subordinate GNU m4 oracle
    pub m4_oracle: Option<M4OracleProfile>,
    /// Paths to all 8 Autoconf binaries
    pub binaries: HashMap<String, BinaryProfile>,
}

/// Profile of a single Autoconf binary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryProfile {
    pub name: String,
    pub path: String,
    pub sha256: String,
    pub version_output: String,
    pub smoke_passed: bool,
}

/// Profile of the subordinate GNU m4 oracle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M4OracleProfile {
    pub path: String,
    pub sha256: String,
    pub version_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OracleFeatures {
    pub supports_autom4te: bool,
    pub supports_autoheader: bool,
    pub supports_autoreconf: bool,
    pub supports_aclocal: bool,
    pub supports_autoscan: bool,
    pub supports_autoupdate: bool,
    pub supports_ifnames: bool,
    pub warning_categories: Vec<String>,
    pub languages: Vec<String>,
}

/// Result of running an oracle command.
#[derive(Debug, Clone)]
pub struct OracleRun {
    pub exit_status: ExitStatus,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Configuration for oracle admission.
#[derive(Debug, Clone)]
pub struct OracleConfig {
    /// Path to autoconf binary; if None, search PATH
    pub autoconf_path: Option<PathBuf>,
    /// Path to m4 binary; if None, search PATH
    pub m4_path: Option<PathBuf>,
    /// Locale to use (default "C")
    pub locale: String,
    /// Shell to use for pipe tests (default "/bin/sh")
    pub shell: String,
    /// Additional environment variables
    pub env: HashMap<String, String>,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            autoconf_path: None,
            m4_path: None,
            locale: "C".to_string(),
            shell: "/bin/sh".to_string(),
            env: HashMap::new(),
        }
    }
}

/// The 8 standard Autoconf binaries to locate.
pub const AUTOCONF_BINARIES: &[&str] = &[
    "autoconf",
    "autoheader",
    "autom4te",
    "autoreconf",
    "aclocal",
    "autoscan",
    "autoupdate",
    "ifnames",
];

/// Locate a binary on the system.
pub fn locate_binary(name: &str, explicit_path: Option<&Path>) -> Result<PathBuf, OracleError> {
    if let Some(path) = explicit_path {
        if path.exists() {
            return Ok(path.to_path_buf());
        }
        return Err(OracleError::NotFound(format!(
            "explicit path for {} not found: {}",
            name,
            path.display()
        )));
    }

    // Search PATH
    if let Ok(path) = which::which(Path::new(name)) {
        return Ok(path);
    }

    // Check common locations
    for dir in &["/usr/bin", "/usr/local/bin"] {
        let candidate = Path::new(dir).join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(OracleError::NotFound(format!(
        "{} not found on PATH or common locations",
        name
    )))
}

/// Run a binary as oracle with given stdin and arguments.
pub fn run_oracle(
    binary_path: &Path,
    args: &[&str],
    stdin: &[u8],
    working_dir: Option<&Path>,
    env: &HashMap<String, String>,
) -> io::Result<OracleRun> {
    let mut cmd = Command::new(binary_path);
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.env_clear();

    // Set controlled environment
    cmd.env("PATH", "/usr/bin:/bin:/usr/local/bin");
    cmd.env("LC_ALL", "C");
    cmd.env("LANG", "C");
    for (k, v) in env {
        cmd.env(k, v);
    }

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let mut child = cmd.spawn()?;

    // Write stdin
    if let Some(mut sin) = child.stdin.take() {
        sin.write_all(stdin)?;
        // stdin is dropped here, closing the pipe
    }

    let output = child.wait_with_output()?;

    Ok(OracleRun {
        exit_status: output.status,
        exit_code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

/// Run the oracle with stdin from a string and capture output as strings.
pub fn run_oracle_text(
    binary_path: &Path,
    args: &[&str],
    stdin: &str,
    working_dir: Option<&Path>,
    env: &HashMap<String, String>,
) -> io::Result<OracleRun> {
    run_oracle(binary_path, args, stdin.as_bytes(), working_dir, env)
}

/// Admit the GNU Autoconf toolchain as the oracle.
///
/// This function:
/// 1. Locates all 8 Autoconf binaries
/// 2. Locates the subordinate GNU m4 binary
/// 3. Captures --version output from each
/// 4. Computes SHA-256 of each binary
/// 5. Detects features
/// 6. Runs smoke tests
/// 7. Builds the OracleProfile
pub fn admit_oracle(config: &OracleConfig) -> Result<OracleProfile, OracleError> {
    // 1. Locate primary autoconf binary
    let autoconf_path = locate_binary("autoconf", config.autoconf_path.as_deref())?;
    let abs_autoconf_path =
        std::fs::canonicalize(&autoconf_path).map_err(|e| OracleError::Io(e.to_string()))?;

    // 2. Capture autoconf version
    let version_run = run_oracle_text(&abs_autoconf_path, &["--version"], "", None, &config.env)
        .map_err(|e| OracleError::Execution(format!("autoconf version check: {}", e)))?;
    let version_output = String::from_utf8_lossy(&version_run.stdout).to_string();

    if !version_output.contains("GNU Autoconf") && !version_output.contains("autoconf") {
        return Err(OracleError::NotGnuAutoconf(format!(
            "binary at {} does not identify as GNU Autoconf:\n{}",
            abs_autoconf_path.display(),
            version_output
        )));
    }

    // 3. Compute sha256 of primary binary
    let binary_bytes =
        std::fs::read(&abs_autoconf_path).map_err(|e| OracleError::Io(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(&binary_bytes);
    let sha256 = format!("{:x}", hasher.finalize());

    // 4. Detect features
    let features = detect_features(&abs_autoconf_path, &config.env)?;

    // 5. Platform info
    let platform = std::env::consts::OS.to_string() + "-" + std::env::consts::ARCH;
    let os_release = read_os_release();

    // 6. Run autoconf smoke test: minimal configure.ac via stdin
    let smoke_input = "AC_INIT([hello], [1.0])\nAC_OUTPUT\n";
    let smoke_result = run_oracle_text(&abs_autoconf_path, &["-"], smoke_input, None, &config.env)
        .map_err(|e| OracleError::Execution(format!("autoconf smoke test: {}", e)))?;

    if smoke_result.exit_code != Some(0) {
        return Err(OracleError::SmokeFailure(format!(
            "autoconf smoke test failed with exit code {:?}: stderr={:?}",
            smoke_result.exit_code,
            String::from_utf8_lossy(&smoke_result.stderr)
        )));
    }

    // 7. Locate all other binaries
    let mut binaries = HashMap::new();
    for name in AUTOCONF_BINARIES {
        match locate_binary(name, None) {
            Ok(path) => {
                let abs_path = std::fs::canonicalize(&path).unwrap_or(path.clone());
                let bin_bytes = std::fs::read(&abs_path).unwrap_or_default();
                let mut h = Sha256::new();
                h.update(&bin_bytes);
                let bin_sha = format!("{:x}", h.finalize());

                let ver = run_oracle_text(&abs_path, &["--version"], "", None, &config.env);
                let ver_output = match ver {
                    Ok(r) => String::from_utf8_lossy(&r.stdout).to_string(),
                    Err(_) => String::new(),
                };

                // Run a minimal smoke test for each binary
                let smoke_ok = match *name {
                    "autoconf" => true, // already tested
                    "autoheader" => {
                        // autoheader needs a configure.ac with AC_CONFIG_HEADERS
                        let ac =
                            "AC_INIT([test], [1.0])\nAC_CONFIG_HEADERS([config.h])\nAC_OUTPUT\n";
                        let r = run_oracle_text(&abs_path, &["-"], ac, None, &config.env);
                        r.map(|r| r.exit_code == Some(0)).unwrap_or(false)
                    }
                    "autom4te" => {
                        let r = run_oracle_text(
                            &abs_path,
                            &["--language=Autoconf", "-"],
                            "AC_INIT\n",
                            None,
                            &config.env,
                        );
                        r.map(|r| r.exit_code == Some(0)).unwrap_or(false)
                    }
                    _ => {
                        let ver_run =
                            run_oracle_text(&abs_path, &["--version"], "", None, &config.env);
                        ver_run.map(|r| r.exit_code == Some(0)).unwrap_or(false)
                    }
                };

                binaries.insert(
                    name.to_string(),
                    BinaryProfile {
                        name: name.to_string(),
                        path: abs_path.to_string_lossy().to_string(),
                        sha256: bin_sha,
                        version_output: ver_output,
                        smoke_passed: smoke_ok,
                    },
                );
            }
            Err(_) => {
                // Binary not found — record as missing
                binaries.insert(
                    name.to_string(),
                    BinaryProfile {
                        name: name.to_string(),
                        path: String::new(),
                        sha256: String::new(),
                        version_output: String::new(),
                        smoke_passed: false,
                    },
                );
            }
        }
    }

    // 8. Locate subordinate GNU m4 oracle
    let m4 = locate_binary("m4", config.m4_path.as_deref()).ok();
    let m4_oracle = m4.and_then(|p| {
        let abs = std::fs::canonicalize(&p).ok()?;
        let bytes = std::fs::read(&abs).ok()?;
        let mut h = Sha256::new();
        h.update(&bytes);
        let m4ver = run_oracle_text(&abs, &["--version"], "", None, &config.env).ok()?;
        Some(M4OracleProfile {
            path: abs.to_string_lossy().to_string(),
            sha256: format!("{:x}", h.finalize()),
            version_output: String::from_utf8_lossy(&m4ver.stdout).to_string(),
        })
    });

    // 9. Build profile
    let profile = OracleProfile {
        kind: extract_profile_kind(&version_output),
        path: abs_autoconf_path.to_string_lossy().to_string(),
        version_output,
        sha256,
        platform,
        locale: config.locale.clone(),
        shell: config.shell.clone(),
        os_release,
        features,
        admitted_at: chrono_now(),
        receipt_registry: vec!["AC.ORACLE.1".to_string()],
        m4_oracle,
        binaries,
    };

    Ok(profile)
}

/// Extract the profile kind from version output.
fn extract_profile_kind(version_output: &str) -> String {
    for line in version_output.lines() {
        // "autoconf (GNU Autoconf) 2.72"
        for part in line.split_whitespace() {
            let cleaned = part.trim_end_matches(',');
            if cleaned
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return format!("gnu_autoconf_{}", cleaned.replace('.', "_"));
            }
        }
    }
    "gnu_autoconf_unknown".to_string()
}

/// Detect oracle features through black-box interrogation.
fn detect_features(
    path: &Path,
    env: &HashMap<String, String>,
) -> Result<OracleFeatures, OracleError> {
    let mut features = OracleFeatures::default();

    // Check --help for supported flags
    let help_run = run_oracle_text(path, &["--help"], "", None, env)
        .map_err(|e| OracleError::Execution(format!("help check: {}", e)))?;
    let help = String::from_utf8_lossy(&help_run.stdout);

    // Warning categories
    for cat in &[
        "cross",
        "gnu",
        "obsolete",
        "override",
        "portability",
        "syntax",
        "unsupported",
        "all",
        "error",
    ] {
        if help.contains(&format!("-W{}", cat)) || help.contains(&format!("--warnings={}", cat)) {
            features.warning_categories.push(cat.to_string());
        }
    }

    // Languages (from autom4te --help)
    let autom4te_path = locate_binary("autom4te", None);
    if let Ok(am4_path) = autom4te_path {
        if let Ok(am4_help) = run_oracle_text(&am4_path, &["--help"], "", None, env) {
            let am4h = String::from_utf8_lossy(&am4_help.stdout);
            for lang in &["Autoconf", "Autotest", "M4sh", "M4sugar"] {
                if am4h.contains(lang) {
                    features.languages.push(lang.to_string());
                }
            }
        }
    }

    // Feature detection
    features.supports_autom4te = locate_binary("autom4te", None).is_ok();
    features.supports_autoheader = locate_binary("autoheader", None).is_ok();
    features.supports_autoreconf = locate_binary("autoreconf", None).is_ok();
    features.supports_aclocal = locate_binary("aclocal", None).is_ok();
    features.supports_autoscan = locate_binary("autoscan", None).is_ok();
    features.supports_autoupdate = locate_binary("autoupdate", None).is_ok();
    features.supports_ifnames = locate_binary("ifnames", None).is_ok();

    Ok(features)
}

fn read_os_release() -> String {
    if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Get a UTC timestamp string.
fn chrono_now() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

/// Save the oracle profile to a JSON file.
pub fn save_profile(profile: &OracleProfile, path: &Path) -> io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(profile)?;
    std::fs::write(path, json)
}

/// Load an oracle profile from a JSON file.
pub fn load_profile(path: &Path) -> io::Result<OracleProfile> {
    let json = std::fs::read_to_string(path)?;
    let profile: OracleProfile = serde_json::from_str(&json)?;
    Ok(profile)
}

// ---------------------------------------------------------------------------
// Cross-Version Matrix — AC.ORACLE.1 Feature 6
//
// Tracks multiple admitted GNU Autoconf versions so we can compare behaviour
// across releases and detect regressions.
// ---------------------------------------------------------------------------

/// A collection of oracle profiles keyed by version string (e.g. "2.73").
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrossVersionMatrix {
    /// Primary (development) oracle version.
    pub primary_version: String,
    /// All admitted profiles, keyed by "major.minor" version.
    pub profiles: HashMap<String, OracleProfile>,
    /// Version pairs that have been compared.
    pub comparisons: Vec<VersionComparison>,
    /// When this matrix was last updated.
    pub updated_at: String,
}

/// Result of comparing output between two oracle versions for the same input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComparison {
    /// Source version (e.g. "2.73").
    pub version_a: String,
    /// Target version (e.g. "2.72").
    pub version_b: String,
    /// The configure.ac input used.
    pub fixture: String,
    /// Whether stdout matched byte-for-byte.
    pub byte_exact: bool,
    /// Whether exit codes matched.
    pub exit_match: bool,
    /// Size of version_a output in bytes.
    pub size_a: usize,
    /// Size of version_b output in bytes.
    pub size_b: usize,
}

impl CrossVersionMatrix {
    /// Create a new matrix with a primary version.
    pub fn new(primary_version: &str) -> Self {
        Self {
            primary_version: primary_version.to_string(),
            profiles: HashMap::new(),
            comparisons: Vec::new(),
            updated_at: chrono_now(),
        }
    }

    /// Admit a profile into the matrix.
    pub fn admit(&mut self, profile: OracleProfile) {
        let version = profile
            .kind
            .strip_prefix("gnu_autoconf_")
            .unwrap_or(&profile.kind)
            .replace('_', ".");
        self.profiles.insert(version, profile);
        self.updated_at = chrono_now();
    }

    /// Compare the same fixture across two admitted versions.
    pub fn compare_versions(
        &mut self,
        version_a: &str,
        version_b: &str,
        fixture: &str,
        fixture_content: &str,
        env: &HashMap<String, String>,
    ) -> Result<VersionComparison, OracleError> {
        let profile_a = self
            .profiles
            .get(version_a)
            .ok_or_else(|| OracleError::NotFound(format!("version {} not admitted", version_a)))?;
        let profile_b = self
            .profiles
            .get(version_b)
            .ok_or_else(|| OracleError::NotFound(format!("version {} not admitted", version_b)))?;

        let path_a = Path::new(&profile_a.path);
        let path_b = Path::new(&profile_b.path);

        let run_a = run_oracle_text(path_a, &["-"], fixture_content, None, env)
            .map_err(|e| OracleError::Execution(format!("version {} run: {}", version_a, e)))?;
        let run_b = run_oracle_text(path_b, &["-"], fixture_content, None, env)
            .map_err(|e| OracleError::Execution(format!("version {} run: {}", version_b, e)))?;

        let cmp = VersionComparison {
            version_a: version_a.to_string(),
            version_b: version_b.to_string(),
            fixture: fixture.to_string(),
            byte_exact: run_a.stdout == run_b.stdout,
            exit_match: run_a.exit_code == run_b.exit_code,
            size_a: run_a.stdout.len(),
            size_b: run_b.stdout.len(),
        };

        self.comparisons.push(cmp.clone());
        self.updated_at = chrono_now();
        Ok(cmp)
    }

    /// List all admitted version strings.
    pub fn admitted_versions(&self) -> Vec<&String> {
        let mut v: Vec<_> = self.profiles.keys().collect();
        v.sort();
        v
    }

    /// Check whether a given version is admitted.
    pub fn is_admitted(&self, version: &str) -> bool {
        self.profiles.contains_key(version)
    }
}

/// Try to admit an additional Autoconf version by checking common installation paths.
///
/// If the version is installed at a non-standard path, use `OracleConfig::autoconf_path`.
pub fn try_admit_version(
    version: &str,
    config: &OracleConfig,
) -> Result<OracleProfile, OracleError> {
    // Check versioned binary names: autoconf-2.72, autoconf2.71, etc.
    let candidates = vec![
        format!("autoconf-{}", version),
        format!("autoconf{}", version.replace('.', "")),
    ];

    let mut found = None;
    for name in &candidates {
        if let Ok(path) = locate_binary(name, None) {
            found = Some(path);
            break;
        }
    }

    // Also try looking in common prefix directories
    if found.is_none() {
        for prefix in &["/usr", "/usr/local", "/opt"] {
            let p = Path::new(prefix).join(format!("bin/autoconf-{}", version));
            if p.exists() {
                found = Some(p);
                break;
            }
        }
    }

    let path = found.ok_or_else(|| {
        OracleError::NotFound(format!(
            "autoconf version {} not found (tried autoconf-{}, autoconf{})",
            version,
            version,
            version.replace('.', "")
        ))
    })?;

    let mut vconfig = config.clone();
    vconfig.autoconf_path = Some(path);
    admit_oracle(&vconfig)
}

/// Errors that can occur during oracle operations.
#[derive(Debug)]
pub enum OracleError {
    NotFound(String),
    NotGnuAutoconf(String),
    Execution(String),
    SmokeFailure(String),
    Io(String),
}

impl std::fmt::Display for OracleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleError::NotFound(s) => write!(f, "oracle not found: {}", s),
            OracleError::NotGnuAutoconf(s) => write!(f, "not GNU Autoconf: {}", s),
            OracleError::Execution(s) => write!(f, "execution error: {}", s),
            OracleError::SmokeFailure(s) => write!(f, "smoke test failed: {}", s),
            OracleError::Io(s) => write!(f, "I/O error: {}", s),
        }
    }
}

impl std::error::Error for OracleError {}

// Minimal inline `which` implementation.
mod which {
    use std::path::{Path, PathBuf};

    pub fn which(cmd: &Path) -> Result<PathBuf, ()> {
        let path_var = std::env::var("PATH").unwrap_or_default();
        for dir in path_var.split(':') {
            let candidate = Path::new(dir).join(cmd);
            if candidate.exists() && is_executable(&candidate) {
                return Ok(candidate);
            }
        }
        Err(())
    }

    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let perms = meta.permissions();
            perms.mode() & 0o111 != 0
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_profile_kind() {
        assert_eq!(
            extract_profile_kind("autoconf (GNU Autoconf) 2.72\n"),
            "gnu_autoconf_2_72"
        );
    }

    #[test]
    fn test_locate_autoconf() {
        let config = OracleConfig::default();
        match locate_binary("autoconf", config.autoconf_path.as_deref()) {
            Ok(path) => {
                eprintln!("Found autoconf at: {}", path.display());
                assert!(path.exists());
            }
            Err(OracleError::NotFound(_)) => {
                eprintln!(
                    "autoconf not found — skipping locate test (CI/container may not have it)"
                );
            }
            Err(e) => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_admit_oracle() {
        let config = OracleConfig::default();
        match admit_oracle(&config) {
            Ok(profile) => {
                eprintln!("Admitted oracle: {:?}", profile.kind);
                eprintln!("  path: {}", profile.path);
                eprintln!("  binaries found: {}", profile.binaries.len());
                assert!(
                    profile.version_output.contains("GNU")
                        || profile.version_output.contains("autoconf")
                );
                // Save smoke test
                let _ = save_profile(&profile, Path::new("/tmp/autoconf-oracle-smoke.json"));
            }
            Err(OracleError::NotFound(_)) => {
                eprintln!("autoconf not found — skipping admission test");
            }
            Err(e) => panic!("admission failed: {}", e),
        }
    }

    // === Cross-Version Matrix tests (AC.ORACLE.1 Feature 6) ===

    #[test]
    fn test_cross_version_matrix_new() {
        let m = CrossVersionMatrix::new("2.73");
        assert_eq!(m.primary_version, "2.73");
        assert!(m.profiles.is_empty());
        assert!(m.comparisons.is_empty());
        assert!(!m.updated_at.is_empty());
    }

    #[test]
    fn test_cross_version_matrix_admit() {
        let config = OracleConfig::default();
        match admit_oracle(&config) {
            Ok(profile) => {
                let _kind = profile.kind.clone();
                let mut m = CrossVersionMatrix::new("2.73");
                m.admit(profile);
                // After admitting, should find the version
                assert!(!m.profiles.is_empty());
                assert!(m.admitted_versions().len() >= 1);
                eprintln!(
                    "CrossVersionMatrix: {} versions admitted: {:?}",
                    m.admitted_versions().len(),
                    m.admitted_versions()
                );
            }
            Err(OracleError::NotFound(_)) => {
                eprintln!("autoconf not found — skipping matrix admit test");
            }
            Err(e) => panic!("admission failed: {}", e),
        }
    }

    #[test]
    fn test_try_admit_version_current() {
        // Try to admit whatever version is on the system (should succeed for installed version)
        let config = OracleConfig::default();
        // The system autoconf is at /usr/bin/autoconf — try_admit_version looks for
        // versioned names like autoconf-2.73. On many systems only the unversioned
        // binary exists, so this may fail gracefully.
        match try_admit_version("2.73", &config) {
            Ok(profile) => {
                eprintln!("Version 2.73 admitted: {}", profile.kind);
                assert!(profile.kind.contains("2_73"));
            }
            Err(OracleError::NotFound(msg)) => {
                eprintln!(
                    "autoconf-2.73 not found (expected on single-version systems): {}",
                    msg
                );
                // This is expected — not a failure
            }
            Err(e) => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_cross_version_matrix_serialization() {
        let m = CrossVersionMatrix::new("2.73");
        // Should round-trip through JSON
        let json = serde_json::to_string(&m).unwrap();
        let m2: CrossVersionMatrix = serde_json::from_str(&json).unwrap();
        assert_eq!(m.primary_version, m2.primary_version);
        assert_eq!(m.profiles.len(), m2.profiles.len());
    }
}
