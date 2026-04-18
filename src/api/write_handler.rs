//! Write handler — HTTP endpoint for ingesting time-series data.
//!
//! POST /api/v1/write
//! Body: JSON IngestBatch
//!
//! Does NOT implement ingest logic — delegates to IngestEngine.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::common::types::IngestBatch;

use super::server::AppState;

/// Handle a write request.
///
/// Deserializes the JSON body into an `IngestBatch`, passes it to
/// the `IngestEngine`, and returns 200 OK on success.
pub async fn handle_write(
    State(state): State<Arc<AppState>>,
    Json(batch): Json<IngestBatch>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .ingest_engine
        .ingest(batch)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            tracing::error!("Ingest error: {e}");
            (StatusCode::BAD_REQUEST, e.to_string())
        })
}
