//! Raft State Machine — applies committed MetadataOp entries.
//!
//! Implements the `openraft::RaftStateMachine` trait. Each committed
//! log entry is a `MetadataOp` that mutates the `MetadataCatalog`.
//!
//! See: docs/phase1_plan.md § raft/state_machine

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::common::error::{Result, RutSeriError};
use crate::coordinator::metadata_catalog::{MetadataCatalog, MetadataOp};

/// Raft state machine wrapping the MetadataCatalog.
///
/// `openraft` calls `apply()` for each committed log entry.
/// Snapshotting serializes the entire MetadataState.
pub struct MetadataStateMachine {
    /// The metadata catalog that this state machine operates on.
    catalog: Arc<MetadataCatalog>,

    /// Last applied log index — used by openraft for consistency.
    last_applied_log: u64,
}

impl MetadataStateMachine {
    pub fn new(catalog: Arc<MetadataCatalog>) -> Self {
        Self {
            catalog,
            last_applied_log: 0,
        }
    }

    /// Apply a committed metadata operation.
    ///
    /// Called by the Raft framework after a log entry is committed
    /// by a quorum of Coordinator nodes.
    pub async fn apply(&mut self, index: u64, op: MetadataOp) -> Result<()> {
        // TODO(engineer): implement openraft::RaftStateMachine::apply
        //
        // self.catalog.apply(op).await?;
        // self.last_applied_log = index;
        // Ok(())
        todo!("TODO(engineer): implement MetadataStateMachine.apply")
    }

    /// Create a snapshot of the current state.
    ///
    /// Used by openraft for log compaction and new-node catch-up.
    pub async fn snapshot(&self) -> Result<(u64, Vec<u8>)> {
        let data = self.catalog.snapshot().await?;
        Ok((self.last_applied_log, data))
    }

    /// Restore state from a snapshot.
    pub async fn restore(&mut self, index: u64, data: &[u8]) -> Result<()> {
        self.catalog.restore(data).await?;
        self.last_applied_log = index;
        Ok(())
    }

    /// Get the last applied log index.
    pub fn last_applied(&self) -> u64 {
        self.last_applied_log
    }
}
