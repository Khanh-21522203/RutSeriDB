//! Internal gRPC service implementation on Storage Nodes.
//!
//! Exposes endpoints that the Coordinator calls:
//! - `WriteBatch`: accept write batches from WriteRouter
//! - `ExecuteQuery`: execute sub-queries from DistributedQueryPlanner
//! - `FlushShard`: admin-triggered flush
//! - `GetReplicationOffset`: failover offset query
//!
//! See: docs/phase1_plan.md § rpc

use std::sync::Arc;

use arrow::record_batch::RecordBatch;

use crate::common::error::{Result, RutSeriError};
use crate::common::types::{IngestBatch, Row, ShardId};
use crate::ingest::engine::IngestEngine;
use crate::query::executor::QueryExecutor;
use crate::replication::manager::ReplicationManager;
use crate::rpc::proto::*;

/// Internal gRPC service running on each Storage Node.
///
/// Delegates to Phase 0 modules (IngestEngine, QueryExecutor)
/// without modifying them — Phase 1 wraps Phase 0, never changes it.
pub struct StorageNodeServer {
    /// Phase 0 ingest engine — handles writes.
    ingest: Arc<IngestEngine>,

    /// Phase 0 query executor — handles sub-queries.
    query: Arc<QueryExecutor>,

    /// Phase 1 replication manager — provides offset info.
    replication: Arc<ReplicationManager>,
}

impl StorageNodeServer {
    pub fn new(
        ingest: Arc<IngestEngine>,
        query: Arc<QueryExecutor>,
        replication: Arc<ReplicationManager>,
    ) -> Self {
        Self {
            ingest,
            query,
            replication,
        }
    }

    /// Start the internal gRPC server.
    ///
    /// Binds to the configured address and serves Coordinator requests.
    pub async fn serve(&self, _addr: &str) -> Result<()> {
        // TODO(engineer): implement tonic gRPC server
        //
        // tonic::transport::Server::builder()
        //     .add_service(storage_node_service_server(self))
        //     .serve(addr.parse()?)
        //     .await?;
        todo!("TODO(engineer): implement StorageNodeServer.serve")
    }

    /// Handle WriteBatch RPC.
    pub async fn handle_write_batch(
        &self,
        request: WriteBatchRequest,
    ) -> Result<WriteBatchResponse> {
        // Delegate to Phase 0 IngestEngine
        let batch = IngestBatch {
            table: request.table,
            rows: request.rows,
        };
        let row_count = batch.rows.len() as u64;

        self.ingest.ingest(batch).await?;

        Ok(WriteBatchResponse {
            rows_written: row_count,
        })
    }

    /// Handle ExecuteQuery RPC.
    pub async fn handle_execute_query(
        &self,
        request: ExecuteQueryRequest,
    ) -> Result<ExecuteQueryResponse> {
        // TODO(engineer): delegate to Phase 0 QueryExecutor
        //
        // let batches = self.query.execute(&request.sql).await?;
        // let ipc_data = serialize_arrow_ipc(&batches)?;
        // Ok(ExecuteQueryResponse {
        //     arrow_ipc_data: ipc_data,
        //     row_count: batches.iter().map(|b| b.num_rows() as u64).sum(),
        // })
        todo!("TODO(engineer): implement handle_execute_query")
    }

    /// Handle FlushShard RPC.
    pub async fn handle_flush_shard(
        &self,
        request: FlushShardRequest,
    ) -> Result<FlushShardResponse> {
        // TODO(engineer): call IngestEngine.flush_shard(request.shard_id)
        todo!("TODO(engineer): implement handle_flush_shard")
    }

    /// Handle GetReplicationOffset RPC.
    pub async fn handle_get_replication_offset(
        &self,
        request: GetReplicationOffsetRequest,
    ) -> Result<GetReplicationOffsetResponse> {
        let offset = self
            .replication
            .replication_offset(request.shard_id)
            .await;

        Ok(GetReplicationOffsetResponse {
            shard_id: request.shard_id,
            offset,
        })
    }
}
