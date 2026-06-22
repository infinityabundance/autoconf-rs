// docgen/mod.rs — Document generation engine for autoconf-rs.
//
// Generates all Markdown and JSON reports from JSON source files.
// Every generated document is freshness-gated: its SHA256 is recorded
// in the doc registry, and `cargo xtask check` fails if sources have changed.

pub mod dsse;
pub mod generate;
pub mod sync_metrics;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Registry of all generated documents and their source hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRegistry {
    pub schema: String,
    pub generated_at: String,
    pub documents: Vec<DocumentEntry>,
}

/// A single document entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEntry {
    pub path: String,
    pub sha256: String,
    pub sources: HashMap<String, String>,
    pub dsse_signature: Option<String>,
}

impl DocumentRegistry {
    pub fn new() -> Self {
        Self {
            schema: "autoconf-rs-doc-registry-v1".to_string(),
            generated_at: chrono_now(),
            documents: Vec::new(),
        }
    }

    /// Verify that all registered documents are fresh.
    ///
    /// A document is stale if:
    /// 1. The generated file is missing
    /// 2. The generated file's SHA256 doesn't match the registry
    /// 3. Any source file's SHA256 doesn't match the registry
    pub fn verify_freshness(&self) -> Result<Vec<String>, Vec<String>> {
        let mut msgs = Vec::new();
        let mut stale = Vec::new();

        for doc in &self.documents {
            let path = Path::new(&doc.path);
            if !path.exists() {
                stale.push(format!(
                    "STALE: {} is missing (run 'cargo xtask generate')",
                    doc.path
                ));
                continue;
            }

            // Check generated document freshness
            match std::fs::read_to_string(path) {
                Ok(contents) => {
                    let mut hasher = Sha256::new();
                    hasher.update(contents.as_bytes());
                    let current_hash = format!("{:x}", hasher.finalize());

                    if current_hash != doc.sha256 {
                        stale.push(format!(
                            "STALE: {} — document hash changed (run 'cargo xtask generate')",
                            doc.path
                        ));
                        continue;
                    }
                }
                Err(e) => {
                    stale.push(format!("STALE: {} — read error: {}", doc.path, e));
                    continue;
                }
            }

            // Check source freshness
            let mut source_stale = false;
            for (src_path, registered_hash) in &doc.sources {
                // Skip non-file sources like "generated" or "default"
                if src_path == "generated" || src_path == "default" {
                    continue;
                }

                let src = Path::new(src_path);
                if !src.exists() {
                    stale.push(format!(
                        "STALE: {} — source '{}' is missing",
                        doc.path, src_path
                    ));
                    source_stale = true;
                    break;
                }

                match std::fs::read_to_string(src) {
                    Ok(src_contents) => {
                        let mut hasher = Sha256::new();
                        hasher.update(src_contents.as_bytes());
                        let current_src_hash = format!("{:x}", hasher.finalize());

                        if &current_src_hash != registered_hash {
                            stale.push(format!(
                                "STALE: {} — source '{}' changed (run 'cargo xtask generate')",
                                doc.path, src_path
                            ));
                            source_stale = true;
                            break;
                        }
                    }
                    Err(e) => {
                        stale.push(format!(
                            "STALE: {} — cannot read source '{}': {}",
                            doc.path, src_path, e
                        ));
                        source_stale = true;
                        break;
                    }
                }
            }

            if !source_stale {
                msgs.push(format!("  OK {} (fresh)", doc.path));
            }
        }

        if stale.is_empty() {
            Ok(msgs)
        } else {
            Err(stale)
        }
    }

    /// Add or update a document entry.
    pub fn register(
        &mut self,
        path: &str,
        contents: &str,
        sources: HashMap<String, String>,
        key: &[u8],
    ) -> Result<(), String> {
        let mut hasher = Sha256::new();
        hasher.update(contents.as_bytes());
        let sha256 = format!("{:x}", hasher.finalize());

        let dsse_sig = dsse::sign_document(path, contents, key);

        // Remove existing entry for this path if present
        self.documents.retain(|d| d.path != path);

        self.documents.push(DocumentEntry {
            path: path.to_string(),
            sha256,
            sources,
            dsse_signature: Some(dsse_sig),
        });

        Ok(())
    }
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}
