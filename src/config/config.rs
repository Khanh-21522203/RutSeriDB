//! TOML-based configuration for RutSeriDB.
//!
//! All configuration sections are defined here. The config is loaded
//! once at startup via `RutSeriConfig::load(path)`.
//!
//! See: docs/architecture.md § Configuration Reference

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::common::error::{Result, RutSeriError};

/// Top-level configuration struct. Maps 1:1 to the TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct RutSeriConfig {
    pub cluster: ClusterConfig,
    pub storage: StorageConfig,
    pub memory: MemoryConfig,
    pub durability: DurabilityConfig,
    pub threads: ThreadConfig,
    pub merge: MergeConfig,
    pub indexes: IndexConfig,

    // ── Phase 1: Distribution ────────────────────────────────────────
    #[serde(default)]
    pub gossip: GossipConfig,

    #[serde(default)]
    pub consistency: ConsistencyConfig,

    #[serde(default)]
    pub raft: RaftConfig,

    /// Per-table overrides. Key = table name.
    #[serde(default)]
    pub tables: std::collections::HashMap<String, TableConfig>,
}

/// Cluster identity and topology.
#[derive(Debug, Clone, Deserialize)]
pub struct ClusterConfig {
    /// Unique identifier for this node.
    pub node_id: String,

    /// Node role: "dev" (all-in-one), "coordinator", or "storage".
    #[serde(default = "default_role")]
    pub role: String,

    /// Address this node advertises to peers.
    #[serde(default = "default_advertise_addr")]
    pub advertise_addr: String,

    /// Total number of shards. Fixed at cluster creation.
    #[serde(default = "default_num_shards")]
    pub num_shards: u32,

    /// Number of replicas per shard (including the leader).
    #[serde(default = "default_replication_factor")]
    pub replication_factor: u32,
}

/// Storage paths.
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// Root directory for all shard data.
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
}

/// Per-component memory budgets.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryConfig {
    /// MemTable flush threshold per shard (bytes). Default: 64 MB.
    #[serde(default = "default_memtable_size")]
    pub memtable_size_bytes: usize,

    /// Read buffer pool size (bytes). Default: 128 MB.
    #[serde(default = "default_read_buffer_size")]
    pub read_buffer_size_bytes: usize,

    /// Index / Bloom cache size (bytes). Default: 32 MB.
    #[serde(default = "default_index_cache_size")]
    pub index_cache_size_bytes: usize,

    /// Replication buffer size (bytes). Default: 16 MB.
    #[serde(default = "default_replication_buffer")]
    pub replication_buffer_bytes: usize,
}

/// WAL durability settings.
#[derive(Debug, Clone, Deserialize)]
pub struct DurabilityConfig {
    /// Durability level: "async", "sync", or "sync_batch".
    #[serde(default = "default_durability_level")]
    pub level: String,

    /// SyncBatch timer interval in milliseconds. Default: 10.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
}

/// Thread pool sizing.
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadConfig {
    /// Tokio async worker threads. Default: num_cpus.
    pub async_worker_threads: Option<usize>,

    /// Blocking I/O pool size. Default: num_cpus / 2.
    pub blocking_io_threads: Option<usize>,

    /// Enable background workers. Default: true.
    #[serde(default = "default_true")]
    pub background_enabled: bool,
}

/// Merge worker configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct MergeConfig {
    /// Enable automatic merging. Default: true.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Trigger merge when a partition has more Parts than this.
    #[serde(default = "default_max_parts")]
    pub max_parts_per_partition: u32,

    /// Target size for merged Part files (bytes).
    #[serde(default = "default_target_part_size")]
    pub target_part_size_bytes: u64,
}

/// Index configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct IndexConfig {
    #[serde(default)]
    pub inverted: InvertedIndexConfig,
}

/// Inverted index settings.
#[derive(Debug, Clone, Deserialize)]
pub struct InvertedIndexConfig {
    /// Enable inverted index. Default: true.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Which tag columns to index. Empty = all tag columns.
    #[serde(default)]
    pub tag_columns: Vec<String>,

    /// Skip indexing a tag key if it has more unique values than this.
    #[serde(default = "default_max_values_per_key")]
    pub max_values_per_key: usize,
}

impl Default for InvertedIndexConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tag_columns: Vec::new(),
            max_values_per_key: 10_000,
        }
    }
}

/// Per-table overrides.
#[derive(Debug, Clone, Deserialize)]
pub struct TableConfig {
    /// Time partition duration (e.g., "1h", "1d").
    pub partition_duration: Option<String>,

    /// Compression override.
    pub compression: Option<String>,

