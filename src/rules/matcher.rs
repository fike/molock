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

use crate::config::Endpoint;
use regex::Regex;
use std::collections::HashMap;

#[derive(Clone)]
pub struct RuleMatcher {
    endpoints: Vec<Endpoint>,
    path_patterns: HashMap<String, Regex>,
    custom_path_regexes: HashMap<String, Regex>,
    headers_regexes: HashMap<String, HashMap<String, Regex>>,
    query_regexes: HashMap<String, HashMap<String, Regex>>,
}

impl RuleMatcher {
    pub fn new(mut endpoints: Vec<Endpoint>) -> Self {
        let mut path_patterns = HashMap::new();
        let mut custom_path_regexes = HashMap::new();
        let mut headers_regexes = HashMap::new();
        let mut query_regexes = HashMap::new();

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

            if let Some(ref path_regex_str) = endpoint.path_regex {
                if let Ok(re) = Regex::new(path_regex_str) {
                    custom_path_regexes.insert(endpoint.name.clone(), re);
                }
            }

            if let Some(ref headers_regex_map) = endpoint.headers_regex {
                let mut re_map = HashMap::new();
                for (header, re_str) in headers_regex_map {
                    if let Ok(re) = Regex::new(re_str) {
                        re_map.insert(header.to_lowercase(), re);
                    }
                }
                headers_regexes.insert(endpoint.name.clone(), re_map);
            }

            if let Some(ref query_regex_map) = endpoint.query_regex {
                let mut re_map = HashMap::new();
                for (param, re_str) in query_regex_map {
                    if let Ok(re) = Regex::new(re_str) {
                        re_map.insert(param.clone(), re);
                    }
                }
                query_regexes.insert(endpoint.name.clone(), re_map);
            }
        }

        Self {
            endpoints,
            path_patterns,
            custom_path_regexes,
            headers_regexes,
            query_regexes,
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
        self.find_match_with_context(method, path, &HashMap::new(), "")
    }

    pub fn find_match_with_context(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
        query: &str,
    ) -> anyhow::Result<&Endpoint> {
        let normalized_request_path = Self::normalize_path(path);

        for endpoint in &self.endpoints {
            if endpoint.method.to_uppercase() != method.to_uppercase() {
                continue;
            }

            // 1. Match Path (Exact/Param/Wildcard OR Custom Regex)
            let path_matches = if let Some(re) = self.custom_path_regexes.get(&endpoint.name) {
                re.is_match(&normalized_request_path)
            } else {
                self.matches_path(&endpoint.path, &normalized_request_path)
            };

            if !path_matches {
                continue;
            }

            // 2. Match Headers Regex
            if !self.matches_headers(&endpoint.name, headers) {
                continue;
            }

            // 3. Match Query Regex
            if !self.matches_query(&endpoint.name, query) {
                continue;
            }

            return Ok(endpoint);
        }

        anyhow::bail!("No matching endpoint found for {} {}", method, path)
    }

    fn matches_headers(&self, endpoint_name: &str, headers: &HashMap<String, String>) -> bool {
        if let Some(re_map) = self.headers_regexes.get(endpoint_name) {
            for (header_name, re) in re_map {
                // Incoming header keys might not be lowercase in the HashMap, 
                // but we pre-lowercased our rule keys.
                let found = headers.iter().find(|(k, _)| k.to_lowercase() == *header_name);

                match found {
                    Some((_, v)) => {
                        if !re.is_match(v) {
                            return false;
                        }
                    }
                    None => return false, // Required header missing
                }
            }
        }
        true
    }

    fn matches_query(&self, endpoint_name: &str, query: &str) -> bool {
        if let Some(re_map) = self.query_regexes.get(endpoint_name) {
            for (param_name, re) in re_map {
                // Lazy scan query string for the parameter
                let found = query.split('&').filter_map(|s| s.split_once('=')).find(|(k, _)| k == param_name);

                match found {
                    Some((_, v)) => {
                        if !re.is_match(v) {
                            return false;
                        }
                    }
                    None => return false, // Required query param missing
                }
            }
        }
        true
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
            schema: None,
            schema_file: None,
            path_regex: None,
            headers_regex: None,
            query_regex: None,
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

    #[test]
    fn test_extract_path_params_duplicate_names() {
        let endpoints = vec![create_test_endpoint("GET", "/users/:id/posts/:id")];
        let matcher = RuleMatcher::new(endpoints);

        let params = matcher.extract_path_params("/users/:id/posts/:id", "/users/123/posts/456");

        // It should return the last value for the duplicate parameter name
        assert_eq!(params.get("id").unwrap(), "456");
    }

    #[test]
    fn test_find_match_with_path_regex() {
        let mut endpoint = create_test_endpoint("GET", "/users/:id");
        endpoint.path_regex = Some(r"^/users/[0-9]+$".to_string());
        let matcher = RuleMatcher::new(vec![endpoint]);

        // Should match numeric ID
        assert!(matcher
            .find_match_with_context("GET", "/users/123", &HashMap::new(), "")
            .is_ok());

        // Should NOT match alphabetic ID
        assert!(matcher
            .find_match_with_context("GET", "/users/abc", &HashMap::new(), "")
            .is_err());
    }

    #[test]
    fn test_find_match_with_headers_regex() {
        let mut endpoint = create_test_endpoint("GET", "/api");
        let mut headers_regex = HashMap::new();
        headers_regex.insert("X-Auth".to_string(), r"^token-[0-9]+$".to_string());
        endpoint.headers_regex = Some(headers_regex);

        let matcher = RuleMatcher::new(vec![endpoint]);

        let mut valid_headers = HashMap::new();
        valid_headers.insert("X-Auth".to_string(), "token-123".to_string());
        assert!(matcher
            .find_match_with_context("GET", "/api", &valid_headers, "")
            .is_ok());

        let mut invalid_headers = HashMap::new();
        invalid_headers.insert("X-Auth".to_string(), "token-abc".to_string());
        assert!(matcher
            .find_match_with_context("GET", "/api", &invalid_headers, "")
            .is_err());
    }

    #[test]
    fn test_find_match_with_query_regex() {
        let mut endpoint = create_test_endpoint("GET", "/api");
        let mut query_regex = HashMap::new();
        query_regex.insert("page".to_string(), r"^[0-9]+$".to_string());
        endpoint.query_regex = Some(query_regex);

        let matcher = RuleMatcher::new(vec![endpoint]);

        assert!(matcher
            .find_match_with_context("GET", "/api", &HashMap::new(), "page=1")
            .is_ok());
        assert!(matcher
            .find_match_with_context("GET", "/api", &HashMap::new(), "page=abc")
            .is_err());
    }
}
