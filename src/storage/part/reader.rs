//! Part reader — reads columnar data from `.rpart` files.
//!
//! See: docs/components.md § Part Writer / Reader
//! See: docs/storage/format.md § Read Flow
//!
//! Read flow:
//! 1. Open file, seek to EOF-32, read Footer
//! 2. MinMax Index check → skip file if range predicates fail
//! 3. Bloom Filter check → skip file if equality predicates fail
//! 4. Read only requested columns (projection pushdown)
//! 5. Decompress + decode columns
//! 6. Apply row-level filter predicates
//! 7. Return matching rows
//!
//! Does NOT:
//! - Manage the Catalog
//! - Decide which files to read (the QueryPlanner decides)

use std::path::Path;

use crate::common::error::Result;
use crate::common::types::Row;
use crate::storage::index::bloom::BloomFilterSet;
use crate::storage::index::minmax::MinMaxIndex;

/// A predicate that can be pushed down to the Part reader.
///
/// The query planner translates SQL WHERE clauses into these.
#[derive(Debug, Clone)]
pub enum Predicate {
    /// `column > value` or `column >= value`
    GreaterThan { column: String, value: PredicateValue, inclusive: bool },
    /// `column < value` or `column <= value`
    LessThan { column: String, value: PredicateValue, inclusive: bool },
    /// `column = value`
    Equals { column: String, value: PredicateValue },
    /// `column BETWEEN low AND high`
    Between { column: String, low: PredicateValue, high: PredicateValue },
}

/// A typed value used in predicates.
#[derive(Debug, Clone)]
pub enum PredicateValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

/// Reads columnar data from `.rpart` files with projection
/// and predicate pushdown.
pub struct PartReader;

impl PartReader {
    /// Read projected columns from a Part file, applying predicates.
    ///
    /// # Arguments
    /// * `part_path` — Absolute path to the `.rpart` file
    /// * `projection` — Column names to read (empty = all columns)
    /// * `predicates` — Filters to push down for early elimination
    ///
    /// # Returns
    /// Rows that survive all filters, containing only projected columns.
    pub fn read(
        part_path: &Path,
        projection: &[String],
        predicates: &[Predicate],
    ) -> Result<Vec<Row>> {
        // TODO(engineer): implement
        //
        // Step 1: Read Footer from EOF-32
        //   let footer = Footer::from_bytes(...)?;
        //
        // Step 2: Read and check MinMax Index
        //   let minmax = self.read_minmax_from_offset(footer.minmax_index_offset)?;
        //   for predicate in predicates:
        //     if minmax rules out the file → return Ok(vec![])
        //
        // Step 3: Read and check Bloom Filters (if present)
        //   if footer.bloom_offset != 0:
        //     let blooms = self.read_blooms(footer.bloom_offset)?;
        //     for equality predicate in predicates:
        //       if bloom.definitely_missing(value) → return Ok(vec![])
        //
        // Step 4: Read FileHeader to get column layout
        //
        // Step 5: Read only projected ColumnHeaders
        //   Find data_offset + data_len for each projected column
        //
        // Step 6: Read + decompress + decode each column block
        //   Use encoding::lz4_decompress, then delta_decode / gorilla_decode / etc.
        //
        // Step 7: Reconstruct rows from columnar data
        //
        // Step 8: Apply row-level predicates
        //
        // Step 9: Return surviving rows

        todo!("PartReader::read")
    }

    /// Read only the MinMax index from a Part file (no column I/O).
    ///
    /// Used by the query planner to prune Parts before reading columns.
    pub fn read_minmax(part_path: &Path) -> Result<MinMaxIndex> {
        // TODO(engineer): implement
        // - Read footer to get minmax_index_offset
        // - Seek to that offset, read the index section
        todo!("PartReader::read_minmax")
    }

    /// Read Bloom filters from a Part file.
    ///
    /// Returns `None` if the Part has no bloom filters (FLAG_BLOOM_PRESENT not set).
    pub fn read_bloom(part_path: &Path) -> Result<Option<BloomFilterSet>> {
        // TODO(engineer): implement
        // - Read footer to get bloom_offset
        // - If bloom_offset == 0, return None
        // - Otherwise, seek and read
        todo!("PartReader::read_bloom")
    }
}
