//! Raft Node — top-level Raft lifecycle management using `openraft`.
//!
//! Wraps `openraft::Raft<TypeConfig>` with our concrete types.
//! Handles: node initialization, proposal, tick, and message handling.
//!
//! See: docs/phase1_plan.md § raft/node

use std::sync::Arc;

use crate::common::error::{Result, RutSeriError};
use crate::common::types::NodeId;
use crate::config::RaftConfig;
use crate::coordinator::metadata_catalog::{MetadataCatalog, MetadataOp};
use crate::raft::log::RaftLogStore;
use crate::raft::state_machine::MetadataStateMachine;

/// Top-level Raft node for metadata consensus.
///
/// Each Coordinator in the cluster runs one `RaftNode`. The Raft group
/// is small (1–3 nodes, always odd for quorum). It only replicates
/// metadata — NOT data-plane traffic.
pub struct RaftNode {
    /// This node's unique ID.
    node_id: NodeId,

    /// Raft configuration.
    config: RaftConfig,

    /// Log storage (persistent).
    log_store: RaftLogStore,

    /// State machine (applies committed ops to MetadataCatalog).
    state_machine: MetadataStateMachine,

    // TODO(engineer): add openraft::Raft instance
    // raft: openraft::Raft<TypeConfig>,
}

impl RaftNode {
    /// Create a new Raft node.
    pub fn new(
        node_id: NodeId,
        config: RaftConfig,
        catalog: Arc<MetadataCatalog>,
    ) -> Self {
        let log_store = RaftLogStore::new(config.data_dir.clone());
        let state_machine = MetadataStateMachine::new(catalog);

        Self {
            node_id,
            config,
            log_store,
            state_machine,
        }
    }

    /// Initialize the Raft node and start the background tick loop.
    ///
    /// Must be called before proposing any operations.
    pub async fn start(&mut self) -> Result<()> {
        // TODO(engineer): implement Raft startup
        //
        // 1. Load log from disk
        //    self.log_store.load()?;
        //
        // 2. Restore state machine from latest snapshot (if any)
        //
        // 3. Create openraft::Raft instance with our TypeConfig
        //    let raft = openraft::Raft::new(
        //        self.node_id.clone(),
        //        self.config.clone(),
        //        network,
        //        self.log_store,
        //        self.state_machine,
        //    ).await?;
        //
        // 4. If this is a fresh cluster, initialize with
        //    raft.initialize(initial_members)?;
        //
        // 5. Start tick loop in background task
        todo!("TODO(engineer): implement RaftNode.start")
    }

    /// Propose a metadata operation to the Raft cluster.
    ///
    /// Blocks until the operation is committed by a quorum and
    /// applied to the state machine.
    pub async fn propose(&self, _op: MetadataOp) -> Result<()> {
        // TODO(engineer): implement proposal
        //
        // 1. Serialize MetadataOp
        // 2. Call self.raft.client_write(op).await
        // 3. Handle errors: NotLeader → return with leader hint
        // 4. Return Ok(()) after commit
        todo!("TODO(engineer): implement RaftNode.propose")
    }

    /// Check if this node is currently the Raft leader.
    pub fn is_leader(&self) -> bool {
        // TODO(engineer): check openraft state
        todo!("TODO(engineer): implement RaftNode.is_leader")
    }

    /// Get the current Raft leader's node ID (if known).
    pub fn current_leader(&self) -> Option<NodeId> {
        // TODO(engineer): query openraft for current leader
        todo!("TODO(engineer): implement RaftNode.current_leader")
    }

    /// Add a new node to the Raft membership.
    pub async fn add_member(&self, _node_id: NodeId, _addr: String) -> Result<()> {
        // TODO(engineer): call openraft change_membership
        todo!("TODO(engineer): implement RaftNode.add_member")
    }

    /// Shut down the Raft node gracefully.
    pub async fn shutdown(&self) -> Result<()> {
        // TODO(engineer): call openraft shutdown
        todo!("TODO(engineer): implement RaftNode.shutdown")
    }
}
