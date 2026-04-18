//! ShardActor — per-shard write processing via async actor pattern.
//!
//! See: docs/components.md § Ingest Engine (Dispatch Queue + Oneshot Flow)
//! See: docs/architecture.md § D11
//!
//! Architecture:
//! - Each shard has one ShardActor running as a Tokio task
//! - Client handlers push `(batch, oneshot::tx)` into an mpsc queue
//! - Client tasks park at `rx.await` (yielding the Tokio worker thread)
//! - The actor drains all pending items, coalesces, does ONE WAL fsync,
//!   inserts into MemTable, then fires all `tx.send(OK)` simultaneously
//!
//! Benefits:
//! - Zero thread blocking during WAL fsync
//! - Group commit: N clients → 1 fsync
//! - Free cancellation detection via dropped oneshot::Receiver

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};

use crate::common::error::{Result, RutSeriError};
use crate::common::schema::TableSchema;
use crate::common::types::{IngestBatch, Row, ShardId};
use crate::storage::memtable::memtable::MemTable;
use crate::storage::wal::writer::WalWriter;

/// Message sent to a ShardActor via its dispatch queue.
pub struct ShardCommand {
    /// The batch of rows to ingest.
    pub batch: IngestBatch,

    /// Oneshot sender for the response. The actor sends Ok(()) after
    /// durable commit (WAL fsynced + MemTable inserted).
    pub response_tx: oneshot::Sender<Result<()>>,
}

/// Handle used by IngestEngine to send work to a specific shard.
///
/// Cheap to clone — contains only an mpsc::Sender.
#[derive(Clone)]
pub struct ShardHandle {
    tx: mpsc::Sender<ShardCommand>,
}

impl ShardHandle {
    /// Send a batch to this shard's actor and await durable commit.
    ///
    /// The caller parks at `rx.await` — the Tokio worker thread is
    /// released while waiting. This is the key to non-blocking ingest.
    pub async fn write(&self, batch: IngestBatch) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ShardCommand {
                batch,
                response_tx: tx,
            })
            .await
            .map_err(|_| RutSeriError::ShardChannelClosed)?;

        // Park until the actor fires the response
        rx.await.map_err(|_| RutSeriError::ShardChannelClosed)?
    }
}

/// Per-shard actor that owns the WAL and MemTable for one shard.
///
/// Runs as a single Tokio task. Never blocks — all I/O is done
/// via spawn_blocking for fsync operations.
pub struct ShardActor {
    /// Shard identifier.
    shard_id: ShardId,

    /// Dispatch queue receiver.
    rx: mpsc::Receiver<ShardCommand>,

    /// WAL writer for this shard.
    wal: WalWriter,

    /// In-memory write buffer.
    memtable: MemTable,

    /// MemTable flush threshold in bytes.
    flush_threshold: usize,

    /// Path to shard data directory.
    shard_dir: PathBuf,

    /// Table schema (for the PartWriter during flush).
    schema: Arc<TableSchema>,
}

impl ShardActor {
    /// Spawn a new ShardActor, returning the handle for sending commands.
    ///
    /// # Arguments
    /// * `shard_id` — Shard identifier
    /// * `shard_dir` — Path to `/data/shard-{id}/`
    /// * `wal` — Pre-constructed WAL writer
    /// * `schema` — Table schema
    /// * `flush_threshold` — MemTable size threshold for triggering flush
    /// * `channel_capacity` — mpsc channel buffer size
    pub fn spawn(
        shard_id: ShardId,
        shard_dir: PathBuf,
        wal: WalWriter,
        schema: Arc<TableSchema>,
        flush_threshold: usize,
        channel_capacity: usize,
    ) -> ShardHandle {
        let (tx, rx) = mpsc::channel(channel_capacity);

        let actor = Self {
            shard_id,
            rx,
            wal,
            memtable: MemTable::new(schema.name.clone()),
            flush_threshold,
            shard_dir,
            schema,
        };

        // Spawn the actor loop as a Tokio task
        tokio::spawn(actor.run());

        ShardHandle { tx }
    }

