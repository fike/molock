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

use crate::config::Endpoint;
use regex::Regex;
use std::collections::HashMap;

#[derive(Clone)]
pub struct RuleMatcher {
    endpoints: Vec<Endpoint>,
    path_patterns: HashMap<String, Regex>,
}

impl RuleMatcher {
    pub fn new(mut endpoints: Vec<Endpoint>) -> Self {
        let mut path_patterns = HashMap::new();

        // Sort endpoints by specificity:
        // 1. Static paths (no : or *)
        // 2. Paths with parameters (:)
        // 3. Paths with wildcards (*)
        // Among those, longer paths come first.
        endpoints.sort_by(|a, b| {
            let a_score = Self::path_specificity_score(&a.path);
            let b_score = Self::path_specificity_score(&b.path);

            if a_score != b_score {
                b_score.cmp(&a_score) // Higher score first
            } else {
                b.path.len().cmp(&a.path.len()) // Longer path first
            }
        });

        for endpoint in &endpoints {
            let normalized_path = Self::normalize_path(&endpoint.path);
            let pattern = Self::compile_path_pattern(&normalized_path);
            path_patterns.insert(endpoint.path.clone(), pattern);
        }

        Self {
            endpoints,
            path_patterns,
        }
    }

    fn path_specificity_score(path: &str) -> u32 {
        if path.contains('*') {
            1
        } else if path.contains(':') {
            2
        } else {
            3
        }
    }

    fn normalize_path(path: &str) -> String {
        let mut normalized = String::new();
        let mut last_was_slash = false;

        for c in path.chars() {
            if c == '/' {
                if !last_was_slash {
                    normalized.push(c);
                    last_was_slash = true;
                }
            } else {
                normalized.push(c);
                last_was_slash = false;
            }
        }

        // Remove trailing slash if not the only character
        if normalized.len() > 1 && normalized.ends_with('/') {
            normalized.pop();
        }

        if normalized.is_empty() {
            "/".to_string()
        } else {
            normalized
        }
    }

    pub fn find_match(&self, method: &str, path: &str) -> anyhow::Result<&Endpoint> {
        let normalized_request_path = Self::normalize_path(path);

        for endpoint in &self.endpoints {
            if endpoint.method.to_uppercase() != method.to_uppercase() {
                continue;
            }

            if self.matches_path(&endpoint.path, &normalized_request_path) {
                return Ok(endpoint);
            }
        }

        anyhow::bail!("No matching endpoint found for {} {}", method, path)
    }

    pub fn extract_path_params(
        &self,
        endpoint_path: &str,
        request_path: &str,
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let normalized_request_path = Self::normalize_path(request_path);

        if let Some(pattern) = self.path_patterns.get(endpoint_path) {
            if let Some(captures) = pattern.captures(&normalized_request_path) {
                let param_names = Self::extract_param_names(endpoint_path);

                for (i, name) in param_names.iter().enumerate() {
                    if let Some(value) = captures.get(i + 1) {
                        params.insert(name.clone(), value.as_str().to_string());
                    }
                }
            }
        }

        params
    }

    fn matches_path(&self, endpoint_path: &str, request_path: &str) -> bool {
        if let Some(pattern) = self.path_patterns.get(endpoint_path) {
            pattern.is_match(request_path)
        } else {
            let normalized_endpoint = Self::normalize_path(endpoint_path);
            normalized_endpoint == request_path
        }
    }

    fn compile_path_pattern(path: &str) -> Regex {
        let mut pattern = String::new();
        let mut in_param = false;
        let _param_name = String::new();

        for c in path.chars() {
            match c {
                ':' => {
                    in_param = true;
                    pattern.push_str("([^/]+)");
                }
                '/' => {
                    if in_param {
                        in_param = false;
                    }
                    pattern.push_str("\\/");
                }
                '*' => {
                    pattern.push_str(".*");
                }
                _ => {
                    if !in_param {
                        pattern.push(c);
                    }
                }
            }
        }

        Regex::new(&format!("^{}$", pattern)).unwrap_or_else(|_| Regex::new("^$").unwrap())
    }

