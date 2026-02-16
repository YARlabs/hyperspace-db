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
pip install -r requirements.txt

echo "🦀 Installing Hyperspace Python SDK in editable mode..."
pip install -e ../sdks/python

# 3. Deploy Infrastructure
echo "🐳 Deploying Docker containers (HyperspaceDB & Competitors)..."
docker-compose up -d

echo "⏳ Waiting for HyperspaceDB to be healthy..."
MAX_RETRIES=30
COUNT=0
until curl -s http://localhost:50050/api/metrics > /dev/null || [ $COUNT -eq $MAX_RETRIES ]; do
    sleep 2
    COUNT=$((COUNT + 1))
    echo -n "."
done

if [ $COUNT -eq $MAX_RETRIES ]; then
    echo "❌ Error: HyperspaceDB failed to start. Logs:"
    docker-compose logs hyperspace
    exit 1
fi
echo -e "\n✅ Infrastructure is ready!"

# 4. Run Benchmark
echo "🎯 Running Performance1024D1M Benchmark..."
python3 -u run_benchmark.py --case=Performance1024D1M

echo "--------------------------------------------------"
echo "✅ Benchmark complete!"
echo "💡 Results are stored in BENCHMARK_RESULTS.md"
echo "--------------------------------------------------"
