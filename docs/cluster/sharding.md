# RutSeriDB — Sharding Design

> **Related:** [architecture.md](../architecture.md) · [replication.md](./replication.md)
> **Version:** 0.1 (Draft)

---

## Shard Key Function

A **shard key** deterministically maps a time-series (identified by its primary tag set) to one specific shard, which is then served by a leader Storage Node.

### Algorithm

```mermaid
flowchart LR
    Tags["Primary tags\nBTreeMap sorted by key"] --> Hash["xxHash64 seed=0\nover key=value\\0 pairs"]
    Hash --> Mod["% num_shards"]
    Mod --> ShardID["Shard index\n[0, num_shards)"]
```

### Properties

| Property | Value |
|----------|-------|
| Hash algorithm | xxHash64 (fast, excellent distribution) |
| Input | Sorted `key=value\0` concatenation of primary tag pairs |
| Output | Integer in `[0, num_shards)` |
| Determinism | Same tags → same shard, on any node, forever |
| Stability | `num_shards` is fixed at cluster creation |

---

## Primary Tags

Each table declares a **primary tag set** — the tags used to compute the shard key. These must be provided at table creation and are immutable.

| Config | Example |
|--------|---------|
| `tables.cpu_metrics.primary_tags` | `["host", "region"]` |

All writes to the table must include every primary tag. Other tags are secondary (stored in data but not used for routing).

---

## Shard Map

The Coordinator maintains the authoritative shard map in its Raft state machine.

```mermaid
flowchart LR
    subgraph ShardMap["Shard Map (4 shards · replication_factor=2)"]
        S0["Shard 0\nLeader: node-a\nReplica: node-d"]
        S1["Shard 1\nLeader: node-b\nReplica: node-e"]
        S2["Shard 2\nLeader: node-c\nReplica: node-f"]
        S3["Shard 3\nLeader: node-a\nReplica: node-d"]
    end
```

> Nodes can lead multiple shards. In the example above, `node-a` leads both shard 0 and shard 3.

---

## Routing a Write

```mermaid
sequenceDiagram
    participant C  as Client
    participant CO as Coordinator
    participant SN as Storage Node (Leader)

    C->>CO: IngestBatch(table="cpu_metrics", tags={host="web-01", region="us-east"}, rows)

    CO->>CO: primary_tags = sort({host: "web-01", region: "us-east"})
    CO->>CO: shard_id = hash(primary_tags) % num_shards  →  e.g. 2
    CO->>CO: leader_node = shard_map[2].leader  →  "node-c"

    CO->>SN: Forward batch to node-c (internal gRPC)
    SN-->>CO: OK
    CO-->>C: OK
```

---

## Routing a Query

```mermaid
sequenceDiagram
    participant C  as Client
    participant CO as Coordinator
    participant SN1 as Storage Node A
    participant SN2 as Storage Node B

    C->>CO: SELECT mean(cpu) FROM cpu_metrics WHERE time > now()-1h AND region='us-east'

    CO->>CO: Parse query · extract time range [now-1h, now]
    CO->>CO: Check per-shard time range bounds in Metadata Catalog
    CO->>CO: All shards may have data → fan-out to all leaders

    par
        CO->>SN1: Sub-query with time filter
        CO->>SN2: Sub-query with time filter
    end

    SN1->>SN1: Filter rows where region='us-east' locally
    SN2->>SN2: Filter rows where region='us-east' locally

    SN1-->>CO: Partial aggregates
    SN2-->>CO: Partial aggregates

    CO->>CO: Final merge
    CO-->>C: ResultSet
```

> **Optimization (v2):** If the query filters on *all* primary tags with equality predicates, the Coordinator can compute the exact `shard_id` and route to a **single** shard — avoiding the fan-out entirely.

---

## Shard Count Trade-offs

| `num_shards` | Pros | Cons |
|--------------|------|------|
| Too few (1–2) | Simple routing; fewer files | Write hotspot; limited parallelism |
| Too many (100+) | Very parallel | Many small files; high catalog overhead |
| **Recommended (v1)** | **8–64** | Balances parallelism and file count |

**Rule of thumb:** `num_shards = 2 × expected_storage_node_count` at cluster creation.

---

## Limitations (v1)

```mermaid
flowchart LR
    L1["❌ num_shards is fixed\nat cluster creation\nChanging requires full data migration (v2)"]
    L2["❌ No automatic rebalancing\nwhen new nodes join\nManual assignment via Admin API"]
    L3["⚠️ Hot time-series risk\nSame primary tags → same shard\nLoad imbalance on high-cardinality keys\n(fine-grained shard splitting in v2)"]
```

---

## Future Improvements (v2)

| Feature | Description |
|---------|-------------|
| **Consistent hashing ring** | Minimize data movement when `num_shards` changes |
| **Shard splitting** | Split a hot shard into two with data migration |
| **Automatic rebalancing** | Coordinator detects imbalance and migrates shards |
| **Tag-based routing shortcut** | Skip fan-out when all primary tags are equality-filtered |
