#!/bin/bash
# Benchmark HyperspaceDB vs Competitors
# Task 3.2 from TODO_ADOPTION.md

set -e

echo "🚀 HyperspaceDB Benchmark Suite"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DIMENSIONS=1024
NUM_VECTORS=100000
BATCH_SIZE=1000

echo -e "${BLUE}Configuration:${NC}"
echo "  Dimensions: $DIMENSIONS"
echo "  Vectors: $NUM_VECTORS"
echo "  Batch Size: $BATCH_SIZE"
echo ""

# Build HyperspaceDB
echo -e "${GREEN}Building HyperspaceDB...${NC}"
cargo build --release --bin hyperspace-server
echo ""

# Start server
echo -e "${GREEN}Starting HyperspaceDB server...${NC}"
./target/release/hyperspace-server &
SERVER_PID=$!
sleep 3
echo "Server PID: $SERVER_PID"
echo ""

# Run benchmarks
echo -e "${GREEN}Running insert benchmark...${NC}"
python3 << 'EOF'
import time
import numpy as np
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")

# Create collection
client.create_collection("benchmark", dimension=1024)

# Benchmark insert
vectors = np.random.rand(100000, 1024).astype(np.float32)
start = time.time()

for i in range(0, len(vectors), 1000):
    batch = vectors[i:i+1000]
    for j, vec in enumerate(batch):
        client.insert(vector=vec.tolist(), metadata={"id": i+j})
    
    if (i + 1000) % 10000 == 0:
        elapsed = time.time() - start
        qps = (i + 1000) / elapsed
        print(f"Inserted {i+1000} vectors | {qps:.0f} QPS")

total_time = time.time() - start
final_qps = len(vectors) / total_time

print(f"\n✅ Insert Complete:")
print(f"   Total: {len(vectors)} vectors")
print(f"   Time: {total_time:.2f}s")
print(f"   QPS: {final_qps:.0f}")
EOF

echo ""
echo -e "${GREEN}Running search benchmark...${NC}"
python3 << 'EOF'
import time
import numpy as np
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")

# Benchmark search
query = np.random.rand(1024).astype(np.float32).tolist()
latencies = []

for i in range(1000):
    start = time.time()
    results = client.search(vector=query, top_k=10)
    latency = (time.time() - start) * 1000  # ms
    latencies.append(latency)

latencies.sort()
p50 = latencies[len(latencies)//2]
p95 = latencies[int(len(latencies)*0.95)]
p99 = latencies[int(len(latencies)*0.99)]
avg = sum(latencies) / len(latencies)

print(f"\n✅ Search Complete (1000 queries):")
print(f"   Avg: {avg:.2f}ms")
print(f"   P50: {p50:.2f}ms")
print(f"   P95: {p95:.2f}ms")
print(f"   P99: {p99:.2f}ms")
EOF

# Cleanup
echo ""
echo -e "${GREEN}Cleaning up...${NC}"
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null || true

echo ""
echo "✅ Benchmark complete!"
echo ""
echo "📊 Results Summary:"
echo "   See output above for detailed metrics"
echo "   Compare with Qdrant/Pinecone/Weaviate benchmarks"
