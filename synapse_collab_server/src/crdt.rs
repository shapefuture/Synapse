//! CRDT/OT logic for distributed, conflict-free collaborative editing
//! (Phase 4 foundation)

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

/// A simple sequence CRDT for text editing (RGA/WOOT-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCRDT {
    /// Map from position to character and its unique ID
    pub chars: BTreeMap<u64, (char, u64)>,
    /// Monotonically increasing version
    pub version: u64,
}

impl TextCRDT {
    pub fn new() -> Self {
        Self {
            chars: BTreeMap::new(),
            version: 1,
        }
    }

    /// Insert a character at a position
    pub fn insert(&mut self, pos: u64, c: char, id: u64) {
        self.chars.insert(pos, (c, id));
        self.version += 1;
    }

    /// Remove a character by position
    pub fn remove(&mut self, pos: u64) {
        self.chars.remove(&pos);
        self.version += 1;
    }

    /// Get the current text
    pub fn text(&self) -> String {
        self.chars.values().map(|(c, _)| *c).collect()
    }
}
