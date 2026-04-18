//! WAL Cleanup Worker — deletes fully checkpointed WAL segments.
//!
//! See: docs/ingestion/wal.md § Replication and WAL Cleanup
//!
//! Cleanup rule (Phase 0 — no replication):
//! A segment is deleted when all its entries have been checkpointed
//! (flushed to Part files).
//!
//! Phase 1 adds: AND all replicas have ACK'd past the segment.

use std::path::Path;

use crate::common::error::Result;

/// Run the WAL cleanup worker loop.
///
/// Periodically scans the WAL directory for segments that are
/// fully checkpointed and safe to delete.
pub async fn run_wal_cleanup(
    shard_wal_dir: &Path,
    last_checkpoint_seq: u64,
) -> Result<()> {
    // TODO(engineer): implement
    //
    // loop {
    //   sleep(cleanup_interval).await;
    //
    //   // List all .rwal files in shard_wal_dir, sorted ascending
    //   // For each segment:
    //   //   Read the last seq in the segment
    //   //   If last_seq <= last_checkpoint_seq:
    //   //     Delete the segment file
    //   //     Log: "Deleted WAL segment {filename}"
    // }

    todo!("run_wal_cleanup")
}
