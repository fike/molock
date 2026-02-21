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

pub mod attributes;
pub mod metrics;
pub mod otel_direct;
pub mod tracer;

pub use metrics::init_metrics;
pub use tracer::init_tracing;

use crate::config::TelemetryConfig;
use anyhow::Context;
use std::time::Duration;
use tracing::{error, info, warn};

/// Check if telemetry debug mode is enabled via environment variable
pub fn is_debug_enabled() -> bool {
    std::env::var("MOLOCK_TELEMETRY_DEBUG")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}

/// Debug logging helper for telemetry operations
pub fn debug_log(message: &str, config: &TelemetryConfig) {
    if is_debug_enabled() {
        info!("[TELEMETRY DEBUG] {}", message);
        info!(
            "[TELEMETRY DEBUG] Config: enabled={}, endpoint={}, protocol={}, timeout={}s",
            config.enabled, config.endpoint, config.protocol, config.timeout_seconds
        );
    }
}

/// Test connectivity to OpenTelemetry collector
async fn test_connectivity(endpoint: &str, protocol: &str) -> anyhow::Result<()> {
    info!(
        "Testing connectivity to {} endpoint: {}",
        protocol, endpoint
    );

    let client = reqwest::Client::new();

    // For HTTP protocol, test the health endpoint
    if protocol == "http" {
        // Try to extract host and port from endpoint
        let health_url = if endpoint.contains("4318") {
            // Replace metrics port with health check port
            endpoint.replace("4318", "8889") + "/"
        } else if let Ok(url) = reqwest::Url::parse(endpoint) {
            // Construct health URL from parsed URL
            let mut health_url = url.clone();
            health_url
                .set_port(Some(8889))
                .map_err(|_| anyhow::anyhow!("Failed to construct health URL from endpoint"))?;
            health_url.set_path("/");
            health_url.to_string()
        } else {
            // Fallback: try common health endpoint
            "http://otel-collector:8889/".to_string()
        };

        if is_debug_enabled() {
            info!(
                "[TELEMETRY DEBUG] Testing connectivity to health endpoint: {}",
                health_url
            );
        }

        match client
            .get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                info!("Successfully connected to OpenTelemetry collector");
                Ok(())
            }
            Ok(response) => {
                let error_msg = format!("Collector returned error status: {}", response.status());
                error!("{}", error_msg);
                Err(anyhow::anyhow!(error_msg))
            }
            Err(e) => {
                let error_msg = format!("Failed to connect to OpenTelemetry collector: {}", e);
                error!("{}", error_msg);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    } else {
        // For gRPC protocol, we can't easily test without gRPC client
        // Just log and return success for now
        info!("gRPC connectivity test not implemented, assuming reachable");
        Ok(())
    }
}

/// Test connectivity with retry logic
async fn test_connectivity_with_retry(endpoint: &str, protocol: &str) -> anyhow::Result<()> {
    let max_retries = 3;
    let mut retry_delay = Duration::from_secs(1);

    for attempt in 1..=max_retries {
        info!(
            "Connectivity test attempt {}/{} to {} endpoint",
            attempt, max_retries, protocol
        );

        match test_connectivity(endpoint, protocol).await {
            Ok(_) => {
                info!("Connectivity test passed on attempt {}", attempt);
                return Ok(());
            }
            Err(e) if attempt == max_retries => {
                error!("All connectivity attempts failed: {}", e);
                return Err(e);
            }
            Err(e) => {
                warn!("Connectivity attempt {} failed: {}", attempt, e);
                warn!("Retrying in {:?}...", retry_delay);
                tokio::time::sleep(retry_delay).await;
                retry_delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}

pub async fn init_telemetry(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Telemetry is disabled");
        return Ok(());
    }

    info!(
        "Initializing telemetry with service name: {}",
        config.service_name
    );

    // Debug logging
    debug_log("Starting telemetry initialization", config);

    // Test connectivity before initialization
    info!("Testing connectivity to OpenTelemetry collector...");
    match test_connectivity_with_retry(&config.endpoint, &config.protocol).await {
        Ok(_) => info!("Connectivity test passed"),
        Err(e) => {
            error!("Connectivity test failed: {}", e);
            error!("OpenTelemetry collector is unreachable. Telemetry data will not be exported.");
            error!(
                "Check if OpenTelemetry collector is running at: {}",
                config.endpoint
            );
            // Log error but continue - telemetry will be initialized but may not work
            // This allows the application to start even if observability is unavailable
        }
    }

    // Add a small delay to avoid race conditions
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Initialize tracing first (which includes logging)
    info!("Starting tracing initialization...");
    init_tracing(config)
        .await
        .context("Failed to initialize tracing")?;
    info!("Tracing initialized, starting metrics...");

    // Another small delay between tracing and metrics
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    init_metrics(config)
        .await
        .context("Failed to initialize metrics")?;

    info!("Telemetry initialized successfully");
    debug_log("Telemetry initialization completed successfully", config);
    Ok(())
}

pub async fn shutdown_telemetry() {
    info!("Shutting down telemetry");

    #[cfg(feature = "otel")]
    {
        // Actual shutdown logic would go here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TelemetryConfig;

    #[tokio::test]
    async fn test_init_disabled_telemetry() {
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

        let result = init_telemetry(&config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_telemetry_config_defaults() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.service_name, "molock");
        assert_eq!(config.endpoint, "http://localhost:4317");
        assert_eq!(config.protocol, "grpc");
        assert_eq!(config.sampling_rate, 1.0);
    }
}
