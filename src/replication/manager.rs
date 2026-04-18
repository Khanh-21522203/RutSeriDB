//! Replication Manager — WAL streaming between leader and replicas.
//!
//! Uses custom TCP with length-prefixed framing for low-overhead
//! streaming on the hot replication path.
//!
//! Leader side: pushes WAL entries after local fsync
//! Replica side: applies entries in-order to local MemTable + WAL
//!
//! See: docs/cluster/replication.md § Normal Operation

use std::collections::HashMap;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::RwLock;

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{NodeId, ShardId};
use crate::storage::wal::entry::WalEntry;

/// Per-replica replication state.
#[derive(Debug)]
struct ReplicaState {
    /// Node ID of the replica.
    node_id: NodeId,

    /// Last acknowledged WAL sequence number.
    acked_seq: u64,

    /// TCP connection to the replica (None if disconnected).
    stream: Option<TcpStream>,
}

/// Replication Manager — runs on both leader and replica nodes.
///
/// On leader: pushes WAL entries to each replica via TCP.
/// On replica: listens for incoming WAL streams and applies entries.
pub struct ReplicationManager {
    /// This node's ID.
    node_id: NodeId,

    /// Per-shard, per-replica replication state (leader side).
    replicas: Arc<RwLock<HashMap<ShardId, Vec<ReplicaState>>>>,

    /// Per-shard replication offset (replica side).
    offsets: Arc<RwLock<HashMap<ShardId, u64>>>,

    /// Replication buffer size in bytes.
    buffer_size: usize,
}

impl ReplicationManager {
    pub fn new(node_id: NodeId, buffer_size: usize) -> Self {
        Self {
            node_id,
            replicas: Arc::new(RwLock::new(HashMap::new())),
            offsets: Arc::new(RwLock::new(HashMap::new())),
            buffer_size,
        }
    }

    // ── Leader Side ──────────────────────────────────────────────────

    /// Push WAL entries to all replicas for a shard (leader side).
    ///
    /// Called after the ShardActor fsync's the WAL. Entries are serialized
    /// with length-prefixed framing and sent over TCP.
    pub async fn push_entries(
        &self,
        _shard_id: ShardId,
        _entries: &[WalEntry],
        _from_seq: u64,
    ) -> Result<()> {
        // TODO(engineer): implement entry push
        //
        // For each replica in self.replicas[shard_id]:
        //   1. Serialize entries with length prefix: [u32:len][payload]
        //   2. Write to TCP stream
        //   3. Await ACK (u64 sequence number)
        //   4. Update replica.acked_seq
        //
        // Handle errors: if replica disconnects, remove from active set
        // and log a warning. ClusterManager will handle re-assignment.
        todo!("TODO(engineer): implement push_entries")
    }

    /// Add a replica to the replication set (leader side).
    pub async fn add_replica(
        &self,
        shard_id: ShardId,
        node_id: NodeId,
        addr: &str,
    ) -> Result<()> {
        // TODO(engineer): connect to replica via TCP, add to replicas map
        todo!("TODO(engineer): implement add_replica")
    }

    // ── Replica Side ─────────────────────────────────────────────────

    /// Start listening for incoming WAL streams (replica side).
    ///
    /// Binds a TCP listener and spawns a task for each incoming
    /// replication connection from a leader.
    pub async fn start_replica_listener(&self, _bind_addr: &str) -> Result<()> {
        // TODO(engineer): implement TCP listener
        //
        // let listener = TcpListener::bind(bind_addr).await?;
        // loop {
        //     let (stream, addr) = listener.accept().await?;
        //     tokio::spawn(self.handle_replication_stream(stream));
        // }
        todo!("TODO(engineer): implement start_replica_listener")
    }

    /// Apply received WAL entries to local storage (replica side).
    ///
    /// 1. Deserialize entries from length-prefixed TCP stream
    /// 2. Append to local WAL
    /// 3. Insert into MemTable
    /// 4. Send ACK back to leader
    pub async fn apply_entries(
        &self,
        _shard_id: ShardId,
        _entries: Vec<WalEntry>,
    ) -> Result<u64> {
        // TODO(engineer): implement entry application
        todo!("TODO(engineer): implement apply_entries")
    }

    /// Get current replication offset for a shard (both leader & replica).
    pub async fn replication_offset(&self, shard_id: ShardId) -> u64 {
        let offsets = self.offsets.read().await;
        offsets.get(&shard_id).copied().unwrap_or(0)
    }

    /// Check if a replica needs snapshot sync instead of streaming.
    pub async fn needs_snapshot(
        &self,
        _shard_id: ShardId,
        _replica_from_seq: u64,
    ) -> bool {
        // TODO(engineer): compare replica_from_seq against oldest
        // buffered WAL entry. If replica is too far behind, return true.
        todo!("TODO(engineer): implement needs_snapshot")
    }
}
