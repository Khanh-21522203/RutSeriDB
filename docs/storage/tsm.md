# TSM — Time-Structured Merge Tree

> **Context:** This document explains the TSM storage engine invented by InfluxDB and how it inspired RutSeriDB's storage design.
> **Version:** 0.1 (Reference)

---

## What Is TSM?

**TSM (Time-Structured Merge Tree)** is a custom storage engine developed by InfluxDB (around 2015–2016) to replace their earlier attempts using general-purpose key-value stores (BoltDB, then LevelDB/RocksDB). The core insight was:

> *LSM Tree was designed for random key-value workloads. Time-series data has a fundamentally different access pattern — time is always the dominant dimension. We should exploit that.*

TSM is best understood as **LSM Tree, restructured around time as the primary axis**.

---

## Why InfluxDB Built TSM

InfluxDB went through several storage engines before TSM:

```mermaid
flowchart LR
    V1["v0.9\nLevelDB\n❌ Too many open files\nPoor compaction for TSDB"] --> V2["v0.9.5\nBoltDB\n❌ Single writer bottleneck\nHigh memory for large datasets"]
    V2 --> V3["v0.10\nRocksDB\n❌ Still suboptimal:\nrow-oriented, wrong compaction strategy"]
    V3 --> V4["v0.11\nTSM\n✅ Purpose-built for TSDB\nColumnar-per-series, time-aware compaction"]
```

The fundamental problems with general-purpose storage engines for TSDB:

| Problem | General KV Store (LSM) | TSM Solution |
|---------|----------------------|--------------|
| Keys are temporal | Treats timestamps as arbitrary keys → poor compression | Time is first-class; delta-encodes timestamps |
| Queries are range scans | Point-lookup optimized | Block index maps series → time range → offset |
| Data is append-only | Designs for updates + deletes | No tombstones; old data naturally falls off by time |
| Many distinct series | One bloom filter per SSTable | Per-series index within each TSM file |
| Column access (one field at a time) | Row-oriented storage | Per-series blocks = effectively columnar per metric |

---

## TSM Architecture

```mermaid
flowchart TB
    Client["Write request\nmeasurement,tags → field=value ts=T"] --> WAL

    subgraph WriteBuffer["Write Buffer"]
        WAL["WAL .wal file\nAppend-only, sequential\nDurability before Cache ack"]
        Cache["Cache\nIn-memory map:\nSeriesKey → sorted (ts, value) list\nmax size: cache-max-memory-size"]
        WAL --> Cache
    end

    Cache -- "flush when full\nor WAL too large" --> L1["Level 1 TSM files\n(small, freshly flushed)"]

    subgraph TSMFiles["TSM File Levels (Compaction)"]
        L1 --> L2["Level 2\n(compacted L1s)"]
        L2 --> L3["Level 3\n(compacted L2s)"]
        L3 --> Opt["Optimized\n(max compression, largest files)"]
    end

    subgraph Query["Query Path"]
        Q["Query:\nSELECT mean(cpu) FROM metrics\nWHERE host='web-01' AND time > now()-1h"]
        Q --> CacheScan["1. Scan Cache\n(recent data)"]
        Q --> TSMScan["2. Scan TSM files\n(historical, via index)"]
        CacheScan --> Merge["Merge + deduplicate\n(Cache takes priority)"]
        TSMScan --> Merge
        Merge --> Result["ResultSet"]
    end
```

---

## TSM Core Data Structures

### 1. Cache (In-Memory)

```mermaid
flowchart TB
    subgraph Cache["Cache — HashMap&lt;SeriesKey, Values&gt;"]
        SK1["SeriesKey:\ncpu,host=web-01,region=us-east\n──────────────────────────────\n(ts=1000, val=12.3)\n(ts=1001, val=14.1)\n(ts=1002, val=11.8)"]
        SK2["SeriesKey:\ncpu,host=db-01,region=us-east\n──────────────────────────────\n(ts=1000, val=5.0)\n(ts=1001, val=5.2)"]
        SK3["SeriesKey:\nmem,host=web-01,region=us-east\n──────────────────────────────\n(ts=1000, val=256)\n(ts=1001, val=312)"]
    end
```

A **SeriesKey** is the combination of measurement name + all tag key-value pairs. It uniquely identifies one time series. The cache maps each SeriesKey to a sorted list of `(timestamp, value)` pairs.

### 2. TSM File Format