    /// Main actor loop.
    ///
    /// See: docs/components.md § Shard Actor Loop
    ///
    /// ```text
    /// loop {
    ///     1. recv() — wait for first command (yields if queue is empty)
    ///     2. try_recv() — drain all remaining commands (group commit)
    ///     3. Coalesce all rows
    ///     4. WAL append(coalesced)
    ///     5. WAL fsync() — ONE call covers N clients
    ///     6. MemTable.insert(rows)
    ///     7. for each tx: tx.send(Ok) — unblock all clients
    ///     8. If MemTable > threshold: trigger flush
    /// }
    /// ```
    async fn run(mut self) {
        loop {
            // Step 1: Wait for the first command
            let first = match self.rx.recv().await {
                Some(cmd) => cmd,
                None => {
                    // Channel closed — all handles dropped, shut down
                    tracing::info!("ShardActor {} shutting down", self.shard_id);
                    return;
                }
            };

            // Step 2: Drain all remaining commands (non-blocking)
            let mut commands = vec![first];
            while let Ok(cmd) = self.rx.try_recv() {
                commands.push(cmd);
            }

            let batch_count = commands.len();

            // Step 3: Coalesce all rows
            let mut all_rows: Vec<Row> = Vec::new();
            let mut senders: Vec<oneshot::Sender<Result<()>>> = Vec::new();

            for cmd in commands {
                all_rows.extend(cmd.batch.rows);
                senders.push(cmd.response_tx);
            }

            // Steps 4-6: WAL append + fsync + MemTable insert
            let result = self.process_batch(all_rows).await;

            // Step 7: Fire ACKs to all waiting clients
            // Note: RutSeriError is not Clone (it wraps std::io::Error),
            // so on failure we convert to a string-based error for broadcast.
            match result {
                Ok(()) => {
                    for tx in senders {
                        let _ = tx.send(Ok(()));
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    for tx in senders {
                        let _ = tx.send(Err(RutSeriError::Internal(msg.clone())));
                    }
                }
            }

            tracing::debug!(
                "ShardActor {}: group committed {} batches",
                self.shard_id,
                batch_count,
            );

            // Step 8: Check flush threshold
            if self.memtable.size_bytes() >= self.flush_threshold {
                self.trigger_flush().await;
            }
        }
    }

    /// WAL append + fsync + MemTable insert for a coalesced batch.
    async fn process_batch(&mut self, rows: Vec<Row>) -> Result<()> {
        // TODO(engineer): implement
        //
        // 1. Build a WalEntry::Write from the rows
        // 2. self.wal.append(&entry)?
        // 3. self.wal.fsync()?
        //    (For actual async: use spawn_blocking for fsync)
        // 4. self.memtable.insert(rows)

        todo!("ShardActor::process_batch")
    }

    /// Trigger an async flush of the MemTable to a Part file.
    ///
    /// The actor continues processing new writes immediately —
    /// the flush runs in a background task.
    async fn trigger_flush(&mut self) {
        // TODO(engineer): implement
        //
        // 1. Take a snapshot: let snap = self.memtable.snapshot();
        // 2. Clear the memtable: self.memtable.clear();
        // 3. Spawn a blocking task:
        //    tokio::task::spawn_blocking(move || {
        //        let meta = PartWriter::flush(&snap, &schema, &parts_dir)?;
        //        catalog.add_part(&snap.table, meta);
        //        catalog.persist(&shard_dir)?;
        //        wal.checkpoint(last_seq, catalog.version)?;
        //        Ok(())
        //    });
        // 4. Notify the IndexBuilder worker (via a channel)

        tracing::info!("ShardActor {}: flush triggered", self.shard_id);
        todo!("ShardActor::trigger_flush")
    }
}
