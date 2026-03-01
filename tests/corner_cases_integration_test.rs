use actix_web::{test, web, App};
use molock::config::types::{Config, Endpoint, Response, ServerConfig};
use molock::rules::RuleEngine;
use molock::server::app::AppState;
use std::collections::HashMap;
use std::sync::Arc;

#[actix_web::test]
async fn test_integration_path_normalization() {
    let mut config = Config::default();
    config.server = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        workers: 1,
        max_request_size: 1024 * 1024,
    };
    
    config.endpoints = vec![Endpoint {
        name: "Test".to_string(),
        method: "GET".to_string(),
        path: "/api/users".to_string(),
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
    let app_state = web::Data::new(AppState {
        _config: config.clone(),
        rule_engine: rule_engine.clone(),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .default_service(web::to(molock::server::request_handler))
    )
    .await;

    // Test with duplicate slashes
    let req = test::TestRequest::get().uri("//api///users").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Test with trailing slash
    let req = test::TestRequest::get().uri("/api/users/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_integration_precedence() {
    let mut config = Config::default();
    config.endpoints = vec![
        Endpoint {
            name: "Wildcard".to_string(),
            method: "GET".to_string(),
            path: "/api/*".to_string(),
            stateful: false,
            state_key: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("Wildcard".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: false,
            }],
        },
        Endpoint {
            name: "Static".to_string(),
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            stateful: false,
            state_key: None,
            responses: vec![Response {
                status: 200,
                delay: None,
                body: Some("Static".to_string()),
                headers: HashMap::new(),
                condition: None,
                probability: None,
                default: false,
            }],
        },
    ];

    let rule_engine = Arc::new(RuleEngine::new(config.endpoints.clone()));
    let app_state = web::Data::new(AppState {
        _config: config.clone(),
        rule_engine,
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .default_service(web::to(molock::server::request_handler))
    )
    .await;

    // Should match Static because it's more specific than Wildcard
    let req = test::TestRequest::get().uri("/api/users").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    
    let body = test::read_body(resp).await;
    assert_eq!(body, web::Bytes::from_static(b"Static"));
}

#[actix_web::test]
async fn test_integration_invalid_utf8_body() {
    let config = Config::default();
    let rule_engine = Arc::new(RuleEngine::new(config.endpoints.clone()));
    let app_state = web::Data::new(AppState {
        _config: config,
        rule_engine,
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .default_service(web::to(molock::server::request_handler))
    )
    .await;

    // Invalid UTF-8 body
    let req = test::TestRequest::post()
        .uri("/any")
        .set_payload(vec![0, 159, 146, 150])
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
