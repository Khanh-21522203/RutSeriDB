//! Query builder — fluent API for querying time-series data.
//!
//! `QueryBuilder` constructs SQL from builder methods and POSTs to
//! `/api/v1/query`. Also supports raw SQL via `client.raw_sql(...)`.

use crate::common::error::{Result, RutSeriError};
use crate::common::types::Timestamp;

use super::types::QueryResult;
use super::RutSeriClient;

/// Builder for constructing a query against a table.
///
/// # Example — Builder style
/// ```rust,no_run
/// let result = client.query("metrics")
///     .select(&["cpu_usage", "memory_mb"])
///     .where_tag("host", "web-01")
///     .where_field_gt("cpu_usage", 50.0)
///     .time_range(start_ns, end_ns)
///     .order_by_time_desc()
///     .limit(100)
///     .execute().await?;
/// ```
///
/// # Example — Raw SQL
/// ```rust,no_run
/// let result = client.raw_sql("SELECT mean(cpu) FROM metrics WHERE host='web-01'")
///     .execute().await?;
/// ```
pub struct QueryBuilder<'a> {
    client: &'a RutSeriClient,

    /// If set, use this exact SQL instead of building from parts.
    raw_sql: Option<String>,

    /// Target table.
    table: String,

    /// Columns to SELECT. Empty = `*`.
    columns: Vec<String>,

    /// WHERE clauses (ANDed together).
    where_clauses: Vec<String>,

    /// Aggregation function (e.g., `mean`, `sum`, `count`).
    aggregate: Option<(String, String)>, // (func, column)

    /// GROUP BY TIME interval.
    group_by_time: Option<String>,

    /// ORDER BY clause.
    order_by: Option<String>,

    /// LIMIT.
    limit: Option<usize>,
}

impl<'a> QueryBuilder<'a> {
    /// Create a builder for a table query.
    pub(crate) fn new(client: &'a RutSeriClient, table: String) -> Self {
        Self {
            client,
            raw_sql: None,
            table,
            columns: Vec::new(),
            where_clauses: Vec::new(),
            aggregate: None,
            group_by_time: None,
            order_by: None,
            limit: None,
        }
    }

    /// Create a builder with raw SQL (bypass builder).
    pub(crate) fn raw(client: &'a RutSeriClient, sql: String) -> Self {
        Self {
            client,
            raw_sql: Some(sql),
            table: String::new(),
            columns: Vec::new(),
            where_clauses: Vec::new(),
            aggregate: None,
            group_by_time: None,
            order_by: None,
            limit: None,
        }
    }

    // ── SELECT ──────────────────────────────────────────────────────

    /// Select specific columns. If not called, selects `*`.
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    // ── WHERE — Tags ────────────────────────────────────────────────

    /// Filter by exact tag value: `WHERE tag_key = 'value'`.
    pub fn where_tag(mut self, key: &str, value: &str) -> Self {
        self.where_clauses
            .push(format!("{key} = '{}'", escape_sql_string(value)));
        self
    }

    /// Filter by tag not equal: `WHERE tag_key != 'value'`.
    pub fn where_tag_ne(mut self, key: &str, value: &str) -> Self {
        self.where_clauses
            .push(format!("{key} != '{}'", escape_sql_string(value)));
        self
    }

    /// Filter by tag matching a regex pattern: `WHERE tag_key =~ 'pattern'`.
    pub fn where_tag_regex(mut self, key: &str, pattern: &str) -> Self {
        self.where_clauses
            .push(format!("{key} =~ '{}'", escape_sql_string(pattern)));
        self
    }

    // ── WHERE — Fields ──────────────────────────────────────────────

    /// Filter: field > value.
    pub fn where_field_gt(mut self, field: &str, value: f64) -> Self {
        self.where_clauses.push(format!("{field} > {value}"));
        self
    }

    /// Filter: field < value.
    pub fn where_field_lt(mut self, field: &str, value: f64) -> Self {
        self.where_clauses.push(format!("{field} < {value}"));
        self
    }

    /// Filter: field >= value.
    pub fn where_field_gte(mut self, field: &str, value: f64) -> Self {
        self.where_clauses.push(format!("{field} >= {value}"));
        self
    }

    /// Filter: field <= value.
    pub fn where_field_lte(mut self, field: &str, value: f64) -> Self {
        self.where_clauses.push(format!("{field} <= {value}"));
        self
    }

    /// Filter: field = value (numeric).
    pub fn where_field_eq(mut self, field: &str, value: f64) -> Self {
        self.where_clauses.push(format!("{field} = {value}"));
        self
    }

