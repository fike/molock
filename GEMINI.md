# Molock Project Guidelines

This file contains the foundational mandates and project context for all AI agents working on Molock.

## Core Mandates
- **Tech Stack**: Molock is a high-performance Rust application built with `actix-web`.
- **Observability**: OpenTelemetry is a core requirement. Every new feature or endpoint must include appropriate spans, metrics, and logs.
- **Testing**: Test-Driven Development (TDD) is mandatory. Line and branch coverage must remain above 80%.
- **Error Handling**: Use `anyhow` for application-level errors and `thiserror` for library-level errors.

## Foundational Skills
### Rust & Actix-web
- Rigorous ownership and borrowing management.
- Async programming using `tokio` runtime.
- Shared state management via `web::Data<T>`.

### OpenTelemetry
- Use `tracing` for structured logging.
- Export telemetry via OTLP.

## Configuration
- Management via `serde` with support for hot-reloading (see `src/config/`).

---
For specific development workflows, TDD protocols, and maintenance scripts, activate the `core-engineering` skill:
`gemini activate-skill core-engineering`