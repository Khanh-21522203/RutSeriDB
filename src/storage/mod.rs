//! Storage layer — WAL, MemTable, Part files, Catalog, Indexes.
//!
//! This module owns all durable and in-memory data structures.
//! It must NEVER import from `ingest`, `query`, `api`, or `background`.

pub mod catalog;
pub mod index;
pub mod memtable;
pub mod part;
pub mod wal;
