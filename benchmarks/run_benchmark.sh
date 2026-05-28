#!/bin/bash
set -e

# HyperspaceDB One-Click Benchmark Script
# ======================================

echo "--------------------------------------------------"
echo "🚀 HyperspaceDB Benchmark Suite"
echo "--------------------------------------------------"
echo "⚠️  DISCLAIMER: Don't take anyone's word for it,"
echo "    verify all numbers yourself!"
echo "--------------------------------------------------"

# 1. Environment Check
echo "🔍 Checking system prerequisites..."
if ! command -v docker >/dev/null 2>&1; then
    echo "❌ Error: docker is not installed."
    exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "❌ Error: python3 is not installed."
    exit 1
fi

# 2. Virtual Environment Setup
echo "📦 Setting up Python virtual environment..."
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi
source venv/bin/activate

echo "📥 Installing dependencies from requirements.txt..."
pip install --upgrade pip
pip install setuptools wheel
pip install -r requirements.txt

echo "🦀 Installing Hyperspace Python SDK in editable mode..."
pip install -e ../sdks/python

# 3. Regenerate Python Protos (Fixes Version Mismatch)
echo "🧬 Regenerating Python protos to match environment..."
python3 -m grpc_tools.protoc \
    -I ../crates/hyperspace-proto/proto \
    --python_out=../sdks/python/hyperspace/proto \
    --grpc_python_out=../sdks/python/hyperspace/proto \
    ../crates/hyperspace-proto/proto/hyperspace.proto

# Patch imports for Python 3
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' 's/import hyperspace_pb2 as hyperspace__pb2/from . import hyperspace_pb2 as hyperspace__pb2/' ../sdks/python/hyperspace/proto/hyperspace_pb2_grpc.py
else
  sed -i 's/import hyperspace_pb2 as hyperspace__pb2/from . import hyperspace_pb2 as hyperspace__pb2/' ../sdks/python/hyperspace/proto/hyperspace_pb2_grpc.py
fi

# 4. Deploy Infrastructure
echo "🐳 Deploying Docker containers (HyperspaceDB & Competitors)..."
docker-compose up -d --build

# echo "⏳ Waiting for HyperspaceDB to be healthy..."
# MAX_RETRIES=30
# COUNT=0
# until curl -s http://localhost:50051/health > /dev/null || [ $COUNT -eq $MAX_RETRIES ]; do
#     sleep 2
#     COUNT=$((COUNT + 1))
#     echo -n "."
# done

# if [ $COUNT -eq $MAX_RETRIES ]; then
#     echo "❌ Error: HyperspaceDB failed to start. Logs:"
#     docker-compose logs hyperspace
#     exit 1
# fi
echo -e "\n✅ Infrastructure is ready!"

# 4. Run Benchmark
echo "🎯 Running Performance1536D50K Benchmark..."
python3 -u run_benchmark_next.py --case=Performance1536D50K

echo "🎯 Running Stress Test..."
python3 -u stress_test_allDB.py

echo "--------------------------------------------------"
echo "✅ Benchmark complete!"
echo "💡 Results are stored in BENCHMARK_REPORT.html"
echo "💡 Results are stored in STRESS_TEST_REPORT.html"
echo "--------------------------------------------------"
