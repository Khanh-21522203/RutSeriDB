//! Bloom Filters — per-column equality predicate pruning.
//!
//! See: docs/storage/indexes.md § Bloom Filters
//!
//! Algorithm: Blocked Bloom Filter (cache-line friendly).
//! Scope: one filter per tag column + per low-cardinality field column.
//! False positive rate: ≤ 1% (configured).
//!
//! Bloom Filters only help equality predicates (WHERE col = value).
//! Range predicates use the MinMax Index instead.

use crate::common::error::Result;

/// A single Bloom filter for one column.
#[derive(Debug, Clone)]
pub struct BloomFilter {
    /// Column name this filter covers.
    pub col_name: String,

    /// The bit array. Size determined by expected element count and FPR.
    bits: Vec<u8>,

    /// Number of hash functions (k).
    num_hashes: u32,
}

impl BloomFilter {
    /// Create a new Bloom filter for a column.
    ///
    /// # Arguments
    /// * `col_name` — Column name
    /// * `expected_items` — Expected number of unique values
    /// * `false_positive_rate` — Target FPR (e.g., 0.01 for 1%)
    pub fn new(col_name: String, expected_items: usize, false_positive_rate: f64) -> Self {
        // TODO(engineer): implement
        // Calculate optimal bit count: m = -n * ln(p) / (ln(2))^2
        // Calculate optimal hash count: k = (m/n) * ln(2)
        // Allocate bit array
        todo!("BloomFilter::new")
    }

    /// Insert a value into the Bloom filter.
    ///
    /// Called during Part flush for each unique value in the column.
    pub fn insert(&mut self, value: &[u8]) {
        // TODO(engineer): implement
        // Hash the value with `num_hashes` different hash functions
        // (use double hashing: h(i) = h1 + i * h2)
        // Set corresponding bits
        todo!("BloomFilter::insert")
    }

    /// Check if a value may be in the set.
    ///
    /// Returns `false` if the value is DEFINITELY NOT present (skip the Part).
    /// Returns `true` if the value MIGHT be present (read the Part).
    pub fn may_contain(&self, value: &[u8]) -> bool {
        // TODO(engineer): implement
        // Hash with same functions as insert
        // Return false if any bit is not set
        todo!("BloomFilter::may_contain")
    }

    /// Serialize the filter to bytes for writing into `.rpart`.
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO(engineer): implement
        todo!("BloomFilter::to_bytes")
    }

    /// Deserialize from bytes.
    pub fn from_bytes(col_name: String, data: &[u8]) -> Result<Self> {
        // TODO(engineer): implement
        todo!("BloomFilter::from_bytes")
    }
}

/// A collection of Bloom filters for all indexed columns in a Part.
#[derive(Debug, Clone)]
pub struct BloomFilterSet {
    pub filters: Vec<BloomFilter>,
}

impl BloomFilterSet {
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }

    /// Add a filter for a column.
    pub fn add(&mut self, filter: BloomFilter) {
        self.filters.push(filter);
    }

    /// Find the filter for a specific column. Returns `None` if not indexed.
    pub fn get(&self, col_name: &str) -> Option<&BloomFilter> {
        self.filters.iter().find(|f| f.col_name == col_name)
    }

    /// Serialize all filters.
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO(engineer): implement (with a header listing filter offsets)
        todo!("BloomFilterSet::to_bytes")
    }

    /// Deserialize.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // TODO(engineer): implement
        todo!("BloomFilterSet::from_bytes")
    }
}
