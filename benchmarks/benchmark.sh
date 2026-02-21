#!/bin/bash
set -e

# Molock Apache Benchmark Test Suite
# ===================================
# This script runs comprehensive performance tests against the Molock mock server
# using Apache Benchmark (ab).

# Configuration
SERVER_PORT=18080  # Different port to avoid conflicts with other services
BENCHMARK_CONCURRENCY=10
BENCHMARK_REQUESTS=1000
BENCHMARK_TIMEOUT=30
SERVER_STARTUP_WAIT=3
SERVER_SHUTDOWN_WAIT=2
DOCKER_MODE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check for ab (Apache Benchmark)
check_ab_installed() {
    log_info "Checking for Apache Benchmark (ab)..."
    if ! command -v ab &> /dev/null; then
        log_error "Apache Benchmark (ab) is not installed."
        echo "Install it with one of the following commands:"
        echo "  Ubuntu/Debian: sudo apt-get install apache2-utils"
        echo "  RHEL/CentOS: sudo yum install httpd-tools"
        echo "  macOS: brew install apache-httpd"
        exit 1
    fi
    log_success "Apache Benchmark found: $(ab -V 2>&1 | head -1)"
}

# Build the project if needed
build_project() {
    log_info "Building Molock project..."
    if ! cargo build --release &> build.log; then
        log_error "Failed to build project. Check build.log for details."
        exit 1
    fi
    log_success "Project built successfully"
}

# Start Molock server in background
start_server() {
    log_info "Starting Molock server on port $SERVER_PORT..."
    
    # Kill any existing server on this port
    lsof -ti:$SERVER_PORT | xargs kill -9 2>/dev/null || true
    
    # Start server in background with benchmark config
    cargo run --release -- --config config/benchmark-config.yaml > server.log 2>&1 &
    SERVER_PID=$!
    
    # Verify server is running
    local max_retries=5
    local retry_count=0
    
    while [ $retry_count -lt $max_retries ]; do
        if curl -s -f http://localhost:$SERVER_PORT/health > /dev/null 2>&1; then
            log_success "Server started successfully (PID: $SERVER_PID)"
            return 0
        fi
        retry_count=$((retry_count + 1))
        sleep 1
    done
    
    log_error "Failed to start server. Check server.log for details."
    kill $SERVER_PID 2>/dev/null || true
    exit 1
}

# Run a single benchmark test
run_benchmark() {
    local test_name="$1"
    local url="$2"
    local method="${3:-GET}"
    local data_file="${4:-}"
    
    log_info "Running benchmark: $test_name"
    echo "================================================"
    echo "Test: $test_name"
    echo "URL: $url"
    echo "Method: $method"
    echo "Requests: $BENCHMARK_REQUESTS"
    echo "Concurrency: $BENCHMARK_CONCURRENCY"
    echo "================================================"
    
    local ab_cmd="ab -n $BENCHMARK_REQUESTS -c $BENCHMARK_CONCURRENCY -t $BENCHMARK_TIMEOUT"
    
    if [ "$method" = "POST" ] && [ -n "$data_file" ]; then
        ab_cmd="$ab_cmd -p $data_file -T application/json"
    fi
    
    # Run the benchmark
    if ! $ab_cmd "$url" 2>&1; then
        log_warning "Benchmark $test_name had issues (non-zero exit code)"
    fi
    
    echo -e "\n"
}

# Run health endpoint benchmark
benchmark_health() {
    run_benchmark "Health Endpoint" "http://localhost:$SERVER_PORT/health"
}

# Run users endpoint benchmark
benchmark_users() {
    run_benchmark "Users Endpoint (static ID)" "http://localhost:$SERVER_PORT/users/123"
    run_benchmark "Users Endpoint (dynamic ID)" "http://localhost:$SERVER_PORT/users/$(date +%s)"
}

# Run echo endpoint benchmark (POST with JSON)
benchmark_echo() {
    # Create a test JSON file
    local test_data_file="/tmp/molock_benchmark_data.json"
    cat > "$test_data_file" << EOF
{
    "test": "benchmark",
    "timestamp": "$(date -Iseconds)",
    "data": {
        "field1": "value1",
        "field2": 12345,
        "field3": true
    }
}
EOF
    
    run_benchmark "Echo Endpoint (POST JSON)" "http://localhost:$SERVER_PORT/echo" "POST" "$test_data_file"
    
    # Clean up
    rm -f "$test_data_file"
}

# Run delayed response benchmark
benchmark_delayed() {
    run_benchmark "Delayed Response (100ms)" "http://localhost:$SERVER_PORT/slow"
}

# Run all benchmarks
run_all_benchmarks() {
    log_info "Starting comprehensive benchmark suite..."
    echo ""
    
    benchmark_health
    benchmark_users
    benchmark_echo
    benchmark_delayed
    
    log_success "All benchmarks completed"
}

