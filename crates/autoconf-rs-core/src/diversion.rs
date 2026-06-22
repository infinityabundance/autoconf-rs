//! Diversion Manager for M4 expansion output ordering.
//!
//! GNU Autoconf relies on M4 diversions to control output ordering:
//! - divert(-1): discard output (used by AC_REQUIRE to defer expansion)
//! - divert(0): normal output
//! - divert(1..N): numbered diversions for reordering
//! - undivert(N): flush diversion N to current output
//! - At EOF, all numbered diversions are undiverted in order.
//!
//! This is critical for AC_REQUIRE: when macro A requires macro B,
//! B's output is diverted to a lower number so it appears before A's.
//!
//! Court: AC.M4.DIVERT.1
//! Panel mandate: "Implement diversions before more macros."

use std::collections::HashMap;

/// Manages M4 diversion state for output ordering.
#[derive(Debug, Clone, Default)]
pub struct DiversionManager {
    /// Current diversion number (-1 = discard, 0 = normal)
    current: i32,
    /// Stored diversion content by number
    buffers: HashMap<i32, Vec<u8>>,
    /// Total bytes written (including discarded)
    total_written: usize,
    /// Total bytes discarded (divert -1)
    total_discarded: usize,
}

impl DiversionManager {
    pub fn new() -> Self {
        Self {
            current: 0,
            buffers: HashMap::new(),
            total_written: 0,
            total_discarded: 0,
        }
    }

    /// Change current diversion.
    /// n = -1: discard output
    /// n = 0: normal output
    /// n > 0: numbered diversion
    pub fn divert(&mut self, n: i32) {
        self.current = n;
    }

    /// Get current diversion number (divnum builtin).
    pub fn divnum(&self) -> i32 {
        self.current
    }

    /// Write data to the current diversion buffer.
    /// If current diversion is -1, data is discarded.
    pub fn write(&mut self, data: &[u8]) {
        self.total_written += data.len();
        if self.current == -1 {
            self.total_discarded += data.len();
            return;
        }
        self.buffers
            .entry(self.current)
            .or_default()
            .extend_from_slice(data);
    }

    /// Undivert a numbered diversion: move its content to the current diversion.
    /// Returns the bytes that were moved.
    pub fn undivert(&mut self, n: i32) -> Vec<u8> {
        if let Some(data) = self.buffers.remove(&n) {
            if self.current != -1 {
                self.buffers
                    .entry(self.current)
                    .or_default()
                    .extend_from_slice(&data);
            }
            data
        } else {
            Vec::new()
        }
    }

    /// Undivert all numbered diversions at EOF.
    /// Returns all diversion content in numeric order.
    pub fn collect_all(&self) -> Vec<u8> {
        let mut keys: Vec<i32> = self.buffers.keys().copied().collect();
        keys.sort();
        let mut output = Vec::new();
        for k in keys {
            if k != -1 {
                if let Some(data) = self.buffers.get(&k) {
                    output.extend_from_slice(data);
                }
            }
        }
        output
    }

    /// Get content of a specific diversion without removing it.
    pub fn peek(&self, n: i32) -> Option<&[u8]> {
        self.buffers.get(&n).map(|v| v.as_slice())
    }

    /// Clear all diversion buffers (for testing/reset).
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.current = 0;
        self.total_written = 0;
        self.total_discarded = 0;
    }

    /// Statistics about diversion usage.
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.buffers.len(), self.total_written, self.total_discarded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divert_discard() {
        let mut dm = DiversionManager::new();
        dm.write(b"normal output\n");
        dm.divert(-1);
        dm.write(b"discarded\n");
        dm.divert(0);
        dm.write(b"more normal\n");

        let output = dm.collect_all();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("normal output"));
        assert!(text.contains("more normal"));
        assert!(!text.contains("discarded"));
        assert_eq!(dm.total_discarded, b"discarded\n".len());
    }

    #[test]
    fn test_divert_reorder() {
        let mut dm = DiversionManager::new();
        dm.write(b"body start\n");

        dm.divert(1);
        dm.write(b"appears first\n");

        dm.divert(2);
        dm.write(b"appears second\n");

        dm.divert(0);
        dm.write(b"body end\n");

        let output = dm.collect_all();
        let text = String::from_utf8_lossy(&output);

        // Diversion 1 should appear before diversion 2 and before body
        let pos_body = text.find("body start").unwrap();
        let pos_first = text.find("appears first").unwrap();
        let pos_second = text.find("appears second").unwrap();

        assert!(pos_body < pos_second, "body should be before diversion 2");
        assert!(
            pos_first < pos_body || pos_first < pos_second,
            "diversion 1 should be early"
        );
    }

    #[test]
    fn test_undivert() {
        let mut dm = DiversionManager::new();
        dm.divert(5);
        dm.write(b"stored data\n");
        dm.divert(0);
        dm.write(b"after store\n");
        dm.undivert(5);
        dm.write(b"final\n");

        let output = dm.collect_all();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("stored data"));
        assert!(text.contains("after store"));
        assert!(text.contains("final"));
    }

    #[test]
    fn test_divnum() {
        let mut dm = DiversionManager::new();
        assert_eq!(dm.divnum(), 0);
        dm.divert(3);
        assert_eq!(dm.divnum(), 3);
        dm.divert(-1);
        assert_eq!(dm.divnum(), -1);
    }

    #[test]
    fn test_clear() {
        let mut dm = DiversionManager::new();
        dm.write(b"data");
        dm.divert(1);
        dm.write(b"more");
        dm.clear();
        assert_eq!(dm.buffers.len(), 0);
        assert_eq!(dm.divnum(), 0);
        assert_eq!(dm.total_written, 0);
    }
}
