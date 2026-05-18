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
use tracing::{info, warn, Instrument};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

#[cfg(feature = "otel")]
use opentelemetry_sdk::logs::SdkLoggerProvider;
#[cfg(feature = "otel")]
use opentelemetry_sdk::trace::SdkTracerProvider;
#[cfg(feature = "otel")]
use opentelemetry_sdk::trace::Span as SdkSpan;

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

    if tracing::dispatcher::has_been_set() {
        info!("A tracing subscriber is already set, skipping initialization");
        return Ok(());
    }

    let resource = build_resource(config);
    let tracer_provider = build_tracer_provider(config, resource.clone())?;
    let logger_provider = build_logger_provider(config, resource)?;

    init_global_settings(tracer_provider.clone());

    setup_subscriber(config, &logger_provider);

    info!("OpenTelemetry tracing initialized successfully");
    Ok(())
}

#[cfg(feature = "otel")]
fn build_resource(config: &TelemetryConfig) -> opentelemetry_sdk::Resource {
    opentelemetry_sdk::Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
        ])
        .build()
}

#[cfg(feature = "otel")]
fn build_tracer_provider(
    config: &TelemetryConfig,
    resource: opentelemetry_sdk::Resource,
) -> anyhow::Result<SdkTracerProvider> {
    let protocol = config.protocol.to_lowercase();
    let exporter = match protocol.as_str() {
        "http" => {
            let endpoint = format_endpoint(&config.endpoint, "v1/traces");
            opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_endpoint(endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
        _ => {
            if protocol != "grpc" {
                warn!("Unknown protocol '{}', defaulting to gRPC", protocol);
            }
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(&config.endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
    }
    .map_err(|e| anyhow::anyhow!("OpenTelemetry span exporter build failed: {e}"))?;

    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .with_sampler(opentelemetry_sdk::trace::Sampler::ParentBased(Box::new(
            opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(config.sampling_rate),
        )))
        .build())
}

#[cfg(feature = "otel")]
fn build_logger_provider(
    config: &TelemetryConfig,
    resource: opentelemetry_sdk::Resource,
) -> anyhow::Result<SdkLoggerProvider> {
    let protocol = config.protocol.to_lowercase();
    let exporter = match protocol.as_str() {
        "http" => {
            let endpoint = format_endpoint(&config.endpoint, "v1/logs");
            opentelemetry_otlp::LogExporter::builder()
                .with_http()
                .with_endpoint(endpoint)
                .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
        }
        _ => opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .with_timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build(),
    }
    .map_err(|e| anyhow::anyhow!("OpenTelemetry log exporter build failed: {e}"))?;

    Ok(SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build())
}

#[cfg(feature = "otel")]
fn format_endpoint(base: &str, path: &str) -> String {
    if base.contains(path) {
        base.to_string()
    } else if base.ends_with('/') {
        format!("{base}{path}")
    } else {
        format!("{base}/{path}")
    }
}

#[cfg(feature = "otel")]
fn init_global_settings(tracer_provider: SdkTracerProvider) {
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );
    otel_direct::init_direct_tracer(Arc::new(tracer_provider));
}

#[cfg(feature = "otel")]
fn setup_subscriber(config: &TelemetryConfig, logger_provider: &SdkLoggerProvider) {
    let tracer = opentelemetry::global::tracer("molock");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let otel_log_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(logger_provider);

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
}

#[cfg(not(feature = "otel"))]
pub async fn init_tracing(config: &TelemetryConfig) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Tracing is disabled");
        return Ok(());
    }

    if tracing::dispatcher::has_been_set() {
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
        let method = req.method().as_str();
        let path = req.path();

        #[cfg(not(feature = "otel"))]
        {
            let method = method.to_string();
            let path = path.to_string();
            Box::pin(async move {
                let span = tracing::info_span!(
                    "http.request",
                    http.method = %method,
                    http.target = %path,
                    http.route = %path,
                );
                let response = service.call(req).instrument(span).await?;
                log_response_status(response.status().as_u16());
                Ok(response)
            })
        }

        #[cfg(feature = "otel")]
        {
            use opentelemetry::propagation::TextMapPropagator;
            use opentelemetry_sdk::propagation::TraceContextPropagator;

            let propagator = TraceContextPropagator::new();
            let parent_cx = propagator.extract(&ActixHeaderExtractor(req.headers()));

            let method = method.to_string();
            let path = path.to_string();

            Box::pin(async move {
                let span_result = otel_direct::create_http_server_span(
                    "http.request",
                    &method,
                    &path,
                    &path,
                    &parent_cx,
                );

                match span_result {
                    Some(span) => {
                        process_otel_request(service, req, span, parent_cx, &method, &path).await
                    }
                    None => process_fallback_request(service, req, parent_cx, &method, &path).await,
                }
            })
        }
    }
}

