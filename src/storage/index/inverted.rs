//! Inverted Index — tag → Part ID mapping at the Catalog level.
//!
//! See: docs/storage/indexes.md § Inverted Index
//!
//! This module provides helper functions for operating on the inverted
//! index that is stored _inside_ the Catalog. The data structure lives
//! in `storage::catalog::catalog::TableCatalog::inverted_index`.
//!
//! The InvertedIndex module exists to keep index logic separate from
//! Catalog persistence logic.

use std::collections::HashMap;

/// Helper to merge new tag entries into an existing inverted index.
///
/// Called by the IndexBuilder worker after scanning a Part file.
///
/// # Arguments
/// * `index` — Mutable reference to the inverted index map
/// * `part_id` — The Part that was scanned
/// * `tag_entries` — Unique `(tag_key, tag_value)` pairs found in the Part
pub fn merge_entries(
    index: &mut HashMap<String, HashMap<String, Vec<uuid::Uuid>>>,
    part_id: uuid::Uuid,
    tag_entries: Vec<(String, String)>,
) {
    for (key, value) in tag_entries {
        index
            .entry(key)
            .or_default()
            .entry(value)
            .or_default()
            .push(part_id);
    }
}

/// Remove a Part ID from all entries in the inverted index.
///
/// Called when a Part is deleted (after merge or compaction).
pub fn remove_part(
    index: &mut HashMap<String, HashMap<String, Vec<uuid::Uuid>>>,
    part_id: &uuid::Uuid,
) {
    for tag_values in index.values_mut() {
        for part_ids in tag_values.values_mut() {
            part_ids.retain(|id| id != part_id);
        }
    }

    // TODO(engineer): optionally prune empty entries to save memory
    // Remove tag_value entries with no Part IDs
    // Remove tag_key entries with no tag_value entries
}

/// Lookup which Part IDs contain a specific tag value.
///
/// Returns an empty Vec if the tag key or value is not indexed.
pub fn lookup(
    index: &HashMap<String, HashMap<String, Vec<uuid::Uuid>>>,
    tag_key: &str,
    tag_value: &str,
) -> Vec<uuid::Uuid> {
    index
        .get(tag_key)
        .and_then(|vals| vals.get(tag_value))
        .cloned()
        .unwrap_or_default()
}
