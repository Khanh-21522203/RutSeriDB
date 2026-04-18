//! Hand-defined gRPC message types.
//!
//! These types avoid requiring a protobuf compiler at build time.
//! They use `prost::Message` derive macros to be compatible with
//! the gRPC wire format.
//!
//! See: docs/phase1_plan.md § rpc

use serde::{Deserialize, Serialize};

use crate::common::types::{FieldValue, Row, ShardId, TagSet, Timestamp};

// ── Write Requests ───────────────────────────────────────────────────

/// Request from Coordinator to StorageNode: ingest a batch of rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteBatchRequest {
    /// Target table name.
    pub table: String,

    /// Target shard ID.
    pub shard_id: ShardId,

    /// Serialized rows to ingest.
    pub rows: Vec<Row>,
}

/// Response from StorageNode after ingesting a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteBatchResponse {
    /// Number of rows successfully ingested.
    pub rows_written: u64,
}

// ── Query Requests ───────────────────────────────────────────────────

/// Sub-query request from Coordinator to StorageNode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteQueryRequest {
    /// SQL sub-query string (with pushed-down filters).
    pub sql: String,
}

/// Sub-query response containing partial Arrow results.
///
/// The actual Arrow RecordBatch is serialized as IPC bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteQueryResponse {
    /// Arrow IPC serialized RecordBatch(es).
    pub arrow_ipc_data: Vec<u8>,

    /// Number of rows in the partial result.
    pub row_count: u64,
}

// ── Admin Requests ───────────────────────────────────────────────────

/// Request to force-flush a shard (admin or shutdown).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlushShardRequest {
    pub shard_id: ShardId,
}

/// Response after flushing a shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlushShardResponse {
    pub success: bool,
}

// ── Replication Offset ───────────────────────────────────────────────

/// Request for the current replication offset (used during failover).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetReplicationOffsetRequest {
    pub shard_id: ShardId,
}

/// Response with the current replication offset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetReplicationOffsetResponse {
    pub shard_id: ShardId,
    pub offset: u64,
}

// ── Node Registration ────────────────────────────────────────────────

/// Heartbeat / registration message from StorageNode to Coordinator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHeartbeat {
    pub node_id: String,
    pub addr: String,
    pub shard_ids: Vec<ShardId>,
    pub timestamp: Timestamp,
}
