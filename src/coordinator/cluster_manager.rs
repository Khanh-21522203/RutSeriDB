//! Cluster Manager — glues SWIM gossip + Raft for failure handling.
//!
//! Subscribes to gossip events and triggers Raft proposals:
//! - NodeDead → query replicas for offset → PromoteLeader
//! - NodeJoined → RegisterNode + AssignShard
//!
//! See: docs/architecture.md § Cluster Management

use std::sync::Arc;

use tracing::{info, warn};

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{NodeId, ShardId};
use crate::coordinator::metadata_catalog::MetadataCatalog;
use crate::gossip::swim::GossipEvent;

/// Cluster Manager — receives gossip events and drives cluster topology.
pub struct ClusterManager {
    /// Metadata catalog — modified via Raft proposals.
    catalog: Arc<MetadataCatalog>,

    // TODO(engineer): add Raft proposer handle
    // raft: Arc<RaftNode>,

    // TODO(engineer): add gossip event receiver
    // gossip_rx: tokio::sync::broadcast::Receiver<GossipEvent>,
}

impl ClusterManager {
    pub fn new(catalog: Arc<MetadataCatalog>) -> Self {
        Self { catalog }
    }

    /// Main event loop — listens for gossip events and reacts.
    ///
    /// Runs as a long-lived Tokio task on each Coordinator.
    pub async fn run(&mut self) -> Result<()> {
        // TODO(engineer): implement event loop
        //
        // loop {
        //     match self.gossip_rx.recv().await {
        //         Ok(GossipEvent::NodeDead(node_id)) => {
        //             self.handle_node_dead(node_id).await?;
        //         }
        //         Ok(GossipEvent::NodeJoined(node_id, info)) => {
        //             self.handle_node_joined(node_id, info).await?;
        //         }
        //         Ok(GossipEvent::NodeAlive(node_id)) => {
        //             info!("Node alive: {node_id}");
        //         }
        //         Ok(GossipEvent::NodeSuspect(node_id)) => {
        //             warn!("Node suspect: {node_id}");
        //         }
        //         Err(e) => {
        //             warn!("Gossip channel error: {e}");
        //             break;
        //         }
        //     }
        // }
        todo!("TODO(engineer): implement ClusterManager.run")
    }

    /// Handle a node being declared dead by SWIM gossip.
    ///
    /// For each shard where the dead node was leader:
    /// 1. Query all replicas for their replication offset
    /// 2. Select the replica with the highest offset
    /// 3. Propose `PromoteLeader` via Raft
    async fn handle_node_dead(&self, _node_id: NodeId) -> Result<()> {
        // TODO(engineer): implement leader failover
        //
        // let shard_map = self.catalog.get_shard_map().await;
        // for assignment in &shard_map {
        //     if assignment.leader == node_id {
        //         let offsets = self.query_replica_offsets(
        //             assignment.shard_id,
        //             &assignment.replicas,
        //         ).await?;
        //         let best = offsets.into_iter()
        //             .max_by_key(|(_, offset)| *offset)
        //             .map(|(id, _)| id);
        //         if let Some(new_leader) = best {
        //             self.propose_promote(assignment.shard_id, new_leader).await?;
        //         }
        //     }
        // }
        todo!("TODO(engineer): implement handle_node_dead")
    }

    /// Query replica nodes for their replication offset.
    async fn query_replica_offsets(
        &self,
        _shard_id: ShardId,
        _replicas: &[NodeId],
    ) -> Result<Vec<(NodeId, u64)>> {
        // TODO(engineer): fan-out RPC calls to each replica
        todo!("TODO(engineer): implement query_replica_offsets")
    }

    /// Propose a PromoteLeader operation via Raft.
    async fn propose_promote(
        &self,
        _shard_id: ShardId,
        _new_leader: NodeId,
    ) -> Result<()> {
        // TODO(engineer): serialize MetadataOp::PromoteLeader
        // and propose to Raft
        todo!("TODO(engineer): implement propose_promote")
    }
}
