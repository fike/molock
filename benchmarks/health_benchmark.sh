#!/bin/bash
set -e

# Health Endpoint Benchmark
# =========================
# Focused benchmark for the health endpoint to measure baseline performance.

# Configuration
SERVER_PORT=18080
BENCHMARK_CONCURRENCY=10
BENCHMARK_REQUESTS=5000  # Higher for baseline
BENCHMARK_TIMEOUT=60

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Health Endpoint Benchmark ===${NC}"
echo "Testing: GET /health"
echo "Port: $SERVER_PORT"
echo "Requests: $BENCHMARK_REQUESTS"
echo "Concurrency: $BENCHMARK_CONCURRENCY"
echo ""

# Check if server is running
if ! curl -s http://localhost:$SERVER_PORT/health > /dev/null; then
    echo "Error: Molock server not running on port $SERVER_PORT"
    echo "Start it with: cargo run --release -- --config config/benchmark-config.yaml"
    exit 1
fi

# Run benchmark with detailed output
echo "Running benchmark..."
echo "===================="

ab -n $BENCHMARK_REQUESTS \
   -c $BENCHMARK_CONCURRENCY \
   -t $BENCHMARK_TIMEOUT \
   -k \
   -e health_benchmark.csv \
   http://localhost:$SERVER_PORT/health

echo ""
echo -e "${GREEN}=== Benchmark Complete ===${NC}"
echo "Results saved to health_benchmark.csv"
echo ""
echo "Key metrics to check:"
echo "1. Requests per second"
echo "2. Time per request (mean)"
echo "3. 99th percentile latency"
echo "4. Error rate (should be 0%)"