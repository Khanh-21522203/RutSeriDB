//! WAL reader — crash recovery via segment replay.
//!
//! See: docs/ingestion/wal.md § Crash Recovery
//!
//! Recovery algorithm:
//! 1. List all .rwal segment files, sorted ascending
//! 2. Find last Checkpoint record → replay_from_seq
//! 3. Re-apply all WalEntry::Write records after the checkpoint
//! 4. Truncate any partial trailing record (CRC mismatch)

use std::path::Path;

use crate::common::error::Result;

use super::entry::{WalEntry, WalRecord};

/// Reads WAL segments for crash recovery.
///
/// Stateless — create a new reader, call `replay()`, discard.
pub struct WalReader;

impl WalReader {
    /// Replay all WAL entries after the last checkpoint.
    ///
    /// Scans all `.rwal` files in `shard_wal_dir` and calls `on_entry`
    /// for each valid `WalEntry::Write` that comes after the last
    /// `WalEntry::Checkpoint`.
    ///
    /// # Arguments
    /// * `shard_wal_dir` — Path to the shard's WAL directory
    /// * `on_entry` — Callback invoked for each replayed entry.
    ///   Receives `(seq, WalEntry)`. Return `Err` to abort replay.
    ///
    /// # Returns
    /// The sequence number of the last replayed entry (or 0 if none).
    ///
    /// # Error Handling
    /// - CRC mismatch on a trailing record: silently truncate (crash mid-write)
    /// - CRC mismatch on a non-trailing record: return `WalCorruption` error
    pub fn replay<F>(shard_wal_dir: &Path, mut on_entry: F) -> Result<u64>
    where
        F: FnMut(u64, WalEntry) -> Result<()>,
    {
        // TODO(engineer): implement
        //
        // Step 1: List and sort .rwal files ascending by segment number
        // Step 2: Read each segment sequentially
        //         - deserialize WalRecord from bytes
        //         - track last Checkpoint seq
        // Step 3: Second pass (or single pass with buffering):
        //         - skip all entries ≤ checkpoint seq
        //         - call on_entry for each Write entry after checkpoint
        // Step 4: Handle partial trailing record gracefully
        //         - if last record has CRC mismatch, truncate the file
        //         - if a mid-file record has CRC mismatch, return error

        todo!("WalReader::replay")
    }
}
