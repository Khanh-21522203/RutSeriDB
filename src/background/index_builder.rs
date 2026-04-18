//! Index Builder Worker — backfills inverted index for new Parts.
//!
//! See: docs/components.md § Background Workers — Index Builder
//! See: docs/storage/indexes.md § Inverted Index — Build Process
//!
//! Trigger: Notification when a new Part is flushed, or startup backfill.
//! Action: Scan new Part for unique tag key/value pairs → update
//!         inverted index in Catalog → persist atomically.

use std::path::Path;

use tokio::sync::mpsc;

use crate::common::error::Result;
use crate::storage::catalog::catalog::Catalog;

/// Notification sent to the IndexBuilder when a Part is flushed.
#[derive(Debug)]
pub struct IndexBuildRequest {
    /// Table that the Part belongs to.
    pub table: String,
    /// Part ID to index.
    pub part_id: uuid::Uuid,
    /// Path to the .rpart file.
    pub part_path: std::path::PathBuf,
}

/// Run the index builder worker loop.
///
/// Listens for `IndexBuildRequest` notifications and backfills the
/// inverted index for each new Part.
pub async fn run_index_builder(
    mut rx: mpsc::Receiver<IndexBuildRequest>,
    catalog: &mut Catalog,
    shard_dir: &Path,
) -> Result<()> {
    // TODO(engineer): implement
    //
    // while let Some(request) = rx.recv().await {
    //   tracing::info!("IndexBuilder: indexing Part {}", request.part_id);
    //
    //   // 1. Open the Part file, read tag columns
    //   //    (PartReader can read specific columns)
    //
    //   // 2. Collect unique (tag_key, tag_value) pairs
    //
    //   // 3. Update catalog inverted index
    //   //    catalog.update_inverted(&request.table, request.part_id, tag_entries);
    //
    //   // 4. Persist catalog atomically
    //   //    catalog.persist(shard_dir)?;
    //
    //   tracing::info!("IndexBuilder: indexed Part {}", request.part_id);
    // }

    todo!("run_index_builder")
}
