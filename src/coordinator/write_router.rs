//! Write Router — routes write batches to the correct shard leader.
//!
//! Algorithm:
//! 1. Extract primary tags from the IngestBatch
//! 2. Compute shard_id = hash(sorted_primary_tags) % num_shards
//! 3. Lookup leader node from MetadataCatalog's shard map
//! 4. Forward the batch to the leader via internal gRPC
//!
//! Does NOT: validate schema, touch WAL/MemTable, or perform leader election.
//!
//! See: docs/components.md § Coordinator — Write Router

use std::sync::Arc;

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{IngestBatch, Row, ShardId};
use crate::coordinator::metadata_catalog::MetadataCatalog;
use crate::rpc::client::StorageNodeClient;

/// Routes write batches to the correct shard leader.
pub struct WriteRouter {
    /// Metadata catalog for shard → leader lookups.
    catalog: Arc<MetadataCatalog>,

    /// RPC client pool for forwarding to storage nodes.
    rpc_client: Arc<StorageNodeClient>,

    /// Total number of shards (static, set at cluster creation).
    num_shards: u32,
}

impl WriteRouter {
    pub fn new(
        catalog: Arc<MetadataCatalog>,
        rpc_client: Arc<StorageNodeClient>,
        num_shards: u32,
    ) -> Self {
        Self {
            catalog,
            rpc_client,
            num_shards,
        }
    }

    /// Route an ingest batch to the correct shard leader.
    ///
    /// Steps:
    /// 1. Compute shard key from the batch's primary tags
    /// 2. Resolve leader node from catalog
    /// 3. Forward via gRPC
    pub async fn route_write(&self, batch: IngestBatch) -> Result<()> {
        // TODO(engineer): implement shard key computation
        // let primary_tags = self.extract_primary_tags(&batch)?;
        // let shard_id = crate::common::shard_key::compute_shard_key(&primary_tags, self.num_shards);
        // let leader = self.catalog.get_shard_leader(shard_id)
        //     .ok_or(RutSeriError::LeaderNotFound(shard_id))?;
        // let leader_addr = self.catalog.get_node_addr(&leader)?;
        // self.rpc_client.write_batch(&leader_addr, batch).await?;
        todo!("TODO(engineer): implement WriteRouter.route_write")
    }

    /// Extract primary tags from the batch for shard key computation.
    fn extract_primary_tags(
        &self,
        _batch: &IngestBatch,
    ) -> Result<std::collections::BTreeMap<String, String>> {
        // TODO(engineer): lookup table schema from catalog,
        // extract primary tag columns from the first row
        todo!("TODO(engineer): implement extract_primary_tags")
    }
}
