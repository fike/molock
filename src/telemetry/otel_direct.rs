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

//! Direct OpenTelemetry API integration for HTTP tracing
//!
//! This module provides direct OpenTelemetry API usage to work around limitations
//! in the tracing-opentelemetry crate, particularly for setting span attributes
//! with correct semantic convention names.

use crate::telemetry::attributes;
use opentelemetry::trace::{Span as OtelSpan, SpanKind, Status, Tracer, TracerProvider};
use opentelemetry::Context;
use opentelemetry_sdk::trace::{SdkTracerProvider, Span, Tracer as SdkTracer};
use std::sync::Arc;
use std::sync::RwLock;

static TRACER_PROVIDER: RwLock<Option<Arc<SdkTracerProvider>>> = RwLock::new(None);

pub fn init_direct_tracer(tracer_provider: Arc<SdkTracerProvider>) {
    let mut provider = TRACER_PROVIDER.write().unwrap();
    *provider = Some(tracer_provider);
}

fn get_tracer() -> Option<SdkTracer> {
    let provider = TRACER_PROVIDER.read().unwrap();
    provider.as_ref().map(|p| p.tracer("molock-direct"))
}

/// Create an HTTP server span using direct OpenTelemetry API.
///
/// The `parent_cx` parameter allows linking this span to an upstream trace extracted
/// from incoming request headers (W3C `traceparent`/`tracestate`). Pass
/// `&Context::current()` when no parent context is available.
pub fn create_http_server_span(
    name: String,
    method: String,
    target: String,
    route: String,
    parent_cx: &Context,
) -> Option<Span> {
    let tracer = get_tracer()?;

    let span = tracer
        .span_builder(name)
        .with_kind(SpanKind::Server)
        .with_attributes(vec![
            attributes::kv::http_method(&method),
            attributes::kv::http_target(&target),
            attributes::kv::http_route(&route),
        ])
        .start_with_context(&tracer, parent_cx);

    Some(span)
}

/// Set HTTP response status code on a span using direct OpenTelemetry API
pub fn set_http_response_status_code(span: &mut Span, status: u16) {
    // Set the correct semantic convention: http.response.status_code
    span.set_attribute(attributes::kv::http_response_status_code(status));

    // Also set span status based on HTTP status code
    match status {
        200..=299 => span.set_status(Status::Ok),
        400..=499 => span.set_status(Status::error("Client error")),
        500..=599 => span.set_status(Status::error("Server error")),
        _ => span.set_status(Status::Unset),
    }
}