```mermaid
flowchart TB
    HDR["Header\n4 bytes magic + 1 byte version"]
    B1["Data Block: cpu,host=web-01\n──────────────────────────────\nTimestamps: [1000,1001,1002,...]\ndelta encoded + snappy compressed\nValues:      [12.3,14.1,11.8,...]\nGorilla XOR + snappy compressed"]
    B2["Data Block: cpu,host=db-01\n──────────────────────────────\nTimestamps: [...]\nValues: [...]"]
    BN["Data Block: ... (one per series key)"]
    IDX["Index Section\n──────────────────────────────\nFor each SeriesKey:\n  → [min_ts, max_ts, block_offset, block_size, type]\nSorted by SeriesKey for binary search"]
    FT["Footer\n8 bytes: offset of Index Section"]

    HDR --> B1 --> B2 --> BN --> IDX --> FT
```

### 3. Index Entry (per series per block)

| Field | Description |
|-------|-------------|
| `series_key` | Measurement + sorted tag set |
| `block_type` | Float, Integer, Bool, String |
| `min_ts` | Minimum timestamp in this block |
| `max_ts` | Maximum timestamp in this block |
| `offset` | Byte offset of block in the TSM file |
| `size` | Byte size of compressed block |

---

## TSM Encodings

| Data Type | Encoding |
|-----------|----------|
| Timestamps | Delta encoding → then `simple8b` (bit-packing) → Snappy |
| Floats | Gorilla XOR (same as Facebook Gorilla paper) |
| Integers | Zig-zag + `simple8b` or RLE if constant |
| Booleans | Bit-packing |
| Strings | Snappy compressed |

The encodings are chosen per-block based on the data characteristics (e.g., if all values are identical → RLE).

---

## TSM Compaction

LSM compacts to manage level overlap across the whole keyspace. TSM compacts to:
1. Reduce the number of files per shard (for query speed)
2. Increase compression (larger blocks compress better)
3. Deduplicate overlapping time ranges (from out-of-order writes)

```mermaid
flowchart TB
    Flush["Cache flush → L1 TSM files\n(many small files, ≤2MB each)"]

    subgraph Compact["Compaction Strategy"]
        C1["Level 1 → Level 2\nWhen: ≥4 L1 files in a shard\nAction: merge + re-encode + compress\nTarget: ≤25MB files"]
        C2["Level 2 → Level 3\nWhen: ≥4 L2 files in a shard\nAction: same\nTarget: ≤1GB files"]
        C3["Level 3 → Optimize\nWhen: many L3 files\nAction: full compaction\nTarget: maximum compression, few files"]
    end

    Flush --> C1 --> C2 --> C3
```

**Key difference from LSM compaction:** TSM compacts *within a shard* (a time window, e.g. 1 week). It never re-organizes data across different time windows. Once a shard's data is outside the retention window, the entire shard directory is deleted — O(1) deletion regardless of data volume.

---

## Shard Groups (Time Partitioning)

TSM always partitions data by time into **shard groups**, analogous to RutSeriDB's time-window partitions:

```mermaid
flowchart LR
    subgraph SG1["Shard Group: 2024-01-01 to 2024-01-07"]
        SH1["Shard 1\n(replication factor)"]
        SH2["Shard 2"]
    end

    subgraph SG2["Shard Group: 2024-01-08 to 2024-01-14"]
        SH3["Shard 3"]
        SH4["Shard 4"]
    end

    subgraph SG3["Shard Group: 2024-01-15 to now"]
        SH5["Shard 5 (active writes)"]
        SH6["Shard 6"]
    end

    SG1 --> SG2 --> SG3
```

- Each shard group covers a fixed time range (configurable, default 1 week for measurements, 1 day for short retention)
- Queries only touch shard groups whose time range overlaps the query's `WHERE time` clause
- Expired shard groups are dropped as a whole directory — infinitely faster than LSM tombstone compaction

---

## TSM vs LSM — Side-by-Side

```mermaid
flowchart TB
    subgraph LSM_Flow["LSM Tree"]
        LW["Random writes\nany key at any time"] --> LM["MemTable\nall keys mixed"]
        LM --> LS0["L0 SSTables\nkey range: full keyspace"]
        LS0 --> LS1["L1 SSTables\nkey range: full keyspace"]
        LS1 --> LS2["L2 SSTables\nkey range: full keyspace"]
    end

    subgraph TSM_Flow["TSM Tree"]
        TW["Append writes\nalways near 'now'"] --> TM["Cache\ngrouped by SeriesKey"]
        TM --> TL1["L1 TSM files\nwithin shard group (time window)"]
        TL1 --> TL2["L2 TSM files\nwithin same shard group"]
        TL2 --> TO["Optimized TSM\nfull compression"]
    end
```

