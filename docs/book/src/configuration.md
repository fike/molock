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

## Error Injection and Resilience Testing

Molock is designed to help developers test how their applications handle failures. You can inject errors using probabilistic or stateful rules.

### Probabilistic Failures (Chaos Engineering)

Inject failures randomly to test circuit breakers and error handling logic.

```yaml
  - name: "Probabilistic Error"
    method: GET
    path: "/api/unstable"
    responses:
      - status: 200
        probability: 0.9
        body: '{"status": "ok"}'
      - status: 503
        probability: 0.1
        body: '{"error": "Service Unavailable"}'
```

### Stateful Failure Patterns (Retry Testing)

Simulate flaky services that fail initially but succeed upon retry by using `stateful: true` and the `request_count` variable.

```yaml
  - name: "Flaky Service"
    method: GET
    path: "/api/retry"
    stateful: true
    responses:
      - status: 500
        condition: "request_count <= 2"
        body: '{"error": "Temporary Failure", "attempt": {{request_count}}}'
      - status: 200
        condition: "request_count > 2"
        body: '{"status": "Success", "after": "{{request_count}} attempts"}'
```

### Simulating Rate Limits

You can simulate API rate limits (HTTP 429) using stateful counters.

```yaml
  - name: "Rate Limiter"
    method: GET
    path: "/api/limited"
    stateful: true
    responses:
      - status: 200
        condition: "request_count <= 100"
        body: '{"data": "some results"}'
      - status: 429
        condition: "request_count > 100"
        headers:
          Retry-After: "3600"
        body: '{"error": "Rate limit exceeded"}'
```
