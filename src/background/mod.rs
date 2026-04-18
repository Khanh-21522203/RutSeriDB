//! Background workers — merge, WAL cleanup, index builder, metrics.

pub mod index_builder;
pub mod merge_worker;
pub mod metrics;
pub mod wal_cleanup;
