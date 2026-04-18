//! Replication module — Phase 1 WAL streaming and snapshot sync.
//!
//! Handles:
//! - Leader → replica WAL streaming (custom TCP, length-prefixed framing)
//! - Full snapshot sync for re-joining replicas
//!
//! See: docs/cluster/replication.md

pub mod manager;
pub mod snapshot;