    // ── WHERE — Time Range ──────────────────────────────────────────

    /// Filter by time range (nanoseconds since epoch).
    ///
    /// Generates: `WHERE timestamp >= start AND timestamp <= end`
    pub fn time_range(mut self, start_ns: Timestamp, end_ns: Timestamp) -> Self {
        self.where_clauses
            .push(format!("timestamp >= {start_ns}"));
        self.where_clauses.push(format!("timestamp <= {end_ns}"));
        self
    }

    /// Filter: timestamp after a given point.
    pub fn time_after(mut self, ns: Timestamp) -> Self {
        self.where_clauses.push(format!("timestamp > {ns}"));
        self
    }

    /// Filter: timestamp before a given point.
    pub fn time_before(mut self, ns: Timestamp) -> Self {
        self.where_clauses.push(format!("timestamp < {ns}"));
        self
    }

    // ── Aggregation ─────────────────────────────────────────────────

    /// Apply an aggregation function.
    ///
    /// # Example
    /// `.aggregate("mean", "cpu_usage")` → `SELECT mean(cpu_usage) FROM ...`
    pub fn aggregate(mut self, func: &str, column: &str) -> Self {
        self.aggregate = Some((func.to_string(), column.to_string()));
        self
    }

    /// Shorthand: `SELECT mean(column) FROM ...`
    pub fn mean(self, column: &str) -> Self {
        self.aggregate("mean", column)
    }

    /// Shorthand: `SELECT sum(column) FROM ...`
    pub fn sum(self, column: &str) -> Self {
        self.aggregate("sum", column)
    }

    /// Shorthand: `SELECT count(column) FROM ...`
    pub fn count(self, column: &str) -> Self {
        self.aggregate("count", column)
    }

    /// Shorthand: `SELECT min(column) FROM ...`
    pub fn min(self, column: &str) -> Self {
        self.aggregate("min", column)
    }

    /// Shorthand: `SELECT max(column) FROM ...`
    pub fn max(self, column: &str) -> Self {
        self.aggregate("max", column)
    }

    // ── GROUP BY ────────────────────────────────────────────────────

    /// Group results by time interval.
    ///
    /// # Example
    /// `.group_by_time("5m")` → `GROUP BY time(5m)`
    pub fn group_by_time(mut self, interval: &str) -> Self {
        self.group_by_time = Some(interval.to_string());
        self
    }

    // ── ORDER BY ────────────────────────────────────────────────────

    /// Order results by timestamp ascending.
    pub fn order_by_time_asc(mut self) -> Self {
        self.order_by = Some("timestamp ASC".to_string());
        self
    }

    /// Order results by timestamp descending.
    pub fn order_by_time_desc(mut self) -> Self {
        self.order_by = Some("timestamp DESC".to_string());
        self
    }

    // ── LIMIT ───────────────────────────────────────────────────────

    /// Limit the number of returned rows.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    // ── Build SQL ───────────────────────────────────────────────────

    /// Build the SQL string from builder state.
    fn build_sql(&self) -> String {
        if let Some(ref sql) = self.raw_sql {
            return sql.clone();
        }

        // SELECT clause
        let select = if let Some((ref func, ref col)) = self.aggregate {
            format!("{func}({col})")
        } else if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut sql = format!("SELECT {select} FROM {}", self.table);

        // WHERE clause
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        // GROUP BY
        if let Some(ref interval) = self.group_by_time {
            sql.push_str(&format!(" GROUP BY time({interval})"));
        }

        // ORDER BY
        if let Some(ref order) = self.order_by {
            sql.push_str(&format!(" ORDER BY {order}"));
        }

        // LIMIT
        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {n}"));
        }

        sql
    }

    // ── Execute ─────────────────────────────────────────────────────

    /// Execute the query and return results.
    pub async fn execute(self) -> Result<QueryResult> {
        let sql = self.build_sql();

        let url = format!("{}/api/v1/query", self.client.base_url());
        let body = serde_json::json!({ "sql": sql });

        let response = self
            .client
            .http()
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RutSeriError::Internal(format!("Query request failed: {e}")))?;

        if response.status().is_success() {
            let result: QueryResult = response
                .json()
                .await
                .map_err(|e| RutSeriError::Internal(format!("Failed to parse response: {e}")))?;
            Ok(result)
        } else {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            Err(RutSeriError::Internal(format!(
                "Query failed ({status}): {body}"
            )))
        }
    }
}

/// Escape single quotes in SQL string literals.
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}
