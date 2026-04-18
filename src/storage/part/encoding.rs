//! Column encoding and decoding for `.rpart` files.
//!
//! See: docs/storage/format.md § Column Encodings
//!
//! Supported encodings:
//! - Delta i64 (timestamps)
//! - Delta-of-delta i64 (integer counters)
//! - Gorilla XOR (IEEE 754 floats)
//! - Dictionary (low-cardinality strings → u16 codes)
//!
//! All encodings produce raw bytes that are then LZ4-compressed
//! before writing to the column block.

use crate::common::error::Result;

// ── Delta Encoding (i64) ─────────────────────────────────────────────

/// Encode a slice of i64 values using delta encoding.
///
/// First value is absolute; subsequent values are stored as
/// differences from the previous value.
///
/// ```text
/// Input:  [1700000000, 1700000010, 1700000020]
/// Output: [1700000000, 10, 10]
/// ```
pub fn delta_encode_i64(values: &[i64]) -> Vec<i64> {
    // TODO(engineer): implement
    todo!("delta_encode_i64")
}

/// Decode delta-encoded i64 values back to absolute values.
pub fn delta_decode_i64(deltas: &[i64]) -> Vec<i64> {
    // TODO(engineer): implement (inverse of delta_encode_i64)
    todo!("delta_decode_i64")
}

// ── Delta-of-Delta Encoding (i64) ────────────────────────────────────

/// Encode using delta-of-delta for monotonically increasing counters.
///
/// Useful when differences are nearly constant (e.g., +10, +10, +10).
/// The delta-of-delta would be [first, first_delta, 0, 0, 0].
pub fn delta_delta_encode_i64(values: &[i64]) -> Vec<i64> {
    // TODO(engineer): implement
    todo!("delta_delta_encode_i64")
}

/// Decode delta-of-delta encoded values.
pub fn delta_delta_decode_i64(encoded: &[i64]) -> Vec<i64> {
    // TODO(engineer): implement
    todo!("delta_delta_decode_i64")
}

// ── Gorilla XOR Encoding (f64) ───────────────────────────────────────

/// Encode f64 values using Gorilla XOR encoding.
///
/// XOR of consecutive IEEE 754 bits. Consecutive time-series floats
/// often have many common leading/trailing zero bits, yielding
/// excellent compression after LZ4.
///
/// Reference: Facebook Gorilla paper (2015), §4.1.2
pub fn gorilla_encode_f64(values: &[f64]) -> Vec<u8> {
    // TODO(engineer): implement
    // For each pair of consecutive values:
    // 1. XOR their raw u64 bits
    // 2. If XOR == 0, emit a 0 bit
    // 3. Otherwise, emit leading zeros count + meaningful bits
    todo!("gorilla_encode_f64")
}

/// Decode Gorilla XOR encoded f64 values.
pub fn gorilla_decode_f64(data: &[u8], count: usize) -> Vec<f64> {
    // TODO(engineer): implement (inverse of gorilla_encode_f64)
    todo!("gorilla_decode_f64")
}

// ── Dictionary Encoding (strings) ────────────────────────────────────

/// A dictionary page for low-cardinality string columns.
///
/// Maps string values to u16 codes. Max 65,535 unique values per Part.
#[derive(Debug, Clone)]
pub struct Dictionary {
    /// Index → string value.
    pub values: Vec<String>,
    /// String value → index.
    pub index: std::collections::HashMap<String, u16>,
}

impl Dictionary {
    /// Build a dictionary from a list of string values.
    ///
    /// Returns the dictionary and the encoded u16 codes.
    pub fn encode(values: &[String]) -> Result<(Self, Vec<u16>)> {
        // TODO(engineer): implement
        // 1. Collect unique values
        // 2. Assign u16 codes (0, 1, 2, ...)
        // 3. Map each input value to its code
        // 4. Error if > 65535 unique values
        todo!("Dictionary::encode")
    }

    /// Decode u16 codes back to string values using this dictionary.
    pub fn decode(&self, codes: &[u16]) -> Vec<String> {
        // TODO(engineer): implement
        todo!("Dictionary::decode")
    }
}

// ── LZ4 Compression Wrapper ─────────────────────────────────────────

/// Compress data using LZ4.
pub fn lz4_compress(data: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(data)
}

/// Decompress LZ4 data.
pub fn lz4_decompress(data: &[u8]) -> Result<Vec<u8>> {
    lz4_flex::decompress_size_prepended(data)
        .map_err(|e| crate::common::error::RutSeriError::InvalidPartFile(
            format!("LZ4 decompression failed: {e}")
        ))
}
