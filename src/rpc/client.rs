//! gRPC client wrappers for Coordinator → StorageNode communication.
//!
//! Provides a typed interface for the Coordinator to call StorageNode
//! endpoints: WriteBatch, ExecuteQuery, FlushShard, GetReplicationOffset.
//!
//! See: docs/phase1_plan.md § rpc

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use arrow::record_batch::RecordBatch;
use tokio::sync::RwLock;

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{IngestBatch, Row, ShardId};
use crate::rpc::proto::*;

/// Client pool for StorageNode gRPC connections.
///
/// Maintains reusable connections keyed by node address.
/// Used by `WriteRouter`, `DistributedQueryPlanner`, and `ClusterManager`.
pub struct StorageNodeClient {
    /// Timeout for RPC calls.
    timeout: Duration,

    // TODO(engineer): add connection pool
    // connections: Arc<RwLock<HashMap<String, tonic::transport::Channel>>>,
}

impl StorageNodeClient {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Send a write batch to a Storage Node.
    pub async fn write_batch(
        &self,
        _addr: &str,
        _batch: IngestBatch,
    ) -> Result<WriteBatchResponse> {
        // TODO(engineer): implement gRPC call
        //
        // 1. Get or create channel from connection pool
        // 2. Serialize IngestBatch → WriteBatchRequest
        // 3. Call StorageNodeService::write_batch
        // 4. Handle timeout / connection errors
        todo!("TODO(engineer): implement StorageNodeClient.write_batch")
    }

    /// Execute a sub-query on a Storage Node.
    pub async fn execute_query(
        &self,
        _addr: &str,
        _sql: String,
    ) -> Result<Vec<RecordBatch>> {
        // TODO(engineer): implement gRPC call
        //
        // 1. Get or create channel
        // 2. Send ExecuteQueryRequest { sql }
        // 3. Receive ExecuteQueryResponse { arrow_ipc_data }
        // 4. Deserialize Arrow IPC → RecordBatch
        todo!("TODO(engineer): implement StorageNodeClient.execute_query")
    }

    /// Force-flush a shard on a Storage Node.
    pub async fn flush_shard(
        &self,
        _addr: &str,
        _shard_id: ShardId,
    ) -> Result<()> {
        // TODO(engineer): implement gRPC call
        todo!("TODO(engineer): implement StorageNodeClient.flush_shard")
    }

    /// Get replication offset for a shard (used during failover).
    pub async fn get_replication_offset(
        &self,
        _addr: &str,
        _shard_id: ShardId,
    ) -> Result<u64> {
        // TODO(engineer): implement gRPC call
        todo!("TODO(engineer): implement get_replication_offset")
    }

    // ── Connection Management ────────────────────────────────────────

    /// Get or create a gRPC channel to a node address.
    async fn get_channel(
        &self,
        _addr: &str,
    ) -> Result<()> {
        // TODO(engineer): implement connection pooling
        // let channel = tonic::transport::Channel::from_shared(addr.to_string())
        //     .map_err(|e| RutSeriError::Rpc(format!("invalid addr: {e}")))?
        //     .timeout(self.timeout)
        //     .connect()
        //     .await
        //     .map_err(|e| RutSeriError::NodeUnreachable {
        //         node_id: "unknown".into(),
        //         addr: addr.into(),
        //     })?;
        todo!("TODO(engineer): implement get_channel")
    }
}
