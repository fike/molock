// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

pub mod executor;
pub mod matcher;
pub mod state;

use crate::config::Endpoint;
use crate::rules::executor::ResponseExecutor;
use crate::rules::matcher::RuleMatcher;
use crate::rules::state::StateManager;
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResponse {
    pub status: u16,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub client_ip: String,
    pub path_params: HashMap<String, String>,
}

#[derive(Clone)]
pub struct RuleEngine {
    matcher: RuleMatcher,
    executor: ResponseExecutor,
    pub compiled_schemas: HashMap<String, Arc<Validator>>,
}

impl RuleEngine {
    /// Creates a new `RuleEngine`.
    ///
    /// # Panics
    ///
    /// Panics if a JSON schema cannot be compiled.
    #[must_use]
    pub fn new(endpoints: &[Endpoint]) -> Self {
        let state_manager = Arc::new(StateManager::new());
        let matcher = RuleMatcher::new(endpoints.to_owned());
        let executor_engine = ResponseExecutor::new(state_manager);
        let mut compiled_schemas = HashMap::new();

        for endpoint in endpoints {
            if let Some(schema_val) = &endpoint.schema {
                let validator = jsonschema::validator_for(schema_val).unwrap_or_else(|e| {
                    panic!("Failed to compile schema for {}: {}", endpoint.name, e);
                });
                compiled_schemas.insert(endpoint.name.clone(), Arc::new(validator));
            }
        }

        Self {
            matcher,
            executor: executor_engine,
            compiled_schemas,
        }
    }

    /// Executes the rule engine for an incoming request.
    ///
    /// # Errors
    ///
    /// Returns an error if no matching rule is found or if execution fails.
    pub async fn execute(
        &self,
        method: &str,
        path: &str,
        query: &str,
        headers: &HashMap<String, String>,
        body: Option<&str>,
        client_ip: &str,
    ) -> anyhow::Result<RuleResponse> {
        let endpoint = self
            .matcher
            .find_match_with_context(method, path, headers, query)?;

        // Validate body schema if configured
        if let Some(validator) = self.compiled_schemas.get(&endpoint.name) {
            if let Some(body_content) = body {
                let json_body: serde_json::Value = serde_json::from_str(body_content)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON body: {e}"))?;

                if let Err(e) = validator.validate(&json_body) {
                    anyhow::bail!("Body validation failed: {e}");
                }
            } else if endpoint.schema.is_some() {
                anyhow::bail!("Body is required for validation but missing");
            }
        }

        let path_params = self.matcher.extract_path_params(&endpoint.path, path);

        let context = ExecutionContext {
            method: method.to_string(),
            path: path.to_string(),
            query: query.to_string(),
            headers: headers.clone(),
            body: body.map(std::string::ToString::to_string),
            client_ip: client_ip.to_string(),
            path_params,
        };

        self.executor.execute(endpoint, &context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Response;

    #[test]
    fn test_rule_engine_creation() {
        let endpoints = vec![Endpoint {
            name: "Test".to_string(),
            method: "GET".to_string(),
            path: "/test".to_string(),
            stateful: false,
            state_key: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("OK".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
            schema: None,
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
        }];

        let _engine = RuleEngine::new(&endpoints);
    }

    #[tokio::test]
    async fn test_execute_no_endpoints() {
        let engine = RuleEngine::new(&[]);
        let result = engine
            .execute("GET", "/test", "", &HashMap::new(), None, "127.0.0.1")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_schema_validation_success() {
        let endpoints = vec![Endpoint {
            name: "SchemaTest".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            stateful: false,
            state_key: None,
            schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "id": { "type": "number" }
                },
                "required": ["id"]
            })),
            schema_file: None,
            responses: vec![Response {
                status: 201,
                delay: None,
                body: Some("Created".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
            path_regex: None,
            headers_regex: None,
            query_regex: None,
        }];

        let engine = RuleEngine::new(&endpoints);
        let result = engine
            .execute(
                "POST",
                "/api",
                "",
                &HashMap::new(),
                Some(r#"{"id": 123}"#),
                "127.0.0.1",
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, 201);
    }

    #[tokio::test]
    async fn test_execute_schema_validation_failure() {
        let endpoints = vec![Endpoint {
            name: "SchemaTest".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            stateful: false,
            state_key: None,
            schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "id": { "type": "number" }
                },
                "required": ["id"]
            })),
            schema_file: None,
            responses: vec![Response {
                status: 201,
                delay: None,
                body: Some("Created".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
            path_regex: None,
            headers_regex: None,
            query_regex: None,
        }];

        let engine = RuleEngine::new(&endpoints);
        let result = engine
            .execute(
                "POST",
                "/api",
                "",
                &HashMap::new(),
                Some(r#"{"id": "not-a-number"}"#),
                "127.0.0.1",
            )
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("validation failed"));
    }

    #[tokio::test]
    async fn test_execute_with_regex_matching() {
        let endpoint = Endpoint {
            name: "RegexTest".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            stateful: false,
            state_key: None,
            schema: None,
            schema_file: None,
            path_regex: None,
            headers_regex: Some({
                let mut h = HashMap::new();
                h.insert("X-Required".to_string(), "^secret$".to_string());
                h
            }),
            query_regex: Some({
                let mut q = HashMap::new();
                q.insert("v".to_string(), "^1$".to_string());
                q
            }),
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("Regex Match".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
        };

        let engine = RuleEngine::new(&[endpoint]);

        // Should match with correct headers and query
        let mut headers = HashMap::new();
        headers.insert("X-Required".to_string(), "secret".to_string());
        let result = engine
            .execute("POST", "/api", "v=1", &headers, None, "127.0.0.1")
            .await;
        assert!(result.is_ok());

        // Should fail with wrong headers
        let mut bad_headers = HashMap::new();
        bad_headers.insert("X-Required".to_string(), "wrong".to_string());
        let result = engine
            .execute("POST", "/api", "v=1", &bad_headers, None, "127.0.0.1")
            .await;
        assert!(result.is_err());

        // Should fail with wrong query
        let result = engine
            .execute("POST", "/api", "v=2", &headers, None, "127.0.0.1")
            .await;
        assert!(result.is_err());
    }
}
