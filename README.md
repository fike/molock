# Molock - High-Performance Mock Server

> ⚠️ **DISCLAIMER: Experimental Project**
>
> This project is an **experimental codebase** created using AI-assisted development.
>
> **Before using in production:**
> - Code has not been manually reviewed by humans
> - Security audits have not been completed
> - API/behavior may change without notice
> - Test coverage meets project standards but may have edge cases
>
> **Use at your own risk.**

Molock is a production-ready mock server for CI/CD pipelines, stress testing, and other testing scenarios. Built in Rust with Actix-web, it provides high-performance, configurable, and observable mock endpoints with OpenTelemetry integration.

## Features

- **High Performance**: Built with Rust and Actix-web for maximum throughput
- **Dynamic Rules**: Configure endpoints with flexible matching rules
- **Response Control**: Add delays, failure injection, and stateful behavior
- **OpenTelemetry Integration**: Built-in tracing, metrics, and logging
- **Hot Reload**: Watch configuration files for live updates
- **Docker Ready**: Production-ready container images
- **Comprehensive Testing**: >80% code coverage with unit and integration tests

## Quick Start

### Prerequisites
- Rust 1.70+
- Docker and Docker Compose (for deployment)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/molock.git
cd molock

# Build the project
make build

# Run tests
make test

# Start the server
make run
```

### Using Docker

```bash
# Build and run with Docker Compose
make docker-build
make docker-run
```

## Configuration

Molock uses YAML configuration files. See `config/molock-config.yaml` for examples:

```yaml
server:
  port: 8080
  workers: 4

endpoints:
  - name: "Get User"
    method: GET
    path: "/users/:id"
    responses:
      - status: 200
        delay: 50ms
        body: '{"id": "{{id}}", "name": "John Doe"}'
      - status: 404
        condition: "id == 'unknown'"
        body: '{"error": "not found"}'

  - name: "Retry Example"
    method: GET
    path: "/retry"
    stateful: true
    responses:
      - status: 200
        condition: "request_count > 2"
        body: "OK"
      - status: 503
        default: true
        body: "Service Unavailable"
```

### Configuration Options

- **Server**: Port, workers, host, and request size limits
- **Telemetry**: OpenTelemetry endpoint, service name, sampling rate
- **Logging**: Log level, format, and OpenTelemetry log integration
- **Endpoints**: HTTP methods, paths with parameters, response rules

### Response Features

- **Delays**: Fixed (`100ms`) or random ranges (`100-500ms`)
- **Conditions**: Simple expressions using request data
- **Probability**: Random response selection with weights
- **Stateful**: Per-client counters for retry logic
- **Templates**: Dynamic response generation with variables

## Observability

Molock integrates with OpenTelemetry for comprehensive observability:

- **Traces**: Request spans with timing and metadata
- **Metrics**: Request counts, errors, and latency histograms
- **Logs**: Structured JSON logging with trace context

### Local Development Stack

```bash
# Start the full observability stack
docker-compose -f deployment/docker-compose.yml up
```

Access the monitoring tools:
- **Jaeger**: http://localhost:16686 (traces)
- **Prometheus**: http://localhost:9090 (metrics)
- **Grafana**: http://localhost:3000 (dashboards)

## API Reference

### Health Check
```http
GET /health
```

Returns server health status.

### Metrics
```http
GET /metrics
```

Prometheus-formatted metrics (when not using OTLP).

### Mock Endpoints

All configured endpoints are available at their specified paths. The server matches requests based on:
- HTTP method
- Path (with parameter support: `/users/:id`)
- Query parameters
- Headers
- Request body

## Development

### Project Structure

```
molock/
├── src/
│   ├── config/     # Configuration loading and parsing
│   ├── server/     # Actix web server setup
│   ├── rules/      # Rule matching and execution
│   ├── telemetry/  # OpenTelemetry integration
│   └── utils/      # Helper functions
├── tests/          # Integration tests
├── config/         # Configuration files
├── deployment/     # Docker and deployment artifacts
└── .ai/           # AI-assisted development documentation
```

### Building and Testing

```bash
# Build release binary
make build

# Run all tests
make test

# Run tests with coverage
make test-coverage

# Check code quality
make lint
make fmt

# Development mode
make dev
```

### Benchmarking

Molock includes comprehensive Apache Benchmark (ab) tests for performance evaluation:

```bash
# Install Apache Benchmark (if not already installed)
sudo apt-get install apache2-utils  # Ubuntu/Debian
brew install apachebench            # macOS
sudo yum install httpd-tools        # RHEL/CentOS

# Run comprehensive benchmark suite
make benchmark

# Run individual benchmark scenarios
make benchmark-health     # Health endpoint benchmarks
make benchmark-users     # User endpoint benchmarks  
make benchmark-delay     # Delayed response benchmarks
make benchmark-post      # POST endpoint benchmarks
```

For detailed benchmarking instructions, see [BENCHMARKING.md](BENCHMARKING.md).

### Adding New Features (TDD Required)

1. **Write failing tests first** following TDD guidelines in `.ai/TDD_GUIDE.md`
2. **Implement minimal solution** to make tests pass
3. **Refactor** while keeping tests green
4. **Ensure 80%+ test coverage** using `make test-coverage`
5. Follow coding conventions in `.ai/Agents.md`
6. Update documentation as needed
7. Run `make test` and `make lint` before committing

**IMPORTANT**: All contributions MUST follow Test-Driven Development (TDD) principles. PRs without adequate test coverage (<80%) will be rejected.

## Deployment

### Docker

```bash
# Build the Docker image
docker build -f deployment/Dockerfile -t molock .

# Run the container
docker run -p 8080:8080 -v ./config:/etc/molock/config molock
```

### Kubernetes

See the `deployment/` directory for example Kubernetes manifests.

### Environment Variables

- `MOLOCK_CONFIG_PATH`: Path to configuration file
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector endpoint
- `OTEL_SERVICE_NAME`: Service name for telemetry
- `RUST_LOG`: Log level (info, debug, trace)

## Contributing

Please read `.ai/CONTRIBUTING.md` for details on our code of conduct and the process for submitting pull requests.

### For AI-Assisted Development

This project supports AI-assisted development workflows:

| Configuration File | Description |
|-------------------|-------------|
| `AGENTS.md` (root) | Project-specific instructions and conventions |
| `.ai/Agents.md` | Extended guidelines, TDD workflow, and skills |
| `.ai/.cursorrules` | IDE-specific rules |

#### Quick Commands

```bash
# Build and test
make build
make test

# Code quality
make lint
make fmt
make test-coverage

# Development
make run
make dev
```

See `.ai/Agents.md` for detailed guidelines, or `AGENTS.md` for a standardized format compatible with AI coding agents.

## License

This project is licensed under the MIT OR Apache-2.0 License - see the LICENSE file for details.

## Support

- **Issues**: Use the GitHub issue tracker
- **Documentation**: Check `.ai/Skills.md` for technical guidance
- **Questions**: Open a discussion on GitHub

## Acknowledgments

- Built with [Actix-web](https://actix.rs/)
- Observability with [OpenTelemetry](https://opentelemetry.io/)
- Configuration with [Serde](https://serde.rs/)
- Testing with [Tokio](https://tokio.rs/)
## License

Molock is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text.
