//! autom4te — Caching M4 wrapper for Autoconf.
//!
//! autom4te is the caching M4 wrapper that autoconf, autoheader, automake,
//! and autoscan all use internally. It caches M4 expansions to avoid
//! re-expansion on subsequent runs.
//!
//! Cache format divergence: autoconf-rs uses JSON-based cache entries with
//! SHA256 freshness checking, rather than GNU m4 frozen files. This is an
//! admitted divergence (NC.CACHE.FORMAT.1) — semantic caching equivalence,
//! not byte-identical frozen file format.
//!
//! Receipt family: AC.CLI.AUTOM4TE.*
//! Court: AC.AUTOM4TE.CACHE.1 — caching with --freeze/--reload
//! Status: Phase 5 — caching operational.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A single cache entry representing one autom4te invocation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    /// SHA256 of the primary input file
    pub input_sha256: String,
    /// SHA256 of all include files (M4 files) concatenated
    pub deps_sha256: String,
    /// The expanded M4 output (configure script text)
    pub output: String,
    /// Autom4te trace lines (for --trace replay)
    pub traces: Vec<String>,
    /// When this cache entry was created (UNIX timestamp)
    pub created_at: u64,
    /// The --language used for this expansion
    pub language: String,
    /// Engine version string (crate version) — cache invalidated on code changes
    #[serde(default)]
    pub engine_version: String,
}

/// Manages autom4te's cache directory (autom4te.cache/).
///
/// Stores cache entries keyed by (input_hash, language). On cache hit,
/// returns cached output without re-expanding M4. On cache miss or --force,
/// expands and stores a new entry.
///
/// Court: AC.AUTOM4TE.CACHE.1
#[derive(Debug, Clone)]
pub struct Autom4teCache {
    /// Path to cache directory (typically autom4te.cache/)
    cache_dir: PathBuf,
    /// Whether to bypass cache and always regenerate
    force: bool,
    /// Loaded cache entries
    entries: HashMap<String, CacheEntry>,
}

impl Autom4teCache {
    /// Create a new cache manager for the given directory.
    /// Creates the directory if it doesn't exist.
    pub fn new(cache_dir: &Path) -> Self {
        let _ = fs::create_dir_all(cache_dir);
        Self {
            cache_dir: cache_dir.to_path_buf(),
            force: false,
            entries: HashMap::new(),
        }
    }

    /// Set force mode — bypass cache and always regenerate.
    pub fn set_force(&mut self, force: bool) {
        self.force = force;
    }

    /// Compute a cache key from input path and language.
    fn cache_key(input_path: &Path, language: &str) -> String {
        format!(
            "{}_{}",
            input_path.display().to_string().replace('/', "_"),
            language
        )
    }

    /// Look up a cached entry. Returns Some(output) on cache hit, None on miss.
    ///
    /// Checks SHA256 of input and all include files against stored hashes.
    /// If any file has changed, the cache is invalidated.
    pub fn lookup(
        &mut self,
        input_path: &Path,
        include_dirs: &[PathBuf],
        language: &str,
    ) -> Option<String> {
        if self.force {
            return None;
        }

        let input_sha256 = Self::file_sha256(input_path);
        let deps_sha256 = Self::deps_sha256(input_path, include_dirs);

        // Try to load the cache entry from disk
        let key = Self::cache_key(input_path, language);
        let entry_path = self.cache_dir.join(format!("{}.json", key));

        if let Ok(data) = fs::read_to_string(&entry_path) {
            if let Ok(entry) = serde_json::from_str::<CacheEntry>(&data) {
                if entry.input_sha256 == input_sha256
                    && entry.deps_sha256 == deps_sha256
                    && entry.engine_version == crate_version()
                {
                    self.entries.insert(key, entry.clone());
                    eprintln!(
                        "autom4te: cache hit for {} ({} bytes output, {} traces)",
                        input_path.display(),
                        entry.output.len(),
                        entry.traces.len()
                    );
                    return Some(entry.output);
                }
                eprintln!(
                    "autom4te: cache stale for {} (input match={}, deps match={}, engine match={})",
                    input_path.display(),
                    entry.input_sha256 == input_sha256,
                    entry.deps_sha256 == deps_sha256,
                    entry.engine_version == crate_version()
                );
            }
        }

        None
    }

