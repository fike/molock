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

1.  **Red**: Write a test that fails (or a benchmark that shows a bottleneck).
2.  **Green**: Write the minimum amount of code to make the test pass.
3.  **Refactor**: Clean up the code while ensuring the tests remain green.

Run all tests frequently:

```bash
cargo test
```

## Quality Standards

We maintain a "Zero Warning" policy. Your Pull Request will not be accepted if it contains lint warnings or fails quality gates.

- **Clippy**: Must pass without any warnings.
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```
- **Formatting**: Must adhere to standard Rust formatting.
  ```bash
  cargo fmt -- --check
  ```
- **Test Coverage**: We maintain a minimum of **80% line and branch coverage**. Use `cargo tarpaulin` (if installed) to verify coverage.

## Task Management

We use **beads (bd)** for internal task tracking.

1.  **Find work**: Run `bd ready` to see available tasks.
2.  **Claim a task**: Run `bd update <id> --claim`.
3.  **Complete a task**: When your work is finished, run `bd close <id>`.

## Branch and Commit Protocol

- **No Direct Commits to `main`**: All changes must go through a feature branch and a Pull Request.
- **Branch Naming**: Use descriptive names like `feature/your-feature-name` or `fix/issue-description`.
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
