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

use crate::config::TelemetryConfig;
use crate::telemetry::attributes;
use crate::telemetry::otel_direct;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::LocalBoxFuture;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;

use std::future::ready;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use tracing::{error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

/// Adapts actix-web's `HeaderMap` to the `opentelemetry::propagation::Extractor`
/// trait so that W3C `traceparent`/`tracestate` headers can be extracted from
/// incoming requests.  `opentelemetry_http::HeaderExtractor` expects `http::HeaderMap`
/// which is a different type from `actix_web`'s internal one.
#[cfg(feature = "otel")]
struct ActixHeaderExtractor<'a>(&'a actix_web::http::header::HeaderMap);

#[cfg(feature = "otel")]
impl opentelemetry::propagation::Extractor for ActixHeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

#[cfg(feature = "otel")]
pub async fn init_tracing(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Tracing is disabled");
        return Ok(());
    }

    info!(
        "Starting OpenTelemetry tracing initialization with endpoint: {}, protocol: {}",
        config.endpoint, config.protocol
    );

    // Debug logging
    if crate::telemetry::is_debug_enabled() {
        info!("[TELEMETRY DEBUG] Tracing initialization starting");
        info!(
            "[TELEMETRY DEBUG] Endpoint: {}, Protocol: {}, Sampling rate: {}",
            config.endpoint, config.protocol, config.sampling_rate
        );
    }

    // Check if a subscriber is already set
    use tracing::dispatcher::has_been_set;
    if has_been_set() {
        info!("A tracing subscriber is already set, skipping initialization");
        return Ok(());
    }

    // Create resource with service name and version
    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
        ])
        .build();

    // Configure OTLP exporter based on protocol
    let protocol = config.protocol.to_lowercase();

    // Debug logging for protocol selection
    if crate::telemetry::is_debug_enabled() {
        info!(
            "[TELEMETRY DEBUG] Selecting exporter for protocol: {}",
            protocol
        );
    }

    let exporter = match protocol.as_str() {
        "grpc" => {
            info!(
                "Configuring gRPC exporter for tracing with endpoint: {}",
                config.endpoint
            );
            if crate::telemetry::is_debug_enabled() {
                info!("[TELEMETRY DEBUG] Using gRPC (tonic) exporter");
            }
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(&config.endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
        "http" => {
            let endpoint = if config.endpoint.contains("/v1/traces") {
                config.endpoint.clone()
            } else if config.endpoint.ends_with("/") {
                format!("{}v1/traces", config.endpoint)
            } else {
                format!("{}/v1/traces", config.endpoint)
            };
            info!(
                "Configuring HTTP exporter for tracing with endpoint: {}",
                endpoint
            );
            if crate::telemetry::is_debug_enabled() {
                info!("[TELEMETRY DEBUG] Using HTTP exporter");
            }
            // For HTTP protocol
            opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_endpoint(&endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
        _ => {
            warn!("Unknown protocol '{}', defaulting to gRPC", protocol);
            if crate::telemetry::is_debug_enabled() {
                info!("[TELEMETRY DEBUG] Unknown protocol, defaulting to gRPC");
            }
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(&config.endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
    }
    .map_err(|e| {
        error!("Failed to build OpenTelemetry span exporter: {}", e);
        anyhow::anyhow!("OpenTelemetry span exporter build failed: {}", e)
    })?;

    // Create tracer provider with the exporter
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource.clone())
        .with_sampler(opentelemetry_sdk::trace::Sampler::ParentBased(Box::new(
            opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(config.sampling_rate),
        )))
        .build();

    // Set as global tracer provider
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    // Configure OTLP log exporter based on protocol
    let log_exporter = match protocol.as_str() {
        "grpc" => {
            info!(
                "Configuring gRPC exporter for logging with endpoint: {}",
                config.endpoint
            );
            opentelemetry_otlp::LogExporter::builder()
                .with_tonic()
                .with_endpoint(&config.endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
        "http" => {
            let endpoint = if config.endpoint.contains("/v1/logs") {
                config.endpoint.clone()
            } else if config.endpoint.ends_with("/") {
                format!("{}v1/logs", config.endpoint)
            } else {
                format!("{}/v1/logs", config.endpoint)
            };
            info!(
                "Configuring HTTP exporter for logging with endpoint: {}",
                endpoint
            );
            opentelemetry_otlp::LogExporter::builder()
                .with_http()
                .with_endpoint(&endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
        _ => opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build(),
    }
    .map_err(|e| {
        error!("Failed to build OpenTelemetry log exporter: {}", e);
        anyhow::anyhow!("OpenTelemetry log exporter build failed: {}", e)
    })?;

    // Create logger provider with the exporter
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource)
        .build();

    // Register W3C TraceContext propagator so incoming traceparent/tracestate headers
    // are extracted and outgoing requests can carry the context forward.
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    // Get a tracer from the global provider for tracing-opentelemetry
    let tracer = opentelemetry::global::tracer("molock");

    // Initialize direct OpenTelemetry tracer for precise attribute control
    otel_direct::init_direct_tracer(Arc::new(tracer_provider));

    // Initialize tracing subscriber with OpenTelemetry layers
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let otel_log_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);

    let subscriber = Registry::default()
        .with(tracing_subscriber::EnvFilter::new(&config.log_level))
        .with(telemetry_layer)
        .with(otel_log_layer);

    if config.log_format == "json" {
        let _ = subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .try_init();
    } else {
        let _ = subscriber.with(tracing_subscriber::fmt::layer()).try_init();
    }

    info!("OpenTelemetry tracing initialized successfully");
    Ok(())
}

#[cfg(not(feature = "otel"))]
pub async fn init_tracing(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Tracing is disabled");
        return Ok(());
    }

    info!("Starting basic tracing initialization (OpenTelemetry feature not enabled)");

    // Check if a subscriber is already set
    use tracing::dispatcher::has_been_set;
    if has_been_set() {
        info!("A tracing subscriber is already set, skipping initialization");
        return Ok(());
    }

    let subscriber =
        Registry::default().with(tracing_subscriber::EnvFilter::new(&config.log_level));

    if config.log_format == "json" {
        let _ = subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .try_init();
    } else {
        let _ = subscriber.with(tracing_subscriber::fmt::layer()).try_init();
    }

    info!("Basic tracing initialized successfully");
    Ok(())
}

pub fn tracing_middleware() -> TracingMiddleware {
    TracingMiddleware
}

pub struct TracingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = TracingMiddlewareService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TracingMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct TracingMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for TracingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        #[cfg(not(feature = "otel"))]
        {
            let method = req.method().to_string();
            let path = req.path().to_string();

            let span = tracing::info_span!(
                "http.request",
                http.method = %method,
                http.target = %path,
                http.route = %path,
            );

            Box::pin(async move {
                let _guard = span.enter();
                let response = service.call(req).await?;
                let status = response.status().as_u16();

                if (200..300).contains(&status) {
                    tracing::info!("Request successful");
                } else if (300..400).contains(&status) {
                    tracing::info!("Redirection");
                } else if (400..500).contains(&status) {
                    tracing::warn!("Client error");
                } else if status >= 500 {
                    tracing::error!("Server error");
                }

                Ok(response)
            })
        }

        #[cfg(feature = "otel")]
        {
            use opentelemetry::propagation::TextMapPropagator;
            use opentelemetry_sdk::propagation::TraceContextPropagator;

            let propagator = TraceContextPropagator::new();
            let parent_cx = propagator.extract(&ActixHeaderExtractor(req.headers()));

            let method = req.method().to_string();
            let path = req.path().to_string();

            Box::pin(async move {
                // Create span using direct OpenTelemetry API for precise control.
                // Pass the extracted parent context so traces from upstream callers
                // are correctly linked (distributed tracing across service boundaries).
                let (direct_span, cx) = match otel_direct::create_http_server_span(
                    "http.request".to_string(),
                    method.clone(),
                    path.clone(),
                    path.clone(),
                    &parent_cx,
                ) {
                    Some(span) => {
                        use opentelemetry::trace::Span as _;
                        use opentelemetry::trace::TraceContextExt as _;
                        // Use the span context to link the tracing span without consuming the direct span
                        let cx = parent_cx.with_remote_span_context(span.span_context().clone());
                        (Some(span), cx)
                    }
                    None => {
                        // Fallback: use a tracing span when the OTel SDK is not initialized.
                        // Still attempt to honour the upstream traceparent via
                        // tracing-opentelemetry's set_parent extension.
                        let span = tracing::span!(
                            tracing::Level::INFO,
                            "http.request",
                            http.method = %method,
                            http.target = %path,
                            http.route = %path,
                            span.kind = "server",
                        );

                        #[cfg(feature = "otel")]
                        {
                            use tracing_opentelemetry::OpenTelemetrySpanExt;
                            let _ = span.set_parent(parent_cx);
                        }

                        let _guard = span.enter();

                        let response = service.call(req).await?;
                        let status = response.status().as_u16();

                        span.record(attributes::http::RESPONSE_STATUS_CODE, status);

                        if (200..300).contains(&status) {
                            tracing::info!("Request successful");
                        } else if (300..400).contains(&status) {
                            tracing::info!("Redirection");
                        } else if (400..500).contains(&status) {
                            tracing::warn!("Client error");
                        } else if status >= 500 {
                            tracing::error!("Server error");
                        }

                        return Ok(response);
                    }
                };

                // Create a tracing span and link it to the OTel context
                let tracing_span = tracing::info_span!(
                    "http.request",
                    http.method = %method,
                    http.target = %path,
                    http.route = %path,
                );

                // Set the OTel context as parent of the tracing span
                use tracing_opentelemetry::OpenTelemetrySpanExt;
                let _ = tracing_span.set_parent(cx);

                // Enter the tracing span so logs inside are correlated
                let response = {
                    let _guard = tracing_span.enter();
                    service.call(req).await?
                };

                let status = response.status().as_u16();

                // Set HTTP response status code using direct OpenTelemetry API.
                if let Some(mut span) = direct_span {
                    otel_direct::set_http_response_status_code(&mut span, status);
                    otel_direct::end_span(span);
                }

                // Log using the tracing span (which now has OTel IDs)
                let _guard = tracing_span.enter();
                if (200..300).contains(&status) {
                    tracing::info!("Request successful");
                } else if (300..400).contains(&status) {
                    tracing::info!("Redirection");
                } else if (400..500).contains(&status) {
                    tracing::warn!("Client error");
                } else if status >= 500 {
                    tracing::error!("Server error");
                }

                Ok(response)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use actix_web::web;
    use actix_web::App;
    use actix_web::HttpResponse;

    #[actix_web::test]
    async fn test_tracing_middleware() {
        let app = test::init_service(App::new().wrap(tracing_middleware()).route(
            "/test",
            web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_different_methods() {
        let app = test::init_service(
            App::new()
                .wrap(tracing_middleware())
                .route(
                    "/test",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                )
                .route(
                    "/test",
                    web::post().to(|| async { HttpResponse::Created().finish() }),
                )
                .route(
                    "/test",
                    web::put().to(|| async { HttpResponse::Ok().finish() }),
                )
                .route(
                    "/test",
                    web::delete().to(|| async { HttpResponse::NoContent().finish() }),
                ),
        )
        .await;

        let get_req = test::TestRequest::get().uri("/test").to_request();
        let get_resp = test::call_service(&app, get_req).await;
        assert_eq!(get_resp.status(), 200);

        let post_req = test::TestRequest::post().uri("/test").to_request();
        let post_resp = test::call_service(&app, post_req).await;
        assert_eq!(post_resp.status(), 201);

        let put_req = test::TestRequest::put().uri("/test").to_request();
        let put_resp = test::call_service(&app, put_req).await;
        assert_eq!(put_resp.status(), 200);

        let delete_req = test::TestRequest::delete().uri("/test").to_request();
        let delete_resp = test::call_service(&app, delete_req).await;
        assert_eq!(delete_resp.status(), 204);
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_different_paths() {
        let app = test::init_service(
            App::new()
                .wrap(tracing_middleware())
                .route(
                    "/api/users",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                )
                .route(
                    "/api/users/{id}",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                )
                .route(
                    "/api/orders",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                ),
        )
        .await;

        let req1 = test::TestRequest::get().uri("/api/users").to_request();
        let resp1 = test::call_service(&app, req1).await;
        assert_eq!(resp1.status(), 200);

        let req2 = test::TestRequest::get().uri("/api/users/123").to_request();
        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 200);

        let req3 = test::TestRequest::get().uri("/api/orders").to_request();
        let resp3 = test::call_service(&app, req3).await;
        assert_eq!(resp3.status(), 200);
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_error_status() {
        let app = test::init_service(App::new().wrap(tracing_middleware()).route(
            "/error",
            web::get().to(|| async { HttpResponse::InternalServerError().finish() }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/error").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);
    }

    #[actix_web::test]
    async fn test_tracing_middleware_single_span_per_request() {
        let app = test::init_service(App::new().wrap(tracing_middleware()).route(
            "/test",
            web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_query_params() {
        let app = test::init_service(App::new().wrap(tracing_middleware()).route(
            "/test",
            web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/test?foo=bar").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_traceparent_header() {
        let app = test::init_service(App::new().wrap(tracing_middleware()).route(
            "/propagate",
            web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;

        // Valid W3C traceparent: version-traceId-parentId-flags
        let req = test::TestRequest::get()
            .uri("/propagate")
            .insert_header((
                "traceparent",
                "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
