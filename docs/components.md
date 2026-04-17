# RutSeriDB — Component Specifications

> **Related:** [architecture.md](./architecture.md)
> **Version:** 0.1 (Draft)

Detailed specifications for every component from the C3 diagrams.

---

## Table of Contents

1. [WAL (Write-Ahead Log)](#wal-write-ahead-log)
2. [MemTable](#memtable)
3. [Part Writer / Reader](#part-writer--reader)
4. [Local Catalog](#local-catalog)
5. [Ingest Engine](#ingest-engine)
6. [Local Query Engine](#local-query-engine)
7. [Replication Manager](#replication-manager)
8. [Coordinator — Write Router](#coordinator--write-router)
9. [Coordinator — Distributed Query Planner](#coordinator--distributed-query-planner)
10. [Coordinator — Cluster Manager](#coordinator--cluster-manager)
11. [Background Workers](#background-workers)

---

## WAL (Write-Ahead Log)

### Purpose

Guarantees durability of unflushed writes. The WAL is the canonical source of truth for data not yet promoted to Part files.

### Segment Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Active : First write
    Active --> Sealed : Reaches max_segment_bytes
    Sealed --> Checkpointed : All rows flushed to Parts AND all replicas ACK'd
    Checkpointed --> [*] : WAL Cleanup Worker deletes file
```

### File Layout

Each shard maintains a directory of numbered segment files:

```mermaid
flowchart LR
    D["/data/shard-id/wal/"] --> S1["00000001.rwal\nsealed"]
    D --> S2["00000002.rwal\nsealed"]
    D --> S3["00000003.rwal\nactive ← appended to"]
```

### Record Structure

Each entry in a segment is a framed record:

| Field | Size | Description |
|-------|------|-------------|
| Magic | 4 B | `RWAL` — sanity marker |
| Seq | 8 B | Monotonically increasing u64, per-shard |
| Len | 4 B | Byte length of Payload |
| Payload | variable | Serialized `WalEntry` |
| CRC32 | 4 B | Integrity check over `[Seq ‖ Len ‖ Payload]` |

### Entry Types

| Entry | Description |
|-------|-------------|
| `Write { table, rows }` | A batch of time-series rows to be inserted |
| `Checkpoint { seq, catalog_ver }` | Marks that all entries ≤ `seq` are safely flushed to Parts |

### Durability Levels

| Level | Behaviour | Latency | Safety |
|-------|-----------|---------|--------|
| `Async` | Buffer in OS page cache; no fsync | ~μs | Data loss on crash |
| `Sync` | fsync after every `Write` entry | ~ms | Durable after ACK |
| `SyncBatch` *(default)* | Background timer fsync every N ms | ~ms amortized | Durable within window |

### Replay Algorithm

```mermaid
flowchart TB
    A([Startup]) --> B["List WAL segment files — sorted ascending"]
    B --> C["Find last Checkpoint record → replay_from_seq"]
    C --> D{"More entries\nafter checkpoint?"}
    D -- Yes --> E["Re-apply WalEntry::Write to MemTable"]
    E --> D
    D -- No --> F["Truncate any partial trailing record\n(CRC mismatch = partial write)"]
    F --> G([Recovery complete])
```

---

## MemTable

### Purpose

In-memory write buffer sorted by `(timestamp, tag_hash)` for efficient merge-flush into Part files.

### Data Structure

An ordered map keyed by `MemKey { timestamp: i64, tag_hash: u64 }`. Sort order is first by timestamp (ascending), then by tag hash to break ties deterministically.

### Concurrency Model

```mermaid
flowchart LR
    WTask["Writer Task\n(one per shard)"] -- "holds Mutex" --> MT["MemTable\nBTreeMap"]
    QTask["Query Task\n(many)"] -- "snapshot clone at\nquery start" --> Snap["MemTable Snapshot\n(read-only)"]
    MT -. "snapshot" .-> Snap
```

### Flush Triggers

| Condition | Action |
|-----------|--------|
| Bytes used > `memtable_size_bytes` | Trigger immediate flush |
| Row count > configured max *(optional)* | Trigger immediate flush |
| Manual flush via Admin API | Trigger flush |
| Graceful shutdown | Flush all shards |

---

## Part Writer / Reader

### Purpose

Converts a MemTable snapshot into an immutable, columnar, compressed `.rpart` file.

### Write Protocol

```mermaid
flowchart TB
    Snap["MemTable Snapshot"] --> Sort["Sort rows by\n(timestamp, tag_hash)"]
    Sort --> Encode["Encode each column independently\n(delta, gorilla, dictionary)"]
    Encode --> Compress["Compress column blocks\n(LZ4 by default)"]
    Compress --> Index["Build Min/Max Index\nall columns: timestamps · tags · fields"]
    Index --> Bloom["Build Bloom Filters\ntag columns + low-cardinality field columns"]
    Bloom --> TmpFile["Write to temp file\n<uuid>.rpart.tmp"]
    TmpFile --> Fsync["fsync temp file"]
    Fsync --> Rename["rename() → <uuid>.rpart\n(atomic on POSIX)"]
    Rename --> Catalog["Update Catalog"]
```

### Read Protocol (Projection + Predicate Pushdown)

```mermaid
flowchart TB
    Open["Open Part file\nRead Footer"] --> MinMax["1️⃣ Min/Max Index\nrange predicate check — all columns"]
    MinMax -- No overlap --> Skip["Skip entire file\nzero I/O on columns"]
    MinMax -- Overlap --> BloomCheck["2️⃣ Bloom Filters\ntag + field equality predicates"]
    BloomCheck -- Definite miss --> Skip
    BloomCheck -- May contain --> ReadCols["Read only requested columns\n(projection pushdown)"]
    ReadCols --> Decompress["Decompress + decode"]
    Decompress --> Filter["Apply row-level filter predicates"]
    Filter --> Output["Return matching rows"]
```

### Column Encodings

| Column Type | Encoding | Compression |
|-------------|----------|-------------|
| `timestamp` | Delta i64 | LZ4 |
| Integer field | Delta-of-delta | LZ4 |
| Float field | Gorilla XOR (IEEE 754) | LZ4 |
| Tag (low cardinality) | Dictionary (u16 codes) | LZ4 |
| String field | Raw bytes | LZ4 |

---

## Local Catalog

### Purpose

Tracks all committed Part files for a shard. The query engine relies on this for Part discovery and time-range pruning.

### Update Protocol

```mermaid
sequenceDiagram
    participant IE as Ingest Engine
    participant FS as File System

    IE->>FS: Write catalog.json.tmp (new version)
    IE->>FS: fsync catalog.json.tmp
    IE->>FS: rename(catalog.json.tmp → catalog.json)
    Note over IE,FS: Atomic — readers always see a consistent catalog
```

### Schema Overview

The catalog stores, per table:
- Table schema (tag names, field types)
- List of `PartMeta` records: `{ id, path, min_ts, max_ts, size_bytes, row_count, created_at }`
- **Inverted index** — maps `(tag_key, tag_value) → [part_id, ...]` for O(1) Part discovery by tag equality
- A monotonically increasing `version` counter

See [storage/indexes.md](./storage/indexes.md) for the full inverted index design.

---

## Ingest Engine

### Purpose

Handles the full write path for one shard via a **Shard Actor** model:
- Client handler tasks create a `oneshot::channel` per request and push `(batch, tx)` into the shard's dispatch queue
- The client task immediately parks at `rx.await` — releasing the Tokio worker thread
- The Shard Actor drains all pending requests, coalesces them, does ONE WAL fsync, then fires all `tx.send(OK)` simultaneously

### Dispatch Queue + Oneshot Flow

```mermaid
sequenceDiagram
    participant CA as Client Handler A
    participant CB as Client Handler B
    participant CC as Client Handler C
    participant Q  as Dispatch Queue (mpsc)
    participant SA as Shard Actor
    participant W  as WAL

    CA->>Q: (batch_A, tx_A).send().await
    CA->>CA: rx_A.await  ← yields thread
    CB->>Q: (batch_B, tx_B).send().await
    CB->>CB: rx_B.await  ← yields thread
    CC->>Q: (batch_C, tx_C).send().await
    CC->>CC: rx_C.await  ← yields thread

    Note over SA: drain all available items
    SA->>Q: recv() → (batch_A, tx_A)
    SA->>Q: try_recv() → (batch_B, tx_B)
    SA->>Q: try_recv() → (batch_C, tx_C)
    SA->>SA: coalesce rows: A+B+C
    SA->>W: WAL append(merged_rows)
    SA->>W: fsync() ← ONE call covers 3 clients
    SA->>SA: memtable.insert(merged_rows)
    SA-->>CA: tx_A.send(OK)  → rx_A wakes
    SA-->>CB: tx_B.send(OK)  → rx_B wakes
    SA-->>CC: tx_C.send(OK)  → rx_C wakes
```

### Shard Actor Loop

```mermaid
flowchart TB
    Start(["loop {"]) --> Recv["batch, tx = queue.recv().await"]
    Recv --> Drain["Drain remaining: try_recv() until Empty\nCoalesce all rows + collect all tx senders"]
    Drain --> Wal["WAL append(coalesced rows)"]
    Wal --> Fsync["WAL fsync() ← one call for all clients"]
    Fsync --> Mem["MemTable.insert(rows)"]
    Mem --> Acks["for tx in senders: tx.send(OK)\nAll clients unblocked simultaneously"]
    Acks --> Flush{"MemTable bytes > threshold?"}
    Flush -- Yes --> TriggerFlush["Spawn Part flush\n(background — actor continues immediately)"]
    Flush -- No --> Start
    TriggerFlush --> Start
```

### Cancellation Safety

If a client disconnects before the actor fires the ACK:
- `rx` (the `oneshot::Receiver`) is dropped when the client task is cancelled
- `tx.send(OK)` returns `Err(SendError)` — actor discards silently
- The rows may still be written (WAL already fsynced) — idempotent retry handles duplicates

### Internal gRPC Interface

The Storage Node exposes these endpoints to the Coordinator:

| RPC | Description |
|-----|-------------|
| `WriteBatch(table, shard_id, rows)` | Ingest a batch of rows |
| `FlushShard(shard_id)` | Force a flush (admin / shutdown) |

---

## Local Query Engine

### Purpose

Executes sub-queries from the Coordinator. Scans local Parts + MemTable snapshot, returns partial results.

### Pipeline

```mermaid
flowchart LR
    SQL["SQL sub-query"] --> Parser["Parser\nSQL → AST"]
    Parser --> Planner["Planner\n• Time range extraction\n• Part pruning via Catalog\n• Bloom filter check"]
    Planner --> MemScan["MemTable Scan\n(snapshot)"]
    Planner --> PartScan["Part Scan\n(column projection)"]
    MemScan --> Filter["Filter\n(vectorized predicates)"]
    PartScan --> Filter
    Filter --> LocalAgg["Local Aggregation\nsum · count · min · max · mean"]
    LocalAgg --> Result["Partial ResultSet\n(Arrow RecordBatch)"]
```

### Supported Operations (v1)

| Operation | Supported |
|-----------|-----------|
| `WHERE time > / < / BETWEEN` | ✅ |
| `WHERE tag = 'value'` | ✅ |
| Field comparison filters | ✅ |
| `SUM · COUNT · MIN · MAX · MEAN` | ✅ |
| `GROUP BY tag` | ✅ |
| `ORDER BY time` | ✅ |
| `LIMIT` | ✅ |
| `JOIN` across tables | ❌ v2 |
| Subqueries | ❌ v2 |

---

## Replication Manager

### Purpose

Streams WAL entries from a shard leader to its replicas. Tracks per-replica lag and triggers snapshot sync when needed.

### Normal Streaming

```mermaid
sequenceDiagram
    participant L  as Leader
    participant R  as Replica

    R->>L: OpenReplicationStream(shard_id, from_seq=N)
    L-->>R: ACK(current_offset)

    loop Continuous push
        L->>R: WalEntries(offset=N, entries=[...])
        R->>R: Apply to local MemTable
        R-->>L: ACK(offset=N)
    end
```

### Snapshot Sync Decision

```mermaid
flowchart TB
    Connect["Replica connects\nwith from_seq=N"] --> Check{"N ≥ leader's\noldest buffered seq?"}
    Check -- Yes --> Stream["Resume normal\nWAL streaming"]
    Check -- No --> Snapshot["Full snapshot sync\n(pull all Part files)"]
    Snapshot --> Stream
```

---

## Coordinator — Write Router

### Purpose

Routes incoming write batches to the correct shard leader.

### Routing Algorithm

```mermaid
flowchart LR
    Batch["IngestBatch\n(table, primary_tags, rows)"] --> Hash["shard_key = hash(primary_tags) % num_shards"]
    Hash --> Lookup["Lookup leader node\nfrom Metadata Catalog"]
    Lookup --> Forward["Forward batch\nto leader via gRPC"]
```

### Failure Handling

| Scenario | Action |
|----------|--------|
| Leader unreachable | Return error to client; Cluster Manager triggers election |
| Leader timeout | Return `DEADLINE_EXCEEDED`; client retries |
| Rebalancing in progress | Buffer writes ≤ 500 ms; retry after migration |

---

## Coordinator — Distributed Query Planner

### Purpose

Translates a client SQL query into per-shard sub-queries, fans them out, and merges results.

### Planner Steps

```mermaid
flowchart TB
    SQL["Client SQL"] --> Parse["Parse → global AST"]
    Parse --> Resolve["Resolve table → find all shards\nholding data for the requested table"]
    Resolve --> Prune["Prune shards outside WHERE time range\n(using per-shard min/max from Metadata Catalog)"]
    Prune --> Rewrite["Rewrite query into per-shard sub-queries\n(push down time filter + projections)"]
    Rewrite --> Fanout["Fan-out: parallel gRPC calls\nto each relevant Storage Node"]
    Fanout --> Collect["Collect Arrow RecordBatches"]
    Collect --> Merge["Final merge:\n• Re-sort by (time, group-by keys)\n• Final aggregation\n• Apply global LIMIT"]
    Merge --> Stream["Stream Arrow IPC to client"]
```

---

## Coordinator — Cluster Manager

### Purpose

Maintains authoritative cluster topology via a Raft state machine (single group, metadata only).

### Metadata Operations

| Operation | Description |
|-----------|-------------|
| `RegisterNode` | Record a new node's address |
| `DeregisterNode` | Remove a departed node |
| `AssignShard` | Map a shard to leader + replica nodes |
| `PromoteLeader` | Elect a new shard leader after failure |
| `RegisterTable` | Record a new table schema |

### Heartbeat & Failure State Machine

```mermaid
stateDiagram-v2
    [*] --> Alive : Node registers
    Alive --> Suspect : 3 missed heartbeats (3 s)
    Suspect --> Alive : Heartbeat received
    Suspect --> Dead : 5 missed heartbeats (5 s)
    Dead --> [*] : Leader election triggered\nfor affected shards
```

---

## Background Workers

All workers run in a dedicated background pool on each Storage Node.

```mermaid
flowchart TB
    BG["Background Worker Pool\n(single Tokio task pool)"]

    BG --> MW["🔀 Merge Worker\nTrigger: Parts per partition > max_parts_per_partition\nAction: merge-sort N Parts → 1 larger Part → update Catalog → delete old Parts"]
    BG --> WC["🗑 WAL Cleanup\nTrigger: segment fully checkpointed + all replicas ACK'd\nAction: delete old .rwal segment files"]
    BG --> MR["📊 Metrics Reporter\nTrigger: every 15 seconds\nAction: expose internal gauges via Prometheus /metrics"]
    BG --> CC["🔍 Catalog Consistency Check\nTrigger: startup + every 1 hour\nAction: verify all catalog Parts exist on disk · log discrepancies"]
    BG --> IB["🗂 Index Builder Worker\nTrigger: new Part flushed (notification) or startup backfill\nAction: scan new Part for unique tag key/value pairs\n→ update inverted index in Catalog (atomic)\n→ remove stale Part IDs after merge or deletion"]
```

### Index Builder — Detailed Flow

```mermaid
sequenceDiagram
    participant IE  as Ingest Engine
    participant IB  as Index Builder Worker
    participant CAT as Catalog

    IE->>CAT: New Part flushed (part-013)
    IE->>IB: Notify(part_id=part-013, table=metrics)
    IB->>IB: Open part-013 · read tag columns
    IB->>IB: Collect unique (tag_key, tag_value) pairs
    IB->>CAT: inverted_index[host][web-01] += part-013
    IB->>CAT: inverted_index[region][us-east] += part-013
    CAT->>CAT: Persist atomically (write-tmp → rename)
```
