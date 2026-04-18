//! HTTP server — starts axum and routes to handlers.
//!
//! Endpoints:
//! - POST /api/v1/write  → write_handler::handle_write
//! - POST /api/v1/query  → query_handler::handle_query
//! - GET  /health        → health check
//!
//! Does NOT implement business logic — delegates to IngestEngine / QueryEngine.

use std::sync::Arc;

use axum::{Router, routing::{get, post}};

use crate::ingest::engine::IngestEngine;

use super::query_handler;
use super::write_handler;

/// Shared application state passed to all handlers.
pub struct AppState {
    pub ingest_engine: IngestEngine,
    // TODO(engineer): add QueryEngine, shared schemas, etc.
}

/// Build the axum Router with all routes.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/write", post(write_handler::handle_write))
        .route("/api/v1/query", post(query_handler::handle_query))
        .route("/health", get(health_check))
        .with_state(state)
}

/// Start the HTTP server.
///
/// # Arguments
/// * `addr` — Listen address (e.g., "0.0.0.0:4000")
/// * `state` — Shared application state
pub async fn start(addr: &str, state: Arc<AppState>) -> crate::common::error::Result<()> {
    let router = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::common::error::RutSeriError::Internal(
            format!("Failed to bind to {addr}: {e}")
        ))?;

    tracing::info!("RutSeriDB listening on {addr}");

    axum::serve(listener, router)
        .await
        .map_err(|e| crate::common::error::RutSeriError::Internal(
            format!("Server error: {e}")
        ))?;

    Ok(())
}

/// Simple health check endpoint.
async fn health_check() -> &'static str {
    "OK"
}