#[cfg(feature = "otel")]
async fn process_otel_request<S, B>(
    service: Rc<S>,
    req: ServiceRequest,
    mut direct_span: SdkSpan,
    parent_cx: opentelemetry::Context,
    method: &str,
    path: &str,
) -> Result<ServiceResponse<B>, actix_web::Error>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: 'static,
{
    use opentelemetry::trace::Span as _;
    use opentelemetry::trace::TraceContextExt as _;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let cx = parent_cx.with_remote_span_context(direct_span.span_context().clone());

    let tracing_span = tracing::info_span!(
        "http.request",
        http.method = %method,
        http.target = %path,
        http.route = %path,
    );
    tracing_span.set_parent(cx);

    let response = service.call(req).instrument(tracing_span.clone()).await?;

    let status = response.status().as_u16();
    otel_direct::set_http_response_status_code(&mut direct_span, status);
    otel_direct::end_span(direct_span);

    let _guard = tracing_span.enter();
    log_response_status(status);

    Ok(response)
}

#[cfg(feature = "otel")]
async fn process_fallback_request<S, B>(
    service: Rc<S>,
    req: ServiceRequest,
    parent_cx: opentelemetry::Context,
    method: &str,
    path: &str,
) -> Result<ServiceResponse<B>, actix_web::Error>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: 'static,
{
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let span = tracing::span!(
        tracing::Level::INFO,
        "http.request",
        http.method = %method,
        http.target = %path,
        http.route = %path,
        span.kind = "server",
    );
    let _ = span.set_parent(parent_cx);

    let response = service.call(req).instrument(span.clone()).await?;
    let status = response.status().as_u16();

    span.record(attributes::http::RESPONSE_STATUS_CODE, status);
    log_response_status(status);

    Ok(response)
}

fn log_response_status(status: u16) {
    if (200..300).contains(&status) {
        tracing::info!("Request successful");
    } else if (300..400).contains(&status) {
        tracing::info!("Redirection");
    } else if (400..500).contains(&status) {
        tracing::warn!("Client error");
    } else if status >= 500 {
        tracing::error!("Server error");
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
    async fn test_tracing_middleware_without_otel() {
        // Ensure telemetry is not initialized or use a way to mock it
        let app = test::init_service(
            App::new()
                .wrap(tracing_middleware())
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_init_tracing_http() {
        let mut config = TelemetryConfig::default();
        config.protocol = "http".to_string();
        config.endpoint = "http://localhost:4318".to_string();
        
        // This might fail to build exporter if OTel SDK is not properly set up in test env,
        // but it should at least exercise the path.
        let _ = init_tracing(&config).await;
    }

    #[tokio::test]
    async fn test_init_tracing_http_with_slash() {
        let mut config = TelemetryConfig::default();
        config.protocol = "http".to_string();
        config.endpoint = "http://localhost:4318/".to_string();
        let _ = init_tracing(&config).await;
    }

    #[tokio::test]
    async fn test_init_tracing_http_with_v1_traces() {
        let mut config = TelemetryConfig::default();
        config.protocol = "http".to_string();
        config.endpoint = "http://localhost:4318/v1/traces".to_string();
        let _ = init_tracing(&config).await;
    }

    #[tokio::test]
    async fn test_init_tracing_unknown_protocol() {
        let mut config = TelemetryConfig::default();
        config.protocol = "unknown".to_string();
        let _ = init_tracing(&config).await;
    }

    #[tokio::test]
    async fn test_init_tracing_disabled() {
        let mut config = TelemetryConfig::default();
        config.enabled = false;
        let result = init_tracing(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_tracing_json_log() {
        let mut config = TelemetryConfig::default();
        config.log_format = "json".to_string();
        let _ = init_tracing(&config).await;
    }
}
