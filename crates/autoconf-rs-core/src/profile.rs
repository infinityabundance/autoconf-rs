//! Oracle profile management for autoconf-rs-core.
//!
//! Provides types and utilities for working with Autoconf oracle profiles
//! within the core crate.
//!
//! Current status: Phase 1 — stub.

use serde::{Deserialize, Serialize};

/// Oracle profile reference for the core crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleProfileRef {
    pub kind: String,
    pub version: String,
    pub path: String,
    pub sha256: String,
}

impl OracleProfileRef {
    pub fn new(kind: &str, version: &str, path: &str, sha256: &str) -> Self {
        Self {
            kind: kind.to_string(),
            version: version.to_string(),
            path: path.to_string(),
            sha256: sha256.to_string(),
        }
    }
}
