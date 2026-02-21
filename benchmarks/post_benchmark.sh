#!/bin/bash
set -e

# POST/JSON Endpoint Benchmark
# ============================
# Benchmark for POST endpoints with JSON payloads to test parsing and processing.

# Configuration
SERVER_PORT=18080
BENCHMARK_CONCURRENCY=15
BENCHMARK_REQUESTS=2000
BENCHMARK_TIMEOUT=60

# Colors
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}=== POST/JSON Endpoint Benchmark ===${NC}"
echo "Testing POST endpoints with JSON payloads"
echo "Measures JSON parsing, request body handling, and response generation"
echo ""

# Check if server is running
if ! curl -s http://localhost:$SERVER_PORT/health > /dev/null; then
    echo "Error: Molock server not running on port $SERVER_PORT"
    echo "Start it with: cargo run --release -- --config config/benchmark-config.yaml"
    exit 1
fi

# Create test JSON payloads of different sizes
echo "Creating test payloads..."

# Small payload (~100 bytes)
cat > /tmp/small_payload.json << EOF
{
    "action": "test",
    "id": 12345,
    "timestamp": "$(date -Iseconds)"
}
EOF

# Medium payload (~1KB)
cat > /tmp/medium_payload.json << EOF
{
    "action": "create_order",
    "order_id": "ORD-$(date +%s)",
    "customer": {
        "id": "CUST-1001",
        "name": "John Doe",
        "email": "john@example.com"
    },
    "items": [
        {"id": "ITEM-001", "name": "Product A", "quantity": 2, "price": 29.99},
        {"id": "ITEM-002", "name": "Product B", "quantity": 1, "price": 49.99},
        {"id": "ITEM-003", "name": "Product C", "quantity": 3, "price": 9.99}
    ],
    "total": 149.94,
    "currency": "USD"
}
EOF

# Large payload (~10KB)
cat > /tmp/large_payload.json << EOF
{
    "action": "bulk_import",
    "batch_id": "BATCH-$(date +%s)",
    "records": [
EOF

# Generate 100 records for large payload
for i in {1..100}; do
    cat >> /tmp/large_payload.json << EOF
        {
            "record_id": "REC-$i",
            "data": {
                "field1": "Value $i",
                "field2": $((i * 100)),
                "field3": $((i % 2 == 0)),
                "metadata": {
                    "created_at": "$(date -Iseconds)",
                    "source": "benchmark",
                    "sequence": $i
                }
            }
        }$(if [ $i -lt 100 ]; then echo ","; fi)
EOF
done

cat >> /tmp/large_payload.json << EOF
    ],
    "summary": {
        "total_records": 100,
        "total_size": "~10KB",
        "processed_at": "$(date -Iseconds)"
    }
}
EOF

echo -e "${PURPLE}Test 1: Small JSON Payload (~100 bytes)${NC}"
echo "---------------------------------------------"
ab -n $BENCHMARK_REQUESTS \
   -c $BENCHMARK_CONCURRENCY \
   -t $BENCHMARK_TIMEOUT \
   -p /tmp/small_payload.json \
   -T "application/json" \
   -k \
   -e post_small.csv \
   http://localhost:$SERVER_PORT/echo
echo ""

echo -e "${PURPLE}Test 2: Medium JSON Payload (~1KB)${NC}"
echo "---------------------------------------------"
ab -n $((BENCHMARK_REQUESTS / 2)) \
   -c $BENCHMARK_CONCURRENCY \
   -t $BENCHMARK_TIMEOUT \
   -p /tmp/medium_payload.json \
   -T "application/json" \
   -k \
   -e post_medium.csv \
   http://localhost:$SERVER_PORT/echo
echo ""

echo -e "${PURPLE}Test 3: Large JSON Payload (~10KB)${NC}"
echo "---------------------------------------------"
ab -n $((BENCHMARK_REQUESTS / 10)) \
   -c $((BENCHMARK_CONCURRENCY / 3)) \
   -t $BENCHMARK_TIMEOUT \
   -p /tmp/large_payload.json \
   -T "application/json" \
   -k \
   -e post_large.csv \
   http://localhost:$SERVER_PORT/echo
echo ""

# Test 4: POST to orders endpoint (if configured)
echo -e "${PURPLE}Test 4: Orders Endpoint (Business Logic)${NC}"
echo "------------------------------------------------"
if curl -s -X POST http://localhost:$SERVER_PORT/orders -H "Content-Type: application/json" -d '{"test": "probe"}' > /dev/null 2>&1; then
    echo "Orders endpoint found. Testing..."
    ab -n $BENCHMARK_REQUESTS \
       -c $BENCHMARK_CONCURRENCY \
       -t $BENCHMARK_TIMEOUT \
       -p /tmp/medium_payload.json \
       -T "application/json" \
       -e post_orders.csv \
       http://localhost:$SERVER_PORT/orders
else
    echo "Orders endpoint not configured or not accepting POST"
    echo "Skipping this test..."
fi

# Clean up
rm -f /tmp/small_payload.json /tmp/medium_payload.json /tmp/large_payload.json

echo ""
echo -e "${BLUE}=== Performance Analysis ===${NC}"
echo "Payload size comparison:"
echo "1. Small (100B): Baseline for minimal overhead"
echo "2. Medium (1KB): Typical API payload size"
echo "3. Large (10KB): Stress test for JSON parsing"
echo ""
echo "Expected trends:"
echo "- Requests/second should decrease as payload size increases"
echo "- Latency should increase with payload size"
echo "- Throughput (MB/s) may increase with larger payloads"
echo ""
echo "Use these results to:"
echo "1. Set appropriate request size limits"
echo "2. Optimize JSON parsing for common payload sizes"
echo "3. Configure timeouts based on expected payload sizes"