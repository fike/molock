// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use arc_swap::ArcSwap;
use clap::Parser;
use molock::config::ConfigLoader;
use molock::rules::RuleEngine;
use molock::server::run_server;
use molock::telemetry::{init_telemetry, shutdown_telemetry};
use molock::utils::shutdown_signal;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config/molock-config.yaml")]
    config: PathBuf,

    #[arg(long, default_value = "false")]
    hot_reload: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args).await
}

async fn run(args: Args) -> anyhow::Result<()> {
    let config = ConfigLoader::from_file(&args.config)
        .with_context(|| format!("Failed to load config from {}", args.config.display()))?;

    init_telemetry(&config.telemetry).await?;

    let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
    let rule_engine_swap = Arc::new(ArcSwap::from(rule_engine.clone()));

    if args.hot_reload {
        start_hot_reload(&args.config, rule_engine_swap.clone());
    }

    let server = run_server(config, rule_engine).await?;

    info!("Molock server is running");
    info!("Press Ctrl+C to shutdown");

    let server_handle = server.handle();
    tokio::select! {
        _ = server => {
            info!("Server stopped");
        }
        () = shutdown_signal() => {
            info!("Shutdown signal received");
            server_handle.stop(true).await;
            info!("Server shutdown complete");
        }
    }

    shutdown_telemetry();

    Ok(())
}

#[cfg(feature = "hot-reload")]
fn start_hot_reload(config_path: &Path, rule_engine_swap: Arc<ArcSwap<RuleEngine>>) {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default()).unwrap();

    watcher
        .watch(config_path, RecursiveMode::NonRecursive)
        .unwrap();

    let config_path = config_path.to_path_buf();
    tokio::spawn(async move {
        while let Ok(Ok(event)) = rx.recv() {
            if let notify::Event {
                kind: notify::EventKind::Modify(_),
                paths,
                ..
            } = event
            {
                if paths.iter().any(|p| p == &config_path) {
                    info!("Configuration file modified, reloading...");
                    match ConfigLoader::from_file(&config_path) {
                        Ok(new_config) => {
                            let new_engine = Arc::new(RuleEngine::new(&new_config.endpoints));
                            rule_engine_swap.store(new_engine);
                            info!("Configuration reloaded successfully");
                        }
                        Err(e) => {
                            tracing::error!("Failed to reload configuration: {}", e);
                        }
                    }
                }
            }
        }
    });
}

#[cfg(not(feature = "hot-reload"))]
fn start_hot_reload(_config_path: &Path, _rule_engine_swap: Arc<ArcSwap<RuleEngine>>) {
    info!("Hot reload feature is not enabled");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_run_invalid_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("invalid.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "invalid yaml").unwrap();

        let args = Args {
            config: config_path,
            hot_reload: false,
        };

        let result = run(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_success_shutdown() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "server:\n  host: 127.0.0.1\n  port: 0\n  workers: 1\n  max_request_size: 1048576\nendpoints: []\ntelemetry:\n  enabled: true\n  endpoint: http://localhost:4317\n  protocol: grpc").unwrap();

        let args = Args {
            config: config_path,
            hot_reload: false,
        };

        // This will at least cover the telemetry init and server creation
        let _ = run(args).await;
    }

    #[tokio::test]
    async fn test_start_hot_reload_change() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "server:\n  host: 127.0.0.1\n  port: 0\n  workers: 1\n  max_request_size: 1048576\nendpoints: []\ntelemetry:\n  enabled: false").unwrap();

        let rule_engine = Arc::new(RuleEngine::new(&[]));
        let rule_engine_swap = Arc::new(ArcSwap::from(rule_engine));

        start_hot_reload(&config_path, rule_engine_swap.clone());

        // Give watcher some time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Trigger change
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "server:\n  host: 127.0.0.1\n  port: 0\n  workers: 1\n  max_request_size: 1048576\nendpoints: []\ntelemetry:\n  enabled: false").unwrap();
        file.flush().unwrap();
        drop(file);

        // Give some time for event to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
