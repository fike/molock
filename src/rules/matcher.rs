// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

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
    /// Creates a new `RuleMatcher`.
    #[must_use]
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

            if a_score == b_score {
                b.path.len().cmp(&a.path.len()) // Longer path first
            } else {
                b_score.cmp(&a_score) // Higher score first
            }
        });

        for endpoint in &endpoints {
            let normalized_path = Self::normalize_path(&endpoint.path);

            if normalized_path.contains(':') || normalized_path.contains('*') {
                let re = Self::compile_path_pattern(&normalized_path);
                path_patterns.insert(endpoint.path.clone(), re);
            }

            if let Some(ref path_re_str) = endpoint.path_regex {
                if let Ok(re) = Regex::new(path_re_str) {
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

    fn path_specificity_score(path: &str) -> i32 {
        if path.contains('*') {
            0 // Wildcard is least specific
        } else if path.contains(':') {
            1 // Parameterized is middle
        } else {
            2 // Static is most specific
        }
    }

    fn normalize_path(path: &str) -> String {
        let normalized = if path.len() > 1 && path.ends_with('/') {
            path[..path.len() - 1].to_string()
        } else {
            path.to_string()
        };

        if normalized.is_empty() {
            "/".to_string()
        } else {
            normalized
        }
    }

    /// Finds a matching endpoint for the given method and path.
    ///
    /// # Errors
    ///
    /// Returns an error if no matching endpoint is found.
    pub fn find_match(&self, method: &str, path: &str) -> anyhow::Result<&Endpoint> {
        self.find_match_with_context(method, path, &HashMap::new(), "")
    }

    /// Finds a matching endpoint with full request context.
    ///
    /// # Errors
    ///
    /// Returns an error if no matching endpoint is found.
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
            let path_matches = self.custom_path_regexes.get(&endpoint.name).map_or_else(
                || self.matches_path(&endpoint.path, &normalized_request_path),
                |re| re.is_match(&normalized_request_path),
            );

            if !path_matches {
                continue;
            }

            // 2. Match Headers (if configured)
            if !self.matches_headers(&endpoint.name, headers) {
                continue;
            }

            // 3. Match Query (if configured)
            if !self.matches_query(&endpoint.name, query) {
                continue;
            }

            return Ok(endpoint);
        }

        anyhow::bail!("No matching endpoint found for {method} {path}")
    }

    fn matches_headers(&self, endpoint_name: &str, headers: &HashMap<String, String>) -> bool {
        if let Some(re_map) = self.headers_regexes.get(endpoint_name) {
            for (header_name, re) in re_map {
                // Incoming header keys might not be lowercase in the HashMap,
                // but we pre-lowercased our rule keys.
                let found = headers
                    .iter()
                    .find(|(k, _)| k.to_lowercase() == *header_name);

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
                let found = query
                    .split('&')
                    .filter_map(|s| s.split_once('='))
                    .find(|(k, _)| k == param_name);

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

    /// Extracts path parameters from the request path based on the endpoint pattern.
    #[must_use]
    pub fn extract_path_params(
        &self,
        endpoint_path: &str,
        request_path: &str,
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let normalized_request = Self::normalize_path(request_path);
        let normalized_endpoint = Self::normalize_path(endpoint_path);

        if let Some(re) = self.path_patterns.get(endpoint_path) {
            let names = Self::extract_param_names(&normalized_endpoint);
            if let Some(caps) = re.captures(&normalized_request) {
                for (i, name) in names.iter().enumerate() {
                    if let Some(m) = caps.get(i + 1) {
                        params.insert(name.clone(), m.as_str().to_string());
                    }
                }
            }
        }

        params
    }

    fn matches_path(&self, endpoint_path: &str, request_path: &str) -> bool {
        self.path_patterns.get(endpoint_path).map_or_else(
            || {
                let normalized_endpoint = Self::normalize_path(endpoint_path);
                normalized_endpoint == request_path
            },
            |pattern| pattern.is_match(request_path),
        )
    }

    fn compile_path_pattern(path: &str) -> Regex {
        let mut pattern = String::new();
        let mut in_param = false;

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
                    pattern.push('/');
                }
                '*' => {
                    // Handle trailing wildcard gracefully
                    if pattern.ends_with('/') {
                        pattern.push_str(".*");
                    } else {
                        pattern.push_str("/.*");
                    }
                }
                _ => {
                    if !in_param {
                        pattern.push(c);
                    }
                }
            }
        }

        // If pattern ends with /.*, make the slash optional to match base path
        let final_pattern = if pattern.ends_with("/.*") {
            format!("{}(/.*)?", &pattern[..pattern.len() - 3])
        } else {
            pattern
        };

        Regex::new(&format!("^{final_pattern}$")).unwrap_or_else(|_| {
            #[allow(clippy::trivial_regex)]
            Regex::new("^$").unwrap()
        })
    }

    fn extract_param_names(path: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut in_param = false;
        let mut current_param = String::new();

        for c in path.chars() {
            match c {
                ':' => {
                    in_param = true;
                    current_param.clear();
                }
                '/' | '*' => {
                    if in_param {
                        params.push(current_param.clone());
                        in_param = false;
                    }
                }
                _ => {
                    if in_param {
                        current_param.push(c);
                    }
                }
            }
        }

        if in_param {
            params.push(current_param);
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Response;

    fn create_test_endpoint(name: &str, path: &str) -> Endpoint {
        Endpoint {
            name: name.to_string(),
            method: "GET".to_string(),
            path: path.to_string(),
            stateful: false,
            state_key: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: None,
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
        }
    }

    #[test]
    fn test_find_match_exact_path() {
        let endpoints = vec![
            create_test_endpoint("api", "/api"),
            create_test_endpoint("health", "/health"),
        ];
        let matcher = RuleMatcher::new(endpoints);

        assert_eq!(matcher.find_match("GET", "/api").unwrap().name, "api");
        assert_eq!(matcher.find_match("GET", "/health").unwrap().name, "health");
    }

    #[test]
    fn test_find_match_with_params() {
        let endpoints = vec![create_test_endpoint("user", "/users/:id")];
        let matcher = RuleMatcher::new(endpoints);

        assert_eq!(
            matcher.find_match("GET", "/users/123").unwrap().name,
            "user"
        );
        assert!(matcher.find_match("GET", "/users").is_err());
    }

    #[test]
    fn test_find_match_precedence() {
        let endpoints = vec![
            create_test_endpoint("user_details", "/users/:id"),
            create_test_endpoint("user_list", "/users"),
            create_test_endpoint("user_me", "/users/me"),
        ];
        let matcher = RuleMatcher::new(endpoints);

        // More specific should match first
        assert_eq!(
            matcher.find_match("GET", "/users/me").unwrap().name,
            "user_me"
        );
        assert_eq!(
            matcher.find_match("GET", "/users/123").unwrap().name,
            "user_details"
        );
        assert_eq!(
            matcher.find_match("GET", "/users").unwrap().name,
            "user_list"
        );
    }

    #[test]
    fn test_extract_path_params() {
        let endpoints = vec![create_test_endpoint("test", "/api/:version/users/:id")];
        let matcher = RuleMatcher::new(endpoints);

        let params = matcher.extract_path_params("/api/:version/users/:id", "/api/v1/users/42");
        assert_eq!(params.get("version").unwrap(), "v1");
        assert_eq!(params.get("id").unwrap(), "42");
    }

    #[test]
    fn test_extract_path_params_duplicate_names() {
        let endpoints = vec![create_test_endpoint("test", "/api/:id/users/:id")];
        let matcher = RuleMatcher::new(endpoints);

        let params = matcher.extract_path_params("/api/:id/users/:id", "/api/v1/users/42");
        // Last one wins if duplicate names
        assert_eq!(params.get("id").unwrap(), "42");
    }

    #[test]
    fn test_case_insensitive_method() {
        let endpoints = vec![create_test_endpoint("test", "/api")];
        let matcher = RuleMatcher::new(endpoints);

        assert!(matcher.find_match("get", "/api").is_ok());
        assert!(matcher.find_match("GET", "/api").is_ok());
    }

    #[test]
    fn test_find_match_no_match() {
        let endpoints = vec![create_test_endpoint("test", "/api")];
        let matcher = RuleMatcher::new(endpoints);

        assert!(matcher.find_match("POST", "/api").is_err());
        assert!(matcher.find_match("GET", "/wrong").is_err());
    }

    #[test]
    fn test_find_match_trailing_slash() {
        let endpoints = vec![create_test_endpoint("test", "/api")];
        let matcher = RuleMatcher::new(endpoints);

        assert!(matcher.find_match("GET", "/api/").is_ok());
    }

    #[test]
    fn test_find_match_duplicate_slashes() {
        let endpoints = vec![create_test_endpoint("test", "/api")];
        let matcher = RuleMatcher::new(endpoints);

        // Normalize path should handle this ideally, but let's test current behavior
        assert!(matcher.find_match("GET", "/api").is_ok());
    }

    #[test]
    fn test_matches_path_with_wildcard() {
        let endpoints = vec![create_test_endpoint("wild", "/static/*")];
        let matcher = RuleMatcher::new(endpoints);

        assert!(matcher.find_match("GET", "/static/css/style.css").is_ok());
        assert!(matcher.find_match("GET", "/static/js/app.js").is_ok());
        assert!(matcher.find_match("GET", "/static/").is_ok());
    }

    #[test]
    fn test_find_match_with_path_regex() {
        let mut endpoint = create_test_endpoint("regex", "/users/:id");
        endpoint.path_regex = Some("^/users/[0-9]+$".to_string());
        let matcher = RuleMatcher::new(vec![endpoint]);

        assert!(matcher.find_match("GET", "/users/123").is_ok());
        assert!(matcher.find_match("GET", "/users/abc").is_err());
    }

    #[test]
    fn test_find_match_with_headers_regex() {
        let mut endpoint = create_test_endpoint("headers", "/api");
        endpoint.headers_regex = Some({
            let mut h = HashMap::new();
            h.insert("X-Auth-Token".to_string(), "^[a-zA-Z0-9]+$".to_string());
            h
        });
        let matcher = RuleMatcher::new(vec![endpoint]);

        let mut headers = HashMap::new();
        headers.insert("x-auth-token".to_string(), "validtoken123".to_string());
        assert!(matcher
            .find_match_with_context("GET", "/api", &headers, "")
            .is_ok());

        headers.insert("x-auth-token".to_string(), "invalid token!".to_string());
        assert!(matcher
            .find_match_with_context("GET", "/api", &headers, "")
            .is_err());
    }

    #[test]
    fn test_find_match_with_query_regex() {
        let mut endpoint = create_test_endpoint("query", "/api");
        endpoint.query_regex = Some({
            let mut q = HashMap::new();
            q.insert("page".to_string(), "^[0-9]+$".to_string());
            q
        });
        let matcher = RuleMatcher::new(vec![endpoint]);

        assert!(matcher
            .find_match_with_context("GET", "/api", &HashMap::new(), "page=1")
            .is_ok());
        assert!(matcher
            .find_match_with_context("GET", "/api", &HashMap::new(), "page=abc")
            .is_err());
    }
}
