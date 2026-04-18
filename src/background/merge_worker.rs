//! Merge Worker — compacts multiple small Parts into fewer larger ones.
//!
//! See: docs/components.md § Background Workers
//!
//! Trigger: Parts per partition > `merge.max_parts_per_partition`
//! Action: merge-sort N Parts → 1 larger Part → update Catalog → delete old Parts
//!
//! Does NOT: handle client requests or block the ingest path.

use std::path::Path;

use crate::common::error::Result;
use crate::storage::catalog::catalog::Catalog;

/// Run the merge worker loop.
///
/// Periodically checks each table's partition for excess Parts
/// and merges them if `max_parts_per_partition` is exceeded.
pub async fn run_merge_worker(
    shard_dir: &Path,
    catalog: &mut Catalog,
    max_parts_per_partition: u32,
    target_part_size_bytes: u64,
) -> Result<()> {
    // TODO(engineer): implement
    //
    // loop {
    //   sleep(check_interval).await;
    //
    //   for (table_name, table_catalog) in &catalog.tables {
    //     if table_catalog.parts.len() > max_parts_per_partition as usize {
    //       // 1. Select N smallest Parts for merging
    //       // 2. Read all rows from those Parts (PartReader::read)
    //       // 3. Merge-sort by (timestamp, tag_hash)
    //       // 4. Write merged Part (PartWriter::flush)
    //       // 5. Update Catalog: add new Part, remove old Parts
    //       // 6. Persist Catalog
    //       // 7. Delete old .rpart files
    //       // 8. Update inverted index (remove old IDs, add new ID)
    //     }
    //   }
    // }

    todo!("run_merge_worker")
}
