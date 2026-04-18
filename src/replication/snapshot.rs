//! Snapshot Sync — full Part file transfer for re-joining replicas.
//!
//! Triggered when a replica's `from_seq` is older than the leader's
//! oldest buffered WAL entry. The leader sends:
//! 1. Current catalog snapshot
//! 2. All Part files referenced by the catalog
//! 3. SnapshotEnd marker with catalog version
//!
//! After snapshot, the replica resumes normal WAL streaming.
//!
//! See: docs/cluster/replication.md § Snapshot Sync

use std::path::Path;

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{NodeId, ShardId};

/// Handles full snapshot sync between leader and replica.
pub struct SnapshotSync;

impl SnapshotSync {
    // ── Leader Side ──────────────────────────────────────────────────

    /// Send a complete snapshot to a replica (leader side).
    ///
    /// Steps:
    /// 1. Read local Catalog for the shard
    /// 2. Send catalog snapshot + current WAL sequence
    /// 3. Stream each .rpart file referenced in the catalog
    /// 4. Send SnapshotEnd with catalog version
    pub async fn send_snapshot(
        _shard_id: ShardId,
        _replica_addr: &str,
        _data_dir: &Path,
    ) -> Result<()> {
        // TODO(engineer): implement snapshot send
        //
        // 1. let catalog = Catalog::load(data_dir)?;
        // 2. let snapshot_seq = wal.current_seq();
        // 3. let conn = TcpStream::connect(replica_addr).await?;
        // 4. send_msg(&conn, SnapshotStart { catalog, snapshot_seq }).await?;
        // 5. for part in catalog.parts() {
        //        let data = tokio::fs::read(part.path).await?;
        //        send_msg(&conn, PartFileChunk { name: part.path, data }).await?;
        //    }
        // 6. send_msg(&conn, SnapshotEnd { catalog_ver: catalog.version }).await?;
        todo!("TODO(engineer): implement send_snapshot")
    }

    // ── Replica Side ─────────────────────────────────────────────────

    /// Receive and apply a snapshot from the leader (replica side).
    ///
    /// Steps:
    /// 1. Receive catalog snapshot
    /// 2. Receive and write each Part file to disk
    /// 3. Receive SnapshotEnd → update local Catalog to that version
    /// 4. Return the snapshot_seq so the caller can resume WAL streaming
    pub async fn receive_snapshot(
        _shard_id: ShardId,
        _data_dir: &Path,
    ) -> Result<u64> {
        // TODO(engineer): implement snapshot receive
        //
        // 1. let msg = recv_msg(&conn).await?; // SnapshotStart
        // 2. loop {
        //        match recv_msg(&conn).await? {
        //            PartFileChunk { name, data } => {
        //                tokio::fs::write(data_dir.join(name), data).await?;
        //            }
        //            SnapshotEnd { catalog_ver } => {
        //                catalog.update_version(catalog_ver)?;
        //                break;
        //            }
        //        }
        //    }
        // 3. Ok(snapshot_seq)
        todo!("TODO(engineer): implement receive_snapshot")
    }
}
