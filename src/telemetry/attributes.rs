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

//! OpenTelemetry semantic convention constants for the Molock project
//!
//! This module provides type-safe constants for OpenTelemetry attribute names
//! following the OpenTelemetry semantic conventions v1.26.0.
//!
//! References:
//! - https://opentelemetry.io/docs/specs/semconv/http/http-spans/
//! - https://opentelemetry.io/docs/specs/semconv/attributes-registry/

/// HTTP semantic conventions
pub mod http {
    /// HTTP request method
    pub const METHOD: &str = "http.method";

    /// HTTP route (matched route)
    pub const ROUTE: &str = "http.route";

    /// Full HTTP request target in the form "scheme://host[:port]/path?query[#fragment]"
    pub const TARGET: &str = "http.target";

    /// HTTP response status code
    pub const RESPONSE_STATUS_CODE: &str = "http.response.status_code";
}

/// Span semantic conventions
pub mod span {
    /// Span kind
    pub const KIND: &str = "span.kind";

    /// Server span kind value
    pub const KIND_SERVER: &str = "server";
}

/// Service semantic conventions
pub mod service {
    #[allow(dead_code)]
    pub const NAME: &str = "service.name";

    #[allow(dead_code)]
    pub const VERSION: &str = "service.version";

    #[allow(dead_code)]
    pub const INSTANCE_ID: &str = "service.instance.id";
}

/// Deployment semantic conventions  
pub mod deployment {
    #[allow(dead_code)]
    pub const ENVIRONMENT: &str = "deployment.environment";
}

/// Error semantic conventions
pub mod error {
    /// Error type
    pub const TYPE: &str = "error.type";
}

/// Network semantic conventions
pub mod network {
    #[allow(dead_code)]
    pub const TRANSPORT: &str = "network.transport";

    #[allow(dead_code)]
    pub const TYPE: &str = "network.type";

    #[allow(dead_code)]
    pub const LOCAL_ADDRESS: &str = "network.local.address";

    #[allow(dead_code)]
    pub const LOCAL_PORT: &str = "network.local.port";

    #[allow(dead_code)]
    pub const PEER_ADDRESS: &str = "network.peer.address";

    #[allow(dead_code)]
    pub const PEER_PORT: &str = "network.peer.port";
}

/// Helper functions for creating OpenTelemetry KeyValue pairs with semantic conventions
pub mod kv {
    use opentelemetry::KeyValue;

    use super::http;

    /// Create a KeyValue for HTTP method
    pub fn http_method(method: impl Into<String>) -> KeyValue {
        KeyValue::new(http::METHOD, method.into())
    }

    /// Create a KeyValue for HTTP route
    pub fn http_route(route: impl Into<String>) -> KeyValue {
        KeyValue::new(http::ROUTE, route.into())
    }

    /// Create a KeyValue for HTTP target
    pub fn http_target(target: impl Into<String>) -> KeyValue {
        KeyValue::new(http::TARGET, target.into())
    }

    /// Create a KeyValue for HTTP response status code
    pub fn http_response_status_code(status: u16) -> KeyValue {
        KeyValue::new(http::RESPONSE_STATUS_CODE, status as i64)
    }

    /// Create a KeyValue for span kind
    pub fn span_kind(kind: impl Into<String>) -> KeyValue {
        KeyValue::new(super::span::KIND, kind.into())
    }

    /// Create a KeyValue for error type
    pub fn error_type(error_type: impl Into<String>) -> KeyValue {
        KeyValue::new(super::error::TYPE, error_type.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_constants() {
        assert_eq!(http::METHOD, "http.method");
        assert_eq!(http::ROUTE, "http.route");
        assert_eq!(http::TARGET, "http.target");
        assert_eq!(http::RESPONSE_STATUS_CODE, "http.response.status_code");
    }

    #[test]
    fn test_span_constants() {
        assert_eq!(span::KIND, "span.kind");
        assert_eq!(span::KIND_SERVER, "server");
    }

    #[test]
    fn test_service_constants() {
        assert_eq!(service::NAME, "service.name");
        assert_eq!(service::VERSION, "service.version");
        assert_eq!(service::INSTANCE_ID, "service.instance.id");
    }

    #[test]
    fn test_deployment_constants() {
        assert_eq!(deployment::ENVIRONMENT, "deployment.environment");
    }

    #[test]
    fn test_error_constants() {
        assert_eq!(error::TYPE, "error.type");
    }

    #[test]
    fn test_network_constants() {
        assert_eq!(network::TRANSPORT, "network.transport");
        assert_eq!(network::TYPE, "network.type");
        assert_eq!(network::LOCAL_ADDRESS, "network.local.address");
        assert_eq!(network::LOCAL_PORT, "network.local.port");
        assert_eq!(network::PEER_ADDRESS, "network.peer.address");
        assert_eq!(network::PEER_PORT, "network.peer.port");
    }

    #[test]
    fn test_kv_http_method() {
        let kv = kv::http_method("GET");
        assert_eq!(kv.key.as_str(), "http.method");
        assert_eq!(kv.value.to_string(), "GET");
    }

    #[test]
    fn test_kv_http_route() {
        let kv = kv::http_route("/api/users");
        assert_eq!(kv.key.as_str(), "http.route");
        assert_eq!(kv.value.to_string(), "/api/users");
    }

    #[test]
    fn test_kv_http_target() {
        let kv = kv::http_target("/api/users?page=1");
        assert_eq!(kv.key.as_str(), "http.target");
        assert_eq!(kv.value.to_string(), "/api/users?page=1");
    }

    #[test]
    fn test_kv_http_response_status_code() {
        let kv = kv::http_response_status_code(200);
        assert_eq!(kv.key.as_str(), "http.response.status_code");
        assert_eq!(kv.value.to_string(), "200");

        let kv = kv::http_response_status_code(404);
        assert_eq!(kv.key.as_str(), "http.response.status_code");
        assert_eq!(kv.value.to_string(), "404");

        let kv = kv::http_response_status_code(500);
        assert_eq!(kv.key.as_str(), "http.response.status_code");
        assert_eq!(kv.value.to_string(), "500");
    }

    #[test]
    fn test_kv_span_kind() {
        let kv = kv::span_kind("server");
        assert_eq!(kv.key.as_str(), "span.kind");
        assert_eq!(kv.value.to_string(), "server");
    }

    #[test]
    fn test_kv_error_type() {
        let kv = kv::error_type("timeout");
        assert_eq!(kv.key.as_str(), "error.type");
        assert_eq!(kv.value.to_string(), "timeout");
    }

    #[test]
    fn test_kv_with_different_input_types() {
        let kv1 = kv::http_method(String::from("POST"));
        let kv2 = kv::http_method("POST");
        assert_eq!(kv1.key.as_str(), kv2.key.as_str());
        assert_eq!(kv1.value.to_string(), kv2.value.to_string());
    }

    #[test]
    fn test_semantic_convention_consistency() {
        assert_eq!(http::RESPONSE_STATUS_CODE, "http.response.status_code");
        assert_ne!(http::RESPONSE_STATUS_CODE, "http.status_code");
    }
}
