//! SQL parser — converts SQL strings to our internal AST.
//!
//! Uses the `sqlparser` crate for correctness, then translates
//! into our own AST types (see `query::ast`).
//!
//! Supported SQL subset (v1):
//! - SELECT ... FROM table WHERE ... GROUP BY ... ORDER BY ... LIMIT
//! - Aggregates: SUM, COUNT, MIN, MAX, MEAN (alias for AVG)
//! - WHERE: comparisons, BETWEEN, AND

use crate::common::error::{Result, RutSeriError};

use super::ast::SelectQuery;

/// Parse a SQL string into our internal AST.
///
/// # Supported syntax (v1)
/// ```sql
/// SELECT [columns | aggregates | *]
/// FROM table_name
/// WHERE [predicates]
/// GROUP BY [columns]
/// ORDER BY [column ASC|DESC]
/// LIMIT n
/// ```
///
/// # Errors
/// Returns `RutSeriError::QueryParse` if the SQL is invalid or
/// uses unsupported features (JOIN, subqueries, etc.).
pub fn parse(sql: &str) -> Result<SelectQuery> {
    // TODO(engineer): implement using sqlparser crate
    //
    // use sqlparser::dialect::GenericDialect;
    // use sqlparser::parser::Parser;
    //
    // Step 1: Parse SQL using sqlparser
    //   let dialect = GenericDialect {};
    //   let ast = Parser::parse_sql(&dialect, sql)
    //       .map_err(|e| RutSeriError::QueryParse(e.to_string()))?;
    //
    // Step 2: Validate we got exactly one SELECT statement
    //   let statement = ast.into_iter().next()
    //       .ok_or(RutSeriError::QueryParse("Empty query".into()))?;
    //
    // Step 3: Extract table name from FROM clause
    //
    // Step 4: Convert WHERE clause to Vec<Filter>
    //   - Binary ops (>, <, >=, <=, =) → Filter variants
    //   - BETWEEN → Filter::Between
    //   - AND → flatten into multiple filters
    //
    // Step 5: Convert SELECT items
    //   - Column refs → Projection::Column
    //   - Function calls (SUM, COUNT, etc.) → Projection::Agg
    //   - * → Projection::Star
    //
    // Step 6: Extract GROUP BY, ORDER BY, LIMIT
    //
    // Step 7: Return SelectQuery

    todo!("parse SQL")
}
