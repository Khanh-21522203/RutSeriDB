//! `.rpart` file format constants and structures.
//!
//! See: docs/storage/format.md
//!
//! Overall file structure:
//! ```text
//! [FileHeader (64B)] [ColBlock 0] [ColBlock 1..N] [MinMax Index]
//! [Dictionary Pages] [Bloom Filters] [Footer (32B)]
//! ```

/// Magic bytes at the start and in the footer of every `.rpart` file.
pub const PART_MAGIC: &[u8; 4] = b"RPRT";

/// Current format version.
pub const FORMAT_VERSION: u16 = 1;

/// File header size in bytes.
pub const FILE_HEADER_SIZE: usize = 64;

/// Footer size in bytes (always at EOF - 32).
pub const FOOTER_SIZE: usize = 32;

/// Flag: bloom filter section is present.
pub const FLAG_BLOOM_PRESENT: u16 = 0x01;

/// Column type identifiers (matches ColumnType enum in common::schema).
pub mod col_type {
    pub const TIMESTAMP: u8 = 0;
    pub const TAG_STR: u8 = 1;
    pub const FIELD_FLOAT: u8 = 2;
    pub const FIELD_INT: u8 = 3;
    pub const FIELD_BOOL: u8 = 4;
    pub const FIELD_STR: u8 = 5;
}

/// Encoding identifiers for column blocks.
pub mod encoding_type {
    pub const RAW: u8 = 0;
    pub const DELTA_I64: u8 = 1;
    pub const DELTA_DELTA_I64: u8 = 2;
    pub const GORILLA_DELTA: u8 = 3;
    pub const DICTIONARY: u8 = 4;
}

/// Compression identifiers for column blocks.
pub mod compression_type {
    pub const NONE: u8 = 0;
    pub const LZ4: u8 = 1;
    pub const ZSTD: u8 = 2;
}

/// The 64-byte file header at the start of every `.rpart` file.
#[derive(Debug, Clone)]
pub struct FileHeader {
    /// Format version (currently 1).
    pub version: u16,
    /// Bitfield flags (e.g., FLAG_BLOOM_PRESENT).
    pub flags: u16,
    /// Total number of columns (timestamp + tags + fields).
    pub num_columns: u32,
    /// Total number of rows.
    pub num_rows: u64,
    /// Minimum timestamp in the file (nanoseconds).
    pub min_timestamp: i64,
    /// Maximum timestamp in the file (nanoseconds).
    pub max_timestamp: i64,
    /// Part UUID.
    pub part_id: uuid::Uuid,
    /// Creation time (Unix seconds).
    pub created_at: i64,
}

/// Per-column metadata within the file.
#[derive(Debug, Clone)]
pub struct ColumnHeader {
    /// Column name.
    pub name: String,
    /// Column type (see `col_type` constants).
    pub col_type: u8,
    /// Encoding type (see `encoding_type` constants).
    pub encoding: u8,
    /// Compression type (see `compression_type` constants).
    pub compression: u8,
    /// Byte offset of compressed data from file start.
    pub data_offset: u64,
    /// Compressed byte length.
    pub data_len: u32,
    /// Uncompressed byte length.
    pub uncompressed_len: u32,
    /// Offset of null bitmap (0 if no nulls).
    pub null_bitmap_offset: u64,
    /// Length of null bitmap.
    pub null_bitmap_len: u32,
}

/// The 32-byte footer at EOF - 32.
#[derive(Debug, Clone)]
pub struct Footer {
    /// Byte offset of the MinMax Index section.
    pub minmax_index_offset: u64,
    /// Byte offset of the Bloom Filter section (0 if absent).
    pub bloom_offset: u64,
    /// Byte offset of the Dictionary Pages section.
    pub dict_pages_offset: u64,
    /// CRC32 of all bytes in the file except the last 4.
    pub file_crc32: u32,
}

impl FileHeader {
    /// Serialize the header to a 64-byte buffer.
    pub fn to_bytes(&self) -> [u8; FILE_HEADER_SIZE] {
        // TODO(engineer): implement binary serialization
        // Layout: magic(4) + version(2) + flags(2) + num_columns(4)
        //       + num_rows(8) + min_ts(8) + max_ts(8) + part_id(16)
        //       + created_at(8) + reserved(4) = 64 bytes
        todo!("FileHeader::to_bytes")
    }

    /// Deserialize from a 64-byte buffer.
    pub fn from_bytes(buf: &[u8; FILE_HEADER_SIZE]) -> crate::common::error::Result<Self> {
        // TODO(engineer): implement binary deserialization
        // Verify magic bytes, check version, extract fields
        todo!("FileHeader::from_bytes")
    }
}

impl Footer {
    /// Serialize the footer to a 32-byte buffer.
    pub fn to_bytes(&self) -> [u8; FOOTER_SIZE] {
        // TODO(engineer): implement
        // Layout: magic(4) + minmax_offset(8) + bloom_offset(8)
        //       + dict_offset(8) + file_crc32(4) = 32 bytes
        todo!("Footer::to_bytes")
    }

    /// Deserialize from a 32-byte buffer.
    pub fn from_bytes(buf: &[u8; FOOTER_SIZE]) -> crate::common::error::Result<Self> {
        // TODO(engineer): implement
        todo!("Footer::from_bytes")
    }
}
