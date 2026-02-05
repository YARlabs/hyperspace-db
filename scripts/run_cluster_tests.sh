#!/bin/bash
set -e

# Build binaries first
echo "🔨 Building binaries..."
cargo build --release --bin hyperspace-server
cargo build --release --bin integration_tests

# Start Leader
echo "🚀 Starting Leader (50051)..."
./target/release/hyperspace-server --port 50051 --http-port 50050 > leader.log 2>&1 &
LEADER_PID=$!

# Start Follower
echo "🚀 Starting Follower (50052)..."
./target/release/hyperspace-server --port 50052 --http-port 50060 --role follower --leader http://localhost:50051 > follower.log 2>&1 &
FOLLOWER_PID=$!

echo "⏳ Waiting 5s for cluster startup..."
sleep 5

# Run Tests
echo "🧪 Running Integration Tests..."
set +e
./target/release/integration_tests
TEST_EXIT_CODE=$?
set -e

# Cleanup
echo "🛑 Stopping Cluster..."
kill $LEADER_PID $FOLLOWER_PID || true
wait $LEADER_PID $FOLLOWER_PID 2>/dev/null || true

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "✅ Tests Passed!"
    rm leader.log follower.log
    exit 0
else
    echo "❌ Tests Failed! (Exit Code: $TEST_EXIT_CODE)"
    echo "--- Leader Logs (Head) ---"
    head -n 20 leader.log
    echo "--- Leader Logs (Tail) ---"
    tail -n 20 leader.log
    echo "--- Follower Logs (Tail) ---"
    tail -n 20 follower.log
    exit 1
fi
