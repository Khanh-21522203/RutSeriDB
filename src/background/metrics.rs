//! Metrics reporter — exposes internal gauges for observability.
//!
//! See: docs/components.md § Background Workers
//!
//! Reports metrics every 15 seconds (configurable).
//! Phase 2 will expose these via a Prometheus /metrics HTTP endpoint.

use crate::common::error::Result;

/// Internal metrics gauges.
///
/// These are collected from various components and reported periodically.
#[derive(Debug, Default)]
pub struct Metrics {
    /// Total rows ingested since startup.
    pub rows_ingested: u64,
    /// Total WAL bytes written.
    pub wal_bytes_written: u64,
    /// Total Part files on disk.
    pub part_files_count: u64,
    /// Current MemTable size across all shards (bytes).
    pub memtable_bytes: u64,
    /// Total queries executed.
    pub queries_executed: u64,
    /// Total Parts pruned by indexes (avoided reads).
    pub parts_pruned: u64,
}

/// Run the metrics reporter loop.
///
/// Collects metrics from components and logs them periodically.
/// In Phase 2, this will be replaced with a Prometheus endpoint.
pub async fn run_metrics_reporter(interval_secs: u64) -> Result<()> {
    // TODO(engineer): implement
    //
    // loop {
    //   tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    //
    //   // Collect metrics from shared state
    //   // Log or expose via /metrics endpoint
    //
    //   tracing::info!(
    //       rows_ingested = metrics.rows_ingested,
    //       part_files = metrics.part_files_count,
    //       memtable_bytes = metrics.memtable_bytes,
    //       queries = metrics.queries_executed,
    //       "Metrics snapshot"
    //   );
    // }

    todo!("run_metrics_reporter")
}
