//! WAL writer — append-only durable log for a single shard.
//!
//! See: docs/ingestion/wal.md
//! See: docs/architecture.md § Durability & Recovery
//!
//! Responsibilities:
//! - Append WalEntry records with framing (magic, seq, CRC)
//! - fsync per configured durability level
//! - Rotate segments when `max_segment_bytes` is exceeded
//! - Write Checkpoint entries to mark flushed data
//!
//! Does NOT:
//! - Know about MemTable, Catalog, or Part files
//! - Decide when to flush (caller decides)

use std::path::PathBuf;

use crate::common::error::{Result, RutSeriError};

use super::entry::{WalEntry, WalRecord};

/// Durability level for WAL writes, parsed from config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityLevel {
    /// No fsync — OS flushes on its own schedule.
    Async,
    /// fsync after every append call.
    Sync,
    /// fsync on a background timer (default: every 10ms).
    SyncBatch,
}

/// Append-only WAL writer for a single shard.
///
/// Each shard has exactly one WalWriter. It manages a directory of
/// numbered segment files (e.g., `00000001.rwal`, `00000002.rwal`).
pub struct WalWriter {
    /// Directory containing WAL segment files for this shard.
    shard_wal_dir: PathBuf,

    /// Current active segment file handle.
    /// TODO(engineer): use `std::fs::File` or `tokio::fs::File`
    active_segment: Option<std::fs::File>,

    /// Current segment number (monotonically increasing).
    current_segment_num: u64,

    /// Next sequence number to assign.
    next_seq: u64,

    /// Bytes written to the active segment.
    active_segment_bytes: u64,

    /// Maximum segment size before rotation.
    max_segment_bytes: u64,

    /// Durability level.
    durability: DurabilityLevel,
}

impl WalWriter {
    /// Create a new WalWriter for a shard.
    ///
    /// # Arguments
    /// * `shard_wal_dir` — Path to the shard's WAL directory (e.g., `/data/shard-0/wal/`)
    /// * `durability` — Durability level from config
    /// * `max_segment_bytes` — Maximum segment file size before rotation
    ///
    /// The directory will be created if it doesn't exist.
    pub fn new(
        shard_wal_dir: PathBuf,
        durability: DurabilityLevel,
        max_segment_bytes: u64,
    ) -> Result<Self> {
        // TODO(engineer): create directory, find latest segment, open it
        todo!("WalWriter::new")
    }

    /// Append a WAL entry. Returns the assigned sequence number.
    ///
    /// Steps:
    /// 1. Assign next sequence number
    /// 2. Serialize the entry into a framed WalRecord
    /// 3. Write to the active segment
    /// 4. If durability = Sync, fsync immediately
    /// 5. If active segment exceeds max_segment_bytes, rotate
    pub fn append(&mut self, entry: &WalEntry) -> Result<u64> {
        // TODO(engineer): implement
        // - Build WalRecord { seq: self.next_seq, entry: entry.clone() }
        // - Write record.to_bytes() to active_segment
        // - Handle rotation if needed
        // - fsync if Sync mode
        // - Increment self.next_seq
        todo!("WalWriter::append")
    }

    /// Force durable persistence of all buffered entries.
    ///
    /// Called by the ShardActor after draining the dispatch queue
    /// (group commit — one fsync covers N clients).
    pub fn fsync(&mut self) -> Result<()> {
        // TODO(engineer): call fsync/fdatasync on the active segment fd
        todo!("WalWriter::fsync")
    }

    /// Write a Checkpoint entry marking that all entries ≤ `seq`
    /// are safely flushed to Part files.
    pub fn checkpoint(&mut self, seq: u64, catalog_ver: u64) -> Result<()> {
        let entry = WalEntry::Checkpoint { seq, catalog_ver };
        self.append(&entry)?;
        self.fsync()?;
        Ok(())
    }

    /// Returns the current (latest written) sequence number.
    pub fn current_seq(&self) -> u64 {
        self.next_seq.saturating_sub(1)
    }

    /// Rotate to a new segment file.
    ///
    /// Seals the current segment and opens a new one with the next
    /// segment number.
    fn rotate_segment(&mut self) -> Result<()> {
        // TODO(engineer): implement
        // 1. fsync current segment
        // 2. Close current file
        // 3. Increment segment number
        // 4. Open new file
        // 5. Reset active_segment_bytes
        todo!("WalWriter::rotate_segment")
    }
}
