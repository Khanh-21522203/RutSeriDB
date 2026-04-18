//! Distributed Query Planner — fan-out SQL queries across storage nodes.
//!
//! Pipeline:
//! 1. Parse SQL → AST (reuses Phase 0 parser)
//! 2. Resolve table → find all shards
//! 3. Prune shards outside WHERE time range
//! 4. Rewrite into per-shard sub-queries (push down time + projections)
//! 5. Fan-out parallel gRPC calls to each relevant StorageNode
//! 6. Collect Arrow RecordBatches
//! 7. Final merge: re-sort, final aggregation, apply LIMIT
//!
//! See: docs/components.md § Coordinator — Distributed Query Planner

use std::sync::Arc;

use arrow::record_batch::RecordBatch;

use crate::common::error::{Result, RutSeriError};
use crate::coordinator::metadata_catalog::MetadataCatalog;
use crate::rpc::client::StorageNodeClient;

/// Distributed query planner and executor.
///
/// Lives on the Coordinator. Translates client SQL into per-shard
/// sub-queries, fans them out, and merges results.
pub struct DistributedQueryPlanner {
    /// Metadata catalog for shard/table resolution.
    catalog: Arc<MetadataCatalog>,

    /// RPC client pool for sub-query fan-out.
    rpc_client: Arc<StorageNodeClient>,
}

impl DistributedQueryPlanner {
    pub fn new(
        catalog: Arc<MetadataCatalog>,
        rpc_client: Arc<StorageNodeClient>,
    ) -> Self {
        Self { catalog, rpc_client }
    }

    /// Execute a distributed query.
    ///
    /// Returns merged Arrow RecordBatches from all relevant storage nodes.
    pub async fn execute(&self, sql: &str) -> Result<Vec<RecordBatch>> {
        // TODO(engineer): implement distributed query execution
        //
        // 1. Parse SQL → AST
        //    let ast = crate::query::parser::parse(sql)?;
        //
        // 2. Resolve table → get all shard assignments
        //    let table = extract_table_name(&ast)?;
        //    let shards = self.catalog.get_shards_for_table(&table);
        //
        // 3. Prune shards outside WHERE time range
        //    let time_range = extract_time_range(&ast)?;
        //    let relevant = self.prune_shards(&shards, &time_range);
        //
        // 4. Fan-out sub-queries in parallel
        //    let futures = relevant.iter().map(|shard| {
        //        let addr = self.catalog.get_node_addr(&shard.leader)?;
        //        self.rpc_client.execute_query(&addr, sql.to_string())
        //    });
        //    let results = futures::future::join_all(futures).await;
        //
        // 5. Merge: re-sort, final aggregation, apply LIMIT
        //    self.merge_results(results, &ast)

        todo!("TODO(engineer): implement DistributedQueryPlanner.execute")
    }

    /// Merge partial RecordBatches from multiple storage nodes.
    ///
    /// Handles: re-sort by time, final aggregation (sum partial sums,
    /// count partial counts, etc.), and global LIMIT.
    fn merge_results(
        &self,
        _partials: Vec<Vec<RecordBatch>>,
    ) -> Result<Vec<RecordBatch>> {
        // TODO(engineer): implement result merging
        // - Re-sort by (timestamp, group-by keys) across all partials
        // - Final aggregation pass (e.g., SUM of partial SUMs)
        // - Apply global LIMIT
        todo!("TODO(engineer): implement merge_results")
    }
}
