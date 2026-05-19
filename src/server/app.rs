// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use crate::rules::RuleEngine;
use crate::server::openapi::ApiDoc;
use crate::telemetry::tracer::tracing_middleware;
use actix_web::dev::Server;
use actix_web::http::header;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use std::sync::Arc;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};

/// Runs the Molock HTTP server.
///
/// # Errors
///
/// Returns an error if the server fails to bind to the specified address or if worker startup fails.
pub async fn run_server(config: Config, rule_engine: Arc<RuleEngine>) -> anyhow::Result<Server> {
    let server_config = config.server.clone();
    let addr = format!("{}:{}", server_config.host, server_config.port);

    info!("Starting server on {addr}");
    info!("Server workers: {}", server_config.workers);
    info!("Max request size: {} bytes", server_config.max_request_size);

    let openapi = ApiDoc::openapi();
    let swagger_urls = vec![(Url::new("Molock API", "/api-docs/openapi.json"), openapi)];

    let server = HttpServer::new(move || {
        let app_state = web::Data::new(AppState {
            config: config.clone(),
            rule_engine: rule_engine.clone(),
        });

        App::new()
            .wrap(tracing_middleware())
            .app_data(app_state)
            .app_data(web::JsonConfig::default().limit(config.server.max_request_size))
            .service(web::resource("/health").to(crate::server::health_handler))
            .service(web::resource("/metrics").to(crate::server::metrics_handler))
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(swagger_urls.clone()))
            .service(web::resource("/api-docs/openapi.json").to(openapi_json_handler))
            .default_service(web::to(crate::server::request_handler))
    })
    .workers(server_config.workers)
    .bind(addr)?
    .run();

    Ok(server)
}

async fn openapi_json_handler() -> impl Responder {
    let openapi = ApiDoc::openapi();
    let json = serde_json::to_string(&openapi).unwrap();
    HttpResponse::Ok()
        .insert_header(header::ContentType::json())
        .body(json)
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub rule_engine: Arc<RuleEngine>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Endpoint, Response};
    use std::collections::HashMap;

    #[test]
    fn test_app_state() {
        let config = Config {
            endpoints: vec![Endpoint {
                name: "Test".to_string(),
                method: "GET".to_string(),
                path: "/test".to_string(),
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
            }],
            ..Config::default()
        };

        let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
        let app_state = AppState {
            config: config.clone(),
            rule_engine: rule_engine.clone(),
        };

        assert_eq!(app_state.config.endpoints.len(), 1);
        assert_eq!(app_state.config.endpoints[0].name, "Test");
    }

    #[actix_web::test]
    async fn test_openapi_json_handler() {
        let resp = openapi_json_handler()
            .await
            .respond_to(&actix_web::test::TestRequest::default().to_http_request());
        assert_eq!(resp.status(), 200);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }
}
