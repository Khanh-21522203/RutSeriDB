//! MemTable — in-memory sorted write buffer for a single shard.
//!
//! See: docs/components.md § MemTable
//!
//! Data structure: `BTreeMap<MemKey, Row>` sorted by
//! `(timestamp ASC, tag_hash ASC)` for efficient merge-flush.
//!
//! Concurrency model (architecture doc):
//! - One writer (the ShardActor) holds exclusive access
//! - Many readers take a snapshot clone at query start
//!
//! Does NOT:
//! - Write to disk
//! - Know about WAL or Part files
//! - Decide when to flush (caller decides based on `size_bytes()`)

use std::collections::BTreeMap;

use crate::common::types::{MemKey, Row};

/// A frozen, read-only snapshot of the MemTable.
///
/// Created by `MemTable::snapshot()` for query reads.
/// The original MemTable can continue accepting writes.
#[derive(Debug, Clone)]
pub struct MemTableSnapshot {
    /// Sorted rows, keyed by (timestamp, tag_hash).
    pub data: BTreeMap<MemKey, Row>,

    /// Table name for this snapshot.
    pub table: String,
}

/// In-memory sorted write buffer for a single shard.
pub struct MemTable {
    /// The sorted rows. Key = (timestamp, tag_hash), Value = Row.
    data: BTreeMap<MemKey, Row>,

    /// Approximate memory usage in bytes.
    /// Updated on every insert.
    estimated_bytes: usize,

    /// Table name this MemTable belongs to.
    table: String,
}

impl MemTable {
    /// Create a new empty MemTable for a table.
    pub fn new(table: String) -> Self {
        Self {
            data: BTreeMap::new(),
            estimated_bytes: 0,
            table,
        }
    }

    /// Insert rows into the sorted structure.
    ///
    /// Each row's MemKey is computed from its timestamp and a hash
    /// of its tag set. Duplicate keys overwrite (last-write-wins).
    ///
    /// # Arguments
    /// * `rows` - Rows to insert.
    pub fn insert(&mut self, rows: Vec<Row>) {
        for row in rows {
            // TODO(engineer): compute tag_hash from row.tags
            // Use xxhash64 over sorted key=value\0 pairs (same as shard_key)
            let tag_hash: u64 = 0; // placeholder

            let key = MemKey {
                timestamp: row.timestamp,
                tag_hash,
            };

            // TODO(engineer): update estimated_bytes accurately
            // Account for key size + row size (recursively)
            self.estimated_bytes += std::mem::size_of::<MemKey>() + 128; // rough estimate

            self.data.insert(key, row);
        }
    }

    /// Current approximate memory usage in bytes.
    ///
    /// Used by the ShardActor to decide when to trigger a flush.
    pub fn size_bytes(&self) -> usize {
        self.estimated_bytes
    }

    /// Number of rows currently in the MemTable.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the MemTable has no rows.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Create a frozen, read-only snapshot for queries.
    ///
    /// This clones the entire BTreeMap. The MemTable can continue
    /// accepting writes after this call returns.
    pub fn snapshot(&self) -> MemTableSnapshot {
        MemTableSnapshot {
            data: self.data.clone(),
            table: self.table.clone(),
        }
    }

    /// Clear all data. Called after a successful flush to Part.
    ///
    /// The caller must ensure the flush is committed before clearing.
    pub fn clear(&mut self) {
        self.data.clear();
        self.estimated_bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::common::types::FieldValue;

    #[test]
    fn test_insert_and_size() {
        let mut mt = MemTable::new("test_table".to_string());
        assert!(mt.is_empty());
        assert_eq!(mt.size_bytes(), 0);

        let row = Row {
            timestamp: 1_000_000_000,
            tags: BTreeMap::from([("host".to_string(), "web-01".to_string())]),
            fields: BTreeMap::from([("cpu".to_string(), FieldValue::Float(42.0))]),
        };

        mt.insert(vec![row]);
        assert_eq!(mt.len(), 1);
        assert!(mt.size_bytes() > 0);
    }

    #[test]
    fn test_snapshot_is_independent() {
        let mut mt = MemTable::new("test_table".to_string());
        let row = Row {
            timestamp: 1,
            tags: BTreeMap::new(),
            fields: BTreeMap::new(),
        };

        mt.insert(vec![row]);
        let snap = mt.snapshot();

        // Clear original — snapshot should be unaffected
        mt.clear();
        assert!(mt.is_empty());
        assert_eq!(snap.data.len(), 1);
    }
}
