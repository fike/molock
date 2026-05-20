# Observability

Molock is designed with observability as a core requirement. It integrates with OpenTelemetry to provide deep insights into its internal operations and the traffic it handles.

## Core Concepts

- **Traces**: Every request handled by Molock generates a span, allowing you to trace the lifecycle of a mock request and see matching logic and delays.
- **Metrics**: Molock exports Prometheus-compatible metrics for request counts, error rates, and latency histograms.
- **Logs**: Structured JSON logs are correlated with trace IDs for easy troubleshooting.

## Local Observability Stack

You can start a complete monitoring stack locally using Docker Compose:

```bash
docker-compose -f deployment/docker-compose.yml up -d
```

### Accessing the Tools

- **Jaeger (Traces)**: [http://localhost:16686](http://localhost:16686)
- **Prometheus (Metrics)**: [http://localhost:9090](http://localhost:9090)
- **Grafana (Dashboards)**: [http://localhost:3000](http://localhost:3000)

## Validating Observability

We provide a script to verify that your OTel configuration is working correctly and that data is reaching the collector:

```bash
./tests/validate_observability.sh
```
