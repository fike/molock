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

use crate::config::{Endpoint, Response};
use crate::rules::state::StateManager;
use crate::rules::{ExecutionContext, RuleResponse};
use anyhow::Context;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

#[derive(Clone)]
pub struct ResponseExecutor {
    state_manager: Arc<StateManager>,
}

impl ResponseExecutor {
    pub fn new(state_manager: Arc<StateManager>) -> Self {
        Self { state_manager }
    }

    pub async fn execute(
        &self,
        endpoint: &Endpoint,
        context: &ExecutionContext,
    ) -> anyhow::Result<RuleResponse> {
        info!(
            endpoint = %endpoint.name,
            method = %context.method,
            path = %context.path,
            "Executing endpoint"
        );

        let state_key = if endpoint.stateful {
            let key = endpoint
                .state_key
                .as_deref()
                .unwrap_or("client_ip")
                .to_string();

            match key.as_str() {
                "client_ip" => context.client_ip.clone(),
                _ => {
                    if let Some(value) = context.headers.get(&key) {
                        value.clone()
                    } else {
                        context.client_ip.clone()
                    }
                }
            }
        } else {
            "".to_string()
        };

        if endpoint.stateful && !state_key.is_empty() {
            self.state_manager.increment_count(&state_key);
        }

        let request_count = if endpoint.stateful && !state_key.is_empty() {
            self.state_manager.get_count(&state_key)
        } else {
            0
        };

        let candidate_responses: Vec<&Response> = endpoint
            .responses
            .iter()
            .filter(|r| self.evaluate_condition(r, context, request_count))
            .collect();

        let selected_response = if candidate_responses.is_empty() {
            endpoint
                .responses
                .iter()
                .find(|r| r.default)
                .context("No matching response and no default response found")?
        } else if candidate_responses.len() == 1 {
            candidate_responses[0]
        } else {
            self.select_by_probability(&candidate_responses)?
        };

        let delay = if let Some(delay_config) = &selected_response.delay {
            let (min, max) = delay_config.parse_range()?;
            if min == max {
                min.as_millis() as u64
            } else {
                let mut rng = rand::thread_rng();
                rng.gen_range(min.as_millis()..=max.as_millis()) as u64
            }
        } else {
            0
        };

        if delay > 0 {
            info!(delay_ms = delay, "Adding delay to response");
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        let body = selected_response
            .body
            .as_ref()
            .map(|body_template| self.render_template(body_template, context, request_count));

        let mut headers = selected_response.headers.clone();
        headers.insert(
            "X-Request-ID".to_string(),
            context
                .headers
                .get("x-request-id")
                .cloned()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        );

        if endpoint.stateful {
            headers.insert("X-Request-Count".to_string(), request_count.to_string());
        }

        Ok(RuleResponse {
            status: selected_response.status,
            body,
            headers,
        })
    }

    fn evaluate_condition(
        &self,
        response: &Response,
        context: &ExecutionContext,
        request_count: u64,
    ) -> bool {
        if let Some(condition) = &response.condition {
            match self.evaluate_expression(condition, context, request_count) {
                Ok(result) => result,
                Err(e) => {
                    tracing::warn!(
                        condition = %condition,
                        error = %e,
                        "Failed to evaluate condition"
                    );
                    false
                }
            }
        } else {
            true
        }
    }

    fn evaluate_expression(
        &self,
        expression: &str,
        _context: &ExecutionContext,
        request_count: u64,
    ) -> anyhow::Result<bool> {
        // Simple expression evaluation
        // In a real implementation, this would use a proper expression evaluator
        let expr = expression.trim().to_lowercase();

        if expr.contains("request_count") {
            // Parse simple comparisons like "request_count > 2"
            let parts: Vec<&str> = expr.split_whitespace().collect();
            if parts.len() == 3 && parts[0] == "request_count" {
                if let Ok(value) = parts[2].parse::<u64>() {
                    match parts[1] {
                        ">" => return Ok(request_count > value),
                        "<" => return Ok(request_count < value),
                        ">=" => return Ok(request_count >= value),
                        "<=" => return Ok(request_count <= value),
                        "==" | "=" => return Ok(request_count == value),
                        "!=" => return Ok(request_count != value),
                        _ => {}
                    }
                }
            }
        }

        // Default to true for simple expressions
        Ok(true)
    }

    fn select_by_probability<'a>(
        &self,
        responses: &[&'a Response],
    ) -> anyhow::Result<&'a Response> {
        let total_probability: f64 = responses.iter().map(|r| r.probability.unwrap_or(0.0)).sum();

        if total_probability == 0.0 {
            anyhow::bail!("No responses with probability specified");
        }

        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen_range(0.0..total_probability);

        let mut cumulative = 0.0;
        for response in responses {
            let probability = response.probability.unwrap_or(0.0);
            cumulative += probability;
            if random_value < cumulative {
                return Ok(response);
            }
        }

        Ok(responses.last().unwrap())
    }

