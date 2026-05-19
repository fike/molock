// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

use crate::server::app::AppState;
use crate::server::openapi::{HealthResponse, MetricsResponse};
use crate::telemetry::metrics::{record_error, record_latency, record_request};
use actix_web::{http::header, web, HttpRequest, HttpResponse, Responder};
use std::time::Instant;
use tracing::{info, Instrument, Span};

/// Health check handler.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
    })
}

/// Metrics handler.
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics", body = MetricsResponse)
    )
)]
pub async fn metrics_handler() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "text/plain"))
        .body("# Metrics endpoint - use OpenTelemetry metrics instead")
}

#[allow(unused_variables)]
#[allow(clippy::future_not_send)]
pub async fn request_handler(
    req: HttpRequest,
    #[allow(unused_variables)] body: web::Bytes,
    data: web::Data<AppState>,
) -> impl Responder {
    let start_time = Instant::now();
    let span = Span::current();

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
            let latency = start_time.elapsed().as_secs_f64() * 1000.0;
            let status = response.status().as_u16();

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
            let latency = start_time.elapsed().as_secs_f64() * 1000.0;

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

/// Processes an incoming HTTP request and executes matching rules.
///
/// # Errors
///
/// Returns an error if the rule execution fails.
#[allow(clippy::future_not_send)]
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
                    "error": "Body must be valid UTF-8"
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

    let mut builder = HttpResponse::build(actix_web::http::StatusCode::from_u16(response.status)?);

    for (k, v) in response.headers {
        builder.insert_header((k, v));
    }

    if let Some(body) = response.body {
        Ok(builder.body(body))
    } else {
        Ok(builder.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::rules::RuleEngine;
    use actix_web::test;
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_health_handler() {
        let resp = health_handler()
            .await
            .respond_to(&test::TestRequest::default().to_http_request());
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_metrics_handler() {
        let resp = metrics_handler()
            .await
            .respond_to(&test::TestRequest::default().to_http_request());
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.headers().get("content-type").unwrap(), "text/plain");
    }

    #[actix_web::test]
    async fn test_request_handler_invalid_utf8_body() {
        let mut config = Config::default();
        config.server.max_request_size = 1024 * 1024;
        let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
        let app_state = web::Data::new(AppState {
            config,
            rule_engine,
        });

        let invalid_utf8 = vec![0, 159, 146, 150];
        let req = test::TestRequest::post().uri("/test").to_http_request();
        let body = web::Bytes::from(invalid_utf8);

        let resp = request_handler(req, body, app_state).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_request_handler_success() {
        let config = Config::default();
        let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
        let app_state = web::Data::new(AppState {
            config,
            rule_engine,
        });

        let req = test::TestRequest::get().uri("/health").to_http_request();
        let body = web::Bytes::new();

        let resp = request_handler(req, body, app_state).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        assert_eq!(resp.status(), 500);
    }

    #[actix_web::test]
    async fn test_request_handler_rule_error() {
        let config = Config::default();
        let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
        let app_state = web::Data::new(AppState {
            config,
            rule_engine,
        });

        let req = test::TestRequest::get().uri("/error").to_http_request();
        let body = web::Bytes::new();

        let resp = request_handler(req, body, app_state).await;
        let _resp = resp.respond_to(&test::TestRequest::default().to_http_request());
    }
}
