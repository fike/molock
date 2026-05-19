// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

use crate::config::TelemetryConfig;
use crate::telemetry::attributes;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing::{error, info, warn};

/// Initializes OpenTelemetry metrics.
///
/// # Errors
///
/// Returns an error if the metric exporter cannot be built.
#[cfg(feature = "otel")]
pub fn init_metrics(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Metrics are disabled");
        return Ok(());
    }

    let protocol = config.protocol.to_lowercase();
    let exporter = if protocol == "grpc" {
        opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
    } else {
        if protocol != "http" {
            warn!("Unknown protocol '{protocol}', defaulting to HTTP");
        }
        let endpoint = if config.endpoint.contains("/v1/metrics") {
            config.endpoint.clone()
        } else if config.endpoint.ends_with('/') {
            format!("{endpoint}v1/metrics", endpoint = config.endpoint)
        } else {
            format!("{endpoint}/v1/metrics", endpoint = config.endpoint)
        };
        info!("Initializing OpenTelemetry metrics with HTTP protocol to: {endpoint}");
        opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
    }
    .map_err(|e| {
        error!("Failed to build OpenTelemetry metric exporter: {e}");
        anyhow::anyhow!("OpenTelemetry metric exporter build failed: {e}")
    })?;

    // Create meter provider with the exporter
    // Wrap exporter in a PeriodicReader for regular export
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(10))
        .build();

    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", "0.1.0"),
        ])
        .build();

    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build();

    opentelemetry::global::set_meter_provider(meter_provider);

    #[cfg(debug_assertions)]
    {
        info!("[TELEMETRY DEBUG] Metrics configured with 10-second export interval and explicit histogram buckets");
    }
    Ok(())
}

/// Initializes metrics when OpenTelemetry is disabled.
///
/// # Errors
///
/// This function is infallible but returns `Result` for API consistency.
#[cfg(not(feature = "otel"))]
pub fn init_metrics(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Metrics are disabled");
        return Ok(());
    }

    #[cfg(debug_assertions)]
    {
        info!("[TELEMETRY DEBUG] Metrics initialized in no-op mode (OTel disabled)");
    }
    Ok(())
}

pub fn record_request(method: &str, path: &str, status: u16) {
    let meter = opentelemetry::global::meter("molock");
    let counter = meter.u64_counter("http.server.request.count").build();

    let attributes = vec![
        attributes::kv::http_method(method),
        attributes::kv::http_route(path),
        attributes::kv::http_response_status_code(status),
    ];

    counter.add(1, &attributes);
}

pub fn record_latency(method: &str, path: &str, latency_ms: f64) {
    let meter = opentelemetry::global::meter("molock");
    let histogram = meter
        .f64_histogram("http.server.request.duration")
        .with_unit("ms")
        .build();

    let attributes = vec![
        attributes::kv::http_method(method),
        attributes::kv::http_route(path),
    ];

    histogram.record(latency_ms, &attributes);
}

pub fn record_error(method: &str, path: &str, error_type: &str) {
    let meter = opentelemetry::global::meter("molock");
    let counter = meter.u64_counter("http.server.request.errors").build();

    let attributes = vec![
        attributes::kv::http_method(method),
        attributes::kv::http_route(path),
        attributes::kv::error_type(error_type),
    ];

    counter.add(1, &attributes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_request() {
        record_request("GET", "/test", 200);
    }

    #[test]
    fn test_record_latency() {
        record_latency("GET", "/test", 10.5);
    }

    #[test]
    fn test_record_error() {
        record_error("GET", "/test", "not_found");
    }

    #[test]
    fn test_record_request_multi() {
        record_request("GET", "/test", 200);
        record_request("POST", "/api", 201);
        record_request("GET", "/test", 404);
    }

    #[test]
    fn test_record_latency_multi() {
        record_latency("GET", "/test", 10.5);
        record_latency("POST", "/api", 50.2);
    }

    #[test]
    fn test_record_error_multi() {
        record_error("GET", "/test", "not_found");
        record_error("POST", "/api", "validation_failed");
    }

    #[test]
    fn test_edge_case_latencies() {
        record_latency("GET", "/test", 0.0);
        record_latency("GET", "/test", 0.001);
        record_latency("GET", "/test", 999999.9);
        record_latency("GET", "/test", -1.0);
    }

    #[test]
    fn test_init_metrics_http() {
        let config = TelemetryConfig {
            protocol: "http".to_string(),
            endpoint: "http://localhost:4318".to_string(),
            ..TelemetryConfig::default()
        };
        let _ = init_metrics(&config);
    }

    #[test]
    fn test_init_metrics_http_with_slash() {
        let config = TelemetryConfig {
            protocol: "http".to_string(),
            endpoint: "http://localhost:4318/".to_string(),
            ..TelemetryConfig::default()
        };
        let _ = init_metrics(&config);
    }

    #[test]
    fn test_init_metrics_http_with_v1_metrics() {
        let config = TelemetryConfig {
            protocol: "http".to_string(),
            endpoint: "http://localhost:4318/v1/metrics".to_string(),
            ..TelemetryConfig::default()
        };
        let _ = init_metrics(&config);
    }

    #[test]
    fn test_init_metrics_unknown_protocol() {
        let config = TelemetryConfig {
            protocol: "unknown".to_string(),
            ..TelemetryConfig::default()
        };
        let _ = init_metrics(&config);
    }
}
