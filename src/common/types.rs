//! Core domain types shared across all modules.
//!
//! These types form the vocabulary of RutSeriDB. Every module uses them
//! for data interchange. They must remain serialization-friendly and
//! free of business logic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ── Timestamp ────────────────────────────────────────────────────────

/// Nanoseconds since Unix epoch.
///
/// i64 gives us a range of ±292 years from epoch, which is sufficient
/// for all practical TSDB workloads.
pub type Timestamp = i64;

// ── Shard ────────────────────────────────────────────────────────────

/// Identifies a shard within the cluster. Range: `[0, num_shards)`.
pub type ShardId = u32;

// ── Tags ─────────────────────────────────────────────────────────────

/// An ordered map of tag key → tag value.
///
/// BTreeMap is used (not HashMap) because:
/// 1. Shard key computation requires deterministic ordering
/// 2. Tags are displayed/logged in sorted order
/// 3. Tag cardinality is low, so BTreeMap overhead is negligible
pub type TagSet = BTreeMap<String, String>;

// ── Field Values ─────────────────────────────────────────────────────

/// A typed measurement value in a time-series row.
///
/// Corresponds to the `col_type` field in the `.rpart` ColumnHeader:
/// - `FieldFloat`  → col_type 2
/// - `FieldInt`    → col_type 3
/// - `FieldBool`   → col_type 4
/// - `FieldStr`    → col_type 5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    Str(String),
}

// ── Row ──────────────────────────────────────────────────────────────

/// A single time-series data point.
///
/// Each row belongs to a table, has a timestamp, a set of tags that
/// identify the series, and a set of fields that carry measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    /// Nanoseconds since Unix epoch.
    pub timestamp: Timestamp,

    /// Tag key-value pairs (e.g., `host=web-01, region=us-east`).
    /// Used for grouping, filtering, and shard routing.
    pub tags: TagSet,

    /// Field name → value (e.g., `cpu=87.5, mem=2048`).
    /// These are the actual measurements.
    pub fields: BTreeMap<String, FieldValue>,
}

// ── Ingest Batch ─────────────────────────────────────────────────────

/// A batch of rows submitted in a single ingest request.
///
/// The API layer deserializes the client payload into this struct,
/// then hands it to `IngestEngine`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestBatch {
    /// Target table name.
    pub table: String,

    /// Rows to insert. All rows must conform to the table's schema.
    pub rows: Vec<Row>,
}

// ── MemTable Key ─────────────────────────────────────────────────────

/// Sort key for rows inside the MemTable.
///
/// Rows are sorted by `(timestamp ASC, tag_hash ASC)` to enable
/// efficient merge-flush into Part files.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemKey {
    pub timestamp: Timestamp,
    pub tag_hash: u64,
}

// ── Part Metadata ────────────────────────────────────────────────────

/// Metadata about a committed `.rpart` file, stored in the Catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartMeta {
    /// Unique Part identifier (UUID v4).
    pub id: uuid::Uuid,

    /// Relative path to the `.rpart` file within the shard directory.
    pub path: String,

    /// Minimum timestamp in this Part.
    pub min_ts: Timestamp,

    /// Maximum timestamp in this Part.
    pub max_ts: Timestamp,

    /// File size in bytes.
    pub size_bytes: u64,

    /// Number of rows in this Part.
    pub row_count: u64,

    /// When this Part was created (Unix seconds).
    pub created_at: i64,
}

// ── Phase 1: Cluster Types ───────────────────────────────────────────

/// Unique identifier for a node in the cluster.
pub type NodeId = String;

/// Role a node plays in the cluster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    /// All-in-one mode for local development.
    Dev,
    /// Routes writes/queries, manages metadata via Raft.
    Coordinator,
    /// Stores data, runs ShardActors, serves sub-queries.
    Storage,
}

/// Liveness state of a node as seen by the SWIM gossip protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    Alive,
    Suspect,
    Dead,
}

/// Information about a node in the cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: NodeId,
    pub role: NodeRole,
    pub addr: String,
    pub state: NodeState,
}

/// Assignment of a shard to leader + replica nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardAssignment {
    pub shard_id: ShardId,
    pub leader: NodeId,
    pub replicas: Vec<NodeId>,
}

/// Read consistency level for distributed queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Read from any single node (including replicas). Fastest, may be stale.
    One,
    /// Read from a quorum of nodes.
    Quorum,
    /// Read from all nodes holding the shard.
    All,
}

impl Default for ConsistencyLevel {
    fn default() -> Self {
        Self::One
    }
}
