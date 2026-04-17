# RutSeriDB — io_uring + Direct I/O (Phase 3)

> **Related:** [architecture.md](../architecture.md) · [storage/format.md](./format.md) · [components.md](../components.md)
> **Status:** Planned — Phase 3
> **Linux kernel requirement:** ≥ 5.6 (registered buffers); ≥ 5.19 recommended

---

## Motivation

RutSeriDB's Phase 0–2 storage engine uses standard POSIX I/O via `tokio::fs` (which internally uses a blocking thread pool with `read()`/`write()`/`fsync()` syscalls). This is correct and portable, but leaves performance on the table in two areas:

| Bottleneck | Root Cause | Phase 3 Solution |
|-----------|-----------|-----------------|
| Syscall overhead per I/O | Every `read()`/`write()` is a context switch | io_uring ring buffer: batch N ops in 1 submit |
| WAL fsync under load | Sequential syscalls even for independent writes | Concurrent WAL submissions via io_uring |
| Part reads scan more files than necessary | OS page cache evicts hot columns when Parts flush | Direct I/O for Part writes: only our buffer pool is the cache |
| Multi-column reads are sequential | One `spawn_blocking` call per column block | Parallel column block reads via io_uring batch submit |

---

## io_uring Primer

io_uring uses two lock-free ring buffers shared between userspace and the kernel:

```mermaid
flowchart LR
    subgraph Userspace["User Process"]
        App["Application\nRutSeriDB"]
        SQ["Submit Queue\n(SQ Ring)\nApp writes ops here"]
        CQ["Completion Queue\n(CQ Ring)\nApp reads results here"]
    end

    subgraph Kernel["Linux Kernel"]
        Worker["io_uring kernel worker\npolls SQ continuously\n(no syscall needed in SQPOLL mode)"]
        FS["File System / Block Layer"]
    end

    App -->|"enqueue ops"| SQ
    SQ -->|"io_uring_submit()\none syscall for N ops"| Worker
    Worker --> FS
    FS -->|"DMA → registered buffer"| CQ
    CQ -->|"io_uring_wait_cqe()"| App
```

**SQPOLL mode** (optional): the kernel spins on the SQ ring when busy — completely eliminates the `io_uring_submit()` syscall too.

---

## Direct I/O (O_DIRECT)

Opening a file with `O_DIRECT` bypasses the OS page cache:

```mermaid
flowchart TB
    subgraph Buffered["Buffered I/O (current)"]
        Write1["write(fd, buf)"] --> PageCache["OS Page Cache\n(kernel RAM)"] --> Disk1["Disk"]
        Read1["read(fd, buf)"] --> PageCache
        Note1["Data may exist in kernel cache AND app buffer\n(double buffering)\nKernel controls eviction — hard to predict"]
    end

    subgraph Direct["Direct I/O (O_DIRECT)"]
        Write2["write(fd, aligned_buf)"] --> Disk2["Disk\n(DMA, no kernel copy)"]
        Read2["read(fd, aligned_buf)"] --> Disk2
        Note2["No kernel copy\nApp controls caching via its own pool\nPredictable — kernel cache not involved"]
    end
```

### Requirements for Direct I/O

| Requirement | Detail |
|-------------|--------|
| Buffer alignment | Buffer start address must be aligned to block size (512 B or 4096 B) |
| I/O size alignment | Read/write size must be a multiple of block size (512 B or 4096 B) |
| File offset alignment | Byte offset must be a multiple of block size |

This means `.rpart` column block boundaries must be padded to 4096-byte alignment (see [format.md § Alignment](./format.md)).

---

## io_uring Registered Buffers

The highest-performance mode: pre-register fixed buffers with the kernel so DMA can write directly into your buffer — true zero-copy from disk.

