# Contributing to Molock

First off, thank you for considering contributing to Molock! It's people like you who make Molock such a great tool for the high-performance testing community.

By contributing, you agree to abide by our standards and follow the development workflow outlined below.

## Project Values

Molock is built on three core pillars:
1.  **Extreme Performance**: Every microsecond counts. We aim for zero-allocation in the hot path.
2.  **Native Observability**: OpenTelemetry is a first-class citizen, not an afterthought.
3.  **Rigorous Quality**: We rely on strict TDD and high test coverage to ensure reliability.

## Environment Setup

To get started with Molock development, you will need:

- **Rust**: Version 1.70 or higher.
- **Docker & Docker Compose**: For running the observability stack (Jaeger, Prometheus, Grafana).
- **Make**: (Optional) For using the provided shortcuts in the `Makefile`.

### Local Stack

You can start the local development environment (OTel collector and dashboards) using:

```bash
docker-compose -f deployment/docker-compose.yml up -d
```

## Development Workflow: TDD Mandate

Molock strictly follows **Test-Driven Development (TDD)**. No feature should be implemented, and no bug should be fixed, without first having a failing test case that demonstrates the need for the change.

### The Red-Green-Refactor Cycle

1.  **Red**: Write a test that fails (e.g., in `tests/integration_test.rs` or a new unit test).
2.  **Green**: Write the minimum amount of code to make the test pass.
3.  **Refactor**: Clean up the code while ensuring the tests remain green.

When testing features that involve observability, ensure you use the `--features otel` flag.

```bash
cargo test --features otel
```

## Quality Standards

We maintain a "Zero Warning" policy. Your Pull Request will not be accepted if it contains lint warnings or fails quality gates.

### Local Verification

Before pushing your changes, run the following commands to ensure everything is in order:

- **Formatting**: Must adhere to standard Rust formatting.
  ```bash
  cargo fmt -- --check
  ```
- **Clippy**: Must pass without any warnings.
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```
- **Tests**: Run unit and integration tests.
  ```bash
  cargo test --all-features
  ```
- **Test Coverage**: We maintain a minimum of **80% line and branch coverage**.
  ```bash
  cargo tarpaulin --all-features --ignore-tests
  ```

### Pre-commit Hooks

We use `pre-commit` to automate these checks. To set it up:

1.  **Install pre-commit**: `pip install pre-commit`
2.  **Install the hooks**: `pre-commit install`

The hooks will run formatting, clippy, and our custom `find_code_smells.sh` script (which includes pedantic and nursery lints) on every commit.

### Integration Testing & Observability

Integration tests verify the end-to-end behavior of Molock, including its interaction with the observability stack (OTel collector, Jaeger, Prometheus).

1.  **Start the stack**:
    ```bash
    docker-compose -f deployment/docker-compose.yml up -d
    ```
2.  **Run integration tests**:
    ```bash
    cargo test --features otel --test integration_test
    ```
3.  **Validate Observability**:
    Use the provided script to verify that spans and metrics are correctly reaching the collector:
    ```bash
    ./tests/validate_observability.sh
    ```

### Security and Auditing

- **Dependency Audit**: Run `cargo audit` to check for known vulnerabilities in dependencies.
- **Secrets Detection**: We use `trufflehog` in CI to prevent accidental commits of secrets.
- **SLSA**: We follow SLSA (Supply-chain Levels for Software Artifacts) guidelines to ensure the integrity of our builds.

## Task Management

We use **beads (bd)** for internal task tracking.

1.  **Understand the workflow**: Run `bd prime` to see the full workflow context and available commands.
2.  **Find work**: Run `bd ready` to see available tasks.
3.  **Claim a task**: Run `bd update <id> --claim`.
4.  **Complete a task**: When your work is finished, run `bd close <id>`.

## Coding Conventions

### Error Handling

We follow a strict separation between application and library errors:
- **Application Logic**: Use `anyhow` for high-level error context and propagation.
- **Library/Core Logic**: Use `thiserror` to define structured, domain-specific error types.

Always provide meaningful context when propagating errors:
```rust
use anyhow::Context;
let config = ConfigLoader::from_file(&path).with_context(|| "Failed to load configuration")?;
```

### Observability (OpenTelemetry)

Every new feature or endpoint must be observable.
- **Spans**: Wrap significant operations in `tracing::span!`.
- **Logs**: Use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`) for structured logging.
- **Metrics**: Register and update relevant metrics (e.g., request counters, latency histograms) in `src/telemetry/metrics.rs`.

### Performance

- **Zero-Allocation**: Avoid unnecessary heap allocations in the hot path (request matching and response generation).
- **Async/Await**: Use non-blocking I/O and avoid holding mutexes across `.await` points.

## Session Completion

When ending a work session, you MUST:
1.  **File issues** for any remaining work.
2.  **Run quality gates** (tests, linter, coverage).
3.  **Update issue status** in `bd`.
4.  **Push to remote**: Ensure your branch is pushed and up to date with origin.

## Branch and Commit Protocol

> [!CAUTION]
> **No Direct Commits to `main`**: Direct commits or pushes to the `main` branch are strictly prohibited and will be blocked by branch protection rules. All changes must go through a feature branch and a Pull Request.

- **Feature Branches**: Use descriptive names like `feature/your-feature-name` or `fix/issue-description`.
- **Conventional Commits**: We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification for our commit messages (e.g., `feat: add regex matching`, `fix: resolve memory leak in tracer`).

## Pull Request Process

1.  Ensure all tests pass and there are no clippy warnings.
2.  Update the documentation (including ADRs if architectural choices were made).
3.  Link your PR to the relevant issue (e.g., `Closes #123`).
4.  Once the PR is merged, the associated task should be marked as closed in `bd`.

## Security

If you discover a security vulnerability, please do **not** open a public issue. Instead, follow the instructions in our `SECURITY.md` (to be implemented) or contact the maintainers directly.

---
*Molock Team - 2026*