# Generate benchmark report
generate_report() {
    log_info "Generating benchmark summary..."
    
    # Create reports directory if it doesn't exist
    mkdir -p benchmarks/reports
    
    local report_file="benchmarks/reports/benchmark_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# Molock Benchmark Report
Generated: $(date)

## Test Environment
- Server Port: $SERVER_PORT
- Benchmark Tool: Apache Benchmark $(ab -V 2>&1 | head -1 | cut -d' ' -f4-)
- Total Requests: $BENCHMARK_REQUESTS per test
- Concurrency Level: $BENCHMARK_CONCURRENCY
- Test Timeout: ${BENCHMARK_TIMEOUT}s

## Server Configuration
\`\`\`yaml
$(head -20 config/molock-config.yaml)
\`\`\`

## Notes
- Benchmarks were run against a locally deployed Molock server
- Server was built in release mode for optimal performance
- Each test simulates realistic load with concurrent connections
- Results may vary based on system resources and background processes

## Next Steps
1. Review individual benchmark outputs for detailed metrics
2. Compare results across different configurations
3. Adjust server parameters (workers, timeouts) based on findings
4. Run benchmarks in isolated environment for consistent results

EOF
    
    log_success "Report generated: $report_file"
}

# Cleanup function
cleanup() {
    if [ "$DOCKER_MODE" = true ]; then
        log_info "Docker mode: No cleanup needed (services remain running)"
        return 0
    fi
    
    log_info "Cleaning up..."
    
    # Kill the server
    if [ -n "$SERVER_PID" ] && kill -0 $SERVER_PID 2>/dev/null; then
        log_info "Stopping Molock server (PID: $SERVER_PID)..."
        kill $SERVER_PID 2>/dev/null || true
        sleep $SERVER_SHUTDOWN_WAIT
    fi
    
    # Clean up any remaining processes
    lsof -ti:$SERVER_PORT | xargs kill -9 2>/dev/null || true
    
    log_success "Cleanup completed"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -c|--concurrency)
                BENCHMARK_CONCURRENCY="$2"
                shift 2
                ;;
            -n|--requests)
                BENCHMARK_REQUESTS="$2"
                shift 2
                ;;
            -p|--port)
                SERVER_PORT="$2"
                shift 2
                ;;
            -d|--docker)
                DOCKER_MODE=true
                SERVER_PORT=8080  # Default Docker port
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Show help
show_help() {
    cat << EOF
Molock Benchmark Runner

Usage: $0 [OPTIONS]

Options:
  -c, --concurrency NUM   Set concurrency level (default: $BENCHMARK_CONCURRENCY)
  -n, --requests NUM      Set total number of requests (default: $BENCHMARK_REQUESTS)
  -p, --port PORT         Set server port (default: $SERVER_PORT)
  -d, --docker            Run benchmarks against Docker Compose stack (port: 8080)
  -h, --help              Show this help message

Examples:
  $0                       # Run all benchmarks with default settings
  $0 -c 20 -n 5000        # Run with 20 concurrent connections, 5000 requests
  $0 -p 18081             # Run on port 18081
  $0 -d                   # Run benchmarks against Docker Compose stack

EOF
}

# Main execution
main() {
    log_info "Starting Molock Benchmark Suite"
    echo "================================================"
    
    # Parse arguments
    parse_args "$@"
    
    if [ "$DOCKER_MODE" = true ]; then
        log_info "Running in Docker mode (port: $SERVER_PORT)"
        log_info "Assuming Docker Compose stack is already running..."
        log_info "If not, run: make docker-run"
        echo ""
    fi
    
    # Set up trap for cleanup (only for non-Docker mode)
    if [ "$DOCKER_MODE" = false ]; then
        trap cleanup EXIT INT TERM
    fi
    
    # Run setup steps
    check_ab_installed
    
    if [ "$DOCKER_MODE" = false ]; then
        build_project
        start_server
    else
        log_info "Skipping local build and server start (Docker mode)"
        # Wait a moment to ensure Docker services are ready
        sleep 2
    fi
    
    # Run benchmarks
    run_all_benchmarks
    
    # Generate report
    generate_report
    
    if [ "$DOCKER_MODE" = true ]; then
        log_info ""
        log_info "Docker benchmark completed!"
        log_info "Observability UIs:"
        log_info "- Jaeger (traces): http://localhost:16686"
        log_info "- Grafana (metrics): http://localhost:3000 (admin/admin)"
        log_info "- Prometheus: http://localhost:9090"
        log_info ""
        log_info "To stop Docker services: make docker-down"
        log_info "To view logs: docker-compose -f deployment/docker-compose.yml logs -f"
    else
        log_success "Benchmark suite completed successfully!"
    fi
}

# Run main function
main "$@"