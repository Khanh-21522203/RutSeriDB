//! Metadata Catalog — Raft-replicated cluster metadata.
//!
//! Stores the authoritative:
//! - Table schemas (column names, types, primary tags)
//! - Shard → {leader, replicas} mapping
//! - Per-shard time range bounds (for query pruning)
//!
//! Replicated across 1–3 Coordinator nodes via `openraft`.
//!
//! See: docs/architecture.md § Cluster Management

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::common::error::{Result, RutSeriError};
use crate::common::schema::TableSchema;
use crate::common::types::{NodeId, NodeInfo, ShardAssignment, ShardId};

// ── Raft Log Entry Types ─────────────────────────────────────────────

/// Operations that are proposed to the Raft log and applied to the
/// metadata state machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataOp {
    /// Register a new node in the cluster.
    RegisterNode { node_id: NodeId, addr: String },

    /// Remove a node from the cluster.
    DeregisterNode { node_id: NodeId },

    /// Assign a shard to a leader and replicas.
    AssignShard {
        shard_id: ShardId,
        leader: NodeId,
        replicas: Vec<NodeId>,
    },

    /// Promote a replica to leader after failure.
    PromoteLeader {
        shard_id: ShardId,
        new_leader: NodeId,
    },

    /// Register a new table schema.
    RegisterTable {
        table: String,
        schema: TableSchema,
        primary_tags: Vec<String>,
    },
}

// ── Metadata State ───────────────────────────────────────────────────

/// In-memory state derived from applying the Raft log.
///
/// This is the data that all Coordinator instances agree on.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataState {
    /// node_id → NodeInfo
    pub nodes: HashMap<NodeId, NodeInfo>,

    /// shard_id → ShardAssignment
    pub shard_map: HashMap<ShardId, ShardAssignment>,

    /// table_name → (schema, primary_tags)
    pub tables: HashMap<String, (TableSchema, Vec<String>)>,

    /// Monotonically increasing version.
    pub version: u64,
}

// ── Metadata Catalog ─────────────────────────────────────────────────

/// Thread-safe metadata catalog backed by Raft consensus.
///
/// The `state` field is updated only by the Raft state machine's
/// `apply()` method. Reads are lock-free (RwLock read guard).
pub struct MetadataCatalog {
    state: Arc<RwLock<MetadataState>>,
}

impl MetadataCatalog {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MetadataState::default())),
        }
    }

    /// Apply a metadata operation (called by Raft state machine after commit).
    pub async fn apply(&self, op: MetadataOp) -> Result<()> {
        // TODO(engineer): implement apply — update internal state
        // based on the operation type.
        //
        // let mut state = self.state.write().await;
        // match op {
        //     MetadataOp::RegisterNode { node_id, addr } => { ... }
        //     MetadataOp::AssignShard { shard_id, leader, replicas } => { ... }
        //     MetadataOp::PromoteLeader { shard_id, new_leader } => { ... }
        //     MetadataOp::RegisterTable { table, schema, primary_tags } => { ... }
        //     ...
        // }
        // state.version += 1;
        todo!("TODO(engineer): implement MetadataCatalog.apply")
    }

    /// Get the leader node ID for a shard.
    pub async fn get_shard_leader(&self, shard_id: ShardId) -> Option<NodeId> {
        let state = self.state.read().await;
        state.shard_map.get(&shard_id).map(|a| a.leader.clone())
    }

    /// Get the address of a node.
    pub async fn get_node_addr(&self, node_id: &NodeId) -> Option<String> {
        let state = self.state.read().await;
        state.nodes.get(node_id).map(|n| n.addr.clone())
    }

    /// Get all shard assignments.
    pub async fn get_shard_map(&self) -> Vec<ShardAssignment> {
        let state = self.state.read().await;
        state.shard_map.values().cloned().collect()
    }

    /// Get table schema and primary tags.
    pub async fn get_table_schema(
        &self,
        table: &str,
    ) -> Option<(TableSchema, Vec<String>)> {
        let state = self.state.read().await;
        state.tables.get(table).cloned()
    }

    /// Snapshot the current state for Raft snapshot.
    pub async fn snapshot(&self) -> Result<Vec<u8>> {
        let state = self.state.read().await;
        serde_json::to_vec(&*state)
            .map_err(|e| RutSeriError::Raft(format!("snapshot serialization: {e}")))
    }

    /// Restore state from a Raft snapshot.
    pub async fn restore(&self, data: &[u8]) -> Result<()> {
        let restored: MetadataState = serde_json::from_slice(data)
            .map_err(|e| RutSeriError::Raft(format!("snapshot deserialization: {e}")))?;
        let mut state = self.state.write().await;
        *state = restored;
        Ok(())
    }
}
