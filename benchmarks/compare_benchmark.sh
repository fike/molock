#!/bin/bash
set -e

# Molock vs Killgrave Benchmark Comparison
# ========================================

# Configuration
MOLOCK_URL="http://localhost:8080"
MOCKSERVER_URL="http://localhost:8081"

# Attempt to increase open files limit
ulimit -n 65535 2>/dev/null || true

CONCURRENCIES=(100 200 300)
REQUESTS=10000
TIMEOUT=30
REPORTS_DIR="benchmarks/reports"
REPORT_FILE="$REPORTS_DIR/MOLOCK_VS_MOCKSERVER_$(date +%Y%m%d_%H%M%S).md"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_header() { echo -e "\n${YELLOW}=== $1 ===${NC}"; }

# Check for ab
if ! command -v ab &> /dev/null; then
    echo "Error: Apache Benchmark (ab) is not installed."
    exit 1
fi

mkdir -p "$REPORTS_DIR"

# Header for the report
cat > "$REPORT_FILE" << EOF
# Benchmark Report: Molock (Rust) vs MockServer (Java)
Date: $(date)
Requests per test: $REQUESTS
Timeout: ${TIMEOUT}s

EOF

run_test() {
    local name="$1"
    local endpoint="$2"
    local method="$3"
    local data_file="$4"

    log_header "Testing Scenario: $name"
    echo "## Scenario: $name" >> "$REPORT_FILE"
    echo "| Tool | Concurrency | Requests/sec | Latency (mean) | 95% Latency | Errors (Fail/Non-2xx) |" >> "$REPORT_FILE"
    echo "| :--- | :--- | :--- | :--- | :--- | :--- |" >> "$REPORT_FILE"

    for c in "${CONCURRENCIES[@]}"; do
        for tool in "Molock" "MockServer"; do
            local url="$MOLOCK_URL"
            [ "$tool" == "MockServer" ] && url="$MOCKSERVER_URL"

            log_info "Running $tool with $c connections on $endpoint..."

            local ab_cmd="ab -n $REQUESTS -c $c -t $TIMEOUT"
            if [ "$method" == "POST" ]; then
                ab_cmd="$ab_cmd -p $data_file -T application/json"
            fi

            local output
            if ! output=$( $ab_cmd "$url$endpoint" 2>&1 ); then
                log_warning "$tool benchmark failed or had issues"
            fi

            local rps=$(echo "$output" | grep "Requests per second:" | awk '{print $4}')
            local latency=$(echo "$output" | grep "Time per request:" | head -1 | awk '{print $4}')
            local p95=$(echo "$output" | grep " 95%" | awk '{print $2}')
            local failed=$(echo "$output" | grep "Failed requests:" | awk '{print $3}')
            local non2xx=$(echo "$output" | grep "Non-2xx responses:" | awk '{print $3}')

            # Defaults
            rps=${rps:-"0.00"}
            latency=${latency:-"N/A"}
            p95=${p95:-"N/A"}
            failed=${failed:-"0"}
            non2xx=${non2xx:-"0"}

            echo "| $tool | $c | $rps | ${latency}ms | ${p95}ms | $failed / $non2xx |" >> "$REPORT_FILE"
        done
    done
    echo "" >> "$REPORT_FILE"
}

# 1. Start Docker
log_info "Starting benchmark environment..."
docker-compose -f deployment/docker-compose-benchmark.yml up -d --build

# Wait for services
log_info "Waiting for services to be ready..."

# Wait for Molock
log_info "Waiting for Molock (8080)..."
until curl -s "$MOLOCK_URL/health" > /dev/null; do
  sleep 1
done

# Wait for MockServer (can take long due to 6GB AlwaysPreTouch)
log_info "Waiting for MockServer (8081)... this may take up to 60s due to 6GB RAM pre-touch..."
until curl -s -X PUT "$MOCKSERVER_URL/mockserver/status" > /dev/null; do
  sleep 2
done

log_success "All services are UP and ready!"

# Create a test POST file
POST_DATA="/tmp/order_data.json"
echo '{"id": "123", "items": [{"id": "p1", "qty": 1}]}' > "$POST_DATA"

# Warm up
log_info "Warming up..."
curl -s "$MOLOCK_URL/health" > /dev/null
curl -s "$MOCKSERVER_URL/health" > /dev/null

# Run Scenarios
run_test "Health Check (Simple GET)" "/health" "GET"
run_test "User Retrieval (Regex Match)" "/users/123" "GET"
run_test "Order Creation (Simple POST)" "/orders" "POST" "$POST_DATA"


log_success "Benchmark completed!"
log_info "Report generated at: $REPORT_FILE"

# Cleanup
log_info "Stopping services..."
docker-compose -f deployment/docker-compose-benchmark.yml down
rm -f "$POST_DATA"

# Print summary
cat "$REPORT_FILE"