| Dimension | LSM | TSM |
|-----------|-----|-----|
| Primary sort key | Arbitrary user key | SeriesKey (measurement + tags) |
| Secondary dimension | None | Time (within each series) |
| Compaction scope | Entire keyspace → complex level management | Single time-window shard → simple |
| Deletion of old data | Tombstones → expensive compaction | Shard group drop → O(1) |
| Write pattern | Random keys | Append-only to recent time window |
| Storage layout | Row-oriented | Per-series columnar (blocks) |
| Bloom filter use | Arbitrary key membership | Series key + time range index |

---

## How RutSeriDB Relates to TSM

RutSeriDB's design was inspired by TSM but takes the columnar idea further:

```mermaid
flowchart TB
    subgraph TSM_Layout["TSM: Per-series columnar"]
        T1["Block: cpu,host=web-01\n[ts: 1000,1001,1002]\n[val: 12.3,14.1,11.8]"]
        T2["Block: cpu,host=db-01\n[ts: 1000,1001]\n[val: 5.0,5.2]"]
        T3["Block: mem,host=web-01\n[ts: 1000,1001]\n[val: 256,312]"]
    end

    subgraph Rut_Layout["RutSeriDB .rpart: True columnar (all series together)"]
        R1["Column: timestamps\n[1000,1000,1001,1001,1002]\ndelta encoded → LZ4"]
        R2["Column: host (dict)\n[0,1,0,1,0]\ndictionary: {0:web-01, 1:db-01}"]
        R3["Column: cpu\n[12.3, 5.0, 14.1, 5.2, 11.8]\nGorilla XOR → LZ4"]
        R4["Column: mem\n[256, null, 312, null, null]\nnull bitmap + delta → LZ4"]
    end

    TSM_Layout -- "RutSeriDB extends\nby merging all series\ninto true columnar layout" --> Rut_Layout
```

| Aspect | TSM | RutSeriDB |
|--------|-----|-----------|
| Column granularity | Per series per field (one block) | Per field across ALL series in a Part |
| Cross-series aggregation | Must read N series blocks | Reads one column block — true vectorized |
| Tag filtering | Series key lookup in index | Bloom filter + inverted index on tag column |
| Time partitioning | Shard groups (e.g. 1 week) | Time-window Parts (e.g. 1 hour) |
| Compaction | 4 levels within a shard | Merge Worker (flat, within a time partition) |
| Multi-node | Shards assigned to nodes | Shards with leader-follower WAL replication |
| WAL | Per-shard WAL | Per-shard WAL (same concept) |

---

## Key Takeaways

```mermaid
flowchart LR
    LSM["LSM Tree\n(LevelDB / RocksDB)\nOptimal for random KV workloads"] --> TSM["TSM\n(InfluxDB)\n'LSM + time as primary axis'\nOptimal for single-server TSDB"]
    TSM --> RutSeriDB["RutSeriDB .rpart\n'TSM + true columnar'\nOptimal for distributed analytical TSDB"]
```

1. **LSM** solves random-write performance at the cost of compaction complexity
2. **TSM** exploits time ordering to simplify compaction and enable per-series compression
3. **RutSeriDB** goes further — merging all series into a true columnar layout (like Parquet) to enable vectorized aggregations across arbitrary tag groups, at the cost of slightly more complex merge logic

The progression is: **row store → per-series columnar (TSM) → true columnar (Parquet-style)** — each step trading write simplicity for query performance on analytical TSDB workloads.

---

## References

- [InfluxDB TSM design document (2016)](https://docs.influxdata.com/influxdb/v1/concepts/storage_engine/)
- [Gorilla: A Fast, Scalable, In-Memory Time Series Database (Facebook, 2015)](https://www.vldb.org/pvldb/vol8/p1816-teller.pdf)
- [Bitcask: A Log-Structured Hash Table (Riak, 2010)](https://riak.com/assets/bitcask-a-log-structured-hash-table-for-fast-key-value-data.pdf)
- [The Log-Structured Merge-Tree (O'Neil et al., 1996)](https://www.cs.umb.edu/~poneil/lsmtree.pdf)
