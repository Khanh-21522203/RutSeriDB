# RutSeriDB — Replication Protocol

> **Related:** [architecture.md](../architecture.md) · [components.md](../components.md)
> **Version:** 0.1 (Draft)

---

## Model

RutSeriDB uses **asynchronous leader-follower replication** per shard. Leader assignment is managed by a single Raft group on the Coordinator (not a per-shard Raft instance).

```mermaid
flowchart TB
    CO["🏛 Coordinator\nRaft metadata group\nmanages leader assignments"]

    subgraph S0["Shard 0"]
        A["Node A 👑 leader"] -- WAL stream --> D["Node D replica"]
    end

    subgraph S1["Shard 1"]
        B["Node B 👑 leader"] -- WAL stream --> E["Node E replica"]
    end

    subgraph S2["Shard 2"]
        C["Node C 👑 leader"] -- WAL stream --> F["Node F replica"]
    end

    CO -- "LeaderAssignment(shard=0, leader=A)" --> S0
    CO -- "LeaderAssignment(shard=1, leader=B)" --> S1
    CO -- "LeaderAssignment(shard=2, leader=C)" --> S2
```

---

## Normal Operation — WAL Streaming

The leader pushes WAL entries immediately after writing them to its own log. The replica applies entries in-order and ACKs after each batch.

```mermaid
sequenceDiagram
    participant R as Replica
    participant L as Leader

    R->>L: OpenReplicationStream(shard_id, from_seq=N)
    L-->>R: ACK(current_offset=M)

    loop Continuous push
        Note over L: New entry written to WAL (seq=M)
        L->>R: WalEntries(offset=M, entries=[...])
        R->>R: Apply entries to local MemTable
        R-->>L: ACK(offset=M)
    end
```

### Transport

| Property | Detail |
|----------|--------|
| Protocol | gRPC bidirectional streaming |
| Buffer | Leader keeps last `replication_buffer_bytes` of WAL in memory |
| ACK frequency | After each batch application |

---

## Snapshot Sync (Replica Re-Join)

When a replica's `from_seq` is older than the leader's oldest buffered WAL entry, a full snapshot sync is required before resuming streaming.

```mermaid
sequenceDiagram
    participant R as Replica
    participant L as Leader

    R->>L: SnapshotRequest(shard_id)
    L-->>R: SnapshotStart(catalog, snapshot_seq=K)

    loop For each .rpart file in catalog
        L-->>R: PartFileChunk(name, data)
    end

    L-->>R: SnapshotEnd(catalog_ver=V)

    R->>R: Write Part files to disk
    R->>R: Update local Catalog to version V

    R->>L: StreamWal(shard_id, from_seq=K+1)
    Note over R,L: Resume normal WAL streaming
```

### When Snapshot Sync is Triggered

```mermaid
flowchart TB
    Connect["Replica connects\nfrom_seq = N"] --> Check{"N ≥ leader's oldest\nbuffered WAL seq?"}
    Check -- Yes --> NormalStream["Resume normal\nWAL streaming"]
    Check -- No --> Snapshot["Full snapshot sync\n1. Pull catalog\n2. Pull all Part files\n3. Resume streaming from snapshot_seq+1"]
    Snapshot --> NormalStream
```

---

## Failover

### Detection

The Coordinator detects node failures via missed heartbeats:

```mermaid
stateDiagram-v2
    [*] --> Alive : Node registers
    Alive --> Suspect : 3 missed heartbeats (3 s)
    Suspect --> Alive : Heartbeat received
    Suspect --> Dead : 5 missed heartbeats (5 s)
    Dead --> [*] : Leader election triggered
```

### Promotion Flow

```mermaid
flowchart TB
    Detect["Coordinator detects leader Node-A dead\nfor shard-0"] --> Query["Query replication_offset\nfrom all shard-0 replicas"]
    Query --> Select["Select replica with\nhighest replication_offset\n→ Node-D"]
    Select --> Raft["Commit MetadataOp::PromoteLeader\nvia Raft consensus"]
    Raft --> Update["Update routing table\n(all Coordinators see new leader)"]
    Update --> Activate["Node-D activates as leader\nbegins accepting writes"]
```

### Data Safety Note

If Node-A had WAL entries **not yet replicated** to Node-D at the time of failure, those entries are lost. This is an explicit trade-off of the async replication model:

> Only writes acknowledged by the **leader's WAL** (per durability config) are guaranteed. Replica lag is best-effort.

---

## gRPC Service Interface

The Replication Manager exposes the following service on each Storage Node:

| RPC | Direction | Description |
|-----|-----------|-------------|
| `StreamWal(shard_id, from_seq)` | Leader → Replica | Bidirectional streaming WAL push |
| `SnapshotRequest(shard_id)` | Replica → Leader | Initiate snapshot sync |

---

## Consistency Guarantees

| Guarantee | Detail |
|-----------|--------|
| **Read-your-writes** | Reads routed to leader only (default) — guaranteed |
| **Monotonic reads** | Leader-only reads are monotonic by definition |
| **Stale reads** *(opt-in, v2)* | Follower reads allowed with configurable max-lag threshold |
| **Replication lag** | No hard bound; replica is best-effort async |
| **Max data loss window** | Time since last replication ACK — typically < 100 ms on LAN |