```mermaid
sequenceDiagram
    participant App as RutSeriDB
    participant Kernel

    App->>Kernel: io_uring_register_buffers(bufs, n)\n(one-time setup at startup)
    Note over App,Kernel: Buffers are now pinned in kernel memory\nDMA can target them directly

    loop Per query
        App->>Kernel: IORING_OP_READ_FIXED(fd, buf_idx, offset, len)
        Note over Kernel: DMA from disk → pinned buffer\nNo kernel-to-user memcpy
        Kernel-->>App: completion event
        App->>App: LZ4 decompress from registered buffer
    end
```

---

## Changes by Component

### WAL Writer

```mermaid
sequenceDiagram
    participant IE  as Ingest Engine
    participant WAL as WAL Writer (Phase 3)
    participant SQ  as io_uring SQ Ring

    IE->>WAL: Batch of writes (10 ms SyncBatch window)
    WAL->>SQ: Enqueue WRITE(fd, buf1, len1)
    WAL->>SQ: Enqueue WRITE(fd, buf2, len2)
    WAL->>SQ: Enqueue WRITE(fd, bufN, lenN)
    WAL->>SQ: Enqueue FDATASYNC(fd)
    WAL->>SQ: io_uring_submit() ← ONE syscall for all
    SQ-->>WAL: All completions via CQ Ring
    WAL-->>IE: OK (all flushed)
```

**File flags:** `O_WRONLY | O_APPEND | O_DIRECT | O_DSYNC`  
**Alignment:** WAL records padded to 512-byte boundaries

### Part File Writer (Flush)

Part files are written with `O_DIRECT` to prevent flushing cold data into the OS page cache — which would evict hot column data being actively queried.

```mermaid
flowchart TB
    Flush["MemTable flush → Part file write"] --> Direct["O_DIRECT write\nPadded to 4096-byte boundaries\nDMA: app buffer → disk\nNo kernel page cache involvement"]
    Direct --> Result["Hot column data stays in\nour managed read buffer pool\n— not evicted by cold flush"]
```

### Part File Reader (Query)

```mermaid
sequenceDiagram
    participant P   as Query Planner
    participant R   as Part Reader (Phase 3)
    participant SQ  as io_uring SQ Ring
    participant RBP as Read Buffer Pool

    P->>R: Read columns [timestamps, cpu] from part-007

    R->>RBP: Check cache for each block
    Note over R: timestamps block: MISS · cpu block: MISS

    R->>SQ: Enqueue READ_FIXED(fd, buf_idx=0, offset=1024, len=512)
    R->>SQ: Enqueue READ_FIXED(fd, buf_idx=1, offset=4096, len=1024)
    R->>SQ: io_uring_submit() ← ONE syscall for both reads
    SQ-->>R: Both completions (potentially concurrent disk reads)

    par Parallel decompression
        R->>R: LZ4 decode buf_idx=0 → timestamps array
        R->>R: LZ4 decode buf_idx=1 → cpu array
    end

    R->>RBP: Cache decoded blocks (LRU eviction)
    R-->>P: (timestamps[], cpu[])
```

---

## .rpart Format Change for Direct I/O (v2)

The `.rpart` format needs one change: **all column block start offsets must be aligned to 4096 bytes**.

```mermaid
flowchart TB
    subgraph v1["v1 format (current)"]
        H1["Header (64 B)"]
        C1a["Col 0 data (variable len)"]
        C1b["Col 1 data (immediately follows)"]
        N1["No alignment padding"]
    end

    subgraph v2["v2 format (Phase 3, Direct I/O)"]
        H2["Header (64 B)"]
        P0["Padding → 4096-byte boundary"]
        C2a["Col 0 data (starts at 4096-byte boundary)"]
        P1["Padding → next 4096-byte boundary"]
        C2b["Col 1 data (starts at 4096-byte boundary)"]
        N2["ColumnHeader.data_offset always % 4096 == 0"]
    end
```

- `version` field in header: `1` → `2`
- Backward compatible: v2 readers can still read v1 files (no Direct I/O on v1 files)
- v1 readers will refuse v2 files (version mismatch)

---

## Read Buffer Pool (Managed Cache)

With Direct I/O, the OS no longer caches anything. RutSeriDB must manage its own cache:

