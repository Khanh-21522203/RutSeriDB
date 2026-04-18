//! Table schema definitions.
//!
//! A `TableSchema` describes the structure of a time-series table:
//! which tags exist, which fields exist and their types, and which
//! tags are used for shard key computation (primary tags).

use serde::{Deserialize, Serialize};

/// The type of a column in a table.
///
/// Maps directly to the `.rpart` ColumnHeader `col_type` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    /// The timestamp column (always present, always first). col_type = 0
    Timestamp,
    /// A string tag column (low cardinality, used for filtering). col_type = 1
    Tag,
    /// A 64-bit floating point field. col_type = 2
    FieldFloat,
    /// A 64-bit signed integer field. col_type = 3
    FieldInt,
    /// A boolean field. col_type = 4
    FieldBool,
    /// A variable-length string field. col_type = 5
    FieldStr,
}

/// Definition of a single column in a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    /// Column name (e.g., "host", "cpu", "temperature").
    pub name: String,

    /// Column type.
    pub col_type: ColumnType,

    /// Whether this column can contain null values.
    pub nullable: bool,
}

/// Schema of a time-series table.
///
/// Created via the Admin API and stored in the Metadata Catalog.
/// Immutable after creation (v1 — schema evolution is out of scope).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    /// Table name (unique within the database).
    pub name: String,

    /// Ordered list of columns. The first column is always `timestamp`.
    pub columns: Vec<ColumnDef>,

    /// Names of tag columns used for shard key computation.
    /// These tags MUST be present in every write request.
    /// Sorted alphabetically for deterministic hashing.
    pub primary_tags: Vec<String>,

    /// Time-based partitioning duration (e.g., "1h" for hourly).
    /// Used by the query planner for time-range pruning.
    pub partition_duration: String,

    /// Compression algorithm for this table's Part files.
    pub compression: CompressionType,
}

/// Supported compression algorithms for `.rpart` column blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Lz4,
    Zstd,
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::Lz4
    }
}

impl TableSchema {
    /// Returns the column definition for a given column name, if it exists.
    pub fn column(&self, name: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Returns all tag column definitions.
    pub fn tag_columns(&self) -> Vec<&ColumnDef> {
        self.columns
            .iter()
            .filter(|c| c.col_type == ColumnType::Tag)
            .collect()
    }

    /// Returns all field column definitions (non-tag, non-timestamp).
    pub fn field_columns(&self) -> Vec<&ColumnDef> {
        self.columns
            .iter()
            .filter(|c| {
                !matches!(c.col_type, ColumnType::Timestamp | ColumnType::Tag)
            })
            .collect()
    }
}
