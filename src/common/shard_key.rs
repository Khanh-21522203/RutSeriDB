//! Shard key computation.
//!
//! Deterministically maps a set of primary tags to a shard index.
//! The algorithm is frozen at cluster creation — changing it would
//! require a full data migration.
//!
//! Algorithm:
//!   1. Sort primary tag key-value pairs alphabetically by key
//!   2. Concatenate as `key=value\0` pairs
//!   3. xxHash64 with seed 0
//!   4. Modulo `num_shards`

use xxhash_rust::xxh64;

use super::types::{ShardId, TagSet};

/// Compute the shard index for a given set of primary tags.
///
/// # Arguments
/// * `tags` - The row's tag set (only primary tags are used)
/// * `primary_tag_names` - Sorted list of primary tag column names
/// * `num_shards` - Total number of shards in the cluster
///
/// # Returns
/// Shard index in `[0, num_shards)`
///
/// # Panics
/// Panics if `num_shards` is 0.
pub fn compute_shard_key(
    tags: &TagSet,
    primary_tag_names: &[String],
    num_shards: u32,
) -> ShardId {
    assert!(num_shards > 0, "num_shards must be > 0");

    // Build the hash input: sorted key=value\0 pairs
    // primary_tag_names is already sorted alphabetically (contract).
    let mut input = Vec::new();
    for key in primary_tag_names {
        if let Some(value) = tags.get(key) {
            input.extend_from_slice(key.as_bytes());
            input.push(b'=');
            input.extend_from_slice(value.as_bytes());
            input.push(0); // null separator
        }
        // TODO(engineer): decide behavior for missing primary tags
        // Option A: skip (current) — produces a different hash
        // Option B: return an error — enforced at IngestEngine level
    }

    let hash = xxh64::xxh64(&input, 0);
    (hash % num_shards as u64) as ShardId
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_deterministic_shard_key() {
        let mut tags = BTreeMap::new();
        tags.insert("host".to_string(), "web-01".to_string());
        tags.insert("region".to_string(), "us-east".to_string());

        let primary = vec!["host".to_string(), "region".to_string()];

        let shard1 = compute_shard_key(&tags, &primary, 8);
        let shard2 = compute_shard_key(&tags, &primary, 8);

        assert_eq!(shard1, shard2, "Same tags must produce same shard");
        assert!(shard1 < 8, "Shard must be in [0, num_shards)");
    }

    #[test]
    fn test_different_tags_may_differ() {
        let primary = vec!["host".to_string()];

        let mut tags_a = BTreeMap::new();
        tags_a.insert("host".to_string(), "web-01".to_string());

        let mut tags_b = BTreeMap::new();
        tags_b.insert("host".to_string(), "db-01".to_string());

        let shard_a = compute_shard_key(&tags_a, &primary, 64);
        let shard_b = compute_shard_key(&tags_b, &primary, 64);

        // They *may* be the same due to hash collision, but likely differ
        // with 64 shards. This test just ensures no panic.
        assert!(shard_a < 64);
        assert!(shard_b < 64);
    }
}
