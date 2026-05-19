// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

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
    /// Creates a new `ResponseExecutor`.
    #[must_use]
    pub const fn new(state_manager: Arc<StateManager>) -> Self {
        Self { state_manager }
    }

    /// Executes the response for a matched endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if no response can be selected or if delay parsing fails.
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
            let key = endpoint.state_key.as_deref().unwrap_or("client_ip");

            if key == "client_ip" {
                context.client_ip.clone()
            } else {
                let key_lower = key.to_lowercase();
                context
                    .headers
                    .get(&key_lower)
                    .map_or_else(|| context.client_ip.clone(), Clone::clone)
            }
        } else {
            String::new()
        };

        if endpoint.stateful && !state_key.is_empty() {
            let _ = self.state_manager.increment_count(&state_key);
        }

        let request_count = if endpoint.stateful && !state_key.is_empty() {
            self.state_manager.get_count(&state_key)
        } else {
            0
        };

        let candidate_responses: Vec<&Response> = endpoint
            .responses
            .iter()
            .filter(|r| Self::evaluate_condition(r, context, request_count))
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
            Self::select_by_probability(&candidate_responses)?
        };

        let delay = if let Some(delay_config) = &selected_response.delay {
            let (min, max) = delay_config.parse_range()?;
            if min == max {
                u64::try_from(min.as_millis()).unwrap_or(u64::MAX)
            } else {
                let mut rng = rand::thread_rng();
                u64::try_from(rng.gen_range(min.as_millis()..=max.as_millis())).unwrap_or(u64::MAX)
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
            .map(|body_template| Self::render_template(body_template, context, request_count));

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
        response: &Response,
        context: &ExecutionContext,
        request_count: u64,
    ) -> bool {
        response
            .condition
            .as_ref()
            .is_none_or(|condition| Self::evaluate_expression(condition, context, request_count))
    }

    fn evaluate_expression(
        expression: &str,
        _context: &ExecutionContext,
        request_count: u64,
    ) -> bool {
        // Simple expression evaluation
        // In a real implementation, this would use a proper expression evaluator
        let expr = expression.trim().to_lowercase();

        if expr.contains("request_count") {
            // Parse simple comparisons like "request_count > 2"
            let parts: Vec<&str> = expr.split_whitespace().collect();
            if parts.len() == 3 && parts[0] == "request_count" {
                if let Ok(value) = parts[2].parse::<u64>() {
                    match parts[1] {
                        ">" => return request_count > value,
                        "<" => return request_count < value,
                        ">=" => return request_count >= value,
                        "<=" => return request_count <= value,
                        "==" | "=" => return request_count == value,
                        "!=" => return request_count != value,
                        _ => {}
                    }
                }
            }
        }

        // Default to true for simple expressions
        true
    }

    fn select_by_probability<'a>(responses: &[&'a Response]) -> anyhow::Result<&'a Response> {
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

    #[allow(clippy::uninlined_format_args)]
    fn render_template(template: &str, context: &ExecutionContext, request_count: u64) -> String {
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

        for param in context.query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                result = result.replace(&format!("{{{{query.{}}}}}", key), value);
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
            body: None,
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
            schema: None,
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
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

        assert!(!ResponseExecutor::evaluate_condition(
            &response, &context, 1
        ));
        assert!(ResponseExecutor::evaluate_condition(&response, &context, 3));
    }

    #[test]
    fn test_render_template() {
        let mut context = create_test_context();
        context
            .path_params
            .insert("id".to_string(), "123".to_string());
        context.query = "name=John&age=30".to_string();

        let template = "User {{id}} ({{query.name}}) from {{client_ip}}";
        let result = ResponseExecutor::render_template(template, &context, 1);

        assert!(result.contains("123"));
        assert!(result.contains("John"));
        assert!(result.contains("127.0.0.1"));
    }

    #[test]
    fn test_render_template_empty_query() {
        let mut context = create_test_context();
        context.query = "".to_string();

        let template = "User {{query.name}}";
        let result = ResponseExecutor::render_template(template, &context, 1);

        assert_eq!(result, "User {{query.name}}");
    }

    #[tokio::test]
    async fn test_execute_stateful_custom_key() {
        let state_manager = Arc::new(StateManager::new());
        let executor = ResponseExecutor::new(state_manager.clone());

        let mut endpoint = create_test_endpoint();
        endpoint.stateful = true;
        endpoint.state_key = Some("X-User-ID".to_string());

        let mut context = create_test_context();
        context
            .headers
            .insert("x-user-id".to_string(), "user1".to_string());

        let result = executor.execute(&endpoint, &context).await.unwrap();
        assert_eq!(
            result.headers.get("X-Request-Count"),
            Some(&"1".to_string())
        );
        assert_eq!(state_manager.get_count("user1"), 1);

        // Test missing custom key falls back to client_ip
        context.headers.remove("x-user-id");
        let _result = executor.execute(&endpoint, &context).await.unwrap();
        assert_eq!(state_manager.get_count("127.0.0.1"), 1);
    }

    #[test]
    fn test_evaluate_expression_operators() {
        let context = create_test_context();

        assert!(ResponseExecutor::evaluate_expression(
            "request_count < 5",
            &context,
            3
        ));
        assert!(ResponseExecutor::evaluate_expression(
            "request_count >= 3",
            &context,
            3
        ));
        assert!(ResponseExecutor::evaluate_expression(
            "request_count <= 3",
            &context,
            3
        ));
        assert!(ResponseExecutor::evaluate_expression(
            "request_count == 3",
            &context,
            3
        ));
        assert!(ResponseExecutor::evaluate_expression(
            "request_count = 3",
            &context,
            3
        ));
        assert!(ResponseExecutor::evaluate_expression(
            "request_count != 4",
            &context,
            3
        ));

        // Invalid operator or format
        assert!(ResponseExecutor::evaluate_expression(
            "request_count ?? 3",
            &context,
            3
        ));
        assert!(ResponseExecutor::evaluate_expression(
            "invalid", &context, 3
        ));
    }

    #[test]
    fn test_select_by_probability_no_probability() {
        let response = Response {
            status: 200,
            delay: None,
            body: None,
            headers: HashMap::new(),
            condition: None,
            probability: None,
            default: false,
        };

        let result = ResponseExecutor::select_by_probability(&[&response]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_range_delay() {
        let state_manager = Arc::new(StateManager::new());
        let executor = ResponseExecutor::new(state_manager);

        let mut endpoint = create_test_endpoint();
        endpoint.responses[0].delay = Some(Delay::Range("10ms-50ms".to_string()));

        let context = create_test_context();
        let result = executor.execute(&endpoint, &context).await.unwrap();
        assert_eq!(result.status, 200);
    }
}