```mermaid
flowchart TB
    Query["Column block requested"] --> Check{"In pool?"}
    Check -- Hit --> Return["Return decoded column\n(zero I/O, zero alloc)"]
    Check -- Miss --> Read["io_uring READ_FIXED\nfrom disk → registered buffer"]
    Read --> Decode["LZ4 decode"]
    Decode --> Insert["Insert decoded block into pool\n(LRU eviction if full)"]
    Insert --> Return

    subgraph Pool["Read Buffer Pool\nconfig: read_buffer_size_bytes (default 128 MB)\nKey: (part_id, col_idx, block_idx)\nValue: decoded column slice\nEviction: LRU"]
        B1["part-001 · col=cpu · blk=0"]
        B2["part-001 · col=ts  · blk=0"]
        B3["part-007 · col=cpu · blk=0"]
    end
```

---

## Performance Impact Estimates

| Operation | Phase 0–2 | Phase 3 | Expected gain |
|-----------|----------|---------|--------------|
| WAL fsync per SyncBatch | 1 fsync syscall | 1 io_uring_submit for N writes + 1 fdatasync | N× write throughput |
| Part read — 2 columns | 2 sequential spawn_blocking | 2 concurrent io_uring READ_FIXED | ~1.5–2× for cold reads |
| Part write (flush) | Pollutes OS page cache | O_DIRECT — no cache pollution | Better cache hit rates on reads |
| Read buffer hit | OS page cache (uncontrolled) | Managed LRU pool (controlled) | More predictable latency |
| Syscall overhead under load | High (many context switches) | Near-zero (ring buffer) | 0.5–2 μs saved per op |

---

## Rust Crates

| Crate | Role |
|-------|------|
| `tokio-uring` | io_uring integration with Tokio runtime — minimal code change from `tokio::fs` |
| `io-uring` | Low-level bindings for buffer registration and advanced features |
| `aligned-vec` | `AlignedVec<u8>` for 4096-byte aligned Direct I/O buffers |

---

## Implementation Plan (Phase 3)

```mermaid
flowchart LR
    P3_1["3.1 — WAL io_uring\nSwitch WAL writes to tokio-uring\nO_DIRECT + O_DSYNC\nBatch per SyncBatch window\nPad WAL records to 512-byte boundary"] --> P3_2

    P3_2["3.2 — .rpart v2 format\nAdd 4096-byte padding to column blocks\nBump format version field\nUpdate Part writer + reader"] --> P3_3

    P3_3["3.3 — Part reader io_uring\nRegister read buffer pool with kernel\nSubmit all column reads per query as\none io_uring batch\nParallel LZ4 decode with Rayon"] --> P3_4

    P3_4["3.4 — Part writer Direct I/O\nOpen Part file with O_DIRECT\nEnsure buffer alignment\nEliminate cold flush cache pollution"] --> P3_5

    P3_5["3.5 — Managed read buffer pool\nLRU pool keyed by (part_id, col_idx)\nReplace implicit OS page cache reliance\nExpose pool hit/miss metrics\nin Prometheus /metrics"]
```

---

## Non-Goals for Phase 3

| Non-Goal | Reason |
|----------|--------|
| SQPOLL mode (kernel-side polling) | Extreme latency reduction; complex; adds CPU cost — revisit in v2 |
| io_uring for MemTable operations | MemTable is in-memory; no I/O involved |
| Removing LZ4 compression in favour of raw binary (QuestDB style) | Compression saves 2–5× disk and network bandwidth; critical for replication |
| mmap-based reads | mmap has TLB shootdown costs under concurrency; Direct I/O + managed pool is better for parallel multi-column reads |

---

## Related Documents

| Document | Relevance |
|----------|-----------|
| [format.md](./format.md) | v2 alignment changes to `.rpart` layout |
| [ingestion/wal.md](../ingestion/wal.md) | WAL I/O path and durability levels |
| [components.md](../components.md) | Part Writer/Reader component details |
| [../architecture.md](../architecture.md) | Phase 3 in Implementation Checklist |