    fn extract_param_names(path: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut in_param = false;
        let mut param_name = String::new();

        for c in path.chars() {
            match c {
                ':' => {
                    in_param = true;
                    param_name.clear();
                }
                '/' => {
                    if in_param && !param_name.is_empty() {
                        params.push(param_name.clone());
                    }
                    in_param = false;
                    param_name.clear();
                }
                _ => {
                    if in_param {
                        param_name.push(c);
                    }
                }
            }
        }

        if in_param && !param_name.is_empty() {
            params.push(param_name);
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Response;
    use std::collections::HashMap;

    fn create_test_endpoint(method: &str, path: &str) -> Endpoint {
        Endpoint {
            name: "Test".to_string(),
            method: method.to_string(),
            path: path.to_string(),
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

    #[test]
    fn test_find_match_exact_path() {
        let endpoints = vec![
            create_test_endpoint("GET", "/api/users"),
            create_test_endpoint("POST", "/api/users"),
        ];

        let matcher = RuleMatcher::new(endpoints);

        let endpoint = matcher.find_match("GET", "/api/users").unwrap();
        assert_eq!(endpoint.method, "GET");
        assert_eq!(endpoint.path, "/api/users");

        let endpoint = matcher.find_match("POST", "/api/users").unwrap();
        assert_eq!(endpoint.method, "POST");
        assert_eq!(endpoint.path, "/api/users");
    }

    #[test]
    fn test_find_match_with_params() {
        let endpoints = vec![create_test_endpoint("GET", "/users/:id")];
        let matcher = RuleMatcher::new(endpoints);

        let endpoint = matcher.find_match("GET", "/users/123").unwrap();
        assert_eq!(endpoint.path, "/users/:id");
    }

    #[test]
    fn test_find_match_no_match() {
        let endpoints = vec![create_test_endpoint("GET", "/api/users")];
        let matcher = RuleMatcher::new(endpoints);

        let result = matcher.find_match("GET", "/api/products");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_path_params() {
        let endpoints = vec![create_test_endpoint("GET", "/users/:id/posts/:post_id")];
        let matcher = RuleMatcher::new(endpoints);

        // First find the endpoint
        let endpoint = matcher.find_match("GET", "/users/123/posts/456").unwrap();
        let params = matcher.extract_path_params(&endpoint.path, "/users/123/posts/456");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("post_id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_extract_param_names() {
        let params = RuleMatcher::extract_param_names("/users/:id/posts/:post_id/comments");
        assert_eq!(params, vec!["id".to_string(), "post_id".to_string()]);

        let params = RuleMatcher::extract_param_names("/static/path");
        assert!(params.is_empty());

        let params = RuleMatcher::extract_param_names("/:single");
        assert_eq!(params, vec!["single".to_string()]);
    }

    #[test]
    fn test_matches_path_with_wildcard() {
        let endpoints = vec![create_test_endpoint("GET", "/api/*")];
        let matcher = RuleMatcher::new(endpoints);

        let endpoint = matcher.find_match("GET", "/api/users").unwrap();
        assert_eq!(endpoint.path, "/api/*");

        let endpoint = matcher.find_match("GET", "/api/users/123").unwrap();
        assert_eq!(endpoint.path, "/api/*");
    }

    #[test]
    fn test_case_insensitive_method() {
        let endpoints = vec![create_test_endpoint("GET", "/test")];
        let matcher = RuleMatcher::new(endpoints);

        let endpoint = matcher.find_match("get", "/test").unwrap();
        assert_eq!(endpoint.method, "GET");
    }

    #[test]
    fn test_find_match_trailing_slash() {
        let endpoints = vec![create_test_endpoint("GET", "/api/users")];
        let matcher = RuleMatcher::new(endpoints);

        // Should match even with trailing slash in the request
        let endpoint = matcher.find_match("GET", "/api/users/").unwrap();
        assert_eq!(endpoint.path, "/api/users");
    }

    #[test]
    fn test_find_match_duplicate_slashes() {
        let endpoints = vec![create_test_endpoint("GET", "/api/users")];
        let matcher = RuleMatcher::new(endpoints);

        // Should match even with duplicate slashes in the request
        let endpoint = matcher.find_match("GET", "//api///users").unwrap();
        assert_eq!(endpoint.path, "/api/users");
    }

    #[test]
    fn test_find_match_precedence() {
        let endpoints = vec![
            create_test_endpoint("GET", "/api/*"),
            create_test_endpoint("GET", "/api/users"),
            create_test_endpoint("GET", "/api/:id"),
        ];
        let matcher = RuleMatcher::new(endpoints);

        // Exact match should win over param or wildcard
        let endpoint = matcher.find_match("GET", "/api/users").unwrap();
        assert_eq!(endpoint.path, "/api/users");

        // Param match should win over wildcard
        let endpoint = matcher.find_match("GET", "/api/123").unwrap();
        assert_eq!(endpoint.path, "/api/:id");
    }
}
