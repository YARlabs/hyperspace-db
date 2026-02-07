#!/bin/bash
# Automated Vector Database Benchmark Runner
# Starts all databases in Docker and runs unified benchmark

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Vector Database Benchmark Suite${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not found. Please install Docker first.${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo -e "${RED}❌ Docker Compose not found. Please install Docker Compose first.${NC}"
    exit 1
fi

# Use docker-compose or docker compose
if command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE="docker-compose"
else
    DOCKER_COMPOSE="docker compose"
fi

# Install Python dependencies
echo -e "${GREEN}📦 Installing Python dependencies...${NC}"
pip install -q numpy qdrant-client weaviate-client pymilvus 2>/dev/null || {
    echo -e "${YELLOW}⚠️  Some Python packages failed to install. Continuing anyway...${NC}"
}

# Build HyperspaceDB Docker image
echo -e "${GREEN}🔨 Building HyperspaceDB Docker image...${NC}"
cd ..
docker build -t hyperspace-db:benchmark -f Dockerfile . || {
    echo -e "${YELLOW}⚠️  HyperspaceDB build failed. Will skip HyperspaceDB benchmark.${NC}"
}
cd benchmarks

# Start all databases
echo -e "${GREEN}🚀 Starting all vector databases...${NC}"
$DOCKER_COMPOSE up -d

# Wait for health checks
echo -e "${GREEN}⏳ Waiting for databases to be ready...${NC}"
sleep 10

# Check health
echo -e "${BLUE}Checking database health:${NC}"

check_health() {
    local name=$1
    local url=$2
    if curl -sf "$url" > /dev/null 2>&1; then
        echo -e "  ✅ $name"
        return 0
    else
        echo -e "  ❌ $name (not responding)"
        return 1
    fi
}

check_health "HyperspaceDB" "http://localhost:50051/health" || true
check_health "Qdrant" "http://localhost:6333/health" || true
check_health "Weaviate" "http://localhost:8080/v1/.well-known/ready" || true
check_health "Milvus" "http://localhost:9091/healthz" || true

echo ""
echo -e "${GREEN}🏃 Running benchmark...${NC}"
echo -e "${YELLOW}This will take approximately 5-10 minutes...${NC}"
echo ""

# Run benchmark
python3 run_benchmark.py

# Copy results to docs
if [ -f "BENCHMARK_RESULTS.md" ]; then
    cp BENCHMARK_RESULTS.md ../docs/BENCHMARK_RESULTS_REAL.md
    echo -e "${GREEN}✅ Results copied to docs/BENCHMARK_RESULTS_REAL.md${NC}"
fi

# Cleanup
echo ""
echo -e "${BLUE}Cleanup:${NC}"
read -p "Stop and remove all containers? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${GREEN}🧹 Cleaning up...${NC}"
    $DOCKER_COMPOSE down -v
    echo -e "${GREEN}✅ Cleanup complete${NC}"
else
    echo -e "${YELLOW}⚠️  Containers still running. Stop with: cd benchmarks && $DOCKER_COMPOSE down${NC}"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Benchmark Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Results: ${BLUE}benchmarks/BENCHMARK_RESULTS.md${NC}"
echo -e "Docs:    ${BLUE}docs/BENCHMARK_RESULTS_REAL.md${NC}"
echo ""
