# RutSeriDB — Architecture (C4 Model)

> **Version:** 0.1 (Draft) · **Last Updated:** 2026-04-17 · **Status:** In Review

This document describes the architecture of **RutSeriDB**, a distributed, scalable time-series database built in Rust. It follows the [C4 model](https://c4model.com/) (Context → Containers → Components → Code) and covers key design decisions, invariants, data flows, and concurrency strategy.

---

## Table of Contents

1. [Goals & Non-Goals](#goals--non-goals)
2. [C1 — System Context](#c1--system-context)
3. [C2 — Container Diagram](#c2--container-diagram)
4. [C3 — Component Diagrams](#c3--component-diagrams)
5. [Data Flows](#data-flows)
6. [Core Invariants](#core-invariants)
7. [Indexing](#indexing)
8. [Concurrency & Threading Model](#concurrency--threading-model)
9. [Memory Management](#memory-management)
10. [Durability & Recovery](#durability--recovery)
11. [Cluster Management](#cluster-management)
12. [Configuration Reference](#configuration-reference)
13. [Design Decision Log](#design-decision-log)
14. [Implementation Checklist](#implementation-checklist)
15. [Open Questions](#open-questions)
16. [Future Work](#future-work)

---

## Goals & Non-Goals

### Goals (v1)

| # | Goal |
|---|------|
| G1 | Horizontal scalability — ingestion and storage scale with more nodes |
| G2 | High write throughput with configurable durability |
| G3 | Efficient time-range and tag-based queries |
| G4 | Fault tolerance — cluster survives minority node failures |
| G5 | SQL-like query surface over time-series data |
| G6 | Operational simplicity — single binary per role, no external dependencies |

### Non-Goals (v1)

| # | Non-Goal | Reason |
|---|----------|--------|
| NG1 | Full SQL compatibility | TSDBs have domain-specific query patterns |
| NG2 | Multi-tenant isolation | Single database instance per cluster |
| NG3 | Arbitrary user-defined secondary indexes | B-tree / hash indexes on arbitrary field columns are out of scope; supported indexes are listed in [storage/indexes.md](./storage/indexes.md) |
| NG4 | ACID transactions across shards | Eventual consistency on replicas is acceptable |
| NG5 | Kubernetes operator / cloud-native autoscaling | Out of scope for v1 |

---

## C1 — System Context

Who interacts with RutSeriDB and what external systems does it depend on?

```mermaid
C4Context
    title System Context — RutSeriDB

    Person(producer, "Time-Series Producer", "IoT devices, microservices, metrics agents")
    Person(operator, "Operator / Analytics", "Grafana, Prometheus scraper, admin CLI")

    System(rutseridb, "RutSeriDB Cluster", "Distributed, multi-node time-series database written in Rust")

    System_Ext(storage, "File System / Object Store", "Local disk, NFS, or S3 — stores WAL segments and Part files")

    Rel(producer, rutseridb, "Write time-series batches", "gRPC / HTTP")
    Rel(operator, rutseridb, "Query, admin, metrics scrape", "gRPC / HTTP / Prometheus")
    Rel(rutseridb, storage, "Persist WAL + Part files", "POSIX fs / S3 API")

    UpdateLayoutConfig($c4ShapeInRow="2", $c4BoundaryInRow="1")
```

### External Interfaces

| Interface | Protocol | Direction | Description |
|-----------|----------|-----------|-------------|
| Write API | gRPC / HTTP/2 | Inbound | Batch ingest of time-series rows |
| Query API | gRPC / HTTP/2 | Inbound | SQL-like queries, streaming results |
| Admin API | HTTP REST | Inbound | Cluster management, health, schema ops |
| Metrics Export | Prometheus text | Outbound | Internal observability metrics |
| Storage | POSIX fs / S3 | Outbound | Durable WAL segments + columnar Part files |

---

## C2 — Container Diagram

A **container** is a deployable unit. RutSeriDB uses three roles, all compiled from one binary (`rutseridb --role=…`).

```mermaid
C4Container
    title Container Diagram — RutSeriDB Cluster

    Person(client, "Client", "Application or service writing/querying data")
    Person(admin, "Operator", "Grafana, admin CLI, Prometheus")

    System_Boundary(cluster, "RutSeriDB Cluster") {
        Container(coord, "Coordinator Node", "Rust · Tokio · Raft", "Routes writes to shard leaders. Distributes queries across storage nodes. Manages cluster metadata via Raft.")
        Container(leader, "Storage Node — Leader", "Rust · Tokio", "Runs Ingest Engine, WAL, MemTable, Part file writer, local Query Engine, and Replication Manager.")
        Container(replica, "Storage Node — Replica", "Rust · Tokio", "Applies replicated WAL from the leader. Acts as failover target when the leader fails.")
    }

    System_Ext(fs, "File System", "Durable storage")

    Rel(client, coord, "Write / Query", "gRPC / HTTP")
    Rel(admin, coord, "Cluster management", "HTTP REST")
    Rel(coord, leader, "Route write · fan-out sub-query", "gRPC (internal)")
    Rel(leader, replica, "Stream WAL entries", "gRPC bidirectional stream")
    Rel(leader, fs, "Persist WAL + Parts", "POSIX fs")
    Rel(replica, fs, "Persist replicated Parts", "POSIX fs")

    UpdateLayoutConfig($c4ShapeInRow="3", $c4BoundaryInRow="1")
```

### Container Summary

| Container | Role | Instances | Notes |
|-----------|------|-----------|-------|
| **Coordinator** | Routing, catalog, query fan-out | 1–3 (odd, for Raft quorum) | Raft group replicates metadata only |
| **Storage Node** | Ingest, store, replicate, query locally | ≥ 1 per shard | Each shard has 1 leader + N replicas |
| **Dev mode** | All roles in one process | 1 | `--role=dev` for local development |

---

## C3 — Component Diagrams

### Coordinator Node

```mermaid
flowchart TB
    GW["🌐 API Gateway\ngRPC + HTTP\n──────────────\n• Rate limiting\n• AuthN (future)\n• Request routing"]

    WR["📍 Write Router\n──────────────\n• Shard key computation\n• Leader node discovery\n• Timeout / retry"]

    QP["🔍 Distributed Query Planner\n──────────────\n• Parse SQL → AST\n• Shard pruning by time range\n• Fan-out sub-queries\n• Merge + sort results"]

    MC["🗂 Metadata Catalog\n──────────────\n• Table schemas\n• Shard → node mapping\n• Partition time ranges\n• Raft-replicated"]

    CM["🩺 Cluster Manager\n──────────────\n• Node heartbeat tracking\n• Leader election\n• Shard rebalancing\n• Node join / leave"]

    GW --> WR
    GW --> QP
    WR --> MC
    QP --> MC
    CM <--> MC
```

---

### Storage Node

```mermaid
flowchart TB
    API["🔌 Node API\ngRPC server\n──────────────\n• Accept writes from Coordinator\n• Accept sub-queries from Coordinator"]

    IE["⚙️ Ingest Engine\n──────────────\n1. Schema validation\n2. WAL append\n3. MemTable insert\n4. Flush trigger\n5. Backpressure control"]

    RM["🔁 Replication Manager\n──────────────\n• Stream WAL → replicas\n• Track replica lag\n• Re-sync on replica join"]

    SE["💾 Storage Engine"]
    MT["MemTable\nin-memory sorted BTree"]
    PF[".rpart Files\ncolumnar · compressed\nimmutable"]
    IDX["Index\ntime range + tag bloom"]
    CAT["Catalog\nJSON · atomic replace"]

    LQE["🔍 Local Query Engine\nParser → Planner → Executor"]

    BG["🔧 Background Workers"]
    MW["Merge Worker"]
    WC["WAL Cleanup"]
    MR["Metrics Reporter"]

    FS[("📁 File System\n/data/shard-id/\n  wal/  parts/  catalog/")]

    API --> IE
    API --> LQE
    IE --> RM
    IE --> SE
    SE --- MT
    SE --- PF
    SE --- IDX
    SE --- CAT
    LQE --> SE
    SE --> BG
    BG --> MW & WC & MR
    SE --> FS
```

---

### Query Engine — Distributed Execution Model

```mermaid
flowchart LR
    Client(["Client"])

    subgraph Coordinator["Coordinator"]
        direction TB
        Parse1["Parser"]
        Analyze["Analyzer\nglobal scope"]
        Merge["Merger\nfinal sort · dedup · aggregation"]
        Parse1 --> Analyze --> Merge
    end

    subgraph SN["Storage Node ×N"]
        direction TB
        Parse2["Parser"]
        Plan["Planner\nlocal Part pruning"]
        Exec["Executor\nScan → Filter → Projection / Agg"]
        Parse2 --> Plan --> Exec
    end

    Client --> Parse1
    Analyze -- "sub-plan per shard" --> Parse2
    Exec -- "partial ResultSet\n(Arrow RecordBatch)" --> Merge
    Merge -- "stream results" --> Client
```

---

## Data Flows

### Ingestion Path

```mermaid
sequenceDiagram
    participant C  as Client
    participant CO as Coordinator
    participant SN as Storage Node (Leader)
    participant R  as Replica(s)

    C->>CO: IngestBatch(table, rows)
    CO->>CO: shard_key = hash(primary_tags) % num_shards
    CO->>CO: Lookup leader node for shard
    CO->>SN: Forward batch (internal gRPC)

    SN->>SN: Validate schema
    SN->>SN: Append to WAL (fsync per durability config)
    SN->>SN: Insert rows into MemTable

    alt MemTable size > threshold
        SN->>SN: Flush MemTable → .rpart file (columnar, compressed)
        SN->>SN: Update Catalog (atomic: write-tmp → rename)
    end

    SN-->>R: Stream WAL entries (async replication)
    SN-->>CO: OK
    CO-->>C: OK
```

**Steps in detail:**

| Step | Description |
|------|-------------|
| 1. Shard routing | `shard_key = hash(primary_tags) % num_shards`; resolve leader |
| 2. Schema validation | Check column types, required primary tags |
| 3. WAL append | Serialize batch; write to WAL; fsync per durability level |
| 4. MemTable insert | Insert rows into in-memory sorted structure by `(timestamp, tag_hash)` |
| 5. Flush decision | Triggered when MemTable bytes > threshold (default 64 MB) |
| 6. Part creation | Columnar `.rpart` file written (compressed, immutable) via atomic rename |
| 7. Catalog update | Atomic entry added for the new Part |
| 8. Replication | WAL tail streamed asynchronously to replica nodes |
| 9. Acknowledge | `OK` returned to coordinator → client |

---

### Query Path

```mermaid
sequenceDiagram
    participant C   as Client
    participant CO  as Coordinator
    participant SN1 as Storage Node A
    participant SN2 as Storage Node B

    C->>CO: Query SQL
    CO->>CO: Parse SQL → AST
    CO->>CO: Analyze — resolve table, column refs, types
    CO->>CO: Plan — identify relevant shards
    CO->>CO: Prune — eliminate shards outside WHERE time range

    par Fan-out sub-queries in parallel
        CO->>SN1: Sub-query (shard A)
        CO->>SN2: Sub-query (shard B)
    end

    SN1->>SN1: Prune local Parts via min/max index
    SN1->>SN1: Scan columns · apply filters · local aggregation
    SN2->>SN2: Prune local Parts via min/max index
    SN2->>SN2: Scan columns · apply filters · local aggregation

    SN1-->>CO: Partial ResultSet (Arrow RecordBatch)
    SN2-->>CO: Partial ResultSet (Arrow RecordBatch)

    CO->>CO: Merge · final aggregation · sort · apply LIMIT
    CO-->>C: ResultSet (Arrow IPC stream)
```

---

### Replication Flow

```mermaid
sequenceDiagram
    participant L  as Storage Node (Leader)
    participant R  as Storage Node (Replica)

    Note over L,R: Normal streaming (ongoing)
    L->>L: Write committed to WAL (seq = N)
    L->>R: ReplicateWAL(offset=N, entries=[...])
    R->>R: Apply WAL entries to local MemTable
    R-->>L: ACK(offset=N)

    Note over L,R: Replica re-join after lag
    R->>L: SnapshotRequest(shard_id)
    L-->>R: SnapshotStart(catalog, seq=K)
    L-->>R: Stream .rpart files (HTTP chunked)
    L-->>R: SnapshotEnd(catalog_ver=V)
    R->>L: StreamWal(shard, from_seq=K+1)
    Note over L,R: Resume normal streaming
```

---

## Core Invariants

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| I1 | **Timestamp ordering** — within a single Part, rows sorted by `timestamp` ascending | MemTable maintains sorted order; Part writer verifies on flush |
| I2 | **Part immutability** — once written, a Part file is never modified | Parts written atomically (write-tmp → rename); no update API |
| I3 | **WAL-before-acknowledge** — no write ACK'd until WAL persisted on leader | Shard Actor sends `oneshot::send(OK)` only after WAL fsync completes |
| I4 | **Single writer per shard** — at most one task mutates a shard's MemTable at a time | Per-shard `mpsc` dispatch queue + dedicated Shard Actor task (replaces Mutex) |
| I5 | **Catalog consistency** — catalog reflects all committed Parts | Catalog update is final step of flush; atomic file replace |
| I6 | **Shard key stability** — a row's shard never changes after first write | Shard key function is frozen at cluster creation |
| I7 | **Monotone replication offset** — replica's applied offset never decreases | WAL protocol rejects out-of-order entries |

---

## Indexing

RutSeriDB supports three index types. All are **read-path only** — they never affect write throughput or Part immutability. See [storage/indexes.md](./storage/indexes.md) for full specifications.

```mermaid
flowchart TB
    subgraph FileLevel["File-level — stored inside each .rpart"]
        MM["1️⃣ Min/Max Index\nCovers: all columns — timestamps, tags, and field values\nUse: skip files outside a value range\nCost: O(1) lookup from file footer"]
        BF["2️⃣ Bloom Filters\nCovers: tag columns + low-cardinality field columns\nUse: skip files that definitely lack an equality value\nFalse positive rate ≤ 1%"]
    end

    subgraph CatalogLevel["Catalog-level — stored in Catalog JSON"]
        INV["3️⃣ Inverted Index  (tag → Part IDs)\nCovers: all tag key/value pairs per table\nUse: retrieve exact Part list for a tag value\nwithout scanning every Part's bloom filter\nBuilt/maintained by Index Builder background worker"]
    end
```

### Index Application Order in the Query Planner

```mermaid
flowchart LR
    A["All Parts\n(Catalog)"] -->|"3️⃣ Inverted Index\ntag equality"| B["Candidate Parts"]
    B -->|"1️⃣ Min/Max\nrange predicates"| C["Surviving Parts"]
    C -->|"2️⃣ Bloom Filter\nremaining equality"| D["Final Parts"]
    D -->|"Column scan\nrow-level filter"| E["Result"]
```

---

## Concurrency & Threading Model

### Per-Node Thread Architecture

| Thread / Pool | Count | Responsibility |
|---------------|-------|----------------|
| **Async Runtime** (Tokio) | `num_cpus` workers | Accept connections, orchestrate async tasks; client handlers park at `rx.await` |
| **Shard Actor** (per shard) | 1 Tokio task per shard | Drains dispatch queue, coalesces batches, WAL append+fsync, MemTable insert, fires oneshot ACKs |
| **Blocking I/O Pool** | `num_cpus / 2` | Part file reads/writes (`spawn_blocking`) |
| **Replication** | 1 per replica peer | WAL streaming, ACK handling |
| **Background** | 1 | Merge, WAL cleanup, metrics, Index Builder |

### Design Options Considered

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| **Actor task + oneshot per request** | Zero blocking; natural group commit; cancellation-safe | Slightly more complex dispatch routing | ✅ v1 |
| Mutex per shard | Simple | Blocks Tokio thread during WAL fsync (~1ms) | ❌ Replaced |
| Sharded fine-grained writers | Higher write throughput | Complex partitioning logic | Deferred v2 |
| Lock-free MemTable | Maximum concurrency | High implementation complexity | Deferred v2 |

**Rationale:** The Actor+oneshot model avoids blocking Tokio threads entirely during WAL fsync, enables natural group commit (drain queue → one fsync covers N clients), and provides free cancellation detection when clients disconnect (dropped `rx` → `tx.send()` returns `Err`).

---

## Memory Management

### Per-Node Memory Budgets

| Component | Default | Configurable |
|-----------|---------|--------------|
| MemTable (per shard) | 64 MB | Yes |
| Read Buffer Pool | 128 MB | Yes |
| Index / Bloom Cache | 32 MB | Yes |
| Replication Buffer | 16 MB | Yes |
| **Node Total Target** | ~256 MB | Yes |

### Backpressure Strategy

| Condition | Response |
|-----------|----------|
| MemTable full | Block new writes; trigger async flush |
| Read buffer exhausted | Queue queries; apply timeout |
| Index cache full | Evict LRU entries |
| Replication buffer full | Apply backpressure to ingest |

---

## Durability & Recovery

### Write Durability Levels

| Level | Behaviour | Latency | Safety |
|-------|-----------|---------|--------|
| `Async` | WAL buffered, no fsync | ~μs | Data loss possible on crash |
| `Sync` | WAL + fsync per batch | ~ms | Durable after ACK |
| `SyncBatch` *(default)* | fsync on background timer | ~ms amortized | Durable within configured window |

**Default:** `SyncBatch` with a 10 ms window.

### Crash Recovery Flow

```mermaid
flowchart TB
    Boot([Boot]) --> ReadCatalog["Read Catalog\ncommitted Parts on disk"]
    ReadCatalog --> OpenWAL["Open WAL\nfind last valid checkpoint offset"]
    OpenWAL --> Replay["Replay WAL entries after checkpoint\n→ Rebuild MemTable"]
    Replay --> Verify["Verify Part checksums\noptional, configurable"]
    Verify --> Register["Register with Coordinator\nannounce alive + replication offset"]
    Register --> Role{Role?}
    Role -- Leader --> AcceptWrites["Accept writes normally"]
    Role -- Replica --> CatchUp["Catch up WAL from leader"]
```

**Guarantee:** All acknowledged writes at the configured durability level are recovered after a crash.

---

## Cluster Management

### Topology Model

```mermaid
flowchart TB
    CO["🏛 Coordinator\nRaft metadata group\n1–3 nodes"]

    subgraph S0["Shard 0  ·  hash range [0, H/N)"]
        A["Node A  👑 leader"]
        D["Node D  replica"]
        A -- WAL stream --> D
    end

    subgraph S1["Shard 1  ·  hash range [H/N, 2H/N)"]
        B["Node B  👑 leader"]
        E["Node E  replica"]
        B -- WAL stream --> E
    end

    subgraph S2["Shard 2  ·  hash range [2H/N, H)"]
        C["Node C  👑 leader"]
        F["Node F  replica"]
        C -- WAL stream --> F
    end

    CO --> S0 & S1 & S2
```

### Key Cluster Operations

| Operation | Mechanism |
|-----------|-----------|
| **Leader election** | Coordinator promotes replica with highest replication offset |
| **Node join** | New node registers; receives shard assignment; starts WAL sync |
| **Node failure** | Coordinator detects via heartbeat timeout (5 s); promotes replica |
| **Shard rebalancing** | Coordinate Part migration + catalog update *(v2)* |
| **Scaling out** | Add nodes; assign new shards and migrate data *(v2)* |

### Heartbeat & Failure Detection

```mermaid
stateDiagram-v2
    [*] --> Alive : Node registers
    Alive --> Suspect : 3 missed heartbeats (3 s)
    Suspect --> Alive : Heartbeat received
    Suspect --> Dead : 5 missed heartbeats (5 s)
    Dead --> [*] : Leader election triggered for affected shards
```

### Shard Key Computation

Each write batch is routed by hashing the **primary tag set**:

- Tags sorted alphabetically (deterministic)
- xxHash64 over `key=value\0` pairs
- Result: `hash % num_shards` → shard index

`num_shards` is fixed at cluster creation. Changing it requires a full data migration.

---

## Configuration Reference

All configuration is provided via a TOML file. Key sections:

| Section | Key Parameters |
|---------|---------------|
| `[cluster]` | `node_id`, `role`, `advertise_addr`, `coordinator`, `num_shards`, `replication_factor` |
| `[storage]` | `data_dir` |
| `[memory]` | `memtable_size_bytes`, `read_buffer_size_bytes`, `index_cache_size_bytes`, `replication_buffer_bytes` |
| `[durability]` | `level` (`async`/`sync`/`sync_batch`), `interval_ms` |
| `[threads]` | `async_worker_threads`, `blocking_io_threads`, `background_enabled` |
| `[merge]` | `enabled`, `max_parts_per_partition`, `target_part_size_bytes` |
| `[indexes]` | `inverted.enabled`, `inverted.tag_columns`, `inverted.max_values_per_key` |
| `[io_uring]` | `enabled` (Phase 3), `sqpoll` (advanced), `registered_buffer_count`, `wal_direct_io`, `part_direct_io` |
| `[tables.<name>]` | `partition_duration`, `compression`, `primary_tags` |

---

## Design Decision Log

| # | Decision | Choice | Alternatives | Rationale |
|---|----------|--------|--------------|-----------|
| D1 | Concurrency model | Single writer per shard | Global single writer, lock-free | Parallelism across shards; simplicity within |
| D2 | Replication model | Async WAL streaming (leader-follower) | Raft per shard, synchronous replication | Simpler than per-shard Raft; acceptable for TSDB |
| D3 | Shard assignment | Hash of primary tag set | Range-based, consistent hashing ring | Simpler; static `num_shards` avoids rehashing |
| D4 | Partition granularity | Hourly | Daily, 15-min | Balances file count vs. query selectivity |
| D5 | Default compression | LZ4 | Zstd, None | Speed over ratio for time-series hot data |
| D6 | Durability default | SyncBatch 10 ms | Sync, Async | Balances safety and write throughput |
| D7 | Storage format | Custom `.rpart` (columnar) | Parquet, Apache Arrow | Full control; learning objective |
| D8 | Coordinator consensus | Raft (single group, metadata only) | etcd external, ZooKeeper | No external deps; metadata is small |
| D9 | Query distribution | Coordinator fan-out | Push-down-only, Spark-like | Simpler model; Coordinator is not a write bottleneck |
| D10 | Index types | Min/Max (all columns) + Bloom Filters (tags + low-cardinality fields) + Inverted (tag→Parts in Catalog) | Full B-tree / hash secondary indexes | Zero write-path cost for file-level indexes; inverted index backfill is async |
| D11 | Ingest write concurrency | Actor task per shard + `oneshot` per request + group commit drain | Mutex per shard, thread per client | Actor never blocks Tokio thread; drain queue → one `fsync` covers N clients; free cancellation via dropped `rx` |

---

## Implementation Checklist

### Phase 0 — Single Node, No Replication

- [ ] `RutSeriConfig` + all sub-configs; TOML loading and validation
- [ ] WAL: append, fsync, CRC verification, replay
- [ ] MemTable: sorted by timestamp; configurable flush threshold
- [ ] **Shard Actor** + per-shard `mpsc` dispatch queue
  - [ ] `oneshot::channel` per ingest request (client parks at `rx.await`)
  - [ ] Drain queue before each `fsync` (group commit)
  - [ ] Cancellation safety: detect dropped `rx` via `tx.send()` returning `Err`
- [ ] `.rpart` columnar file writer + reader (with LZ4)
  - [ ] Min/Max Index for all columns (built at flush time)
  - [ ] Bloom Filters for tag columns + configured field columns (built at flush time)
- [ ] Local Catalog: JSON, atomic replace
  - [ ] Inverted index schema in Catalog
- [ ] Local Query Engine: parse → plan → scan → aggregate
  - [ ] Index-aware planner (apply Inverted → Min/Max → Bloom in order)
- [ ] gRPC / HTTP API server (ingest + query endpoints)
- [ ] Background workers: Merge, WAL cleanup, Metrics, **Index Builder**

### Phase 1 — Distribution

- [ ] Shard key computation
- [ ] Coordinator: Raft-replicated Metadata Catalog
- [ ] Coordinator: Write Router
- [ ] Coordinator: Query fan-out + result merger
  - [ ] Inverted index lookup in distributed query planning
- [ ] Storage Node: internal gRPC server
- [ ] Storage Node: WAL replication (leader → replica streaming)
  - [ ] Inverted index replicated as part of Catalog replication
- [ ] Cluster Manager: heartbeat, leader election, node registration

### Phase 2 — Operations & Hardening

- [ ] Prometheus metrics endpoint
- [ ] Admin API (cluster status, table stats, shard info)
- [ ] Automatic leader promotion on node failure
- [ ] Shard rebalancing
- [ ] Per-table resource quotas

### Phase 3 — I/O Performance (io_uring + Direct I/O)

See [storage/io_uring.md](./storage/io_uring.md) for the full design.

- [ ] WAL writer: switch to `tokio-uring`; batch writes per `SyncBatch` window; `O_DIRECT | O_DSYNC`
  - [ ] Pad WAL records to 512-byte boundaries for Direct I/O alignment
- [ ] `.rpart` v2 format: 4096-byte aligned column block offsets
  - [ ] Update Part writer to emit aligned format
  - [ ] Reader: detect v2 and use `O_DIRECT`; fall back to buffered I/O for v1
- [ ] Part reader: `io_uring` batch submit for all columns per query
  - [ ] Register read buffer pool with kernel (`IORING_REGISTER_BUFFERS`)
  - [ ] Parallel LZ4 decode with Rayon after completions
- [ ] Part writer (flush): `O_DIRECT` to prevent cold-data page cache pollution
- [ ] Managed read buffer pool (LRU, keyed by `part_id + col_idx`)
  - [ ] Expose pool hit/miss ratio via Prometheus metrics

---

## Open Questions

| # | Question | Impact |
|---|----------|--------|
| Q1 | **Shard count mutability** — Can `num_shards` change post-creation? Requires full rehash. | High |
| Q2 | **Follower reads** — Allow stale reads from replicas to reduce leader load? | Medium |
| Q3 | **Cross-shard transactions** — Should multi-table or multi-shard atomic writes ever be supported? | Medium |
| Q4 | **Hot config reload** — Can table configs or memory limits be updated at runtime? | Low |
| Q5 | **Object storage (S3)** — Should `.rpart` files be tiered to S3 for cold data? | Medium |
| Q6 | **Compaction policy** — Time-based TTL + LRU eviction for old Parts? | Medium |

---

## Future Work

- **Follower reads** — stale reads from replicas with bounded lag
- **S3 tiering** — automatic offload of cold Parts to object storage
- **io_uring + Direct I/O** — batch WAL writes, parallel column reads, O_DIRECT Part flush to eliminate cache pollution; see [storage/io_uring.md](./storage/io_uring.md)
- **Kubernetes operator** — automated cluster lifecycle management
- **Multi-tenancy** — namespace isolation with per-tenant quotas
- **Native Grafana datasource plugin**
- **Continuous aggregation** — pre-compute rollups at ingest time
- **PromQL / InfluxQL compatibility** — broader ecosystem adoption

---

## Related Documents

| Document | Description |
|----------|-------------|
| [components.md](./components.md) | Detailed component specifications |
| [storage/format.md](./storage/format.md) | `.rpart` file format |
| [storage/indexes.md](./storage/indexes.md) | Index design — Min/Max, Bloom Filters, Inverted Index |
| [ingestion/wal.md](./ingestion/wal.md) | WAL format and durability guarantees |
| [cluster/replication.md](./cluster/replication.md) | Replication protocol details |
| [cluster/sharding.md](./cluster/sharding.md) | Shard key design and routing |
| [storage/tsm.md](./storage/tsm.md) | TSM reference — how RutSeriDB's storage relates to InfluxDB's TSM engine |
| [storage/io_uring.md](./storage/io_uring.md) | Phase 3 io_uring + Direct I/O design — WAL batching, parallel column reads, managed buffer pool |
