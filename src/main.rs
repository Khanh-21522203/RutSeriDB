//! RutSeriDB — binary entry point.
//!
//! Parses CLI arguments, loads configuration, initializes all
//! components, and starts the HTTP server.

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;

use rutseridb::api::server::{self, AppState};
use rutseridb::config::config::RutSeriConfig;

/// RutSeriDB — distributed time-series database.
#[derive(Parser, Debug)]
#[command(name = "rutseridb", version, about)]
struct Cli {
    /// Path to the TOML configuration file.
    #[arg(short, long, default_value = "rutseridb.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    tracing::info!("Starting RutSeriDB...");

    // Load configuration
    // TODO(engineer): implement once config file exists
    // let config = RutSeriConfig::load(&cli.config)?;

    // TODO(engineer): Initialize components in order:
    //
    // 1. Create shard data directories
    //    for shard_id in 0..config.cluster.num_shards {
    //        create_dir_all(data_dir/shard-{id}/{wal,parts,catalog})?;
    //    }
    //
    // 2. WAL recovery: replay WAL for each shard
    //    for shard_id in 0..config.cluster.num_shards {
    //        let wal_dir = data_dir/shard-{id}/wal;
    //        let memtable = WalReader::replay(wal_dir, |seq, entry| {
    //            memtable.insert(entry.rows);
    //        })?;
    //    }
    //
    // 3. Load Catalogs from disk
    //    let catalogs = load each shard's catalog.json
    //
    // 4. Create WalWriters for each shard
    //
    // 5. Spawn ShardActors (one per shard)
    //    let handles = ShardActor::spawn(...) for each shard
    //
    // 6. Create IngestEngine with all shard handles
    //
    // 7. Spawn background workers
    //    - MergeWorker
    //    - WALCleanup
    //    - IndexBuilder
    //    - MetricsReporter
    //
    // 8. Start HTTP server
    //    let state = Arc::new(AppState { ingest_engine });
    //    server::start(&config.cluster.advertise_addr, state).await?;

    tracing::info!("RutSeriDB ready");

    // Placeholder: just print hello for now
    println!("RutSeriDB — skeleton ready. Implement the TODOs!");

    Ok(())
}
