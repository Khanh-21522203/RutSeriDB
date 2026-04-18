//! Gossip module — SWIM protocol for node membership & failure detection.
//!
//! AP layer, independent of Raft. Propagates membership changes
//! peer-to-peer in O(log N) rounds.
//!
//! See: docs/architecture.md § Failure Detection — SWIM Gossip

pub mod membership;
pub mod swim;
