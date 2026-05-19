// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::needless_for_each)]

use serde::Serialize;
use utoipa::OpenApi;
use utoipa::ToSchema;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Molock API",
        version = "0.1.0",
        description = "High-performance mock server for CI/CD pipelines and testing"
    ),
    paths(request_handler_path),
    components(schemas(HealthResponse, ErrorResponse))
)]
pub struct ApiDoc;

/// A placeholder function to provide `OpenAPI` documentation for the catch-all request handler.
#[utoipa::path(
    get,
    path = "/{any:.*}",
    responses(
        (status = 200, description = "Mock response - returns configured mock response"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[allow(dead_code)]
pub const fn request_handler_path() {}

#[derive(ToSchema, Serialize)]
pub struct HealthResponse {
    #[schema(example = "healthy")]
    pub status: String,
}

#[derive(ToSchema, Serialize)]
pub struct MetricsResponse {
    #[schema(example = "# HELP http_server_request_count ...")]
    pub metrics: String,
}

#[derive(ToSchema, Serialize)]
pub struct ErrorResponse {
    #[schema(example = "Internal server error")]
    pub error: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub request_id: String,
}
