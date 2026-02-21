# Molock Benchmarking Guide

This document provides comprehensive guidance for benchmarking the Molock high-performance mock server using Apache Benchmark (ab).

## Prerequisites

1. **Apache Benchmark (ab)**: Install using your package manager:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install apache2-utils
   
   # macOS
   brew install apachebench
   
   # RHEL/CentOS
   sudo yum install httpd-tools
   ```

2. **Molock Server**: Ensure the server is built and ready:
   ```bash
   make build
   ```

## Available Benchmark Targets

The Makefile provides several benchmark targets:

| Target | Description | Command |
|--------|-------------|---------|
| `make benchmark` | Run comprehensive benchmark suite | `make benchmark` |
| `make benchmark-all` | Run all benchmark scenarios | `make benchmark-all` |
| `make benchmark-health` | Benchmark health endpoints | `make benchmark-health` |
| `make benchmark-users` | Benchmark user endpoints | `make benchmark-users` |
| `make benchmark-delay` | Benchmark delayed responses | `make benchmark-delay` |
| `make benchmark-post` | Benchmark POST endpoints | `make benchmark-post` |

## Benchmark Scripts

### 1. Main Benchmark Runner (`benchmarks/benchmark.sh`)
The primary script that runs all benchmark scenarios with comprehensive reporting.

**Features:**
- Tests multiple endpoint types (health, users, POST, delayed)
- Colored output for better readability
- Error handling and cleanup
- Performance report generation
- Configurable concurrency and request counts

**Usage:**
```bash
./benchmarks/benchmark.sh [concurrency] [requests] [port]
# Default: concurrency=100, requests=10000, port=18080
```

### 2. Individual Benchmark Scripts

#### Health Endpoint Benchmark (`benchmarks/health_benchmark.sh`)
Tests the `/health` endpoint which returns a simple JSON response.

**Command:**
```bash
./benchmarks/health_benchmark.sh [concurrency] [requests] [port]
```

**Expected Performance:**
- Very high throughput (10k+ requests/second)
- Low latency (< 1ms average)
- Minimal resource usage

#### User Endpoints Benchmark (`benchmarks/users_benchmark.sh`)
Tests user-related endpoints with path parameters:
- `/users` - List all users
- `/users/1` - Get specific user
- `/users/123` - Get user with ID 123

**Command:**
```bash
./benchmarks/users_benchmark.sh [concurrency] [requests] [port]
```

**Expected Performance:**
- High throughput (5k+ requests/second)
- Slightly higher latency due to route matching
- Consistent performance across different IDs

#### Delayed Response Benchmark (`benchmarks/delay_benchmark.sh`)
Tests endpoints with artificial delays:
- `/delay/100` - 100ms delay
- `/delay/500` - 500ms delay
- `/delay/1000` - 1 second delay

**Command:**
```bash
./benchmarks/delay_benchmark.sh [concurrency] [requests] [port]
```

**Expected Performance:**
- Throughput limited by delay duration
- Tests server's ability to handle concurrent delayed requests
- Useful for testing timeout handling and connection pooling

#### POST Endpoints Benchmark (`benchmarks/post_benchmark.sh`)
Tests POST endpoints with JSON payloads of varying sizes:
- `/echo` - Echo back POST data
- Small (100B), Medium (1KB), and Large (10KB) payloads

**Command:**
```bash
./benchmarks/post_benchmark.sh [concurrency] [requests] [port]
```

**Expected Performance:**
- Throughput decreases with larger payloads
- Tests JSON parsing and serialization performance
- Memory usage monitoring important

## Configuration

### Port Configuration
Benchmark scripts use port **18080** by default to avoid conflicts with the default Molock port (8080). You can change this by:
1. Modifying the scripts directly
2. Passing port as a command-line argument
3. Using a different configuration file

### Test Configuration
The benchmarks use the existing `config/molock-config.yaml` file which includes:
- Health check endpoint
- User endpoints with path parameters
- POST echo endpoint
- Delayed response endpoints

## Performance Expectations

Based on Molock's architecture (Rust + Actix-web), here are expected performance metrics:

### Hardware Baseline (4-core CPU, 8GB RAM)
| Endpoint Type | Requests/sec | Avg Latency | 95th %ile | Memory Usage |
|---------------|--------------|-------------|-----------|--------------|
| Health Check | 15,000-20,000 | < 1ms | < 2ms | < 50MB |
| User Endpoints | 8,000-12,000 | 1-2ms | < 5ms | < 100MB |
| POST (1KB) | 5,000-8,000 | 2-5ms | < 10ms | < 150MB |
| Delayed (100ms) | 800-1,200 | ~100ms | < 110ms | < 100MB |

### Factors Affecting Performance
1. **Concurrency Level**: Higher concurrency increases throughput but may increase latency
2. **Payload Size**: Larger payloads reduce throughput and increase memory usage
3. **Network Latency**: Local testing vs remote testing
4. **System Resources**: CPU cores, memory, and disk I/O

## Running Benchmarks

### Quick Start
```bash
# Build the server
make build

