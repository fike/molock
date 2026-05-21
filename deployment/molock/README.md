# Molock Helm Chart

A Helm chart for deploying Molock, a high-performance mock server with native observability.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.0+

## Installing the Chart

To install the chart with the release name `my-molock`:

```bash
helm install my-molock ./deployment/molock
```

## Configuration

The following table lists the configurable parameters of the Molock chart and their default values.

| Parameter | Description | Default |
| --------- | ----------- | ------- |
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Image repository | `molock` |
| `image.tag` | Image tag | `.Chart.AppVersion` |
| `service.type` | Service type | `ClusterIP` |
| `service.port` | Service port | `8080` |
| `config` | Molock application configuration | (see values.yaml) |
| `resources` | Resource requests and limits | (see values.yaml) |

### Customizing Molock Rules

You can customize the mock endpoints by modifying the `config` section in `values.yaml` or by providing a custom values file:

```bash
helm install my-molock ./deployment/molock -f my-values.yaml
```

Example `my-values.yaml`:

```yaml
config:
  endpoints:
    - name: "My New Endpoint"
      method: GET
      path: "/hello"
      responses:
        - status: 200
          body: '{"message": "Hello World"}'
```

## Observability

By default, the chart expects an OpenTelemetry collector to be available at `http://otel-collector:4317`. You can update this in `values.yaml`:

```yaml
config:
  telemetry:
    endpoint: "http://your-otel-collector:4317"
```
