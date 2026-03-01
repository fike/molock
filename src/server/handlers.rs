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

use crate::server::app::AppState;
use crate::server::openapi::{HealthResponse, MetricsResponse};
use crate::telemetry::metrics::{record_error, record_latency, record_request};
use actix_web::http::header;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use std::time::Instant;
use tracing::info;
use tracing::Instrument;
use tracing::Span;

#[utoipa::path(
    get,
    path = "/health",
    tag = "System",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    )
)]
pub async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "molock",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[utoipa::path(
    get,
    path = "/metrics",
    tag = "System",
    responses(
        (status = 200, description = "Prometheus metrics endpoint", body = MetricsResponse)
    )
)]
pub async fn metrics_handler() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "text/plain"))
        .body("# Metrics endpoint - use OpenTelemetry metrics instead")
}

#[allow(unused_variables)]
pub async fn request_handler(
    req: HttpRequest,
    #[allow(unused_variables)] body: web::Bytes,
    data: web::Data<AppState>,
) -> impl Responder {
    let start_time = Instant::now();
    let span = Span::current();

    // Note: http.method and http.target are already set in the tracing middleware
    // at span creation time, so we don't need to record them here

    let request_id = uuid::Uuid::new_v4().to_string();
    span.record("request.id", &request_id);

    info!(
        method = %req.method(),
        path = %req.uri().path(),
        request_id = %request_id,
        "Processing request"
    );

    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let result = process_request(req, body, data).instrument(span).await;

    match result {
        Ok(response) => {
            let latency = start_time.elapsed().as_millis() as f64;
            let status = response.status().as_u16();

            // Record metrics
            record_request(&method, &path, status);
            record_latency(&method, &path, latency);

            info!(
                request_id = %request_id,
                status = status,
                latency_ms = latency,
                "Request completed"
            );
            response
        }
        Err(e) => {
            let latency = start_time.elapsed().as_millis() as f64;

            // Record error metric
            record_request(&method, &path, 500);
            record_latency(&method, &path, latency);
            record_error(&method, &path, "internal_error");

            tracing::error!(
                request_id = %request_id,
                error = %e,
                latency_ms = latency,
                "Request processing failed"
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "request_id": request_id
            }))
        }
    }
}

async fn process_request(
    req: HttpRequest,
    body: web::Bytes,
    data: web::Data<AppState>,
) -> anyhow::Result<HttpResponse> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();
    let headers = req
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body_str = if body.is_empty() {
        None
    } else {
        match String::from_utf8(body.to_vec()) {
            Ok(s) => Some(s),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid UTF-8 sequence in request body"
                })));
            }
        }
    };

    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    let response = data
        .rule_engine
        .execute(
            &method,
            &path,
            &query,
            &headers,
            body_str.as_deref(),
            &client_ip,
        )
        .await?;

    let mut http_response = HttpResponse::build(
        actix_web::http::StatusCode::from_u16(response.status)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    );

    for (key, value) in response.headers {
        http_response.insert_header((key, value));
    }

    if let Some(body) = response.body {
        Ok(http_response.body(body))
    } else {
        Ok(http_response.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Config;
    use crate::rules::RuleEngine;
    use actix_web::test;
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_health_handler() {
        let resp = health_handler().await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        assert_eq!(resp.status(), 200);

        // Check that it's a JSON response
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[actix_web::test]
    async fn test_metrics_handler() {
        let resp = metrics_handler().await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.headers().get("content-type").unwrap(), "text/plain");
    }

    #[actix_web::test]
    async fn test_request_handler_invalid_utf8_body() {
        let mut config = Config::default();
        config.server.max_request_size = 1024 * 1024;
        let rule_engine = Arc::new(RuleEngine::new(config.endpoints.clone()));
        let app_state = web::Data::new(AppState {
            _config: config,
            rule_engine,
        });

        // Create a request with invalid UTF-8 body
        let invalid_utf8 = vec![0, 159, 146, 150]; // Not valid UTF-8
        let req = test::TestRequest::post().uri("/api/test").to_http_request();
        let body = web::Bytes::from(invalid_utf8);

        let resp = request_handler(req, body, app_state).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        // Should return 400 Bad Request because the body is not valid UTF-8
        assert_eq!(resp.status(), 400);
    }
}
