# Configuration Guide

Molock is highly configurable via YAML files. The default configuration can be found in `config/molock-config.yaml`.

## Server Configuration

The `server` section defines the basic network settings for the mock server.

```yaml
server:
  port: 8080
  workers: 4
  host: "0.0.0.0"
  max_request_size: 10485760  # 10MB
```

## Telemetry Configuration

Molock has native support for OpenTelemetry.

```yaml
telemetry:
  enabled: true
  service_name: "molock"
  service_version: "0.1.0"
  endpoint: "http://localhost:4317"
  protocol: "grpc"
  sampling_rate: 1.0
  log_level: "info"
  log_format: "json"
```

## Endpoints

Endpoints are the heart of Molock. Each endpoint consists of a `name`, `method`, `path`, and a list of `responses`.

### Request Matching

Molock matches incoming requests based on:
- **Method**: HTTP method (GET, POST, etc.)
- **Path**: Supports parameters (e.g., `/users/:id`) and regex.
- **Conditions**: Logic applied to headers, query parameters, and body.

### Response Features

- **Status**: The HTTP status code to return.
- **Body**: The response payload, supporting templates (e.g., `{{id}}`).
- **Headers**: Custom HTTP headers.
- **Delay**: Fixed (e.g., `50ms`) or random delays.
- **Probability**: Failure injection or random response selection.
- **Stateful Logic**: Simulate retries using state counters (e.g., `request_count`).

### Example: Dynamic Response

```yaml
  - name: "Get User"
    method: GET
    path: "/users/:id"
    responses:
      - status: 200
        delay: 50ms
        body: '{"id": "{{id}}", "name": "John Doe"}'
      - status: 404
        condition: "id == 'unknown'"
        body: '{"error": "User not found"}'
```
