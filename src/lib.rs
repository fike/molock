// SPDX-FileCopyrightText: 2026 Molock Team
// SPDX-License-Identifier: Apache-2.0

//! # Molock - High-Performance Mock Server
//!
//! Molock is a production-ready mock server designed for high-throughput environments,
//! CI/CD pipelines, and stress testing. Built in Rust with Actix-web, it provides
//! configurable and observable mock endpoints with native OpenTelemetry integration.
//!
//! ## Core Pillars
//!
//! 1.  🚀 **Extreme Performance**: Leveraging Rust and Actix-web for zero-allocation hot paths and maximum throughput.
//! 2.  🔭 **Native Observability**: OpenTelemetry is a first-class citizen, providing out-of-the-box traces, metrics, and logs.
//! 3.  🛡️ **Rigorous Quality**: Developed using strict Test-Driven Development (TDD) with high coverage and SLSA security standards.
//!
//! ## Example Usage
//!
//! While Molock is typically used as a standalone binary configured via YAML, you can also
//! interact with its core components programmatically:
//!
//! ```rust,no_run
//! use molock::config::ConfigLoader;
//! use molock::rules::RuleEngine;
//! use molock::server::run_server;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration
//!     let config = ConfigLoader::from_file("config/molock-config.yaml")?;
//!
//!     // Initialize the rule engine
//!     let rule_engine = Arc::new(RuleEngine::new(&config.endpoints));
//!
//!     // Start the server
//!     let server = run_server(config, rule_engine).await?;
//!
//!     // server.await?;
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod rules;
pub mod server;
pub mod telemetry;
pub mod utils;
