//! WAL entry types and record framing.
//!
//! See: docs/ingestion/wal.md § WAL Record Format
//!
//! Physical record layout (per entry):
//! ```text
//! | Magic (4B) | Seq (8B) | Len (4B) | Payload (variable) | CRC32 (4B) |
//! ```
//!
//! CRC32 covers `[Seq ‖ Len ‖ Payload]`.

use serde::{Deserialize, Serialize};

use crate::common::types::Row;

/// Magic bytes at the start of every WAL record.
pub const WAL_MAGIC: &[u8; 4] = b"RWAL";

/// Fixed overhead per WAL record: magic(4) + seq(8) + len(4) + crc(4) = 20 bytes.
pub const WAL_RECORD_OVERHEAD: usize = 20;

/// A logical WAL entry — the payload inside a WAL record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalEntry {
    /// A batch of rows to insert into a table.
    Write {
        table: String,
        rows: Vec<Row>,
    },

    /// Checkpoint marker: all entries ≤ `seq` are safely flushed to Part files.
    Checkpoint {
        /// The sequence number up to which data is flushed.
        seq: u64,
        /// The catalog version at the time of checkpoint.
        catalog_ver: u64,
    },
}

/// A framed WAL record as it appears on disk.
///
/// Engineers implementing the WAL writer/reader should serialize/deserialize
/// this struct to/from bytes.
#[derive(Debug)]
pub struct WalRecord {
    /// Monotonically increasing sequence number (per shard, never resets).
    pub seq: u64,

    /// The logical entry payload.
    pub entry: WalEntry,
}

impl WalRecord {
    /// Serialize this record to bytes (for writing to a WAL segment).
    ///
    /// Format: `RWAL` | seq (8B LE) | len (4B LE) | payload | crc32 (4B LE)
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO(engineer): implement serialization
        // 1. Serialize `self.entry` to bytes (e.g., using bincode or serde_json)
        // 2. Build the framed record: magic + seq + len + payload + crc32
        // 3. CRC32 covers [seq ‖ len ‖ payload]
        todo!("WalRecord::to_bytes")
    }

    /// Deserialize a record from bytes (for reading during replay).
    ///
    /// Returns `None` if the buffer is too short or the magic doesn't match.
    /// Returns `Err` if CRC verification fails.
    pub fn from_bytes(buf: &[u8]) -> crate::common::error::Result<Option<(Self, usize)>> {
        // TODO(engineer): implement deserialization
        // 1. Check magic bytes
        // 2. Read seq, len
        // 3. Read payload, compute CRC, verify
        // 4. Deserialize WalEntry from payload
        // 5. Return (record, total_bytes_consumed)
        todo!("WalRecord::from_bytes")
    }
}
