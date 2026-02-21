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

use crate::config::types::Config;
use anyhow::Context;
use serde_yaml;
use std::fs;
use std::path::Path;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> anyhow::Result<Config> {
        let config: Config =
            serde_yaml::from_str(content).with_context(|| "Failed to parse YAML configuration")?;

        Self::validate(&config)?;

        Ok(config)
    }

    fn validate(config: &Config) -> anyhow::Result<()> {
        if config.server.port == 0 {
            anyhow::bail!("Server port cannot be 0");
        }

        if config.server.workers == 0 {
            anyhow::bail!("Number of workers cannot be 0");
        }

        if config.telemetry.sampling_rate < 0.0 || config.telemetry.sampling_rate > 1.0 {
            anyhow::bail!("Sampling rate must be between 0.0 and 1.0");
        }

        // Validate telemetry endpoint URL
        if config.telemetry.enabled {
            Self::validate_telemetry_config(&config.telemetry)?;
        }

        for endpoint in &config.endpoints {
            Self::validate_endpoint(endpoint)?;
        }

        Ok(())
    }

    fn validate_telemetry_config(
        config: &crate::config::types::TelemetryConfig,
    ) -> anyhow::Result<()> {
        // Validate endpoint URL
        if config.endpoint.is_empty() {
            anyhow::bail!("Telemetry endpoint cannot be empty");
        }

        // Try to parse the URL to validate format
        if let Ok(url) = reqwest::Url::parse(&config.endpoint) {
            // Check if URL has a scheme
            if url.scheme().is_empty() {
                anyhow::bail!("Telemetry endpoint must have a scheme (http:// or https://)");
            }

            // Check for valid schemes
            let scheme = url.scheme();
            if scheme != "http" && scheme != "https" {
                anyhow::bail!("Telemetry endpoint must use http:// or https:// scheme");
            }

            // Check if URL has a host
            if url.host().is_none() {
                anyhow::bail!("Telemetry endpoint must have a host");
            }
        } else {
            anyhow::bail!("Invalid telemetry endpoint URL format: {}", config.endpoint);
        }

        // Validate protocol
        let protocol = config.protocol.to_lowercase();
        if protocol != "http" && protocol != "grpc" {
            anyhow::bail!(
                "Telemetry protocol must be 'http' or 'grpc', got '{}'",
                config.protocol
            );
        }

        // Validate timeout
        if config.timeout_seconds == 0 {
            anyhow::bail!("Telemetry timeout must be greater than 0");
        }

        // Validate export batch size
        if config.export_batch_size == 0 {
            anyhow::bail!("Telemetry export batch size must be greater than 0");
        }

        // Validate export timeout
        if config.export_timeout_millis == 0 {
            anyhow::bail!("Telemetry export timeout must be greater than 0");
        }

        Ok(())
    }

    fn validate_endpoint(endpoint: &crate::config::types::Endpoint) -> anyhow::Result<()> {
        if endpoint.name.is_empty() {
            anyhow::bail!("Endpoint name cannot be empty");
        }

        if endpoint.method.is_empty() {
            anyhow::bail!("Endpoint method cannot be empty");
        }

        if endpoint.path.is_empty() {
            anyhow::bail!("Endpoint path cannot be empty");
        }

        if endpoint.responses.is_empty() {
            anyhow::bail!("Endpoint must have at least one response");
        }

        let default_responses: Vec<_> = endpoint.responses.iter().filter(|r| r.default).collect();

        if default_responses.len() > 1 {
            anyhow::bail!("Endpoint can have at most one default response");
        }

        for response in &endpoint.responses {
            Self::validate_response(response)?;
        }

        Ok(())
    }

    fn validate_response(response: &crate::config::types::Response) -> anyhow::Result<()> {
        if response.status < 100 || response.status >= 600 {
            anyhow::bail!("Invalid HTTP status code: {}", response.status);
        }

        if let Some(probability) = response.probability {
            if probability < 0.0 || probability > 1.0 {
                anyhow::bail!("Probability must be between 0.0 and 1.0");
            }
        }

        if let Some(delay) = &response.delay {
            if let Err(e) = delay.parse_duration() {
                anyhow::bail!("Invalid delay format: {}", e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_config() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true
  service_name: "test"

logging:
  level: "info"

endpoints:
  - name: "Test"
    method: GET
    path: "/test"
    responses:
      - status: 200
        body: "OK"
        "#;

        let config = ConfigLoader::from_str(config_str).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.endpoints.len(), 1);
        assert_eq!(config.endpoints[0].name, "Test");
    }

    #[test]
    fn test_invalid_port() {
        let config_str = r#"
server:
  port: 0
  workers: 4

telemetry:
  enabled: true

logging:
  level: "info"

endpoints: []
        "#;

        let result = ConfigLoader::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("port cannot be 0"));
    }

    #[test]
    fn test_invalid_sampling_rate() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true
  sampling_rate: 1.5

logging:
  level: "info"

endpoints: []
        "#;

        let result = ConfigLoader::from_str(config_str);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Sampling rate must be between"));
    }

    #[test]
    fn test_empty_endpoint_name() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true

logging:
  level: "info"

endpoints:
  - name: ""
    method: GET
    path: "/test"
    responses:
      - status: 200
        "#;

        let result = ConfigLoader::from_str(config_str);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Endpoint name cannot be empty"));
    }

    #[test]
    fn test_multiple_default_responses() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true

logging:
  level: "info"

endpoints:
  - name: "Test"
    method: GET
    path: "/test"
    responses:
      - status: 200
        default: true
      - status: 404
        default: true
        "#;

        let result = ConfigLoader::from_str(config_str);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at most one default response"));
    }

    #[test]
    fn test_invalid_delay_format() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true

logging:
  level: "info"

endpoints:
  - name: "Test"
    method: GET
    path: "/test"
    responses:
      - status: 200
        delay: "invalid"
        "#;

        let result = ConfigLoader::from_str(config_str);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid delay format"));
    }

    #[test]
    fn test_invalid_telemetry_endpoint() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true
  endpoint: "not-a-valid-url"
  protocol: "http"

endpoints: []
        "#;

        let result = ConfigLoader::from_str(config_str);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid telemetry endpoint URL format"));
    }

    #[test]
    fn test_invalid_telemetry_protocol() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true
  endpoint: "http://localhost:4318"
  protocol: "invalid-protocol"

endpoints: []
        "#;

        let result = ConfigLoader::from_str(config_str);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Telemetry protocol must be 'http' or 'grpc'"));
    }

    #[test]
    fn test_valid_telemetry_config() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true
  endpoint: "http://localhost:4318"
  protocol: "http"
  sampling_rate: 0.5
  timeout_seconds: 30
  export_batch_size: 512
  export_timeout_millis: 30000

endpoints: []
        "#;

        let config = ConfigLoader::from_str(config_str).unwrap();
        assert!(config.telemetry.enabled);
        assert_eq!(config.telemetry.endpoint, "http://localhost:4318");
        assert_eq!(config.telemetry.protocol, "http");
        assert_eq!(config.telemetry.sampling_rate, 0.5);
    }

    #[test]
    fn test_valid_grpc_telemetry_config() {
        let config_str = r#"
server:
  port: 8080
  workers: 4

telemetry:
  enabled: true
  endpoint: "http://localhost:4317"
  protocol: "grpc"

endpoints: []
        "#;

        let config = ConfigLoader::from_str(config_str).unwrap();
        assert!(config.telemetry.enabled);
        assert_eq!(config.telemetry.endpoint, "http://localhost:4317");
        assert_eq!(config.telemetry.protocol, "grpc");
    }
}