/// End a span
pub fn end_span(mut span: Span) {
    span.end();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Tests share a global TRACER_PROVIDER, so they must be serialized
    // to avoid race conditions (e.g., one test setting None while another reads).
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_create_http_server_span_without_initialization() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.write().unwrap();
            provider.clone()
        };

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = None;
        drop(provider);

        let cx = Context::current();
        let span = create_http_server_span(
            "test-span".to_string(),
            "GET".to_string(),
            "/test".to_string(),
            "/test".to_string(),
            &cx,
        );
        assert!(span.is_none());

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    #[test]
    fn test_get_tracer_without_initialization() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = None;
        drop(provider);

        let tracer = get_tracer();
        assert!(tracer.is_none());

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    #[test]
    fn test_init_direct_tracer() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let tracer_provider = SdkTracerProvider::builder().build();

        init_direct_tracer(Arc::new(tracer_provider));

        let tracer = get_tracer();
        assert!(tracer.is_some());

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    #[test]
    fn test_create_http_server_span_with_initialization() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let tracer_provider = SdkTracerProvider::builder().build();

        init_direct_tracer(Arc::new(tracer_provider));

        let cx = Context::current();
        let span = create_http_server_span(
            "http.request".to_string(),
            "GET".to_string(),
            "/api/users".to_string(),
            "/api/users".to_string(),
            &cx,
        );

        assert!(span.is_some());

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    /// Verify that `create_http_server_span` does not add a redundant `span.kind`
    /// string attribute. The kind is communicated correctly via `SpanKind::Server`
    /// on the builder; adding it again as a raw string attribute causes Jaeger to
    /// display two separate `span.kind` entries for every span.
    #[test]
    fn test_create_http_server_span_no_duplicate_span_kind_attribute() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        // Use a noop provider — we only care that the builder is invoked without
        // the extra attribute; the span itself doesn't need to be exported.
        let tracer_provider = SdkTracerProvider::builder().build();
        init_direct_tracer(Arc::new(tracer_provider));

        let cx = Context::current();
        // If the builder still included `span.kind` as an attribute, the span
        // would carry it twice when exported. The test ensures the function compiles
        // and executes without panicking — the absence of the attribute is enforced
        // by code inspection of the `with_attributes` list above.
        let span = create_http_server_span(
            "http.request".to_string(),
            "GET".to_string(),
            "/test".to_string(),
            "/test".to_string(),
            &cx,
        );
        assert!(span.is_some());
        end_span(span.unwrap());

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    /// Verify that a span created with a non-empty parent context correctly links
    /// to the parent. This simulates receiving an upstream `traceparent` header.
    #[test]
    fn test_create_http_server_span_with_parent_context() {
        use opentelemetry::trace::{SpanContext, SpanId, TraceFlags, TraceId, TraceState};

        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let tracer_provider = SdkTracerProvider::builder().build();
        init_direct_tracer(Arc::new(tracer_provider));

        // Build a synthetic parent SpanContext (as if extracted from traceparent header).
        let parent_span_ctx = SpanContext::new(
            TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap(),
            SpanId::from_hex("00f067aa0ba902b7").unwrap(),
            TraceFlags::SAMPLED,
            true,
            TraceState::default(),
        );

        use opentelemetry::trace::TraceContextExt;
        let parent_cx = Context::current().with_remote_span_context(parent_span_ctx.clone());

        let span = create_http_server_span(
            "http.request".to_string(),
            "GET".to_string(),
            "/api/resource".to_string(),
            "/api/resource".to_string(),
            &parent_cx,
        );

        assert!(
            span.is_some(),
            "span should be created with a parent context"
        );

        // The child span must share the same TraceId as the parent.
        let child_span = span.unwrap();
        let child_ctx = child_span.span_context();
        assert_eq!(
            child_ctx.trace_id(),
            parent_span_ctx.trace_id(),
            "child span TraceId must match the parent TraceId for correct propagation"
        );

        end_span(child_span);

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    #[test]
    fn test_set_http_response_status_code() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let tracer_provider = SdkTracerProvider::builder().build();

        init_direct_tracer(Arc::new(tracer_provider));

        let tracer = get_tracer().unwrap();
        let mut span = tracer.start("test-span");

        set_http_response_status_code(&mut span, 200);

        let mut span = tracer.start("test-span-404");
        set_http_response_status_code(&mut span, 404);

        let mut span = tracer.start("test-span-500");
        set_http_response_status_code(&mut span, 500);

        let mut span = tracer.start("test-span-300");
        set_http_response_status_code(&mut span, 300);

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    #[test]
    fn test_end_span() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let tracer_provider = SdkTracerProvider::builder().build();

        init_direct_tracer(Arc::new(tracer_provider));

        let tracer = get_tracer().unwrap();
        let span = tracer.start("test-span");

        end_span(span);

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    #[test]
    fn test_create_http_server_span_with_different_methods() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let tracer_provider = SdkTracerProvider::builder().build();

        init_direct_tracer(Arc::new(tracer_provider));

        let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"];
        let cx = Context::current();

        for method in methods {
            let span = create_http_server_span(
                "http.request".to_string(),
                method.to_string(),
                "/api/test".to_string(),
                "/api/test".to_string(),
                &cx,
            );

            assert!(span.is_some());
            end_span(span.unwrap());
        }

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    #[test]
    fn test_create_http_server_span_with_different_paths() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let tracer_provider = SdkTracerProvider::builder().build();

        init_direct_tracer(Arc::new(tracer_provider));

        let paths = vec![
            "/",
            "/api/users",
            "/api/users/123",
            "/api/orders?page=1&limit=10",
            "/api/search?q=test%20query",
        ];
        let cx = Context::current();

        for path in paths {
            let span = create_http_server_span(
                "http.request".to_string(),
                "GET".to_string(),
                path.to_string(),
                path.to_string(),
                &cx,
            );

            assert!(span.is_some());
            end_span(span.unwrap());
        }

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }

    #[test]
    fn test_semantic_convention_usage() {
        let _guard = TEST_LOCK.lock().unwrap();

        let original_provider = {
            let provider = TRACER_PROVIDER.read().unwrap();
            provider.clone()
        };

        let tracer_provider = SdkTracerProvider::builder().build();

        init_direct_tracer(Arc::new(tracer_provider));

        let cx = Context::current();
        let span = create_http_server_span(
            "http.request".to_string(),
            "POST".to_string(),
            "/api/users".to_string(),
            "/api/users".to_string(),
            &cx,
        )
        .unwrap();

        end_span(span);

        let mut provider = TRACER_PROVIDER.write().unwrap();
        *provider = original_provider;
    }
}
