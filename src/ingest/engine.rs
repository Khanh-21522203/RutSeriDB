//! IngestEngine — top-level write path entry point.
//!
//! See: docs/components.md § Ingest Engine
//!
//! Responsibilities:
//! - Validate incoming rows against the table schema
//! - Compute the shard key from primary tags
//! - Dispatch the batch to the correct ShardActor via its handle
//! - Await the oneshot response (durable commit confirmation)
//!
//! Does NOT:
//! - Implement WAL, MemTable, or Part logic (ShardActor does that)
//! - Serve queries

use std::collections::HashMap;
use std::sync::Arc;

use crate::common::error::{Result, RutSeriError};
use crate::common::schema::TableSchema;
use crate::common::shard_key;
use crate::common::types::{IngestBatch, ShardId};

use super::shard_actor::ShardHandle;

/// Top-level ingest API. One instance per node.
///
/// Holds references to all ShardActors and table schemas.
pub struct IngestEngine {
    /// Shard ID → actor handle. One per shard on this node.
    shard_handles: HashMap<ShardId, ShardHandle>,

    /// Table name → schema. Shared with the QueryEngine.
    schemas: Arc<HashMap<String, TableSchema>>,

    /// Total number of shards in the cluster.
    num_shards: u32,
}

impl IngestEngine {
    /// Create a new IngestEngine.
    ///
    /// # Arguments
    /// * `shard_handles` — Map of shard ID → actor handle
    /// * `schemas` — Shared table schemas
    /// * `num_shards` — Total shards in the cluster
    pub fn new(
        shard_handles: HashMap<ShardId, ShardHandle>,
        schemas: Arc<HashMap<String, TableSchema>>,
        num_shards: u32,
    ) -> Self {
        Self {
            shard_handles,
            schemas,
            num_shards,
        }
    }

    /// Ingest a batch of rows.
    ///
    /// Validates schema, computes shard key, dispatches to ShardActor,
    /// and awaits durable commit before returning.
    pub async fn ingest(&self, batch: IngestBatch) -> Result<()> {
        // Step 1: Lookup table schema
        let schema = self.schemas.get(&batch.table).ok_or_else(|| {
            RutSeriError::UnknownTable(batch.table.clone())
        })?;

        // Step 2: Validate rows against schema
        self.validate_batch(&batch, schema)?;

        // Step 3: Compute shard key from primary tags of the first row
        // (All rows in a batch must have the same primary tags —
        //  the API layer enforces this, or we group by shard here)
        let shard_id = if let Some(first_row) = batch.rows.first() {
            shard_key::compute_shard_key(
                &first_row.tags,
                &schema.primary_tags,
                self.num_shards,
            )
        } else {
            return Ok(()); // empty batch, nothing to do
        };

        // Step 4: Dispatch to the ShardActor
        let handle = self.shard_handles.get(&shard_id).ok_or_else(|| {
            RutSeriError::Ingest(format!("No shard actor for shard {shard_id}"))
        })?;

        // Step 5: Await durable commit (oneshot response from actor)
        handle.write(batch).await
    }

    /// Validate that all rows in a batch conform to the table schema.
    fn validate_batch(&self, batch: &IngestBatch, schema: &TableSchema) -> Result<()> {
        // TODO(engineer): implement validation
        //
        // For each row:
        // 1. Check that all primary_tags are present in row.tags
        // 2. Check that field names match schema columns
        // 3. Check that field types match column types
        //    (e.g., FieldValue::Float for ColumnType::FieldFloat)
        // 4. Return RutSeriError::SchemaValidation on first mismatch
        //
        // For now, accept all rows:
        for row in &batch.rows {
            for tag_name in &schema.primary_tags {
                if !row.tags.contains_key(tag_name) {
                    return Err(RutSeriError::MissingPrimaryTag(tag_name.clone()));
                }
            }
        }

        Ok(())
    }
}
