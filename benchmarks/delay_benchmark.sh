#!/bin/bash
set -e

# Delayed Response Benchmark
# ==========================
# Benchmark for endpoints with artificial delays to test timeout handling
# and concurrent request management.

# Configuration
SERVER_PORT=18080
BENCHMARK_CONCURRENCY=5   # Lower concurrency for delayed responses
BENCHMARK_REQUESTS=100    # Fewer requests due to delays
BENCHMARK_TIMEOUT=120     # Longer timeout for delayed responses

# Colors
BLUE='\033[0;34m'
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${BLUE}=== Delayed Response Benchmark ===${NC}"
echo "Testing endpoints with artificial delays"
echo "This measures how well the server handles concurrent delayed requests"
echo ""

# Check if server is running
if ! curl -s http://localhost:$SERVER_PORT/health > /dev/null; then
    echo "Error: Molock server not running on port $SERVER_PORT"
    echo "Start it with: cargo run --release -- --config config/benchmark-config.yaml"
    exit 1
fi

# Verify delayed endpoint exists
echo "Checking delayed endpoint..."
if ! curl -s --max-time 5 http://localhost:$SERVER_PORT/slow > /dev/null; then
    echo -e "${RED}Warning: /slow endpoint not configured or not responding${NC}"
    echo "Please ensure your config has a delayed response endpoint like:"
    echo "  - name: \"Delayed Response\""
    echo "    method: GET"
    echo "    path: \"/slow\""
    echo "    responses:"
    echo "      - status: 200"
    echo "        delay: 3s"
    echo "        body: '{\"message\": \"Delayed response\"}'"
    exit 1
fi

echo -e "${GREEN}Delayed endpoint found. Starting benchmarks...${NC}"
echo ""

# Test 1: Single delayed request (baseline)
echo -e "${BLUE}Test 1: Single Delayed Request (Baseline)${NC}"
echo "-----------------------------------------------"
time curl -s --max-time 10 http://localhost:$SERVER_PORT/slow > /dev/null
echo "Expected: ~3 seconds (config delay)"
echo ""

# Test 2: Concurrent delayed requests
echo -e "${BLUE}Test 2: Concurrent Delayed Requests${NC}"
echo "-----------------------------------------"
echo "Testing $BENCHMARK_CONCURRENCY concurrent requests with delays"
echo "This tests the server's ability to handle multiple waiting requests"

ab -n $BENCHMARK_REQUESTS \
   -c $BENCHMARK_CONCURRENCY \
   -t $BENCHMARK_TIMEOUT \
   -k \
   -e delayed_concurrent.csv \
   http://localhost:$SERVER_PORT/slow
echo ""

# Test 3: Mixed workload (some delayed, some fast)
echo -e "${BLUE}Test 3: Mixed Workload Benchmark${NC}"
echo "--------------------------------------"
echo "Testing mixed endpoints (health = fast, slow = delayed)"

# Create a file with mixed URLs
cat > /tmp/mixed_urls.txt << EOF
http://localhost:$SERVER_PORT/health
http://localhost:$SERVER_PORT/slow
http://localhost:$SERVER_PORT/health
http://localhost:$SERVER_PORT/health
http://localhost:$SERVER_PORT/slow
EOF

echo "Running mixed workload (75% fast, 25% slow)..."
ab -n $((BENCHMARK_REQUESTS * 4)) \
   -c $BENCHMARK_CONCURRENCY \
   -t $BENCHMARK_TIMEOUT \
   -T "application/json" \
   -p /tmp/mixed_urls.txt \
   -e delayed_mixed.csv

# Clean up
rm -f /tmp/mixed_urls.txt

echo ""
echo -e "${GREEN}=== Benchmark Insights ===${NC}"
echo "Key metrics to analyze:"
echo "1. Total time vs concurrent connections"
echo "2. Request rate for delayed endpoints"
echo "3. Connection handling under load"
echo "4. Timeout and error rates"
echo ""
echo "Expected patterns:"
echo "- Single delayed request: ~3s"
echo "- Concurrent requests: Should complete in ~3s + overhead"
echo "- Mixed workload: Health endpoints should not be blocked by slow ones"