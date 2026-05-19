// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub server: ServerConfig,
    pub telemetry: TelemetryConfig,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_workers")]
    pub workers: usize,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_max_request_size")]
    pub max_request_size: usize,
}

const fn default_port() -> u16 {
    8080
}

const fn default_workers() -> usize {
    4
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

const fn default_max_request_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_service_name")]
    pub service_name: String,
    #[serde(default = "default_service_version")]
    pub service_version: String,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f64,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default = "default_export_batch_size")]
    pub export_batch_size: usize,
    #[serde(default = "default_export_timeout_millis")]
    pub export_timeout_millis: u64,
}

const fn default_enabled() -> bool {
    true
}

fn default_service_name() -> String {
    "molock".to_string()
}

fn default_endpoint() -> String {
    "http://localhost:4317".to_string()
}

fn default_protocol() -> String {
    "grpc".to_string()
}

const fn default_sampling_rate() -> f64 {
    1.0
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_service_version() -> String {
    "0.1.0".to_string()
}

const fn default_timeout_seconds() -> u64 {
    30
}

const fn default_export_batch_size() -> usize {
    512
}

const fn default_export_timeout_millis() -> u64 {
    30000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub name: String,
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub stateful: bool,
    #[serde(default)]
    pub state_key: Option<String>,
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
    #[serde(default)]
    pub schema_file: Option<String>,
    #[serde(default)]
    pub path_regex: Option<String>,
    #[serde(default)]
    pub headers_regex: Option<HashMap<String, String>>,
    #[serde(default)]
    pub query_regex: Option<HashMap<String, String>>,
    pub responses: Vec<Response>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    #[serde(default)]
    pub delay: Option<Delay>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub probability: Option<f64>,
    #[serde(default)]
    pub default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Delay {
    Fixed(String),
    Range(String),
}

impl Delay {
    /// Parses the delay into a `Duration`.
    ///
    /// # Errors
    ///
    /// Returns an error if the delay format is invalid or if the range is logically incorrect.
    pub fn parse_duration(&self) -> anyhow::Result<Duration> {
        match self {
            Self::Fixed(delay_str) => parse_duration_str(delay_str),
            Self::Range(range_str) => {
                let parts: Vec<&str> = range_str.split('-').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Invalid delay range format: {range_str}");
                }
                let min = parse_duration_str(parts[0])?;
                let max = parse_duration_str(parts[1])?;
                if min > max {
                    anyhow::bail!("Min delay cannot be greater than max delay");
                }
                Ok(min)
            }
        }
    }

    /// Parses the delay into a range of `Duration`s.
    ///
    /// # Errors
    ///
    /// Returns an error if the delay format is invalid or if the range is logically incorrect.
    pub fn parse_range(&self) -> anyhow::Result<(Duration, Duration)> {
        match self {
            Self::Fixed(delay_str) => {
                let duration = parse_duration_str(delay_str)?;
                Ok((duration, duration))
            }
            Self::Range(range_str) => {
                let parts: Vec<&str> = range_str.split('-').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Invalid delay range format: {range_str}");
                }
                let min = parse_duration_str(parts[0])?;
                let max = parse_duration_str(parts[1])?;
                if min > max {
                    anyhow::bail!("Min delay cannot be greater than max delay");
                }
                Ok((min, max))
            }
        }
    }
}

fn parse_duration_str(duration_str: &str) -> anyhow::Result<Duration> {
    let duration_str = duration_str.trim();
    if let Some(stripped) = duration_str.strip_suffix("ms") {
        let ms = stripped
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid milliseconds: {e}"))?;
        Ok(Duration::from_millis(ms))
    } else if let Some(stripped) = duration_str.strip_suffix('s') {
        let secs = stripped
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid seconds: {e}"))?;
        Ok(Duration::from_secs(secs))
    } else {
        anyhow::bail!("Invalid duration format: {duration_str}");
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            workers: default_workers(),
            host: default_host(),
            max_request_size: default_max_request_size(),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            service_name: default_service_name(),
            service_version: default_service_version(),
            endpoint: default_endpoint(),
            protocol: default_protocol(),
            sampling_rate: default_sampling_rate(),
            log_level: default_log_level(),
            log_format: default_log_format(),
            timeout_seconds: default_timeout_seconds(),
            export_batch_size: default_export_batch_size(),
            export_timeout_millis: default_export_timeout_millis(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixed_delay() {
        let delay = Delay::Fixed("100ms".to_string());
        assert_eq!(delay.parse_duration().unwrap(), Duration::from_millis(100));

        let delay = Delay::Fixed("2s".to_string());
        assert_eq!(delay.parse_duration().unwrap(), Duration::from_secs(2));
    }

    #[test]
    fn test_parse_range_delay() {
        let delay = Delay::Range("100ms-500ms".to_string());
        let (min, max) = delay.parse_range().unwrap();
        assert_eq!(min, Duration::from_millis(100));
        assert_eq!(max, Duration::from_millis(500));
    }

    #[test]
    fn test_invalid_delay_format() {
        let delay = Delay::Fixed("100".to_string());
        assert!(delay.parse_duration().is_err());

        let delay = Delay::Range("100ms-".to_string());
        assert!(delay.parse_range().is_err());
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.workers, 4);
        assert!(config.telemetry.enabled);
        assert_eq!(config.telemetry.log_level, "info");
    }

    #[test]
    fn test_endpoint_deserialization_with_schema() {
        let json = r#"{
            "name": "Test Schema",
            "method": "POST",
            "path": "/api/data",
            "schema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" }
                }
            },
            "responses": []
        }"#;
        let endpoint: Endpoint = serde_json::from_str(json).unwrap();
        assert!(endpoint.schema.is_some());
        let schema = endpoint.schema.unwrap();
        assert_eq!(schema["properties"]["id"]["type"], "integer");
    }

    #[test]
    fn test_endpoint_deserialization_with_schema_file() {
        let json = r#"{
            "name": "Test Schema File",
            "method": "POST",
            "path": "/api/data",
            "schema_file": "schemas/user.json",
            "responses": []
        }"#;
        let endpoint: Endpoint = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.schema_file, Some("schemas/user.json".to_string()));
    }

    #[test]
    fn test_endpoint_deserialization_with_regex() {
        let json = r#"{
            "name": "Test Regex",
            "method": "GET",
            "path": "/users/:id",
            "path_regex": "^/users/[0-9]+$",
            "headers_regex": {
                "X-Auth-Token": "^[a-zA-Z0-9]+$"
            },
            "query_regex": {
                "page": "^[0-9]+$"
            },
            "responses": []
        }"#;
        let endpoint: Endpoint = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.path_regex, Some("^/users/[0-9]+$".to_string()));
        assert_eq!(
            endpoint.headers_regex.as_ref().unwrap().get("X-Auth-Token"),
            Some(&"^[a-zA-Z0-9]+$".to_string())
        );
        assert_eq!(
            endpoint.query_regex.as_ref().unwrap().get("page"),
            Some(&"^[0-9]+$".to_string())
        );
    }
}
