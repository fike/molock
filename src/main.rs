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

mod config;
mod rules;
mod server;
mod telemetry;
mod utils;

use crate::config::ConfigLoader;
use crate::rules::RuleEngine;
use crate::server::run_server;
use crate::telemetry::{init_telemetry, shutdown_telemetry};
use crate::utils::shutdown_signal;
use anyhow::Context;
use arc_swap::ArcSwap;
use clap::Parser;
use std::path::PathBuf;
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

    let config = ConfigLoader::from_file(&args.config)
        .with_context(|| format!("Failed to load config from {:?}", args.config))?;

    init_telemetry(&config.telemetry).await?;

    let rule_engine = Arc::new(RuleEngine::new(config.endpoints.clone()));
    let rule_engine_swap = Arc::new(ArcSwap::from(rule_engine.clone()));

    if args.hot_reload {
        start_hot_reload(&args.config, rule_engine_swap.clone()).await?;
    }

    let server = run_server(config, rule_engine).await?;

    info!("Molock server is running");
    info!("Press Ctrl+C to shutdown");

    let server_handle = server.handle();
    tokio::select! {
        _ = server => {
            info!("Server stopped");
        }
        _ = shutdown_signal() => {
            info!("Shutdown signal received");
            server_handle.stop(true).await;
            info!("Server shutdown complete");
        }
    }

    shutdown_telemetry().await;

    Ok(())
}

#[cfg(feature = "hot-reload")]
async fn start_hot_reload(
    config_path: &PathBuf,
    rule_engine_swap: Arc<ArcSwap<RuleEngine>>,
) -> anyhow::Result<()> {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;

    watcher.watch(config_path, RecursiveMode::NonRecursive)?;

    let config_path = config_path.clone();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv() {
            match event {
                notify::Event {
                    kind: notify::EventKind::Modify(_),
                    paths,
                    ..
                } => {
                    if paths.iter().any(|p| p == &config_path) {
                        info!("Configuration file modified, reloading...");
                        match ConfigLoader::from_file(&config_path) {
                            Ok(new_config) => {
                                let new_engine = Arc::new(RuleEngine::new(new_config.endpoints));
                                rule_engine_swap.store(new_engine);
                                info!("Configuration reloaded successfully");
                            }
                            Err(e) => {
                                tracing::error!("Failed to reload configuration: {}", e);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    Ok(())
}

#[cfg(not(feature = "hot-reload"))]
async fn start_hot_reload(
    _config_path: &PathBuf,
    _rule_engine_swap: Arc<ArcSwap<RuleEngine>>,
) -> anyhow::Result<()> {
    info!("Hot reload feature is not enabled");
    Ok(())
}
