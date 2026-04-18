//! Raft module — Metadata consensus using `openraft`.
//!
//! A single Raft group across 1–3 Coordinator nodes replicates
//! the MetadataCatalog (schema + shard map). NOT per-shard Raft.
//!
//! See: docs/architecture.md § Design Decision D8

pub mod log;
pub mod node;
pub mod state_machine;
