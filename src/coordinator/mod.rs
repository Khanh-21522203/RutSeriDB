//! Coordinator module — Phase 1 distribution layer.
//!
//! Runs on `--role=coordinator` nodes. Handles:
//! - Write routing to shard leaders
//! - Distributed query fan-out + Arrow merge
//! - Raft-replicated metadata catalog (schema + shard map)
//! - Cluster management (SWIM gossip integration + leader election)
//! - Read routing for follower reads (consistency=ONE)
//!
//! See: docs/phase1_plan.md

pub mod cluster_manager;
pub mod metadata_catalog;
pub mod query_planner;
pub mod read_router;
pub mod write_router;
