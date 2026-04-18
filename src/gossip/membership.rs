//! Membership Table — tracks node liveness states for SWIM gossip.
//!
//! State machine per node: Alive → Suspect → Dead
//!
//! See: docs/architecture.md § Cluster Management

use std::collections::HashMap;
use std::sync::RwLock;

use crate::common::types::{NodeId, NodeInfo, NodeState};

/// Thread-safe membership table for SWIM gossip.
///
/// Every node maintains its own copy. Gossip messages piggyback
/// membership updates so views converge in O(log N) rounds.
pub struct MembershipTable {
    /// node_id → (NodeInfo, incarnation_number)
    nodes: RwLock<HashMap<NodeId, (NodeInfo, u64)>>,
}

impl MembershipTable {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
        }
    }

    /// Add or update a node in the membership table.
    pub fn upsert(&self, info: NodeInfo, incarnation: u64) {
        let mut nodes = self.nodes.write().unwrap();
        let entry = nodes.entry(info.node_id.clone()).or_insert((info.clone(), 0));
        // Only update if incarnation is newer (prevents stale info)
        if incarnation >= entry.1 {
            entry.0 = info;
            entry.1 = incarnation;
        }
    }

    /// Mark a node as Suspect.
    pub fn mark_suspect(&self, node_id: &NodeId) {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(entry) = nodes.get_mut(node_id) {
            entry.0.state = NodeState::Suspect;
        }
    }

    /// Mark a node as Dead.
    pub fn mark_dead(&self, node_id: &NodeId) {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(entry) = nodes.get_mut(node_id) {
            entry.0.state = NodeState::Dead;
        }
    }

    /// Mark a node as Alive (e.g., after receiving an ACK).
    pub fn mark_alive(&self, node_id: &NodeId) {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(entry) = nodes.get_mut(node_id) {
            entry.0.state = NodeState::Alive;
        }
    }

    /// Remove a dead node from the table.
    pub fn remove(&self, node_id: &NodeId) {
        let mut nodes = self.nodes.write().unwrap();
        nodes.remove(node_id);
    }

    /// Get a snapshot of current membership.
    pub fn snapshot(&self) -> Vec<(NodeId, NodeState)> {
        let nodes = self.nodes.read().unwrap();
        nodes
            .iter()
            .map(|(id, (info, _))| (id.clone(), info.state))
            .collect()
    }

    /// Select a random alive peer (excluding self).
    pub fn random_peer(&self, self_id: &NodeId) -> Option<NodeId> {
        use rand::seq::IteratorRandom;
        let nodes = self.nodes.read().unwrap();
        nodes
            .iter()
            .filter(|(id, (info, _))| {
                *id != self_id && info.state == NodeState::Alive
            })
            .map(|(id, _)| id.clone())
            .choose(&mut rand::rng())
    }

    /// Select N random alive peers (excluding self).
    pub fn random_peers(&self, self_id: &NodeId, n: usize) -> Vec<NodeId> {
        use rand::seq::IteratorRandom;
        let nodes = self.nodes.read().unwrap();
        nodes
            .iter()
            .filter(|(id, (info, _))| {
                *id != self_id && info.state == NodeState::Alive
            })
            .map(|(id, _)| id.clone())
            .choose_multiple(&mut rand::rng(), n)
    }

    /// Get the count of alive nodes.
    pub fn alive_count(&self) -> usize {
        let nodes = self.nodes.read().unwrap();
        nodes
            .values()
            .filter(|(info, _)| info.state == NodeState::Alive)
            .count()
    }
}
