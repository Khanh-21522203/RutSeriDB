//! RutSeriDB client library — fluent, type-safe API for writes and queries.
//!
//! Instead of manually constructing JSON and calling HTTP endpoints,
//! users interact with RutSeriDB through builder methods:
//!
//! ```rust,no_run
//! let client = RutSeriClient::connect("http://localhost:4000").await?;
//!
//! // Write with builder — no SQL needed
//! client.write("metrics")
//!     .tag("host", "web-01")
//!     .field("cpu", 90.5)
//!     .timestamp(1_700_000_000_000_000_000)
//!     .send().await?;
//!
//! // Query with builder
//! let rows = client.query("metrics")
//!     .select(&["cpu", "memory"])
//!     .where_tag("host", "web-01")
//!     .execute().await?;
//!
//! // Raw SQL escape hatch
//! let rows = client.raw_sql("SELECT mean(cpu) FROM metrics")
//!     .execute().await?;
//! ```
//!
//! The client communicates over HTTP/JSON with the existing axum server.
//! No server changes are required.

pub mod query;
pub mod types;
pub mod write;

use std::time::Duration;

use reqwest::Client as HttpClient;

use crate::common::error::{Result, RutSeriError};

use self::query::QueryBuilder;
use self::types::QueryResult;
use self::write::{BatchBuilder, WriteBuilder};

/// Default timeout for HTTP requests.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// RutSeriDB client — the main entry point for interacting with the database.
///
/// Wraps an HTTP client and provides fluent builder methods for writes
/// and queries. Reuses connections via `reqwest`'s internal pool.
pub struct RutSeriClient {
    /// Base URL of the RutSeriDB server (e.g., `http://localhost:4000`).
    base_url: String,

    /// Reusable HTTP client with connection pooling.
    http: HttpClient,
}

impl RutSeriClient {
    /// Connect to a RutSeriDB server.
    ///
    /// Validates connectivity by calling the `/health` endpoint.
    ///
    /// # Arguments
    /// * `url` — Base URL (e.g., `"http://localhost:4000"`)
    ///
    /// # Example
    /// ```rust,no_run
    /// let client = RutSeriClient::connect("http://localhost:4000").await?;
    /// ```
    pub async fn connect(url: &str) -> Result<Self> {
        let base_url = url.trim_end_matches('/').to_string();

        let http = HttpClient::builder()
            .timeout(DEFAULT_TIMEOUT)
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| RutSeriError::Internal(format!("HTTP client error: {e}")))?;

        // Health check — verify the server is reachable.
        let health_url = format!("{base_url}/health");
        http.get(&health_url)
            .send()
            .await
            .map_err(|e| {
                RutSeriError::Internal(format!(
                    "Cannot connect to RutSeriDB at {base_url}: {e}"
                ))
            })?;

        Ok(Self { base_url, http })
    }

    /// Create a client without a health check (useful for testing or
    /// when the server may not be running yet).
    pub fn new(url: &str) -> Result<Self> {
        let base_url = url.trim_end_matches('/').to_string();

        let http = HttpClient::builder()
            .timeout(DEFAULT_TIMEOUT)
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| RutSeriError::Internal(format!("HTTP client error: {e}")))?;

        Ok(Self { base_url, http })
    }

    // ── Write Builders ──────────────────────────────────────────────

    /// Start building a single write to a table.
    ///
    /// # Example
    /// ```rust,no_run
    /// client.write("metrics")
    ///     .tag("host", "web-01")
    ///     .field("cpu", 90.5)
    ///     .timestamp_now()
    ///     .send().await?;
    /// ```
    pub fn write<'a>(&'a self, table: &str) -> WriteBuilder<'a> {
        WriteBuilder::new(self, table.to_string())
    }

    /// Start building a batch of writes to a table.
    ///
    /// Amortises the HTTP round-trip by sending multiple rows in one request.
    ///
    /// # Example
    /// ```rust,no_run
    /// client.batch("metrics")
    ///     .row(|r| r.tag("host", "web-01").field("cpu", 90.5).timestamp_now())
    ///     .row(|r| r.tag("host", "web-02").field("cpu", 45.2).timestamp_now())
    ///     .send().await?;
    /// ```
    pub fn batch<'a>(&'a self, table: &str) -> BatchBuilder<'a> {
        BatchBuilder::new(self, table.to_string())
    }

    // ── Query Builders ──────────────────────────────────────────────

    /// Start building a query against a table.
    ///
    /// # Example
    /// ```rust,no_run
    /// let result = client.query("metrics")
    ///     .select(&["cpu", "memory"])
    ///     .where_tag("host", "web-01")
    ///     .time_range(start, end)
    ///     .execute().await?;
    /// ```
    pub fn query<'a>(&'a self, table: &str) -> QueryBuilder<'a> {
        QueryBuilder::new(self, table.to_string())
    }

    /// Execute a raw SQL query.
    ///
    /// Escape hatch for complex queries that the builder cannot express.
    ///
    /// # Example
    /// ```rust,no_run
    /// let result = client.raw_sql("SELECT mean(cpu) FROM metrics WHERE host='web-01'")
    ///     .execute().await?;
    /// ```
    pub fn raw_sql<'a>(&'a self, sql: &str) -> QueryBuilder<'a> {
        QueryBuilder::raw(self, sql.to_string())
    }

    // ── Internal ────────────────────────────────────────────────────

    /// Base URL of the connected server.
    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Shared HTTP client.
    pub(crate) fn http(&self) -> &HttpClient {
        &self.http
    }
}
