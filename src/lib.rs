//! RutSeriDB — a distributed time-series database written in Rust.
//!
//! This is the library root. It re-exports all modules for use by
//! `main.rs` and integration tests.
//!
//! NOTE: `allow(unused)` is set globally during skeleton phase.
//! Remove this once engineers start implementing the TODOs.
#![allow(unused)]

// ── Phase 0: Single-Node Core ────────────────────────────────────────
pub mod api;
pub mod background;
pub mod common;
pub mod config;
pub mod ingest;
pub mod query;
pub mod storage;

// ── Phase 1: Distribution ────────────────────────────────────────────
pub mod coordinator;
pub mod gossip;
pub mod raft;
pub mod replication;
pub mod rpc;

// ── Client Library ───────────────────────────────────────────────────
pub mod client;
