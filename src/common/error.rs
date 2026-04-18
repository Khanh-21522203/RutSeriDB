//! Unified error types for RutSeriDB.
//!
//! All modules return `RutSeriError`. Engineers may add new variants
//! as needed — just keep the enum exhaustive and well-documented.

use thiserror::Error;

/// Top-level error type for RutSeriDB.
///
/// Every public API in the crate returns `Result<T, RutSeriError>`.
/// Add new variants as new failure modes are discovered.
#[derive(Debug, Error)]
pub enum RutSeriError {
    // ── I/O ──────────────────────────────────────────────────────────
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // ── WAL ──────────────────────────────────────────────────────────
    #[error("WAL corruption: {0}")]
    WalCorruption(String),

    #[error("WAL entry CRC mismatch at seq {seq}: expected {expected:#010x}, got {actual:#010x}")]
    WalCrcMismatch {
        seq: u64,
        expected: u32,
        actual: u32,
    },

    // ── Schema ───────────────────────────────────────────────────────
    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    #[error("Unknown table: {0}")]
    UnknownTable(String),

    #[error("Missing required primary tag: {0}")]
    MissingPrimaryTag(String),

    // ── Part file ────────────────────────────────────────────────────
    #[error("Invalid part file: {0}")]
    InvalidPartFile(String),

    #[error("Unsupported part format version: {0}")]
    UnsupportedVersion(u16),

    // ── Catalog ──────────────────────────────────────────────────────
    #[error("Catalog error: {0}")]
    Catalog(String),

    // ── Query ────────────────────────────────────────────────────────
    #[error("Query parse error: {0}")]
    QueryParse(String),

    #[error("Query planning error: {0}")]
    QueryPlan(String),

    #[error("Query execution error: {0}")]
    QueryExec(String),

    // ── Ingest ───────────────────────────────────────────────────────
    #[error("Ingest error: {0}")]
    Ingest(String),

    #[error("Shard dispatch channel closed")]
    ShardChannelClosed,

    #[error("Client disconnected before acknowledgement")]
    ClientDisconnected,

    // ── Config ───────────────────────────────────────────────────────
    #[error("Configuration error: {0}")]
    Config(String),

    // ── Serialization ────────────────────────────────────────────────
    #[error("Serialization error: {0}")]
    Serialization(String),

    // ── Phase 1: Cluster ─────────────────────────────────────────────
    #[error("Cluster error: {0}")]
    Cluster(String),

    #[error("Node unreachable: {node_id} at {addr}")]
    NodeUnreachable { node_id: String, addr: String },

    #[error("Leader not found for shard {0}")]
    LeaderNotFound(u32),

    // ── Phase 1: Replication ─────────────────────────────────────────
    #[error("Replication error: {0}")]
    Replication(String),

    #[error("Replication lag too large for shard {shard_id}: replica at seq {replica_seq}, leader at {leader_seq}")]
    ReplicationLagExceeded {
        shard_id: u32,
        replica_seq: u64,
        leader_seq: u64,
    },

    // ── Phase 1: Raft ────────────────────────────────────────────────
    #[error("Raft error: {0}")]
    Raft(String),

    // ── Phase 1: RPC ─────────────────────────────────────────────────
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("RPC timeout to {addr} after {timeout_ms} ms")]
    RpcTimeout { addr: String, timeout_ms: u64 },

    // ── Phase 1: Gossip ──────────────────────────────────────────────
    #[error("Gossip error: {0}")]
    Gossip(String),

    // ── Internal ─────────────────────────────────────────────────────
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, RutSeriError>;
