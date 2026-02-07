# HyperspaceDB Architecture

## System Overview

```mermaid
graph TB
    subgraph "Client Layer"
        PY[Python SDK]
        TS[TypeScript SDK]
        WASM[WASM Module]
        LC[LangChain]
    end
    
    subgraph "API Layer"
        GRPC[gRPC Server<br/>Tonic]
    end
    
    subgraph "Core Engine"
        IDX[HNSW Index<br/>Hyperbolic/Euclidean]
        STORE[Vector Store<br/>MMap/RAM]
        QUANT[Quantization<br/>ScalarI8/Binary]
        MERKLE[Merkle Tree<br/>256 Buckets]
    end
    
    subgraph "Storage Layer"
        WAL[Write-Ahead Log]
        SNAP[Snapshots<br/>rkyv]
        CHUNKS[Segmented Files<br/>chunk_*.hyp]
    end
    
    subgraph "Replication"
        LEADER[Leader Node]
        FOLLOWER[Follower Node]
    end
    
    PY --> GRPC
    TS --> GRPC
    LC --> GRPC
    WASM --> STORE
    WASM --> IDX
    
    GRPC --> IDX
    IDX --> STORE
    IDX --> QUANT
    IDX --> MERKLE
    
    STORE --> CHUNKS
    STORE --> WAL
    IDX --> SNAP
    
    LEADER --> MERKLE
    FOLLOWER --> MERKLE
    MERKLE -.Delta Sync.-> FOLLOWER
    
    style WASM fill:#f9f,stroke:#333,stroke-width:2px
    style MERKLE fill:#9f9,stroke:#333,stroke-width:2px
    style IDX fill:#99f,stroke:#333,stroke-width:2px
```

## Data Flow: Insert Operation

```mermaid
sequenceDiagram
    participant Client
    participant gRPC
    participant Index
    participant Store
    participant WAL
    participant Merkle
    
    Client->>gRPC: insert(vector, metadata)
    gRPC->>Index: insert(vector)
    Index->>Store: allocate(id)
    Store->>WAL: append(id, vector)
    WAL-->>Store: ✓
    Store->>Store: write to mmap
    Store-->>Index: internal_id
    Index->>Index: build HNSW graph
    Index->>Merkle: update_bucket(id)
    Merkle->>Merkle: recompute hash
    Merkle-->>Index: ✓
    Index-->>gRPC: internal_id
    gRPC-->>Client: success
```

## Data Flow: Search Operation

```mermaid
sequenceDiagram
    participant Client
    participant gRPC
    participant Index
    participant Store
    participant SIMD
    
    Client->>gRPC: search(query, k=10)
    gRPC->>Index: search(query, k)
    Index->>Index: select entry point
    loop HNSW Traversal
        Index->>Store: get_vector(id)
        Store->>SIMD: distance(query, vec)
        SIMD-->>Index: distance
        Index->>Index: update candidates
    end
    Index->>Index: sort top-k
    Index-->>gRPC: [(id, dist), ...]
    gRPC-->>Client: results
```

## Replication Flow: Merkle Delta Sync

```mermaid
sequenceDiagram
    participant Follower
    participant Leader
    participant Merkle
    
    Follower->>Leader: get_root_hash()
    Leader->>Merkle: compute_root()
    Merkle-->>Leader: root_hash
    Leader-->>Follower: root_hash
    
    alt Hashes Match
        Follower->>Follower: Already in sync ✓
    else Hashes Differ
        loop For each bucket (0..256)
            Follower->>Leader: get_bucket_hash(i)
            Leader-->>Follower: bucket_hash
            alt Bucket differs
                Follower->>Leader: sync_bucket(i)
                Leader->>Leader: serialize bucket
                Leader-->>Follower: bucket_data
                Follower->>Follower: apply bucket
            end
        end
    end
    
    Follower->>Follower: Sync complete ✓
```

## Storage Layout

```mermaid
graph LR
    subgraph "Disk"
        WAL[wal.log]
        SNAP[index.snap]
        C0[chunk_0.hyp]
        C1[chunk_1.hyp]
        CN[chunk_N.hyp]
    end
    
    subgraph "Memory"
        MMAP[Memory Map]
        IDX_MEM[HNSW Graph]
    end
    
    C0 -.mmap.-> MMAP
    C1 -.mmap.-> MMAP
    CN -.mmap.-> MMAP
    SNAP -.load.-> IDX_MEM
    WAL -.replay.-> IDX_MEM
    
    style MMAP fill:#ff9,stroke:#333,stroke-width:2px
    style IDX_MEM fill:#9ff,stroke:#333,stroke-width:2px
```

## Component Details

### HNSW Index Structure

