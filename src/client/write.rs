//! Write builders — fluent API for inserting time-series data.
//!
//! `WriteBuilder` constructs a single row; `BatchBuilder` collects
//! multiple rows. Both serialize to `IngestBatch` JSON and POST
//! to `/api/v1/write`.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{FieldValue, IngestBatch, Row, TagSet, Timestamp};

use super::RutSeriClient;

// ── WriteBuilder (single row) ───────────────────────────────────────

/// Builder for a single time-series write.
///
/// # Example
/// ```rust,no_run
/// client.write("metrics")
///     .tag("host", "web-01")
///     .tag("region", "us-east")
///     .field("cpu_usage", 90.5)
///     .field_int("memory_mb", 4096)
///     .timestamp_now()
///     .send().await?;
/// ```
pub struct WriteBuilder<'a> {
    client: &'a RutSeriClient,
    table: String,
    tags: TagSet,
    fields: BTreeMap<String, FieldValue>,
    timestamp: Option<Timestamp>,
}

impl<'a> WriteBuilder<'a> {
    pub(crate) fn new(client: &'a RutSeriClient, table: String) -> Self {
        Self {
            client,
            table,
            tags: BTreeMap::new(),
            fields: BTreeMap::new(),
            timestamp: None,
        }
    }

    /// Add a tag key-value pair.
    pub fn tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a float field.
    pub fn field(mut self, name: &str, value: f64) -> Self {
        self.fields
            .insert(name.to_string(), FieldValue::Float(value));
        self
    }

    /// Add an integer field.
    pub fn field_int(mut self, name: &str, value: i64) -> Self {
        self.fields
            .insert(name.to_string(), FieldValue::Int(value));
        self
    }

    /// Add a boolean field.
    pub fn field_bool(mut self, name: &str, value: bool) -> Self {
        self.fields
            .insert(name.to_string(), FieldValue::Bool(value));
        self
    }

    /// Add a string field.
    pub fn field_str(mut self, name: &str, value: &str) -> Self {
        self.fields
            .insert(name.to_string(), FieldValue::Str(value.to_string()));
        self
    }

    /// Set an explicit timestamp (nanoseconds since Unix epoch).
    pub fn timestamp(mut self, ns: Timestamp) -> Self {
        self.timestamp = Some(ns);
        self
    }

    /// Use the current system time as the timestamp.
    pub fn timestamp_now(mut self) -> Self {
        let ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        self.timestamp = Some(ns);
        self
    }

    /// Send the write to the server.
    ///
    /// Returns `Ok(())` on success, or an error with the server's response.
    pub async fn send(self) -> Result<()> {
        let timestamp = self.timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64
        });

        if self.fields.is_empty() {
            return Err(RutSeriError::Ingest(
                "Write must have at least one field".to_string(),
            ));
        }

        let row = Row {
            timestamp,
            tags: self.tags,
            fields: self.fields,
        };

        let batch = IngestBatch {
            table: self.table,
            rows: vec![row],
        };

        send_batch(self.client, &batch).await
    }
}

// ── BatchBuilder (multiple rows) ────────────────────────────────────

/// Builder for sending multiple rows in a single HTTP request.
///
/// # Example
/// ```rust,no_run
/// client.batch("metrics")
///     .row(|r| r.tag("host", "web-01").field("cpu", 90.5).timestamp_now())
///     .row(|r| r.tag("host", "web-02").field("cpu", 45.2).timestamp_now())
///     .send().await?;
/// ```
pub struct BatchBuilder<'a> {
    client: &'a RutSeriClient,
    table: String,
    rows: Vec<Row>,
}

impl<'a> BatchBuilder<'a> {
    pub(crate) fn new(client: &'a RutSeriClient, table: String) -> Self {
        Self {
            client,
            table,
            rows: Vec::new(),
        }
    }

    /// Add a row using a closure that receives a `RowBuilder`.
    pub fn row<F>(mut self, f: F) -> Self
    where
        F: FnOnce(RowBuilder) -> RowBuilder,
    {
        let builder = f(RowBuilder::new());
        self.rows.push(builder.build());
        self
    }

    /// Add a pre-built `Row` directly.
    pub fn add_row(mut self, row: Row) -> Self {
        self.rows.push(row);
        self
    }

    /// Number of rows currently in the batch.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Send the entire batch to the server in one HTTP request.
    pub async fn send(self) -> Result<()> {
        if self.rows.is_empty() {
            return Err(RutSeriError::Ingest(
                "Batch must have at least one row".to_string(),
            ));
        }

        let batch = IngestBatch {
            table: self.table,
            rows: self.rows,
        };

        send_batch(self.client, &batch).await
    }
}

// ── RowBuilder (used inside BatchBuilder closures) ──────────────────

/// Builder for a single row inside a `BatchBuilder` closure.
///
/// Used with `batch.row(|r| r.tag("host", "web-01").field("cpu", 90.5))`.
pub struct RowBuilder {
    tags: TagSet,
    fields: BTreeMap<String, FieldValue>,
    timestamp: Option<Timestamp>,
}

impl RowBuilder {
    fn new() -> Self {
        Self {
            tags: BTreeMap::new(),
            fields: BTreeMap::new(),
            timestamp: None,
        }
    }

    /// Add a tag.
    pub fn tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a float field.
    pub fn field(mut self, name: &str, value: f64) -> Self {
        self.fields
            .insert(name.to_string(), FieldValue::Float(value));
        self
    }

    /// Add an integer field.
    pub fn field_int(mut self, name: &str, value: i64) -> Self {
        self.fields
            .insert(name.to_string(), FieldValue::Int(value));
        self
    }

    /// Add a boolean field.
    pub fn field_bool(mut self, name: &str, value: bool) -> Self {
        self.fields
            .insert(name.to_string(), FieldValue::Bool(value));
        self
    }

    /// Add a string field.
    pub fn field_str(mut self, name: &str, value: &str) -> Self {
        self.fields
            .insert(name.to_string(), FieldValue::Str(value.to_string()));
        self
    }

    /// Set timestamp (nanoseconds since epoch).
    pub fn timestamp(mut self, ns: Timestamp) -> Self {
        self.timestamp = Some(ns);
        self
    }

    /// Use current time as timestamp.
    pub fn timestamp_now(mut self) -> Self {
        let ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        self.timestamp = Some(ns);
        self
    }

    fn build(self) -> Row {
        let timestamp = self.timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64
        });
        Row {
            timestamp,
            tags: self.tags,
            fields: self.fields,
        }
    }
}

// ── Shared HTTP logic ───────────────────────────────────────────────

/// POST an `IngestBatch` to `/api/v1/write`.
async fn send_batch(client: &RutSeriClient, batch: &IngestBatch) -> Result<()> {
    let url = format!("{}/api/v1/write", client.base_url());

    let response = client
        .http()
        .post(&url)
        .json(batch)
        .send()
        .await
        .map_err(|e| RutSeriError::Internal(format!("Write request failed: {e}")))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(RutSeriError::Internal(format!(
            "Write failed ({status}): {body}"
        )))
    }
}
