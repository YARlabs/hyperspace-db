#!/bin/bash
# Complete test and build verification script for HyperspaceDB

set -e

echo "======================================"
echo "HyperspaceDB - Complete Build & Test"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 1. Backend Check
echo -e "${YELLOW}[1/5] Running cargo check...${NC}"
cargo check -p hyperspace-server
echo -e "${GREEN}✅ Backend check passed${NC}"
echo ""

# 2. Backend Tests
echo -e "${YELLOW}[2/5] Running Rust unit tests...${NC}"
cargo test --lib
echo -e "${GREEN}✅ Rust tests passed${NC}"
echo ""

# 3. Frontend Build
echo -e "${YELLOW}[3/5] Building frontend...${NC}"
cd dashboard
npm run build
cd ..
echo -e "${GREEN}✅ Frontend build passed${NC}"
echo ""

# 4. Full Release Build
echo -e "${YELLOW}[4/5] Building release binary...${NC}"
cargo build --release -p hyperspace-server
echo -e "${GREEN}✅ Release build passed${NC}"
echo ""

# 5. Integration Tests (optional - requires running server)
echo -e "${YELLOW}[5/5] Integration tests${NC}"
echo "To run integration tests:"
echo "  1. Start server: HYPERSPACE_API_KEY=test_key_12345 ./target/release/hyperspace-server"
echo "  2. Run tests: python3 tests/integration_test.py"
echo ""

echo "======================================"
echo -e "${GREEN}✅ ALL CHECKS PASSED${NC}"
echo "======================================"
echo ""
echo "Next steps:"
echo "  • Run server: ./target/release/hyperspace-server"
echo "  • Access dashboard: http://localhost:50050"
echo "  • View logs: tail -f data/*/wal/*.log"
