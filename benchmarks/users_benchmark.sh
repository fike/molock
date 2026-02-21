#!/bin/bash
set -e

# Users Endpoint Benchmark
# ========================
# Benchmark for user endpoints with path parameters and different scenarios.

# Configuration
SERVER_PORT=18080
BENCHMARK_CONCURRENCY=20  # Higher concurrency for user endpoints
BENCHMARK_REQUESTS=3000
BENCHMARK_TIMEOUT=60

# Colors
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=== Users Endpoint Benchmark Suite ===${NC}"
echo "Server Port: $SERVER_PORT"
echo ""

# Check if server is running
if ! curl -s http://localhost:$SERVER_PORT/health > /dev/null; then
    echo "Error: Molock server not running on port $SERVER_PORT"
    echo "Start it with: cargo run --release -- --config config/benchmark-config.yaml"
    exit 1
fi

# Test 1: Static user ID (caching test)
echo -e "${YELLOW}Test 1: Static User ID (ID: 123)${NC}"
echo "----------------------------------------"
ab -n $BENCHMARK_REQUESTS \
   -c $BENCHMARK_CONCURRENCY \
   -t $BENCHMARK_TIMEOUT \
   -e users_static.csv \
   http://localhost:$SERVER_PORT/users/123
echo ""

# Test 2: Dynamic user IDs (no caching)
echo -e "${YELLOW}Test 2: Dynamic User IDs${NC}"
echo "--------------------------------"
echo "Generating dynamic user IDs..."
for i in {1..5}; do
    user_id=$((RANDOM % 10000 + 1000))
    echo "  Batch $i: Testing user ID $user_id"
    ab -n $((BENCHMARK_REQUESTS / 5)) \
       -c $BENCHMARK_CONCURRENCY \
       -t $((BENCHMARK_TIMEOUT / 5)) \
       -q \
       http://localhost:$SERVER_PORT/users/$user_id
done
echo ""

# Test 3: Mixed user IDs (simulating real traffic)
echo -e "${YELLOW}Test 3: Mixed User IDs Pattern${NC}"
echo "---------------------------------------"
echo "Simulating realistic user access pattern..."
cat > /tmp/user_ids.txt << EOF
/users/1001
/users/1002
/users/1003
/users/1004
/users/1005
/users/1001
/users/1002
/users/1003
/users/9999  # Non-existent user (should return 404)
/users/unknown  # Special case from config
EOF

ab -n $BENCHMARK_REQUESTS \
   -c $BENCHMARK_CONCURRENCY \
   -t $BENCHMARK_TIMEOUT \
   -T "application/json" \
   -p /tmp/user_ids.txt \
   -e users_mixed.csv \
   http://localhost:$SERVER_PORT/

# Clean up
rm -f /tmp/user_ids.txt

echo ""
echo -e "${BLUE}=== Benchmark Analysis ===${NC}"
echo "Three scenarios tested:"
echo "1. Static ID: Measures caching/optimization potential"
echo "2. Dynamic IDs: Measures path parameter parsing performance"
echo "3. Mixed pattern: Simulates realistic traffic with error cases"
echo ""
echo "Compare results between static and dynamic IDs to understand"
echo "the impact of path parameter parsing on performance."