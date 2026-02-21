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

pub async fn run_server(config: Config, rule_engine: Arc<RuleEngine>) -> anyhow::Result<Server> {
    let server_config = config.server.clone();
    let addr = format!("{}:{}", server_config.host, server_config.port);

    info!("Starting server on {}", addr);
    info!("Server workers: {}", server_config.workers);
    info!("Max request size: {} bytes", server_config.max_request_size);

    let openapi = ApiDoc::openapi();
    let swagger_urls = vec![(Url::new("Molock API", "/api-docs/openapi.json"), openapi)];

    let server = HttpServer::new(move || {
        let app_state = web::Data::new(AppState {
            _config: config.clone(),
            rule_engine: rule_engine.clone(),
        });

        App::new()
            .wrap(tracing_middleware())
            .app_data(app_state.clone())
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
    pub _config: Config,
    pub rule_engine: Arc<RuleEngine>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Endpoint, Response};
    use std::collections::HashMap;

    #[test]
    fn test_app_state() {
        let mut config = Config::default();
        config.endpoints = vec![Endpoint {
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
        }];

        let rule_engine = Arc::new(RuleEngine::new(config.endpoints.clone()));
        let app_state = AppState {
            _config: config.clone(),
            rule_engine: rule_engine.clone(),
        };

        assert_eq!(app_state._config.endpoints.len(), 1);
        assert_eq!(app_state._config.endpoints[0].name, "Test");
    }
}