    fn render_template(
        &self,
        template: &str,
        context: &ExecutionContext,
        request_count: u64,
    ) -> String {
        let mut result = template.to_string();

        result = result.replace("{{request_count}}", &request_count.to_string());
        result = result.replace("{{method}}", &context.method);
        result = result.replace("{{path}}", &context.path);
        result = result.replace("{{client_ip}}", &context.client_ip);
        result = result.replace("{{timestamp}}", &chrono::Utc::now().to_rfc3339());
        result = result.replace("{{uuid}}", &uuid::Uuid::new_v4().to_string());
        result = result.replace("{{request_id}}", &uuid::Uuid::new_v4().to_string());

        for (key, value) in &context.path_params {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }

        if let Some(query) = context.query.split('?').next() {
            for param in query.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    result = result.replace(&format!("{{{{query.{}}}}}", key), value);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Delay, Endpoint, Response};
    use std::collections::HashMap;

    fn create_test_context() -> ExecutionContext {
        ExecutionContext {
            method: "GET".to_string(),
            path: "/test".to_string(),
            query: "".to_string(),
            headers: HashMap::new(),
            client_ip: "127.0.0.1".to_string(),
            path_params: HashMap::new(),
        }
    }

    fn create_test_endpoint() -> Endpoint {
        Endpoint {
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
        }
    }

    #[tokio::test]
    async fn test_execute_basic_response() {
        let state_manager = Arc::new(StateManager::new());
        let executor = ResponseExecutor::new(state_manager);
        let endpoint = create_test_endpoint();
        let context = create_test_context();

        let result = executor.execute(&endpoint, &context).await.unwrap();
        assert_eq!(result.status, 200);
        assert_eq!(result.body, Some("OK".to_string()));
    }

    #[tokio::test]
    async fn test_execute_with_delay() {
        let state_manager = Arc::new(StateManager::new());
        let executor = ResponseExecutor::new(state_manager);

        let mut endpoint = create_test_endpoint();
        endpoint.responses[0].delay = Some(Delay::Fixed("100ms".to_string()));

        let context = create_test_context();

        let start = std::time::Instant::now();
        let result = executor.execute(&endpoint, &context).await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(result.status, 200);
        assert!(elapsed >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_execute_stateful() {
        let state_manager = Arc::new(StateManager::new());
        let executor = ResponseExecutor::new(state_manager.clone());

        let mut endpoint = create_test_endpoint();
        endpoint.stateful = true;

        let context = create_test_context();

        let result1 = executor.execute(&endpoint, &context).await.unwrap();
        let result2 = executor.execute(&endpoint, &context).await.unwrap();

        assert_eq!(
            result1.headers.get("X-Request-Count"),
            Some(&"1".to_string())
        );
        assert_eq!(
            result2.headers.get("X-Request-Count"),
            Some(&"2".to_string())
        );
        assert_eq!(state_manager.get_count("127.0.0.1"), 2);
    }

    #[test]
    fn test_evaluate_condition() {
        let state_manager = Arc::new(StateManager::new());
        let executor = ResponseExecutor::new(state_manager);

        let response = Response {
            status: 200,
            delay: None,
            body: None,
            headers: HashMap::new(),
            condition: Some("request_count > 2".to_string()),
            probability: None,
            default: false,
        };

        let context = create_test_context();

        assert!(!executor.evaluate_condition(&response, &context, 1));
        assert!(executor.evaluate_condition(&response, &context, 3));
    }

    #[test]
    fn test_render_template() {
        let state_manager = Arc::new(StateManager::new());
        let executor = ResponseExecutor::new(state_manager);

        let mut context = create_test_context();
        context
            .path_params
            .insert("id".to_string(), "123".to_string());

        let template = "User {{id}} from {{client_ip}}";
        let result = executor.render_template(template, &context, 1);

        assert!(result.contains("123"));
        assert!(result.contains("127.0.0.1"));
    }

    #[test]
    fn test_select_by_probability() {
        let state_manager = Arc::new(StateManager::new());
        let executor = ResponseExecutor::new(state_manager);

        let responses = vec![
            Response {
                status: 200,
                delay: None,
                body: None,
                headers: HashMap::new(),
                condition: None,
                probability: Some(0.3),
                default: false,
            },
            Response {
                status: 500,
                delay: None,
                body: None,
                headers: HashMap::new(),
                condition: None,
                probability: Some(0.7),
                default: false,
            },
        ];

        let refs: Vec<&Response> = responses.iter().collect();
        let selected = executor.select_by_probability(&refs).unwrap();

        assert!(selected.status == 200 || selected.status == 500);
    }
}
