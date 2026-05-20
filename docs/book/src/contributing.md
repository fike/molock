# Contributing

Thank you for your interest in contributing to Molock!

The source of truth for our contribution guidelines is the [CONTRIBUTING.md](https://github.com/fike/molock/blob/main/CONTRIBUTING.md) file in the root of the repository.

## Key Principles

- **TDD (Test-Driven Development)**: All features and bug fixes must have accompanying tests.
- **Zero-Warning Policy**: Code must pass `clippy` and `fmt` without any warnings.
- **Branch Protocol**: Never commit directly to `main`. Always use feature branches.
- **Observability**: New features must include appropriate telemetry (spans, metrics, logs).

## Development Workflow

1.  Find an issue in our tracker (`bd ready`).
2.  Claim the issue (`bd update <id> --claim`).
3.  Create a feature branch.
4.  Implement your changes following the TDD cycle.
5.  Submit a Pull Request.
