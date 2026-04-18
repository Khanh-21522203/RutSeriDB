//! Client-side response types.
//!
//! These types deserialize the JSON responses from the RutSeriDB server.
//! They mirror the server-side `QueryResponse` but are owned by the client
//! module to avoid coupling to server internals.

use serde::{Deserialize, Serialize};

/// Result of a query execution.
///
/// Returned by `QueryBuilder::execute()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// JSON-serialized rows from the server.
    pub rows: Vec<serde_json::Value>,

    /// Total number of rows returned.
    pub row_count: usize,
}

impl QueryResult {
    /// Whether the result set is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Number of rows.
    pub fn len(&self) -> usize {
        self.row_count
    }

    /// Iterate over rows as `ResultRow` wrappers for convenient field access.
    pub fn iter(&self) -> impl Iterator<Item = ResultRow<'_>> {
        self.rows.iter().map(ResultRow)
    }

    /// Get a specific row by index.
    pub fn get(&self, index: usize) -> Option<ResultRow<'_>> {
        self.rows.get(index).map(ResultRow)
    }

    /// Collect all values of a specific column as f64.
    ///
    /// Useful for plotting or calculations.
    pub fn column_f64(&self, name: &str) -> Vec<Option<f64>> {
        self.rows
            .iter()
            .map(|row| row.get(name).and_then(|v| v.as_f64()))
            .collect()
    }

    /// Collect all values of a specific column as i64.
    pub fn column_i64(&self, name: &str) -> Vec<Option<i64>> {
        self.rows
            .iter()
            .map(|row| row.get(name).and_then(|v| v.as_i64()))
            .collect()
    }

    /// Collect all values of a specific column as strings.
    pub fn column_str(&self, name: &str) -> Vec<Option<&str>> {
        self.rows
            .iter()
            .map(|row| row.get(name).and_then(|v| v.as_str()))
            .collect()
    }
}

/// A lightweight wrapper around a single JSON row for field access.
///
/// Provides typed getters without requiring deserialization into a struct.
///
/// # Example
/// ```rust,no_run
/// for row in result.iter() {
///     let ts = row.timestamp().unwrap();
///     let cpu = row.field_f64("cpu_usage").unwrap_or(0.0);
///     let host = row.tag("host").unwrap_or("unknown");
///     println!("{ts}: {host} cpu={cpu}");
/// }
/// ```
pub struct ResultRow<'a>(pub(crate) &'a serde_json::Value);

impl<'a> ResultRow<'a> {
    /// Get the timestamp (nanoseconds since epoch).
    pub fn timestamp(&self) -> Option<i64> {
        self.0.get("timestamp").and_then(|v| v.as_i64())
    }

    /// Get a tag value by key.
    pub fn tag(&self, key: &str) -> Option<&str> {
        self.0
            .get("tags")
            .and_then(|t| t.get(key))
            .and_then(|v| v.as_str())
    }

    /// Get a float field value.
    pub fn field_f64(&self, name: &str) -> Option<f64> {
        self.0
            .get("fields")
            .and_then(|f| f.get(name))
            .and_then(|v| v.as_f64())
    }

    /// Get an integer field value.
    pub fn field_i64(&self, name: &str) -> Option<i64> {
        self.0
            .get("fields")
            .and_then(|f| f.get(name))
            .and_then(|v| v.as_i64())
    }

    /// Get a boolean field value.
    pub fn field_bool(&self, name: &str) -> Option<bool> {
        self.0
            .get("fields")
            .and_then(|f| f.get(name))
            .and_then(|v| v.as_bool())
    }

    /// Get a string field value.
    pub fn field_str(&self, name: &str) -> Option<&str> {
        self.0
            .get("fields")
            .and_then(|f| f.get(name))
            .and_then(|v| v.as_str())
    }

    /// Get any value by top-level key (raw JSON access).
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    /// Access the underlying JSON value.
    pub fn as_json(&self) -> &serde_json::Value {
        self.0
    }
}
