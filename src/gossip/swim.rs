//! SWIM Gossip Protocol — failure detection via direct + indirect probing.
//!
//! Each node periodically probes a random peer:
//! 1. Direct ping (every `probe_interval_ms`)
//! 2. If no ACK within 500ms → indirect probe via `fanout` random peers
//! 3. If still no ACK → mark Suspect
//! 4. Gossip "Suspect" to random peers
//! 5. If no recovery within `suspect_timeout_ms` → mark Dead
//! 6. Gossip "Dead" → triggers Raft leader election via ClusterManager
//!
//! See: docs/architecture.md § Failure Detection — SWIM Gossip

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{NodeId, NodeInfo, NodeState};
use crate::config::GossipConfig;
use crate::gossip::membership::MembershipTable;

/// Events emitted by the SWIM gossip protocol.
///
/// `ClusterManager` subscribes to these events to drive failover logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipEvent {
    /// A node has been confirmed alive (direct or indirect ACK).
    NodeAlive(NodeId),

    /// A node is suspected of being down (no direct or indirect ACK).
    NodeSuspect(NodeId),

    /// A node has been declared dead after the suspect timeout expired.
    NodeDead(NodeId),

    /// A new node has joined the cluster.
    NodeJoined(NodeId, NodeInfo),
}

/// SWIM gossip agent — runs on every node (Coordinator and Storage).
///
/// Spawns a background Tokio task that:
/// - Probes a random peer every `probe_interval_ms`
/// - Piggybacks membership updates on probe messages
/// - Emits `GossipEvent`s for `ClusterManager` to act on
pub struct SwimAgent {
    /// This node's identity.
    node_id: NodeId,

    /// SWIM configuration.
    config: GossipConfig,

    /// Membership table — tracks all known nodes and their states.
    membership: Arc<MembershipTable>,

    /// Broadcast channel for gossip events.
    event_tx: broadcast::Sender<GossipEvent>,
}

impl SwimAgent {
    pub fn new(node_id: NodeId, config: GossipConfig) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            node_id,
            config,
            membership: Arc::new(MembershipTable::new()),
            event_tx,
        }
    }

    /// Subscribe to gossip events.
    pub fn subscribe(&self) -> broadcast::Receiver<GossipEvent> {
        self.event_tx.subscribe()
    }

    /// Get a snapshot of current membership.
    pub fn members(&self) -> Vec<(NodeId, NodeState)> {
        self.membership.snapshot()
    }

    /// Start the gossip protocol background task.
    ///
    /// This spawns a Tokio task that runs the SWIM probe loop.
    pub async fn start(&self) -> Result<()> {
        // TODO(engineer): implement SWIM probe loop
        //
        // loop {
        //     tokio::time::sleep(Duration::from_millis(self.config.probe_interval_ms)).await;
        //
        //     // 1. Select random peer from membership table
        //     let target = self.membership.random_peer(&self.node_id)?;
        //
        //     // 2. Direct probe: send ping, wait 500ms for ACK
        //     match self.direct_probe(&target).await {
        //         Ok(()) => continue,  // alive
        //         Err(_) => {
        //             // 3. Indirect probe: ask `fanout` random peers to ping target
        //             let probers = self.membership.random_peers(&self.node_id, self.config.fanout);
        //             let any_ack = self.indirect_probe(&target, &probers).await;
        //
        //             if !any_ack {
        //                 // 4. Mark suspect
        //                 self.membership.mark_suspect(&target);
        //                 let _ = self.event_tx.send(GossipEvent::NodeSuspect(target.clone()));
        //
        //                 // 5. Start suspect timer
        //                 // After suspect_timeout_ms → mark Dead
        //             }
        //         }
        //     }
        // }
        todo!("TODO(engineer): implement SwimAgent.start")
    }

    /// Join the cluster by contacting seed nodes.
    pub async fn join(&self, _seeds: &[String]) -> Result<()> {
        // TODO(engineer): implement cluster join
        // For each seed: send JoinRequest, receive membership table
        todo!("TODO(engineer): implement SwimAgent.join")
    }

    /// Send a direct ping to a target node.
    async fn direct_probe(&self, _target: &NodeId) -> Result<()> {
        // TODO(engineer): send UDP/TCP ping, await ACK with timeout
        todo!("TODO(engineer): implement direct_probe")
    }

    /// Ask indirect probers to ping the target on our behalf.
    async fn indirect_probe(
        &self,
        _target: &NodeId,
        _probers: &[NodeId],
    ) -> bool {
        // TODO(engineer): fan-out ping-req to probers, await any ACK
        todo!("TODO(engineer): implement indirect_probe")
    }
}
