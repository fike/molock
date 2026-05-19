use actix_web::{test, web, App};
use molock::config::types::{Config, Endpoint, Response};
use molock::rules::RuleEngine;
use molock::server::app::AppState;
use std::collections::HashMap;
use std::sync::Arc;

#[actix_web::test]
async fn test_integration_path_specificity() {
    let config = Config {
        endpoints: vec![
            Endpoint {
                name: "user_details".to_string(),
                method: "GET".to_string(),
                path: "/users/:id".to_string(),
                stateful: false,
                state_key: None,
                responses: vec![Response {
                    status: 200,
                    delay: None,
                    body: Some("User Detail".to_string()),
                    headers: HashMap::new(),
                    condition: None,
                    probability: None,
                    default: true,
                }],
                schema: None,
                schema_file: None,
                path_regex: None,
                headers_regex: None,
                query_regex: None,
            },
            Endpoint {
                name: "user_me".to_string(),
                method: "GET".to_string(),
                path: "/users/me".to_string(),
                stateful: false,
                state_key: None,
                responses: vec![Response {
                    status: 200,
                    delay: None,
                    body: Some("User Me".to_string()),
                    headers: HashMap::new(),
                    condition: None,
                    probability: None,
                    default: true,
                }],
                schema: None,
                schema_file: None,
                path_regex: None,
                headers_regex: None,
                query_regex: None,
            },
        ],
        ..Config::default()
    };

    let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
    let app_state = web::Data::new(AppState {
        config: config.clone(),
        rule_engine: rule_engine.clone(),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .default_service(web::to(molock::server::request_handler)),
    )
    .await;

    // "/users/me" is more specific than "/users/:id", so it should win
    let req = test::TestRequest::get().uri("/users/me").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    assert_eq!(body, web::Bytes::from_static(b"User Me"));

    // "/users/123" should match "/users/:id"
    let req = test::TestRequest::get().uri("/users/123").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    assert_eq!(body, web::Bytes::from_static(b"User Detail"));
}

#[actix_web::test]
async fn test_integration_wildcard_fallback() {
    let config = Config {
        endpoints: vec![
            Endpoint {
                name: "api_v1".to_string(),
                method: "GET".to_string(),
                path: "/api/v1/*".to_string(),
                stateful: false,
                state_key: None,
                responses: vec![Response {
                    status: 200,
                    delay: None,
                    body: Some("API V1".to_string()),
                    headers: HashMap::new(),
                    condition: None,
                    probability: None,
                    default: true,
                }],
                schema: None,
                schema_file: None,
                path_regex: None,
                headers_regex: None,
                query_regex: None,
            },
            Endpoint {
                name: "catch_all".to_string(),
                method: "GET".to_string(),
                path: "/*".to_string(),
                stateful: false,
                state_key: None,
                responses: vec![Response {
                    status: 404,
                    delay: None,
                    body: Some("Not Found".to_string()),
                    headers: HashMap::new(),
                    condition: None,
                    probability: None,
                    default: true,
                }],
                schema: None,
                schema_file: None,
                path_regex: None,
                headers_regex: None,
                query_regex: None,
            },
        ],
        ..Config::default()
    };

    let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
    let app_state = web::Data::new(AppState {
        config: config.clone(),
        rule_engine: rule_engine.clone(),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .default_service(web::to(molock::server::request_handler)),
    )
    .await;

    // Should match "/api/v1/*"
    let req = test::TestRequest::get()
        .uri("/api/v1/resource")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Should match "/*"
    let req = test::TestRequest::get().uri("/other").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_integration_invalid_utf8_body() {
    let config = Config::default();
    let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
    let app_state = web::Data::new(AppState {
        config,
        rule_engine,
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .default_service(web::to(molock::server::request_handler)),
    )
    .await;

    // Send invalid UTF-8 body
    let req = test::TestRequest::post()
        .uri("/any")
        .set_payload(vec![0, 159, 146, 150])
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