    /// Primary tag columns for shard key computation.
    pub primary_tags: Option<Vec<String>>,
}

// ── Phase 1: Distribution Configs ────────────────────────────────────

/// SWIM gossip protocol settings.
#[derive(Debug, Clone, Deserialize)]
pub struct GossipConfig {
    /// Direct probe interval (ms). Default: 1000.
    #[serde(default = "default_probe_interval")]
    pub probe_interval_ms: u64,

    /// Time before Suspect → Dead (ms). Default: 2000.
    #[serde(default = "default_suspect_timeout")]
    pub suspect_timeout_ms: u64,

    /// Number of indirect probes per round. Default: 3.
    #[serde(default = "default_fanout")]
    pub fanout: usize,

    /// Bootstrap peer addresses for cluster join.
    #[serde(default)]
    pub seed_nodes: Vec<String>,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            probe_interval_ms: 1000,
            suspect_timeout_ms: 2000,
            fanout: 3,
            seed_nodes: Vec::new(),
        }
    }
}

/// Read consistency settings.
#[derive(Debug, Clone, Deserialize)]
pub struct ConsistencyConfig {
    /// Read consistency level: "one", "quorum", or "all". Default: "one".
    #[serde(default = "default_read_level")]
    pub read_level: String,
}

impl Default for ConsistencyConfig {
    fn default() -> Self {
        Self {
            read_level: "one".to_string(),
        }
    }
}

/// Raft consensus settings for the Coordinator metadata group.
#[derive(Debug, Clone, Deserialize)]
pub struct RaftConfig {
    /// Election timeout range start (ms). Default: 1000.
    #[serde(default = "default_election_timeout")]
    pub election_timeout_ms: u64,

    /// Leader heartbeat interval (ms). Default: 250.
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_ms: u64,

    /// Number of log entries before triggering a snapshot. Default: 1000.
    #[serde(default = "default_snapshot_threshold")]
    pub snapshot_threshold: u64,

    /// Directory for Raft log and snapshot storage.
    #[serde(default = "default_raft_data_dir")]
    pub data_dir: PathBuf,
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            election_timeout_ms: 1000,
            heartbeat_interval_ms: 250,
            snapshot_threshold: 1000,
            data_dir: PathBuf::from("./raft-data"),
        }
    }
}

// ── Defaults ─────────────────────────────────────────────────────────

fn default_role() -> String { "dev".to_string() }
fn default_advertise_addr() -> String { "127.0.0.1:4000".to_string() }
fn default_num_shards() -> u32 { 8 }
fn default_replication_factor() -> u32 { 1 }
fn default_data_dir() -> PathBuf { PathBuf::from("./data") }
fn default_memtable_size() -> usize { 64 * 1024 * 1024 } // 64 MB
fn default_read_buffer_size() -> usize { 128 * 1024 * 1024 } // 128 MB
fn default_index_cache_size() -> usize { 32 * 1024 * 1024 } // 32 MB
fn default_replication_buffer() -> usize { 16 * 1024 * 1024 } // 16 MB
fn default_durability_level() -> String { "sync_batch".to_string() }
fn default_interval_ms() -> u64 { 10 }
fn default_true() -> bool { true }
fn default_max_parts() -> u32 { 8 }
fn default_target_part_size() -> u64 { 256 * 1024 * 1024 } // 256 MB
fn default_max_values_per_key() -> usize { 10_000 }

// Phase 1 defaults
fn default_probe_interval() -> u64 { 1000 }
fn default_suspect_timeout() -> u64 { 2000 }
fn default_fanout() -> usize { 3 }
fn default_read_level() -> String { "one".to_string() }
fn default_election_timeout() -> u64 { 1000 }
fn default_heartbeat_interval() -> u64 { 250 }
fn default_snapshot_threshold() -> u64 { 1000 }
fn default_raft_data_dir() -> PathBuf { PathBuf::from("./raft-data") }

// ── Loading ──────────────────────────────────────────────────────────

impl RutSeriConfig {
    /// Load configuration from a TOML file.
    ///
    /// # Errors
    /// Returns `RutSeriError::Config` if the file can't be read or parsed.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RutSeriError::Config(format!("Failed to read config: {e}")))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| RutSeriError::Config(format!("Failed to parse config: {e}")))?;

        // TODO(engineer): add semantic validation
        // - node_id is non-empty
        // - role is one of ["dev", "coordinator", "storage"]
        // - num_shards > 0
        // - data_dir exists or can be created
        // - durability.level is one of ["async", "sync", "sync_batch"]
        // - consistency.read_level is one of ["one", "quorum", "all"]
        // - raft.election_timeout_ms > raft.heartbeat_interval_ms

        Ok(config)
    }
}
