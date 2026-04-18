//! Raft Log Storage — persistent log for openraft.
//!
//! Stores Raft log entries on disk. Each entry contains a serialized
//! `MetadataOp`. Supports append, truncate, and snapshot compaction.
//!
//! See: docs/phase1_plan.md § raft/log

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::common::error::{Result, RutSeriError};
use crate::coordinator::metadata_catalog::MetadataOp;

/// A single entry in the Raft log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftLogEntry {
    /// Raft term number.
    pub term: u64,

    /// Log index (monotonically increasing).
    pub index: u64,

    /// The metadata operation to apply.
    pub op: MetadataOp,
}

/// Persistent Raft log storage.
///
/// Provides the storage backend for `openraft::RaftLogStorage`.
/// Entries are serialized as JSON lines for simplicity (v1).
pub struct RaftLogStore {
    /// Directory where log files are stored.
    data_dir: PathBuf,

    /// In-memory log entries (loaded at startup, appended at runtime).
    entries: Vec<RaftLogEntry>,

    /// Index of the last compacted entry (entries before this are snapshotted).
    compacted_index: u64,
}

impl RaftLogStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            entries: Vec::new(),
            compacted_index: 0,
        }
    }

    /// Load log entries from disk at startup.
    pub fn load(&mut self) -> Result<()> {
        // TODO(engineer): implement log loading
        // Read JSON lines from data_dir/raft.log
        // Populate self.entries
        todo!("TODO(engineer): implement RaftLogStore.load")
    }

    /// Append entries to the log.
    pub fn append(&mut self, entries: Vec<RaftLogEntry>) -> Result<()> {
        // TODO(engineer): implement log append
        // 1. Serialize entries as JSON lines
        // 2. Append to data_dir/raft.log
        // 3. fsync
        // 4. Add to in-memory entries
        todo!("TODO(engineer): implement RaftLogStore.append")
    }

    /// Truncate the log at the given index (exclusive).
    ///
    /// All entries with index >= `from_index` are removed.
    pub fn truncate(&mut self, _from_index: u64) -> Result<()> {
        // TODO(engineer): implement log truncation
        todo!("TODO(engineer): implement RaftLogStore.truncate")
    }

    /// Get log entries in the range [start, end).
    pub fn get_entries(&self, start: u64, end: u64) -> Vec<&RaftLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.index >= start && e.index < end)
            .collect()
    }

    /// Get the last log entry.
    pub fn last_entry(&self) -> Option<&RaftLogEntry> {
        self.entries.last()
    }

    /// Compact the log up to the given index.
    ///
    /// Entries with index <= `up_to` can be removed because they're
    /// included in the latest snapshot.
    pub fn compact(&mut self, up_to: u64) -> Result<()> {
        // TODO(engineer): implement log compaction
        // Remove entries with index <= up_to
        // Update compacted_index
        // Rewrite log file
        todo!("TODO(engineer): implement RaftLogStore.compact")
    }
}
