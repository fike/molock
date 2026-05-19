// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

pub mod attributes;
pub mod metrics;
pub mod otel_direct;
pub mod tracer;

use crate::config::TelemetryConfig;
use anyhow::Context;
use std::time::Duration;
use tracing::{error, info, warn};

/// Check if telemetry debug mode is enabled via environment variable
#[must_use]
pub fn is_debug_enabled() -> bool {
    std::env::var("MOLOCK_TELEMETRY_DEBUG").is_ok_and(|v| v.to_lowercase() == "true" || v == "1")
}

/// Debug logging helper for telemetry operations
pub fn debug_log(message: &str, config: &TelemetryConfig) {
    if is_debug_enabled() {
        info!(
            "[TELEMETRY DEBUG] {message} (Service: {}, Endpoint: {}, Protocol: {})",
            config.service_name, config.endpoint, config.protocol
        );
    }
}

/// Test connectivity to the OpenTelemetry collector
async fn test_connectivity(endpoint: &str, protocol: &str) -> anyhow::Result<()> {
    if protocol == "http" {
        let client = reqwest::Client::new();

        let health_url = if endpoint.contains("4318") {
            endpoint.replace("4318", "8889") + "/"
        } else if let Ok(url) = reqwest::Url::parse(endpoint) {
            let mut health_url = url;
            let _ = health_url.set_port(Some(8889));
            health_url.set_path("/");
            health_url.to_string()
        } else {
            "http://otel-collector:8889/".to_string()
        };

        debug_log(
            &format!("Testing HTTP connectivity to {health_url}"),
            &TelemetryConfig::default(),
        );

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
                let error_msg = format!(
                    "Collector returned error status: {status}",
                    status = response.status()
                );
                error!("{error_msg}");
                Err(anyhow::anyhow!(error_msg))
            }
            Err(e) => {
                let error_msg = format!("Failed to connect to OpenTelemetry collector: {e}");
                error!("{error_msg}");
                Err(anyhow::anyhow!(error_msg))
            }
        }
    } else {
        info!("gRPC connectivity test not implemented, assuming reachable");
        Ok(())
    }
}

async fn test_connectivity_with_retry(endpoint: &str, protocol: &str) -> anyhow::Result<()> {
    let max_retries = 3;
    let mut retry_delay = Duration::from_secs(1);

    for attempt in 1..=max_retries {
        info!("Connectivity test attempt {attempt}/{max_retries} to {protocol} endpoint");

        match test_connectivity(endpoint, protocol).await {
            Ok(()) => {
                info!("Connectivity test passed on attempt {attempt}");
                return Ok(());
            }
            Err(e) if attempt == max_retries => {
                error!("All connectivity attempts failed: {e}");
                return Err(e);
            }
            Err(e) => {
                warn!("Connectivity attempt {attempt} failed: {e}");
                warn!("Retrying in {retry_delay:?}...");
                tokio::time::sleep(retry_delay).await;
                retry_delay *= 2;
            }
        }
    }

    unreachable!()
}

/// Initializes all telemetry components (tracing, logging, and metrics).
///
/// # Errors
///
/// Returns an error if tracing or metrics initialization fails.
pub async fn init_telemetry(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Telemetry is disabled");
        return Ok(());
    }

    info!(
        "Initializing telemetry with service name: {}",
        config.service_name
    );

    debug_log("Starting telemetry initialization", config);

    info!("Testing connectivity to OpenTelemetry collector...");
    match test_connectivity_with_retry(&config.endpoint, &config.protocol).await {
        Ok(()) => info!("Connectivity test passed"),
        Err(e) => {
            error!("Connectivity test failed: {e}");
            error!("OpenTelemetry collector is unreachable. Telemetry data will not be exported.");
            error!(
                "Check if OpenTelemetry collector is running at: {endpoint}",
                endpoint = config.endpoint
            );
        }
    }

    info!("Starting tracing initialization...");
    #[cfg(feature = "otel")]
    tracer::init_tracing(config).context("Failed to initialize tracing")?;
    #[cfg(not(feature = "otel"))]
    tracer::init_tracing(config).context("Failed to initialize tracing")?;

    info!("Tracing initialized, starting metrics...");

    #[cfg(feature = "otel")]
    metrics::init_metrics(config).context("Failed to initialize metrics")?;
    #[cfg(not(feature = "otel"))]
    metrics::init_metrics(config).context("Failed to initialize metrics")?;

    info!("Telemetry initialized successfully");
    debug_log("Telemetry initialization completed successfully", config);
    Ok(())
}

pub fn shutdown_telemetry() {
    info!("Shutting down telemetry");

    #[cfg(feature = "otel")]
    {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TelemetryConfig;

    #[test]
    fn test_is_debug_enabled() {
        std::env::set_var("MOLOCK_TELEMETRY_DEBUG", "true");
        assert!(is_debug_enabled());
        std::env::set_var("MOLOCK_TELEMETRY_DEBUG", "1");
        assert!(is_debug_enabled());
        std::env::set_var("MOLOCK_TELEMETRY_DEBUG", "false");
        assert!(!is_debug_enabled());
        std::env::remove_var("MOLOCK_TELEMETRY_DEBUG");
        assert!(!is_debug_enabled());
    }

    #[test]
    fn test_debug_log() {
        let config = TelemetryConfig::default();
        std::env::set_var("MOLOCK_TELEMETRY_DEBUG", "true");
        debug_log("Test message", &config);
        std::env::remove_var("MOLOCK_TELEMETRY_DEBUG");
    }

    #[tokio::test]
    async fn test_init_telemetry_connectivity_failure() {
        let config = TelemetryConfig {
            enabled: true,
            service_name: "test".to_string(),
            service_version: "0.1.0".to_string(),
            endpoint: "http://invalid-host:4318".to_string(),
            protocol: "http".to_string(),
            sampling_rate: 1.0,
            log_level: "info".to_string(),
            log_format: "json".to_string(),
            timeout_seconds: 1,
            export_batch_size: 512,
            export_timeout_millis: 1000,
        };

        let result = init_telemetry(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_telemetry_grpc() {
        let config = TelemetryConfig {
            enabled: true,
            service_name: "test".to_string(),
            service_version: "0.1.0".to_string(),
            endpoint: "http://localhost:4317".to_string(),
            protocol: "grpc".to_string(),
            sampling_rate: 1.0,
            log_level: "info".to_string(),
            log_format: "json".to_string(),
            timeout_seconds: 1,
            export_batch_size: 512,
            export_timeout_millis: 1000,
        };

        let result = init_telemetry(&config).await;
        assert!(result.is_ok());
    }
}
