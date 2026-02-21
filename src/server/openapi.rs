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

use serde::Serialize;
use utoipa::OpenApi;
use utoipa::ToSchema;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Molock API",
        description = "High-performance mock server API for CI/CD pipelines and testing",
        version = "0.1.0",
        contact(
            name = "Molock Team",
            url = "https://github.com/your-org/molock"
        ),
        license(
            name = "MIT OR Apache-2.0",
            url = "https://github.com/your-org/molock/blob/main/LICENSE"
        )
    ),
    paths(
        super::handlers::health_handler,
        super::handlers::metrics_handler,
        request_handler_path
    ),
    components(
        schemas(
            HealthResponse,
            MetricsResponse,
            ErrorResponse
        )
    ),
    tags(
        (name = "System", description = "System endpoints"),
        (name = "Mock", description = "Mock endpoint handlers")
    )
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    path = "/{path:.*}",
    tag = "Mock",
    responses(
        (status = 200, description = "Mock response - returns configured mock response"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[allow(dead_code)]
pub fn request_handler_path() {}

#[derive(ToSchema, Serialize)]
pub struct HealthResponse {
    #[schema(example = "healthy")]
    pub status: String,
    #[schema(example = "molock")]
    pub service: String,
    #[schema(example = "2026-01-01T00:00:00Z")]
    pub timestamp: String,
}

#[derive(ToSchema, Serialize)]
pub struct MetricsResponse {
    #[schema(example = "# Metrics endpoint - use OpenTelemetry metrics instead")]
    pub message: String,
}

#[derive(ToSchema, Serialize)]
pub struct ErrorResponse {
    #[schema(example = "Internal server error")]
    pub error: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub request_id: String,
}