    /// Store a new cache entry after successful M4 expansion.
    pub fn store(
        &mut self,
        input_path: &Path,
        include_dirs: &[PathBuf],
        language: &str,
        output: &str,
        traces: &[String],
    ) {
        let entry = CacheEntry {
            input_sha256: Self::file_sha256(input_path),
            deps_sha256: Self::deps_sha256(input_path, include_dirs),
            output: output.to_string(),
            traces: traces.to_vec(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            language: language.to_string(),
            engine_version: crate_version(),
        };

        let key = Self::cache_key(input_path, language);
        let entry_path = self.cache_dir.join(format!("{}.json", key));

        if let Ok(json) = serde_json::to_string_pretty(&entry) {
            let mut f = fs::File::create(&entry_path).ok();
            if let Some(ref mut f) = f {
                let _ = f.write_all(json.as_bytes());
            }
            self.entries.insert(key, entry);
            eprintln!(
                "autom4te: cached {} ({} bytes, {} traces) → {}",
                input_path.display(),
                output.len(),
                traces.len(),
                entry_path.display()
            );
        }
    }

    /// Get cached trace lines for replay.
    pub fn get_traces(&self, input_path: &Path, language: &str) -> Vec<String> {
        let key = Self::cache_key(input_path, language);
        self.entries
            .get(&key)
            .map(|e| e.traces.clone())
            .unwrap_or_default()
    }

    /// Clear all cache entries for the given input file.
    pub fn invalidate(&self, input_path: &Path, language: &str) {
        let key = Self::cache_key(input_path, language);
        let entry_path = self.cache_dir.join(format!("{}.json", key));
        let _ = fs::remove_file(&entry_path);
    }

    /// Count of cache entries currently loaded.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// SHA256 hash of a file's contents.
    fn file_sha256(path: &Path) -> String {
        if let Ok(data) = fs::read(path) {
            let mut hasher = Sha256::new();
            hasher.update(&data);
            format!("{:x}", hasher.finalize())
        } else {
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        }
    }

    /// SHA256 of all M4 include files found in include_dirs.
    /// Concatenates all .m4 files in include dirs and hashes the result.
    fn deps_sha256(_input_path: &Path, include_dirs: &[PathBuf]) -> String {
        let mut hasher = Sha256::new();
        for dir in include_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                files.sort_by_key(|e| e.file_name());
                for entry in files {
                    let path = entry.path();
                    if path.extension().map(|e| e == "m4").unwrap_or(false) {
                        if let Ok(data) = fs::read(&path) {
                            hasher.update(&data);
                        }
                    }
                }
            }
        }
        format!("{:x}", hasher.finalize())
    }

    /// List all cached entries in the cache directory.
    pub fn list_entries(&self) -> Vec<String> {
        let mut entries = Vec::new();
        if let Ok(dir) = fs::read_dir(&self.cache_dir) {
            for entry in dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Some(name) = path.file_stem() {
                        entries.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        entries.sort();
        entries
    }
}

/// Get the current crate version for cache invalidation.
/// Uses CARGO_PKG_VERSION at compile time.
fn crate_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_store_and_lookup() {
        let tmp = std::env::temp_dir().join("ac_cache_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // Create a test configure.ac
        let ac_path = tmp.join("configure.ac");
        fs::write(&ac_path, "AC_INIT([test], [1.0])\nAC_OUTPUT\n").unwrap();

        // Create include dir with a .m4 file
        let m4_dir = tmp.join("m4");
        fs::create_dir_all(&m4_dir).unwrap();
        fs::write(m4_dir.join("extra.m4"), "# extra macros\n").unwrap();

        let cache_dir = tmp.join("autom4te.cache");
        let mut cache = Autom4teCache::new(&cache_dir);

        // First lookup should miss (empty cache)
        let result = cache.lookup(&ac_path, &[m4_dir.clone()], "Autoconf");
        assert!(result.is_none(), "empty cache should miss");

        // Store an entry
        cache.store(
            &ac_path,
            &[m4_dir.clone()],
            "Autoconf",
            "# test output\n",
            &["trace1".to_string(), "trace2".to_string()],
        );

        // Second lookup should hit
        let result = cache.lookup(&ac_path, &[m4_dir.clone()], "Autoconf");
        assert!(result.is_some(), "stored cache should hit");
        assert_eq!(result.unwrap(), "# test output\n");

        // Traces should be retrievable
        let traces = cache.get_traces(&ac_path, "Autoconf");
        assert_eq!(traces.len(), 2);

        // Invalidate and verify miss
        cache.invalidate(&ac_path, "Autoconf");
        let result = cache.lookup(&ac_path, &[m4_dir.clone()], "Autoconf");
        assert!(result.is_none(), "invalidated cache should miss");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_cache_force_mode() {
        let tmp = std::env::temp_dir().join("ac_cache_force_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let ac_path = tmp.join("configure.ac");
        fs::write(&ac_path, "AC_INIT([force], [2.0])\nAC_OUTPUT\n").unwrap();

        let cache_dir = tmp.join("autom4te.cache");
        let mut cache = Autom4teCache::new(&cache_dir);

        // Store an entry
        cache.store(&ac_path, &[], "Autoconf", "cached output", &[]);

        // Normal lookup should hit
        assert!(cache.lookup(&ac_path, &[], "Autoconf").is_some());

        // Force mode should bypass cache
        cache.set_force(true);
        assert!(cache.lookup(&ac_path, &[], "Autoconf").is_none());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_cache_stale_on_input_change() {
        let tmp = std::env::temp_dir().join("ac_cache_stale_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let ac_path = tmp.join("configure.ac");
        fs::write(&ac_path, "AC_INIT([v1], [1.0])\nAC_OUTPUT\n").unwrap();

        let cache_dir = tmp.join("autom4te.cache");
        let mut cache = Autom4teCache::new(&cache_dir);

        // Store
        cache.store(&ac_path, &[], "Autoconf", "output v1", &[]);

        // Verify hit
        assert!(cache.lookup(&ac_path, &[], "Autoconf").is_some());

        // Change input file
        fs::write(&ac_path, "AC_INIT([v2], [2.0])\nAC_OUTPUT\n").unwrap();

        // Should now miss (stale)
        assert!(cache.lookup(&ac_path, &[], "Autoconf").is_none());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_cache_entry_count() {
        let tmp = std::env::temp_dir().join("ac_cache_count_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let ac1 = tmp.join("configure1.ac");
        let ac2 = tmp.join("configure2.ac");
        fs::write(&ac1, "AC_INIT([one], [1.0])\n").unwrap();
        fs::write(&ac2, "AC_INIT([two], [2.0])\n").unwrap();

        let cache_dir = tmp.join("autom4te.cache");
        let mut cache = Autom4teCache::new(&cache_dir);

        cache.store(&ac1, &[], "Autoconf", "out1", &[]);
        cache.store(&ac2, &[], "Autoconf", "out2", &[]);

        assert_eq!(cache.entry_count(), 2);

        let entries = cache.list_entries();
        assert_eq!(entries.len(), 2);

        let _ = fs::remove_dir_all(&tmp);
    }
}
