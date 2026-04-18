//! Query handler — HTTP endpoint for executing SQL queries.
//!
//! POST /api/v1/query
//! Body: JSON { "sql": "SELECT ..." }
//! Response: JSON array of Arrow RecordBatch (serialized as JSON rows)
//!
//! Does NOT implement query logic — delegates to QueryEngine.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::server::AppState;

/// Request body for a query.
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub sql: String,
}

/// Response body for a query.
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    /// JSON-serialized rows. In Phase 1 this will be Arrow IPC.
    pub rows: Vec<serde_json::Value>,
    /// Number of rows returned.
    pub row_count: usize,
}

/// Handle a query request.
///
/// Parses the SQL query, executes it via the QueryEngine, and returns
/// the results as JSON.
pub async fn handle_query(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, String)> {
    // TODO(engineer): implement
    //
    // 1. Parse SQL: let ast = query::parser::parse(&request.sql)?;
    // 2. Plan: let plan = query::planner::plan(ast, &catalog, &shard_dir)?;
    // 3. Execute: let batches = query::executor::execute(&plan, memtable_snap)?;
    // 4. Convert Arrow RecordBatches to JSON rows
    //    (use arrow::json::writer for serialization)
    // 5. Return QueryResponse { rows, row_count }

    Err((
        StatusCode::NOT_IMPLEMENTED,
        "Query endpoint not yet implemented".to_string(),
    ))
}
