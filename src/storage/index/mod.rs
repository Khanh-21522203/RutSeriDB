//! Index structures — MinMax, Bloom Filters, Inverted Index.
//!
//! All indexes are read-path optimizations only — they never affect
//! write throughput or Part immutability.

pub mod bloom;
pub mod inverted;
pub mod minmax;
