/*
 * Copyright 2026 Molock Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::config::TelemetryConfig;
use crate::telemetry::attributes;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing::{error, info, warn};

#[cfg(feature = "otel")]
pub async fn init_metrics(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Metrics are disabled");
        return Ok(());
    }

    info!(
        "Initializing OpenTelemetry metrics with endpoint: {}, protocol: {}",
        config.endpoint, config.protocol
    );

    // Debug logging
    if crate::telemetry::is_debug_enabled() {
        info!("[TELEMETRY DEBUG] Metrics initialization starting");
        info!(
            "[TELEMETRY DEBUG] Endpoint: {}, Protocol: {}, Export period: 10s",
            config.endpoint, config.protocol
        );
    }

    // Create resource with service name and version
    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
        ])
        .build();

    // Configure OTLP exporter based on protocol
    let protocol = config.protocol.to_lowercase();

    // Debug logging for protocol selection
    if crate::telemetry::is_debug_enabled() {
        info!(
            "[TELEMETRY DEBUG] Selecting metrics exporter for protocol: {}",
            protocol
        );
    }

    let exporter = match protocol.as_str() {
        "grpc" => {
            info!(
                "Configuring gRPC exporter for metrics with endpoint: {}",
                config.endpoint
            );
            if crate::telemetry::is_debug_enabled() {
                info!("[TELEMETRY DEBUG] Using gRPC (tonic) exporter for metrics");
            }
            opentelemetry_otlp::MetricExporter::builder()
                .with_tonic()
                .with_endpoint(&config.endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
        "http" => {
            let endpoint = if config.endpoint.contains("/v1/metrics") {
                config.endpoint.clone()
            } else if config.endpoint.ends_with("/") {
                format!("{}v1/metrics", config.endpoint)
            } else {
                format!("{}/v1/metrics", config.endpoint)
            };
            info!(
                "Configuring HTTP exporter for metrics with endpoint: {}",
                endpoint
            );
            if crate::telemetry::is_debug_enabled() {
                info!("[TELEMETRY DEBUG] Using HTTP exporter for metrics");
            }
            opentelemetry_otlp::MetricExporter::builder()
                .with_http()
                .with_endpoint(&endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
        _ => {
            warn!("Unknown protocol '{}', defaulting to gRPC", protocol);
            if crate::telemetry::is_debug_enabled() {
                info!("[TELEMETRY DEBUG] Unknown protocol, defaulting to gRPC for metrics");
            }
            opentelemetry_otlp::MetricExporter::builder()
                .with_tonic()
                .with_endpoint(&config.endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
    }
    .map_err(|e| {
        error!("Failed to build OpenTelemetry metric exporter: {}", e);
        anyhow::anyhow!("OpenTelemetry metric exporter build failed: {}", e)
    })?;

    // Create meter provider with the exporter
    // Wrap exporter in a PeriodicReader for regular export
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(10))
        .build();

    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build();

    // Set as global meter provider
    opentelemetry::global::set_meter_provider(meter_provider);

    info!("OpenTelemetry metrics initialized successfully");

    // Debug logging
    if crate::telemetry::is_debug_enabled() {
        info!("[TELEMETRY DEBUG] Metrics configured with 10-second export interval and explicit histogram buckets");
    }
    Ok(())
}

#[cfg(not(feature = "otel"))]
pub async fn init_metrics(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Metrics are disabled");
        return Ok(());
    }

    info!("Initializing basic metrics (OpenTelemetry feature not enabled)");
    Ok(())
}

#[cfg(feature = "otel")]
pub fn record_request(method: &str, path: &str, status: u16) {
    use opentelemetry::global;

    let meter = global::meter("molock");
    let counter = meter
        .u64_counter("http_server_request_count_total")
        .with_description("Total number of HTTP requests")
        .build();

    let attributes = vec![
        attributes::kv::http_method(method),
        attributes::kv::http_route(path),
        // Use correct semantic convention and type (i64, not String)
        attributes::kv::http_response_status_code(status),
    ];

    // Debug logging for metrics recording
    if crate::telemetry::is_debug_enabled() {
        tracing::debug!(
            method = %method,
            path = %path,
            status = %status,
            ?attributes,
            "[TELEMETRY DEBUG] Recording request counter metric"
        );
    }

    counter.add(1, &attributes);

    // Also log for debugging
    tracing::info!(
        method = %method,
        path = %path,
        status = %status,
        "Request completed"
    );
}

#[cfg(feature = "otel")]
pub fn record_error(method: &str, path: &str, error_type: &str) {
    use opentelemetry::global;

    let meter = global::meter("molock");
    let counter = meter
        .u64_counter("http_server_error_count_total")
        .with_description("Total number of HTTP errors")
        .build();

    let attributes = vec![
        attributes::kv::http_method(method),
        attributes::kv::http_route(path),
        attributes::kv::error_type(error_type),
    ];

    // Debug logging for metrics recording
    if crate::telemetry::is_debug_enabled() {
        tracing::debug!(
            method = %method,
            path = %path,
            error_type = %error_type,
            ?attributes,
            "[TELEMETRY DEBUG] Recording error counter metric"
        );
    }

    counter.add(1, &attributes);

    tracing::error!(
        method = %method,
        path = %path,
        error_type = %error_type,
        "Request error"
    );
}

#[cfg(feature = "otel")]
pub fn record_latency(method: &str, path: &str, latency_ms: f64) {
    use opentelemetry::global;

    let meter = global::meter("molock");

    // Note: OpenTelemetry SDK uses default buckets for histograms
    // Default buckets are appropriate for HTTP request latencies
    let histogram = meter
        .f64_histogram("http_server_request_duration")
        .with_description("HTTP request duration in seconds")
        .with_unit("s")
        .build();

    let attributes = vec![
        attributes::kv::http_method(method),
        attributes::kv::http_route(path),
    ];

    // Convert milliseconds to seconds for Prometheus compatibility
    let latency_seconds = latency_ms / 1000.0;

    // Debug logging for metrics recording
    if crate::telemetry::is_debug_enabled() {
        tracing::debug!(
            method = %method,
            path = %path,
            latency_ms = %latency_ms,
            latency_seconds = %latency_seconds,
            "[TELEMETRY DEBUG] Recording latency metric"
        );
    }

    histogram.record(latency_seconds, &attributes);

    tracing::debug!(
        method = %method,
        path = %path,
        latency_ms = %latency_ms,
        latency_seconds = %latency_seconds,
        "Request latency"
    );
}

#[cfg(not(feature = "otel"))]
pub fn record_request(method: &str, path: &str, status: u16) {
    info!(
        method = %method,
        path = %path,
        status = %status,
        "Request completed"
    );
}

#[cfg(not(feature = "otel"))]
pub fn record_error(method: &str, path: &str, error_type: &str) {
    tracing::error!(
        method = %method,
        path = %path,
        error_type = %error_type,
        "Request error"
    );
}

#[cfg(not(feature = "otel"))]
pub fn record_latency(method: &str, path: &str, latency_ms: f64) {
    tracing::debug!(
        method = %method,
        path = %path,
        latency_ms = %latency_ms,
        "Request latency"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TelemetryConfig;

    #[tokio::test]
    async fn test_init_metrics_disabled() {
        let config = TelemetryConfig {
            enabled: false,
            service_name: "test".to_string(),
            service_version: "0.1.0".to_string(),
            endpoint: "http://localhost:4317".to_string(),
            protocol: "grpc".to_string(),
            sampling_rate: 1.0,
            log_level: "info".to_string(),
            log_format: "json".to_string(),
            timeout_seconds: 30,
            export_batch_size: 512,
            export_timeout_millis: 30000,
        };

        let result = init_metrics(&config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_record_functions() {
        record_request("GET", "/test", 200);
        record_error("GET", "/test", "timeout");
        record_latency("GET", "/test", 100.0);
    }

    #[test]
    fn test_record_request_with_different_status_codes() {
        record_request("GET", "/api/users", 200);
        record_request("POST", "/api/users", 201);
        record_request("PUT", "/api/users/1", 200);
        record_request("DELETE", "/api/users/1", 204);
        record_request("GET", "/api/users", 404);
        record_request("POST", "/api/users", 400);
        record_request("GET", "/api/users", 500);
    }

    #[test]
    fn test_record_error_with_different_error_types() {
        record_error("GET", "/api/users", "timeout");
        record_error("POST", "/api/users", "validation_error");
        record_error("PUT", "/api/users/1", "database_error");
        record_error("DELETE", "/api/users/1", "authorization_error");
        record_error("GET", "/api/users", "network_error");
    }

    #[test]
    fn test_record_latency_with_different_values() {
        record_latency("GET", "/api/users", 10.5);
        record_latency("POST", "/api/users", 150.0);
        record_latency("PUT", "/api/users/1", 75.2);
        record_latency("DELETE", "/api/users/1", 25.0);
        record_latency("GET", "/api/users", 1000.0);
    }

    #[test]
    fn test_record_functions_with_special_characters() {
        record_request("GET", "/api/users?page=1&limit=10", 200);
        record_error("POST", "/api/users/{id}", "not_found");
        record_latency("GET", "/api/users/search?q=test%20query", 45.3);
    }

    #[test]
    fn test_record_functions_with_empty_path() {
        record_request("GET", "", 200);
        record_error("POST", "", "error");
        record_latency("GET", "", 50.0);
    }

    #[test]
    fn test_record_functions_with_long_path() {
        let long_path = "/api/v1/users/12345/orders/67890/items/abcde/fghij/klmno/pqrst/uvwxyz";
        record_request("GET", long_path, 200);
        record_error("POST", long_path, "error");
        record_latency("GET", long_path, 200.0);
    }

    #[test]
    fn test_metrics_function_names_consistency() {
        record_request("GET", "/test", 200);
        record_error("GET", "/test", "error");
        record_latency("GET", "/test", 100.0);
    }

    #[test]
    fn test_edge_case_status_codes() {
        record_request("GET", "/test", 0);
        record_request("GET", "/test", 100);
        record_request("GET", "/test", 599);
        record_request("GET", "/test", 999);
    }

    #[test]
    fn test_edge_case_latencies() {
        record_latency("GET", "/test", 0.0);
        record_latency("GET", "/test", 0.001);
        record_latency("GET", "/test", 999999.9);
        record_latency("GET", "/test", -1.0);
    }
}
