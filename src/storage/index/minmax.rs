//! MinMax Index — per-column min/max values for Part-level pruning.
//!
//! See: docs/storage/indexes.md § Min/Max Index
//!
//! Covers ALL columns (timestamps, tags, field values).
//! Enables O(1) range predicate pruning at the file level before
//! any column data is read.

use crate::common::error::Result;

/// A single min/max entry for one column.
#[derive(Debug, Clone)]
pub struct MinMaxEntry {
    /// Column index within the Part.
    pub col_idx: u16,
    /// Column name (for predicate matching).
    pub col_name: String,
    /// Minimum value (encoded as u64 bits — type-specific).
    pub min_val: u64,
    /// Maximum value (encoded as u64 bits — type-specific).
    pub max_val: u64,
}

/// MinMax index for all columns in a Part file.
#[derive(Debug, Clone)]
pub struct MinMaxIndex {
    pub entries: Vec<MinMaxEntry>,
}

impl MinMaxIndex {
    /// Build a MinMax index from column data during flush.
    ///
    /// Called by PartWriter for each column after encoding.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Add a min/max entry for a column.
    ///
    /// The caller is responsible for encoding values correctly:
    /// - Timestamps / integers: raw i64/u64 bits
    /// - Floats: IEEE 754 f64 bits
    /// - Tags / strings: xxHash64 of lexicographic min/max string
    /// - Bools: 0 (false) or 1 (true)
    pub fn add_entry(&mut self, col_idx: u16, col_name: String, min_val: u64, max_val: u64) {
        self.entries.push(MinMaxEntry {
            col_idx,
            col_name,
            min_val,
            max_val,
        });
    }

    /// Check if a range predicate on a column can be satisfied.
    ///
    /// Returns `false` if the Part can be skipped (no overlap).
    /// Returns `true` if the Part might contain matching rows.
    pub fn may_contain_range(&self, col_name: &str, query_min: u64, query_max: u64) -> bool {
        // TODO(engineer): implement
        // Find the entry for col_name.
        // Return false if: entry.max_val < query_min OR entry.min_val > query_max
        // Return true otherwise (overlap exists).
        todo!("MinMaxIndex::may_contain_range")
    }

    /// Serialize the index to bytes for writing into the `.rpart` file.
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO(engineer): implement
        // Format: for each entry: col_idx(2B) + min_val(8B) + max_val(8B) = 18B per entry
        todo!("MinMaxIndex::to_bytes")
    }

    /// Deserialize from bytes.
    pub fn from_bytes(data: &[u8], num_columns: u32) -> Result<Self> {
        // TODO(engineer): implement
        todo!("MinMaxIndex::from_bytes")
    }
}
