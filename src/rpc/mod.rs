//! RPC module — node-to-node communication layer.
//!
//! Provides:
//! - `proto`: Hand-defined message types (no protoc dependency)
//! - `client`: Typed gRPC client wrappers for StorageNode calls
//! - `server`: Internal gRPC service implementation on StorageNodes
//!
//! See: docs/phase1_plan.md § rpc

pub mod client;
pub mod proto;
pub mod server;
