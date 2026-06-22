//! TraceLog event caching for autom4te/autoheader — panel mandate.
//!
//! GNU Autoconf's autom4te caches frozen M4 state (.m4f) so that
//! autoheader can extract AC_DEFINE trace events without re-running
//! the full M4 expansion. This module provides equivalent functionality
//! using JSON-serialized TraceLog events with SHA256 integrity.
//!
//! Court: AC.TRACE.CACHE.1
//! Receipt: AC.M4.AUTOM4TE.CACHE.1 (extension)

use crate::trace::{AutoconfEvent, TraceLog};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// A serializable trace event for cache storage.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CachedTraceEvent {
    pub event_type: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub file: Option<String>,
    pub line: Option<usize>,
}

impl From<&AutoconfEvent> for CachedTraceEvent {
    fn from(event: &AutoconfEvent) -> Self {
        match event {
            AutoconfEvent::Init {
                package,
                version,
                bug_report: _,
                tarname: _,
                origin: _,
            } => CachedTraceEvent {
                event_type: "AC_INIT".into(),
                name: Some(package.clone()),
                value: Some(version.clone()),
                file: None,
                line: None,
            },
            AutoconfEvent::Define {
                name,
                value,
                description: _,
                origin: _,
            } => CachedTraceEvent {
                event_type: "AC_DEFINE".into(),
                name: Some(name.clone()),
                value: value.clone(),
                file: None,
                line: None,
            },
            AutoconfEvent::Subst {
                name,
                value,
                origin: _,
            } => CachedTraceEvent {
                event_type: "AC_SUBST".into(),
                name: Some(name.clone()),
                value: value.clone(),
                file: None,
                line: None,
            },
            AutoconfEvent::ConfigFile {
                output,
                inputs: _,
                origin: _,
            } => CachedTraceEvent {
                event_type: "AC_CONFIG_FILES".into(),
                name: Some(output.clone()),
                value: None,
                file: None,
                line: None,
            },
            AutoconfEvent::ConfigHeader {
                output,
                templates: _,
                origin: _,
            } => CachedTraceEvent {
                event_type: "AC_CONFIG_HEADERS".into(),
                name: Some(output.clone()),
                value: None,
                file: None,
                line: None,
            },
            AutoconfEvent::CheckFunc {
                function,
                actions: _,
                origin: _,
            } => CachedTraceEvent {
                event_type: "AC_CHECK_FUNC".into(),
                name: Some(function.clone()),
                value: None,
                file: None,
                line: None,
            },
            AutoconfEvent::CheckHeader {
                header,
                actions: _,
                origin: _,
            } => CachedTraceEvent {
                event_type: "AC_CHECK_HEADER".into(),
                name: Some(header.clone()),
                value: None,
                file: None,
                line: None,
            },
            _ => CachedTraceEvent {
                event_type: "OTHER".into(),
                name: None,
                value: None,
                file: None,
                line: None,
            },
        }
    }
}

/// Cache entry for a configure.ac input with its trace events.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TraceCacheEntry {
    pub input_hash: String,
    pub events: Vec<CachedTraceEvent>,
    pub configure_output_hash: String,
}

/// Trace event cache manager.
#[derive(Default)]
pub struct TraceCache {
    entries: HashMap<String, TraceCacheEntry>,
}

impl TraceCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store trace events for a given input, keyed by SHA256 of input.
    pub fn store(&mut self, input: &str, trace_log: &TraceLog, output: &str) -> String {
        let input_hash = format!("{:x}", Sha256::digest(input.as_bytes()));
        let output_hash = format!("{:x}", Sha256::digest(output.as_bytes()));
        let events: Vec<CachedTraceEvent> = trace_log
            .events
            .iter()
            .map(CachedTraceEvent::from)
            .collect();

        let entry = TraceCacheEntry {
            input_hash: input_hash.clone(),
            events,
            configure_output_hash: output_hash,
        };
        self.entries.insert(input_hash.clone(), entry);
        input_hash
    }

    /// Look up cached trace events by input hash.
    pub fn lookup(&self, input_hash: &str) -> Option<&TraceCacheEntry> {
        self.entries.get(input_hash)
    }

    /// Get all AC_DEFINE events from the cache (for autoheader).
    pub fn get_defines(&self, input_hash: &str) -> Vec<CachedTraceEvent> {
        self.entries
            .get(input_hash)
            .map(|e| {
                e.events
                    .iter()
                    .filter(|ev| ev.event_type == "AC_DEFINE")
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if cache has an entry for this input.
    pub fn contains(&self, input_hash: &str) -> bool {
        self.entries.contains_key(input_hash)
    }

    /// Flush cache to JSON file.
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(path, json)
    }

    /// Load cache from JSON file.
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let entries: HashMap<String, TraceCacheEntry> = serde_json::from_str(&json)?;
        Ok(Self { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::Span;

    #[test]
    fn test_cache_store_and_lookup() {
        let mut cache = TraceCache::new();
        let mut log = TraceLog::new();
        log.push(AutoconfEvent::Define {
            name: "HAVE_FOO".into(),
            value: Some("1".into()),
            description: None,
            origin: Span::new("configure.ac", 1, 1),
        });
        log.push(AutoconfEvent::Subst {
            name: "CC".into(),
            value: Some("gcc".into()),
            origin: Span::new("configure.ac", 2, 1),
        });

        let hash = cache.store(
            "AC_INIT([t],[1.0])\nAC_DEFINE([HAVE_FOO],[1])\nAC_OUTPUT\n",
            &log,
            "fake_output",
        );
        assert!(cache.contains(&hash));

        let defines = cache.get_defines(&hash);
        assert_eq!(defines.len(), 1);
        assert_eq!(defines[0].name.as_deref(), Some("HAVE_FOO"));
        assert_eq!(defines[0].value.as_deref(), Some("1"));
    }

    #[test]
    fn test_cache_empty() {
        let cache = TraceCache::new();
        assert!(cache.get_defines("nonexistent").is_empty());
    }

    #[test]
    fn test_cache_init_event() {
        let mut cache = TraceCache::new();
        let mut log = TraceLog::new();
        log.push(AutoconfEvent::Init {
            package: "test".into(),
            version: "2.0".into(),
            bug_report: None,
            tarname: None,
            origin: Span::new("f", 1, 1),
        });
        let hash = cache.store("input", &log, "out");
        let entry = cache.lookup(&hash).unwrap();
        assert!(!entry.events.is_empty());
        assert_eq!(entry.events[0].event_type, "AC_INIT");
    }
}
