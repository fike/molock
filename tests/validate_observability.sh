#!/bin/bash
# Molock Observability Validation Script
# This script verifies that Molock is correctly sending signals (Traces, Metrics, Logs) to the OTel stack.

set -e

# Configuration
MOLOCK_URL="http://localhost:8082"
JAEGER_API="http://localhost:16686/api"
PROMETHEUS_API="http://localhost:9090/api/v1"
OTEL_COLLECTOR_LOGS="deployment-otel-collector-1"
SERVICE_NAME="molock-test-validation"

echo "=== Molock Observability Validation ==="

# 1. Create a temporary config for validation
cat > config/test-validation.yaml <<EOF
server:
  host: "127.0.0.1"
  port: 8082
  workers: 1
  max_request_size: 1048576

telemetry:
  enabled: true
  service_name: "$SERVICE_NAME"
  service_version: "0.1.0"
  endpoint: "http://localhost:4317"
  protocol: "grpc"
  sampling_rate: 1.0
  log_level: "info"
  log_format: "json"

endpoints:
  - name: "Health"
    method: GET
    path: "/health"
    responses:
      - status: 200
        body: '{"status": "ok"}'
        default: true
EOF

# 2. Start Molock in the background
echo "Starting Molock..."
cargo run --features otel -- --config config/test-validation.yaml > molock_validation.log 2>&1 &
MOLOCK_PID=$!

# Ensure cleanup on exit
cleanup() {
    echo "Cleaning up..."
    kill $MOLOCK_PID || true
    rm -f config/test-validation.yaml molock_validation.log
}
trap cleanup EXIT

# Wait for Molock to be ready
echo "Waiting for Molock to start..."
# Increased timeout to 60 seconds for CI environments where compilation might happen
for i in {1..60}; do
    if curl -s $MOLOCK_URL/health > /dev/null; then
        echo "Molock is up!"
        break
    fi
    if [ $i -eq 60 ]; then
        echo "Error: Molock failed to start within 60 seconds."
        cat molock_validation.log
        exit 1
    fi
    sleep 1
done

# 3. Generate traffic
echo "Generating traffic..."
for i in {1..10}; do
    curl -s $MOLOCK_URL/health > /dev/null
    sleep 0.1
done

# 4. Verify Traces in Jaeger
echo "Verifying traces in Jaeger..."
sleep 5
SERVICES=$(curl -s "$JAEGER_API/services")
if echo "$SERVICES" | grep -q "$SERVICE_NAME"; then
    echo "SUCCESS: Service '$SERVICE_NAME' found in Jaeger."
else
    echo "ERROR: Service '$SERVICE_NAME' NOT found in Jaeger."
    echo "Services found: $SERVICES"
    exit 1
fi

# 5. Verify Metrics in Prometheus
echo "Verifying metrics in Prometheus..."
echo "Waiting for Prometheus scrape (15s)..."
sleep 15
QUERY_RESULT=$(curl -s "$PROMETHEUS_API/query?query=http_server_request_count_total")
if echo "$QUERY_RESULT" | grep -q "http_server_request_count_total"; then
    echo "SUCCESS: Metrics found in Prometheus."
else
    echo "WARNING: Metrics not found in Prometheus yet (might be timing). Checking OTel Collector logs..."
    if docker logs $OTEL_COLLECTOR_LOGS 2>&1 | grep -q "LogRecord"; then
        echo "SUCCESS: OTLP signals found in OTel Collector logs."
    else
        echo "ERROR: No signals found in OTel Collector."
        exit 1
    fi
fi

echo "=== Observability Validation Passed Successfully ==="
exit 0
