#!/bin/bash
# Legacy benchmark script - redirects to new unified benchmark system

set -e

echo "⚠️  This script has been superseded by the unified benchmark suite."
echo ""
echo "📊 For reproducible, fair benchmarks against Qdrant, Weaviate, and Milvus:"
echo ""
echo "   cd benchmarks"
echo "   ./run_all.sh"
echo ""
echo "This will:"
echo "  1. Start all databases in Docker"
echo "  2. Run identical workload on each"
echo "  3. Generate comparison report"
echo ""
read -p "Run unified benchmark now? (y/N) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    cd "$(dirname "$0")/../benchmarks"
    ./run_all.sh
else
    echo "Cancelled. See benchmarks/README.md for details."
fi
