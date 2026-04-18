//! Query executor — PhysicalPlan → Arrow RecordBatch results.
//!
//! See: docs/components.md § Local Query Engine
//!
//! Pipeline:
//!   PhysicalPlan
//!     → PartReader.read() for each Part (projection + predicates)
//!     → MemTable snapshot scan
//!     → Merge results
//!     → Apply aggregations (SUM, COUNT, MIN, MAX, MEAN)
//!     → Apply GROUP BY, ORDER BY, LIMIT
//!     → Return Vec<RecordBatch>
//!
//! Does NOT:
//! - Modify any data
//! - Know about WAL

use std::sync::Arc;

use arrow::record_batch::RecordBatch;

use crate::common::error::{Result, RutSeriError};
use crate::storage::memtable::memtable::MemTableSnapshot;

use super::planner::PhysicalPlan;

/// Execute a physical plan and return Arrow RecordBatches.
///
/// # Arguments
/// * `plan` — The physical plan produced by the planner
/// * `memtable_snapshot` — Optional MemTable snapshot for in-memory data
///
/// # Returns
/// A Vec of Arrow RecordBatches containing the query results.
/// Using Arrow from Phase 0 enables seamless Phase 1 streaming
/// over gRPC (distributed query merge without refactoring).
pub fn execute(
    plan: &PhysicalPlan,
    memtable_snapshot: Option<&MemTableSnapshot>,
) -> Result<Vec<RecordBatch>> {
    // TODO(engineer): implement
    //
    // Step 1: Scan Parts
    //   let mut all_rows = Vec::new();
    //   for part_plan in &plan.parts_to_scan {
    //       let predicates = convert_filters_to_predicates(&plan.query.filters);
    //       let rows = PartReader::read(
    //           &part_plan.path,
    //           &part_plan.projected_columns,
    //           &predicates,
    //       )?;
    //       all_rows.extend(rows);
    //   }
    //
    // Step 2: Scan MemTable snapshot (if present)
    //   if let Some(snap) = memtable_snapshot {
    //       // Filter rows from MemTable matching the query predicates
    //       // Add matching rows to all_rows
    //   }
    //
    // Step 3: Merge and deduplicate
    //   // Sort by (timestamp, tag_hash) — same order as MemTable/Part
    //
    // Step 4: Apply aggregations
    //   if !plan.query.aggregations.is_empty() {
    //       // Group rows by group_by columns
    //       // For each group, compute aggregate values
    //       // (SUM, COUNT, MIN, MAX, MEAN)
    //   }
    //
    // Step 5: Apply ORDER BY
    //   // Sort results by ORDER BY columns
    //
    // Step 6: Apply LIMIT
    //   // Truncate to LIMIT rows
    //
    // Step 7: Convert to Arrow RecordBatch
    //   // Build arrow::datatypes::Schema from column names/types
    //   // Build arrow::array::ArrayRef for each column
    //   // RecordBatch::try_new(schema, columns)

    todo!("executor::execute")
}