# Run comprehensive benchmarks
make benchmark

# Or run individual benchmarks
make benchmark-health
make benchmark-users
make benchmark-delay
make benchmark-post
```

### Custom Benchmark Run
```bash
# Run with custom parameters
./benchmarks/benchmark.sh 50 5000 18080
# 50 concurrent users, 5000 total requests, port 18080
```

## Interpreting Results

### Key Metrics to Monitor
1. **Requests per second**: Throughput measurement
2. **Time per request**: Average latency
3. **Transfer rate**: Network throughput
4. **Failed requests**: Error rate should be 0%
5. **Connection times**: Connect, processing, waiting, total

### Example Output Analysis
```
Concurrency Level:      100
Time taken for tests:   1.234 seconds
Complete requests:      10000
Failed requests:        0
Requests per second:    8100.37 [#/sec] (mean)
Time per request:       12.340 [ms] (mean)
Time per request:       0.123 [ms] (mean, across all concurrent requests)
Transfer rate:          645.45 [Kbytes/sec] received
```

### Performance Bottlenecks
1. **CPU Bound**: High CPU usage indicates computation limits
2. **Memory Bound**: Increasing memory usage with load
3. **I/O Bound**: Disk or network limitations
4. **Connection Limits**: File descriptor or connection pool limits

## Advanced Benchmarking

### Stress Testing
```bash
# Extreme load test
./benchmarks/benchmark.sh 1000 100000 18080
```

### Long-running Tests
```bash
# 30-second test with 50 concurrent users
ab -n 1000000 -c 50 -t 30 http://localhost:18080/health
```

### Comparing Configurations
1. Run benchmarks with different server configurations
2. Compare performance before/after optimizations
3. Test with different payload sizes and endpoint combinations

## Troubleshooting

### Common Issues
1. **Port already in use**: Change port in script or kill existing process
2. **Apache Benchmark not found**: Install apache2-utils package
3. **Server not starting**: Check logs and ensure binary is built
4. **High error rate**: Reduce concurrency or check server resources

### Debug Mode
```bash
# Run with verbose output
DEBUG=1 ./benchmarks/benchmark.sh
```

## Best Practices

1. **Warm-up**: Run a small test before full benchmark
2. **Multiple Runs**: Average results from 3+ runs
3. **System Monitoring**: Monitor CPU, memory, and network during tests
4. **Baseline**: Establish performance baseline for comparison
5. **Documentation**: Record test conditions and results

## Integration with CI/CD

Add benchmark tests to your CI pipeline:
```yaml
# Example GitHub Actions workflow
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: sudo apt-get install -y apache2-utils
      - run: make build
      - run: make benchmark-health
      - run: make benchmark-users
```

## Next Steps

1. **Automated Performance Regression Testing**
2. **Load Testing with Distributed Tools** (wrk, vegeta, k6)
3. **Real-world Scenario Simulation**
4. **Comparative Analysis** with other mock servers
5. **Resource Usage Profiling** under sustained load