//! Query planner — AST → PhysicalPlan with index-aware Part pruning.
//!
//! See: docs/architecture.md § Indexing — Index Application Order
//! See: docs/storage/indexes.md § Index Interaction in the Query Planner
//!
//! Pruning order (cheapest first):
//! 1. Inverted Index (tag equality → exact Part list) — O(1) lookup
//! 2. Min/Max Index (time/value range → prune Parts) — O(1) per Part
//! 3. Bloom Filters (remaining equality → prune Parts) — O(1) per Part
//! 4. Column scan with row-level filtering
//!
//! Does NOT:
//! - Read column data (the Executor does that)
//! - Modify any data

use std::path::PathBuf;

use crate::common::error::Result;
use crate::common::types::PartMeta;
use crate::storage::catalog::catalog::Catalog;

use super::ast::{Filter, SelectQuery};

/// A physical execution plan produced by the planner.
#[derive(Debug)]
pub struct PhysicalPlan {
    /// The original query (for executor reference).
    pub query: SelectQuery,

    /// Parts that survived all index pruning — the executor reads these.
    pub parts_to_scan: Vec<PartScanPlan>,

    /// Whether to also scan the MemTable snapshot.
    pub scan_memtable: bool,
}

/// Plan for scanning a single Part file.
#[derive(Debug)]
pub struct PartScanPlan {
    /// Path to the .rpart file.
    pub path: PathBuf,
    /// Part metadata (for logging / debugging).
    pub meta: PartMeta,
    /// Column names to read (projection pushdown).
    pub projected_columns: Vec<String>,
}

/// Create a physical plan by pruning Parts using all available indexes.
///
/// # Arguments
/// * `query` — Parsed SELECT query
/// * `catalog` — Shard catalog (Part registry + inverted index)
/// * `shard_dir` — Path to shard data directory
///
/// # Returns
/// A `PhysicalPlan` listing the Parts the executor should scan.
pub fn plan(
    query: SelectQuery,
    catalog: &Catalog,
    shard_dir: &std::path::Path,
) -> Result<PhysicalPlan> {
    // TODO(engineer): implement the 3-stage pruning pipeline
    //
    // Step 1: Get all Parts for the target table
    //   let all_parts = catalog.list_parts(&query.table);
    //
    // Step 2: Inverted Index pruning (tag equality filters)
    //   For each Filter::Equals where the column is a tag:
    //     candidate_part_ids = catalog.lookup_inverted(table, tag_key, tag_value)
    //     intersect with current candidates
    //
    // Step 3: Min/Max Index pruning (range filters)
    //   For each surviving Part:
    //     Read MinMax index from the Part file (PartReader::read_minmax)
    //     For each range filter (GreaterThan, LessThan, Between):
    //       If MinMax says no overlap → remove from candidates
    //
    // Step 4: Bloom Filter pruning (remaining equality filters)
    //   For each surviving Part:
    //     Read Bloom filters (PartReader::read_bloom)
    //     For each equality filter on tag/field columns:
    //       If Bloom definitely misses → remove from candidates
    //
    // Step 5: Build PartScanPlan for each surviving Part
    //   Include only projected columns (from query.projection)
    //
    // Step 6: Return PhysicalPlan

    todo!("planner::plan")
}

/// Extract column names needed for projection from the query.
///
/// Used to determine which columns to read from Part files.
pub fn extract_projected_columns(query: &SelectQuery) -> Vec<String> {
    // TODO(engineer): implement
    // - Walk projection list
    // - For Projection::Column(name) → include name
    // - For Projection::Agg(agg) → include agg.column
    // - For Projection::Star → return empty (meaning all columns)
    // - Also include columns mentioned in filters, group_by, order_by

    todo!("extract_projected_columns")
}
