//! AST node types — our internal representation of parsed SQL.
//!
//! We wrap `sqlparser`'s AST with our own types to:
//! 1. Control exactly which SQL features are supported
//! 2. Decouple our planner/executor from the parser crate's API
//! 3. Add TSDB-specific nodes (e.g., time range predicates)

/// A parsed SELECT query in our internal AST.
#[derive(Debug, Clone)]
pub struct SelectQuery {
    /// Target table name.
    pub table: String,

    /// Columns to project. Empty = all columns (`SELECT *`).
    pub projection: Vec<Projection>,

    /// WHERE clause filters.
    pub filters: Vec<Filter>,

    /// GROUP BY column names.
    pub group_by: Vec<String>,

    /// Aggregation functions to compute.
    pub aggregations: Vec<Aggregation>,

    /// ORDER BY clause.
    pub order_by: Vec<OrderBy>,

    /// LIMIT clause. None = no limit.
    pub limit: Option<usize>,
}

/// A projected column or expression.
#[derive(Debug, Clone)]
pub enum Projection {
    /// A raw column name (e.g., `SELECT host, cpu`).
    Column(String),

    /// An aggregate function (e.g., `SELECT mean(cpu)`).
    Agg(Aggregation),

    /// Wildcard (`SELECT *`).
    Star,
}

/// A filter predicate from the WHERE clause.
#[derive(Debug, Clone)]
pub enum Filter {
    /// `column > value` or `column >= value`
    GreaterThan {
        column: String,
        value: LiteralValue,
        inclusive: bool,
    },
    /// `column < value` or `column <= value`
    LessThan {
        column: String,
        value: LiteralValue,
        inclusive: bool,
    },
    /// `column = value`
    Equals {
        column: String,
        value: LiteralValue,
    },
    /// `column BETWEEN low AND high`
    Between {
        column: String,
        low: LiteralValue,
        high: LiteralValue,
    },
}

/// An aggregation function.
#[derive(Debug, Clone)]
pub struct Aggregation {
    /// Function name: sum, count, min, max, mean.
    pub func: AggFunc,
    /// Column to aggregate.
    pub column: String,
    /// Optional alias: `SELECT mean(cpu) AS avg_cpu`.
    pub alias: Option<String>,
}

/// Supported aggregation functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggFunc {
    Sum,
    Count,
    Min,
    Max,
    Mean,
}

/// A literal value in a SQL expression.
#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

/// ORDER BY clause item.
#[derive(Debug, Clone)]
pub struct OrderBy {
    pub column: String,
    pub descending: bool,
}
