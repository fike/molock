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

pub mod executor;
pub mod matcher;
pub mod state;

use crate::config::Endpoint;
use executor::ResponseExecutor;
use jsonschema::Validator;
use matcher::RuleMatcher;
use state::StateManager;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct RuleEngine {
    matcher: RuleMatcher,
    executor: ResponseExecutor,
    pub compiled_schemas: HashMap<String, Arc<Validator>>,
}

impl RuleEngine {
    pub fn new(endpoints: Vec<Endpoint>) -> Self {
        let state_manager = Arc::new(StateManager::new());
        let matcher = RuleMatcher::new(endpoints.clone());
        let executor = ResponseExecutor::new(state_manager.clone());
        let mut compiled_schemas = HashMap::new();

        for endpoint in &endpoints {
            if let Some(schema_val) = &endpoint.schema {
                let validator = jsonschema::validator_for(schema_val).unwrap_or_else(|e| {
                    panic!("Failed to compile schema for {}: {}", endpoint.name, e);
                });
                compiled_schemas.insert(endpoint.name.clone(), Arc::new(validator));
            } else if let Some(schema_path) = &endpoint.schema_file {
                // In a real implementation, we would load from file.
                // For now, let's just handle it as a placeholder.
                let content = std::fs::read_to_string(schema_path).unwrap_or_else(|e| {
                    panic!(
                        "Failed to read schema file {} for {}: {}",
                        schema_path, endpoint.name, e
                    );
                });
                let schema_val: serde_json::Value =
                    serde_json::from_str(&content).unwrap_or_else(|e| {
                        panic!(
                            "Failed to parse JSON from schema file {} for {}: {}",
                            schema_path, endpoint.name, e
                        );
                    });
                let validator = jsonschema::validator_for(&schema_val).unwrap_or_else(|e| {
                    panic!("Failed to compile schema for {}: {}", endpoint.name, e);
                });
                compiled_schemas.insert(endpoint.name.clone(), Arc::new(validator));
            }
        }

        Self {
            matcher,
            executor,
            compiled_schemas,
        }
    }

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

        // Schema validation
        if let Some(validator) = self.compiled_schemas.get(&endpoint.name) {
            let body_val: serde_json::Value = if let Some(body_str) = body {
                match serde_json::from_str(body_str) {
                    Ok(val) => val,
                    Err(e) => {
                        return Ok(RuleResponse {
                            status: 400,
                            body: Some(
                                serde_json::json!({
                                    "error": "Invalid JSON payload",
                                    "details": e.to_string()
                                })
                                .to_string(),
                            ),
                            headers: HashMap::new(),
                        });
                    }
                }
            } else {
                serde_json::Value::Null
            };

            if !validator.is_valid(&body_val) {
                let errors: Vec<String> = validator
                    .iter_errors(&body_val)
                    .map(|err| err.to_string())
                    .collect();

                return Ok(RuleResponse {
                    status: 400,
                    body: Some(
                        serde_json::json!({
                            "error": "Validation error",
                            "details": errors
                        })
                        .to_string(),
                    ),
                    headers: HashMap::new(),
                });
            }
        }

        let context = ExecutionContext {
            method: method.to_string(),
            path: path.to_string(),
            query: query.to_string(),
            headers: headers.clone(),
            client_ip: client_ip.to_string(),
            path_params: self.matcher.extract_path_params(&endpoint.path, path),
        };

        self.executor.execute(endpoint, &context).await
    }
}

