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
use matcher::RuleMatcher;
use state::StateManager;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct RuleEngine {
    matcher: RuleMatcher,
    executor: ResponseExecutor,
}

impl RuleEngine {
    pub fn new(endpoints: Vec<Endpoint>) -> Self {
        let state_manager = Arc::new(StateManager::new());
        let matcher = RuleMatcher::new(endpoints.clone());
        let executor = ResponseExecutor::new(state_manager.clone());

        Self { matcher, executor }
    }

    pub async fn execute(
        &self,
        method: &str,
        path: &str,
        query: &str,
        headers: &HashMap<String, String>,
        _body: Option<&str>,
        client_ip: &str,
    ) -> anyhow::Result<RuleResponse> {
        let endpoint = self.matcher.find_match(method, path)?;

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
}