```mermaid
graph TD
    L2_0[Layer 2: Entry Point]
    L1_0[Layer 1: Node 0]
    L1_1[Layer 1: Node 1]
    L0_0[Layer 0: Node 0]
    L0_1[Layer 0: Node 1]
    L0_2[Layer 0: Node 2]
    L0_3[Layer 0: Node 3]
    
    L2_0 --> L1_0
    L2_0 --> L1_1
    L1_0 --> L0_0
    L1_0 --> L0_1
    L1_1 --> L0_2
    L1_1 --> L0_3
    L0_0 --> L0_1
    L0_1 --> L0_2
    L0_2 --> L0_3
    
    style L2_0 fill:#f99,stroke:#333,stroke-width:2px
    style L1_0 fill:#9f9,stroke:#333,stroke-width:2px
    style L1_1 fill:#9f9,stroke:#333,stroke-width:2px
```

### Merkle Tree Structure

```mermaid
graph TD
    ROOT[Root Hash]
    B0[Bucket 0<br/>Hash]
    B1[Bucket 1<br/>Hash]
    B255[Bucket 255<br/>Hash]
    
    V0_0[Vec 0]
    V0_1[Vec 256]
    V1_0[Vec 1]
    V1_1[Vec 257]
    V255_0[Vec 255]
    
    ROOT --> B0
    ROOT --> B1
    ROOT --> B255
    
    B0 --> V0_0
    B0 --> V0_1
    B1 --> V1_0
    B1 --> V1_1
    B255 --> V255_0
    
    style ROOT fill:#f99,stroke:#333,stroke-width:3px
    style B0 fill:#9f9,stroke:#333,stroke-width:2px
    style B1 fill:#9f9,stroke:#333,stroke-width:2px
    style B255 fill:#9f9,stroke:#333,stroke-width:2px
```

## Edge-Cloud Federation

```mermaid
graph TB
    subgraph "Browser (WASM)"
        WASM_APP[Web App]
        WASM_DB[HyperspaceDB<br/>WASM Core]
        IDB[IndexedDB]
    end
    
    subgraph "Cloud"
        SERVER[HyperspaceDB<br/>Server]
        DISK[Persistent<br/>Storage]
    end
    
    WASM_APP --> WASM_DB
    WASM_DB --> IDB
    WASM_DB -.Merkle Sync.-> SERVER
    SERVER --> DISK
    
    style WASM_DB fill:#f9f,stroke:#333,stroke-width:2px
    style SERVER fill:#99f,stroke:#333,stroke-width:2px
```

## Technology Stack

```mermaid
graph LR
    subgraph "Core"
        RUST[Rust<br/>Nightly]
        SIMD[SIMD<br/>AVX2/NEON]
        TOKIO[Tokio<br/>Async Runtime]
    end
    
    subgraph "Storage"
        MMAP[memmap2<br/>Zero-Copy]
        RKYV[rkyv<br/>Serialization]
    end
    
    subgraph "Network"
        TONIC[Tonic<br/>gRPC]
        PROTO[Protobuf]
    end
    
    subgraph "WASM"
        WBIND[wasm-bindgen]
        REXIE[rexie<br/>IndexedDB]
    end
    
    RUST --> SIMD
    RUST --> TOKIO
    RUST --> MMAP
    RUST --> RKYV
    RUST --> TONIC
    TONIC --> PROTO
    RUST --> WBIND
    WBIND --> REXIE
```

## Performance Characteristics

| Operation | Latency | Throughput | Notes |
|-----------|---------|------------|-------|
| **Insert** | 110 μs | 9,087 QPS | Async WAL + Background indexing |
| **Search (1M)** | 0.18 ms (p99) | 14,600 QPS | SIMD distance + HNSW |
| **Snapshot** | 500 ms | - | Background task, non-blocking |
| **Merkle Sync** | 11s (1% delta) | - | Bucket-level granularity |
| **WASM Load** | 50 ms | - | IndexedDB deserialization |

## Deployment Topologies

### Single Node
```
┌─────────────────┐
│  HyperspaceDB   │
│   (Standalone)  │
└─────────────────┘
```

### Leader-Follower
```
┌─────────┐    Merkle    ┌───────────┐
│ Leader  │─────Sync────▶│ Follower  │
└─────────┘              └───────────┘
```

### Multi-Region
```
┌─────────┐              ┌─────────┐
│ US-East │◀────Sync────▶│ EU-West │
└─────────┘              └─────────┘
     │                        │
     └────────Sync────────────┘
              │
         ┌─────────┐
         │ AP-South│
         └─────────┘
```

### Edge-Cloud
```
┌──────────┐              ┌──────────┐
│ Browser  │              │  Cloud   │
│  (WASM)  │◀────Sync────▶│  Server  │
└──────────┘              └──────────┘
```