pub struct ExecutionContext {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: HashMap<String, String>,
    pub client_ip: String,
    pub path_params: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RuleResponse {
    pub status: u16,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Response;
    use std::collections::HashMap;

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
                default: false,
            }],
            schema: None,
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
        }];

        let _engine = RuleEngine::new(endpoints);
    }

    #[tokio::test]
    async fn test_execute_no_endpoints() {
        let engine = RuleEngine::new(vec![]);
        let result = engine
            .execute("GET", "/test", "", &HashMap::new(), None, "127.0.0.1")
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_rule_engine_compiles_valid_schema() {
        let endpoints = vec![Endpoint {
            name: "Schema Test".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            stateful: false,
            state_key: None,
            schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "age": { "type": "integer" }
                }
            })),
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("OK".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
        }];

        let engine = RuleEngine::new(endpoints);
        assert!(engine.compiled_schemas.contains_key("Schema Test"));
    }

    #[test]
    fn test_rule_engine_compiles_schema_from_file() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        }"#;
        temp_file.write_all(schema_json.as_bytes()).unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let endpoints = vec![Endpoint {
            name: "File Schema Test".to_string(),
            method: "POST".to_string(),
            path: "/api/file".to_string(),
            stateful: false,
            state_key: None,
            schema: None,
            schema_file: Some(path),
            path_regex: None,
            headers_regex: None,
            query_regex: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("OK".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
        }];

        let engine = RuleEngine::new(endpoints);
        assert!(engine.compiled_schemas.contains_key("File Schema Test"));
    }

    #[test]
    #[should_panic(expected = "Failed to compile schema for Schema Test")]
    fn test_rule_engine_panics_on_invalid_schema() {
        let endpoints = vec![Endpoint {
            name: "Schema Test".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            stateful: false,
            state_key: None,
            schema: Some(serde_json::json!({
                "type": "invalid_type"
            })),
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("OK".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
        }];

        let _engine = RuleEngine::new(endpoints);
    }

    #[tokio::test]
    async fn test_execute_schema_validation_success() {
        let endpoints = vec![Endpoint {
            name: "Schema Test".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            stateful: false,
            state_key: None,
            schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer" }
                },
                "required": ["id"]
            })),
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("OK".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
        }];

        let engine = RuleEngine::new(endpoints);
        let result = engine
            .execute(
                "POST",
                "/api",
                "",
                &HashMap::new(),
                Some(r#"{"id": 123}"#),
                "127.0.0.1",
            )
            .await
            .unwrap();

        assert_eq!(result.status, 200);
        assert_eq!(result.body, Some("OK".to_string()));
    }

    #[tokio::test]
    async fn test_execute_schema_validation_failure() {
        let endpoints = vec![Endpoint {
            name: "Schema Test".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            stateful: false,
            state_key: None,
            schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer" }
                },
                "required": ["id"]
            })),
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("OK".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
        }];

        let engine = RuleEngine::new(endpoints);
        let result = engine
            .execute(
                "POST",
                "/api",
                "",
                &HashMap::new(),
                Some(r#"{"id": "not an integer"}"#),
                "127.0.0.1",
            )
            .await
            .unwrap();

        assert_eq!(result.status, 400);
        assert!(result.body.unwrap().contains("Validation error"));
    }

    #[tokio::test]
    async fn test_execute_schema_validation_invalid_json() {
        let endpoints = vec![Endpoint {
            name: "Schema Test".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            stateful: false,
            state_key: None,
            schema: Some(serde_json::json!({ "type": "object" })),
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("OK".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: true,
            }],
        }];

        let engine = RuleEngine::new(endpoints);
        let result = engine
            .execute(
                "POST",
                "/api",
                "",
                &HashMap::new(),
                Some(r#"{"invalid": json"#),
                "127.0.0.1",
            )
            .await
            .unwrap();

        assert_eq!(result.status, 400);
        assert!(result.body.unwrap().contains("Invalid JSON payload"));
    }

    #[tokio::test]
    async fn test_execute_with_regex_matching() {
        let endpoint = Endpoint {
            name: "Regex Test".to_string(),
            method: "GET".to_string(),
            path: "/api".to_string(),
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
        };

        let engine = RuleEngine::new(vec![endpoint]);

        // Should match with correct headers and query
        let mut headers = HashMap::new();
        headers.insert("X-Required".to_string(), "secret".to_string());
        let result = engine
            .execute("GET", "/api", "v=1", &headers, None, "127.0.0.1")
            .await;
        assert!(result.is_ok());

        // Should NOT match if header is wrong
        let mut bad_headers = HashMap::new();
        bad_headers.insert("X-Required".to_string(), "wrong".to_string());
        let result = engine
            .execute("GET", "/api", "v=1", &bad_headers, None, "127.0.0.1")
            .await;
        assert!(result.is_err());
    }
}
