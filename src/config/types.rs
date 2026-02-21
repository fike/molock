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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn default_port() -> u16 {
    8080
}

fn default_workers() -> usize {
    4
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_max_request_size() -> usize {
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

fn default_enabled() -> bool {
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

fn default_sampling_rate() -> f64 {
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

fn default_timeout_seconds() -> u64 {
    30
}

fn default_export_batch_size() -> usize {
    512
}

fn default_export_timeout_millis() -> u64 {
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
    pub fn parse_duration(&self) -> anyhow::Result<Duration> {
        match self {
            Delay::Fixed(delay_str) => parse_duration_str(delay_str),
            Delay::Range(range_str) => {
                let parts: Vec<&str> = range_str.split('-').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Invalid delay range format: {}", range_str);
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

    pub fn parse_range(&self) -> anyhow::Result<(Duration, Duration)> {
        match self {
            Delay::Fixed(delay_str) => {
                let duration = parse_duration_str(delay_str)?;
                Ok((duration, duration))
            }
            Delay::Range(range_str) => {
                let parts: Vec<&str> = range_str.split('-').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Invalid delay range format: {}", range_str);
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
    if duration_str.ends_with("ms") {
        let ms = duration_str[..duration_str.len() - 2]
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid milliseconds: {}", e))?;
        Ok(Duration::from_millis(ms))
    } else if duration_str.ends_with('s') {
        let secs = duration_str[..duration_str.len() - 1]
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid seconds: {}", e))?;
        Ok(Duration::from_secs(secs))
    } else {
        anyhow::bail!("Invalid duration format: {}", duration_str);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            telemetry: TelemetryConfig::default(),
            endpoints: Vec::new(),
        }
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
        assert_eq!(config.telemetry.enabled, true);
        assert_eq!(config.telemetry.log_level, "info");
    }
}
