# Getting Started

To get started with Molock, you'll need to install the project and run the server.

## Prerequisites

- **Rust**: Version 1.70 or higher.
- **Docker & Docker Compose**: For running the observability stack (Jaeger, Prometheus, Grafana).
- **Make**: (Optional) For using the provided shortcuts in the `Makefile`.

## Installation

```bash
# Clone the repository
git clone https://github.com/fike/molock.git
cd molock

# Build the project
make build

# Run tests
make test

# Start the server with default configuration
make run
```

## Kubernetes Deployment

Molock can be deployed to Kubernetes using the provided Helm chart located in `deployment/molock`.

```bash
# Install the chart
helm install my-molock ./deployment/molock
```

See the [Helm Chart README](../../../deployment/molock/README.md) for detailed configuration options.

## Immediate Usage

Once the server is running, you can test it with `curl`:

```bash
# Health check (with dynamic timestamp)
curl http://localhost:8080/health

# Path parameter matching
curl http://localhost:8080/users/123

# Regex/Condition matching (triggers 404 if not found)
curl http://localhost:8080/users/unknown
```
