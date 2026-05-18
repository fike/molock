use molock::config::types::{Config, ServerConfig};
use molock::rules::RuleEngine;
use molock::server::run_server;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_server_startup_and_shutdown() {
    let mut config = Config::default();
    config.server = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        workers: 1,
        max_request_size: 1024 * 1024,
    };

    let rule_engine = Arc::new(RuleEngine::new(config.endpoints.clone()));

    let server = run_server(config, rule_engine)
        .await
        .expect("Failed to start server");
    let server_handle = server.handle();

    // Spawn server in background
    let server_task = tokio::spawn(server);

    // Shutdown the server immediately
    server_handle.stop(true).await;

    // Ensure the task finishes
    let _ = timeout(Duration::from_secs(1), server_task).await;
}
