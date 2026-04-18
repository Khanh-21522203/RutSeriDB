//! Local Catalog — persistent registry of committed Part files.
//!
//! See: docs/components.md § Local Catalog
//! See: docs/storage/indexes.md § Inverted Index
//!
//! The Catalog stores, per table:
//! - List of PartMeta records
//! - Inverted index: (tag_key, tag_value) → [part_id, ...]
//! - A monotonically increasing version counter
//!
//! Persistence model:
//!   1. Write `catalog.json.tmp` (new version)
//!   2. fsync
//!   3. rename → `catalog.json` (atomic on POSIX)
//!
//! Does NOT:
//! - Read/write Part file contents
//! - Know about WAL
//! - Make query planning decisions

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::common::error::{Result, RutSeriError};
use crate::common::types::PartMeta;

/// The persisted Catalog structure for a single shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    /// Monotonically increasing version counter.
    /// Incremented on every mutation (add/remove Part, update index).
    pub version: u64,

    /// Table name → list of Parts for that table.
    pub tables: HashMap<String, TableCatalog>,
}

/// Per-table catalog data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCatalog {
    /// All committed Parts for this table.
    pub parts: Vec<PartMeta>,

    /// Inverted index: tag_key → tag_value → [part_id, ...]
    ///
    /// Used for O(1) Part discovery by tag equality predicates.
    /// Built asynchronously by the IndexBuilder background worker.
    #[serde(default)]
    pub inverted_index: HashMap<String, HashMap<String, Vec<uuid::Uuid>>>,
}

impl Catalog {
    /// Create a new empty Catalog.
    pub fn new() -> Self {
        Self {
            version: 0,
            tables: HashMap::new(),
        }
    }

    /// Register a newly flushed Part file.
    pub fn add_part(&mut self, table: &str, meta: PartMeta) {
        let tc = self.tables.entry(table.to_string()).or_insert_with(|| {
            TableCatalog {
                parts: Vec::new(),
                inverted_index: HashMap::new(),
            }
        });
        tc.parts.push(meta);
        self.version += 1;
    }

    /// Remove a Part (after merge or deletion).
    pub fn remove_part(&mut self, table: &str, part_id: &uuid::Uuid) {
        if let Some(tc) = self.tables.get_mut(table) {
            tc.parts.retain(|p| &p.id != part_id);

            // TODO(engineer): also remove part_id from inverted_index entries
            // Iterate all tag keys/values and remove this part_id from their lists

            self.version += 1;
        }
    }

    /// List all Parts for a table.
    pub fn list_parts(&self, table: &str) -> Vec<&PartMeta> {
        self.tables
            .get(table)
            .map(|tc| tc.parts.iter().collect())
            .unwrap_or_default()
    }

    /// Lookup inverted index: which Parts contain this tag value?
    ///
    /// Returns an empty Vec if the tag key or value is not indexed.
    pub fn lookup_inverted(
        &self,
        table: &str,
        tag_key: &str,
        tag_value: &str,
    ) -> Vec<uuid::Uuid> {
        self.tables
            .get(table)
            .and_then(|tc| tc.inverted_index.get(tag_key))
            .and_then(|vals| vals.get(tag_value))
            .cloned()
            .unwrap_or_default()
    }

    /// Update the inverted index for a Part.
    ///
    /// Called by the IndexBuilder background worker after scanning
    /// a newly flushed Part for tag key/value pairs.
    pub fn update_inverted(
        &mut self,
        table: &str,
        part_id: uuid::Uuid,
        tag_entries: Vec<(String, String)>,
    ) {
        let tc = self.tables.entry(table.to_string()).or_insert_with(|| {
            TableCatalog {
                parts: Vec::new(),
                inverted_index: HashMap::new(),
            }
        });

        for (key, value) in tag_entries {
            tc.inverted_index
                .entry(key)
                .or_default()
                .entry(value)
                .or_default()
                .push(part_id);
        }
        self.version += 1;
    }

    /// Persist the catalog to disk atomically.
    ///
    /// Protocol: write `catalog.json.tmp` → fsync → rename to `catalog.json`
    pub fn persist(&self, shard_dir: &Path) -> Result<()> {
        // TODO(engineer): implement atomic write
        // let tmp_path = shard_dir.join("catalog.json.tmp");
        // let final_path = shard_dir.join("catalog.json");
        // 1. serde_json::to_vec_pretty(self)
        // 2. std::fs::write(&tmp_path, bytes)
        // 3. fsync the tmp file
        // 4. std::fs::rename(tmp_path, final_path)
        todo!("Catalog::persist")
    }

    /// Load catalog from disk.
    pub fn load(shard_dir: &Path) -> Result<Self> {
        let path = shard_dir.join("catalog.json");
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| RutSeriError::Catalog(format!("Failed to read catalog: {e}")))?;
        let catalog: Self = serde_json::from_str(&content)
            .map_err(|e| RutSeriError::Catalog(format!("Failed to parse catalog: {e}")))?;
        Ok(catalog)
    }
}
