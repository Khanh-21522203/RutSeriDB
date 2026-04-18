//! Read Router — routes read queries to replicas for follower reads.
//!
//! Supports consistency levels:
//! - `ONE`: read from any single replica (fastest, may be stale)
//! - `QUORUM`: read from a quorum and pick the most recent
//! - `ALL`: read from all replicas (strongest, slowest)
//!
//! See: docs/architecture.md § Configuration Reference — consistency

use std::sync::Arc;

use arrow::record_batch::RecordBatch;

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{ConsistencyLevel, NodeId, ShardId};
use crate::coordinator::metadata_catalog::MetadataCatalog;
use crate::rpc::client::StorageNodeClient;

/// Routes read queries considering consistency level.
///
/// When `consistency=ONE`, reads can be served from any replica,
/// reducing load on leaders and improving read throughput.
pub struct ReadRouter {
    /// Metadata catalog for node/shard lookups.
    catalog: Arc<MetadataCatalog>,

    /// RPC client for sub-query execution.
    rpc_client: Arc<StorageNodeClient>,

    /// Default consistency level.
    consistency: ConsistencyLevel,
}

impl ReadRouter {
    pub fn new(
        catalog: Arc<MetadataCatalog>,
        rpc_client: Arc<StorageNodeClient>,
        consistency: ConsistencyLevel,
    ) -> Self {
        Self {
            catalog,
            rpc_client,
            consistency,
        }
    }

    /// Select a node to read from based on the consistency level.
    ///
    /// - `ONE`: pick any node (leader or replica), prefer least-loaded
    /// - `QUORUM`: return a quorum of nodes
    /// - `ALL`: return all nodes
    pub async fn select_read_targets(
        &self,
        shard_id: ShardId,
    ) -> Result<Vec<NodeId>> {
        // TODO(engineer): implement target selection
        //
        // let assignment = self.catalog.get_shard_map().await
        //     .into_iter()
        //     .find(|a| a.shard_id == shard_id)
        //     .ok_or(RutSeriError::LeaderNotFound(shard_id))?;
        //
        // match self.consistency {
        //     ConsistencyLevel::One => {
        //         // Round-robin or random pick from leader + replicas
        //         let mut all = vec![assignment.leader.clone()];
        //         all.extend(assignment.replicas.clone());
        //         Ok(vec![all[rand::random::<usize>() % all.len()].clone()])
        //     }
        //     ConsistencyLevel::Quorum => {
        //         let total = 1 + assignment.replicas.len();
        //         let quorum = total / 2 + 1;
        //         // Pick `quorum` nodes
        //         todo!()
        //     }
        //     ConsistencyLevel::All => {
        //         let mut all = vec![assignment.leader.clone()];
        //         all.extend(assignment.replicas.clone());
        //         Ok(all)
        //     }
        // }
        todo!("TODO(engineer): implement ReadRouter.select_read_targets")
    }

    /// Execute a query at the specified consistency level.
    pub async fn execute_read(
        &self,
        _sql: &str,
        _shard_id: ShardId,
    ) -> Result<Vec<RecordBatch>> {
        // TODO(engineer): implement consistency-aware read
        // 1. Select targets
        // 2. Fan-out query to targets
        // 3. For QUORUM/ALL: compare results, pick most recent
        // 4. Return
        todo!("TODO(engineer): implement ReadRouter.execute_read")
    }
}
