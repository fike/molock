# Molock - AI Agent Guide

This guide provides instructions for AI agents contributing to the Molock project.

## Coding Conventions

### Formatting
- Always run `cargo fmt` before finalizing changes
- Use 4 spaces for indentation (rustfmt default)
- Maximum line length: 100 characters
- Trailing commas in multi-line structs and enums

### Linting
- Run `cargo clippy -- -D warnings` and fix all warnings
- Enable all default clippy lints
- Address performance suggestions from clippy

### Naming
- Use `snake_case` for variables, functions, and modules
- Use `PascalCase` for types (structs, enums, traits)
- Use `SCREAMING_SNAKE_CASE` for constants
- Prefer descriptive names over abbreviations

### Documentation
- Document all public items with `///` comments
- Include examples for complex functions
- Document error conditions and return values
- Use markdown in documentation comments

## Module Structure

The project follows this structure:
```
src/
├── config/     # Configuration loading and parsing
├── server/     # Actix web server setup
├── rules/      # Rule matching and execution
├── telemetry/  # OpenTelemetry integration
└── utils/      # Helper functions
```

### Adding New Modules
1. Create `mod.rs` file declaring the module
2. Add module to parent's `mod.rs`
3. Follow existing patterns in similar modules
4. Add tests in the same file or `tests/` directory

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

### Examples
```
feat(rules): add support for regex path matching
fix(config): handle missing configuration file gracefully
docs(telemetry): add OpenTelemetry setup guide
test(server): add integration tests for health endpoint
```

## Using the Makefile

### Common Tasks
```bash
# Build the project
make build

# Run tests
make test

# Check code quality
make lint
make fmt

# Run with coverage
make test-coverage

# Development
make dev
make run
```

### Docker Operations
```bash
# Build Docker image
make docker-build

# Run with docker-compose
make docker-run
```

## Common Development Tasks

### Adding a New Rule Type
1. Update `src/config/types.rs` to add new rule variant
2. Update `src/rules/matcher.rs` to handle matching logic
3. Update `src/rules/executor.rs` to implement execution
4. Add tests in respective modules
5. Update example configuration

### Modifying Configuration Schema
1. Update structs in `src/config/types.rs`
2. Update `src/config/loader.rs` for validation
3. Add serde attributes for deserialization
4. Update example configuration file
5. Add migration notes if breaking changes

### Extending Telemetry
1. Add new spans or metrics in `src/telemetry/`
2. Update telemetry initialization
3. Add configuration options if needed
4. Update documentation

### Adding New Endpoints
1. Add handler in `src/server/handlers.rs`
2. Register route in `src/server/app.rs`
3. Add tests for the endpoint
4. Update OpenAPI documentation if applicable

## Test-Driven Development (TDD) Guidelines

### TDD Workflow for All New Features
1. **Write failing tests first** before implementing any new functionality
2. **Implement minimal code** to make tests pass
3. **Refactor** while keeping tests green
4. **Repeat** for each new feature or bug fix

### Unit Tests (Mandatory)
- **REQUIRED**: Every public function must have unit tests
- **REQUIRED**: Every module must have comprehensive test coverage
- Place tests in the same file as the code being tested using `#[cfg(test)]`
- Mock external dependencies using appropriate mocking libraries
- Test all error conditions and edge cases
- Test both success and failure paths

### Integration Tests
- Place in `tests/` directory
- Use `#[actix_web::test]` for async tests
- Test end-to-end functionality
- Clean up resources after tests
- Test API contracts and integration points

### Test-First Development Requirements
- **NEW FEATURES**: Write tests before implementing functionality
- **BUG FIXES**: Write failing test that reproduces the bug before fixing
- **REFACTORING**: Ensure existing tests pass before and after refactoring
- **CODE REVIEW**: Reject PRs without adequate test coverage

### Coverage Requirements
- **MINIMUM**: >80% line coverage for all modules
- **MINIMUM**: >80% branch coverage for critical paths
- **TARGET**: >90% line coverage for new code
- Run `make test-coverage` to check coverage
- **REJECTION CRITERIA**: PRs with coverage below 80% will be rejected

### Test Discovery and Enforcement
- **AUTOMATIC CHECK**: Run `cargo test --lib` to verify all unit tests pass
- **TEST GAP ANALYSIS**: When modifying code, check for missing tests:
  - Use `grep -r "fn test_" src/` to find existing tests
  - Use `cargo tarpaulin --lib` to identify uncovered lines
  - Add tests for any untested functions discovered
- **LEGACY CODE**: When working with untested legacy code:
  1. First write tests for existing behavior
  2. Then make changes with confidence
  3. Ensure tests continue to pass

## Error Handling

### Application Errors
- Use `anyhow::Result` for application code
- Use `thiserror::Error` for library errors
- Convert to `actix_web::Error` for HTTP handlers
- Include context with `anyhow::Context`

### HTTP Errors
- Use appropriate HTTP status codes
- Include error details in response body
- Log errors with appropriate severity
- Don't expose internal details in production

## Performance Considerations

### Async Patterns
- Use `async` for I/O operations
- Avoid blocking in async contexts
- Use `tokio::spawn` for concurrent tasks
- Implement backpressure where needed

### Memory Management
- Use `Arc` for shared read-only data
- Use `Mutex` or `RwLock` for shared mutable data
- Avoid unnecessary cloning
- Use `Cow` for borrowed or owned data

### Database/Storage
- Use connection pooling
- Implement query caching where appropriate
- Batch operations when possible
- Monitor memory usage

## Security Best Practices

### Input Validation
- Validate all user input
- Sanitize configuration values
- Use type-safe parsing
- Implement rate limiting

### Dependencies
- Keep dependencies up to date
- Audit dependencies regularly
- Use minimal feature sets
- Monitor security advisories

### Deployment
- Run as non-root user
- Use secure defaults
- Implement health checks
- Monitor logs and metrics

## Observability Testing

### Running Observability Stack
```bash
# Start all observability services
cd deployment
docker-compose up -d

# Wait for services to be ready
sleep 10
```

### Available Observability Endpoints
| Service | URL | Description |
|---------|-----|-------------|
| **Molock** | http://localhost:8080 | Application server |
| **Health** | http://localhost:8080/health | Health check |
| **Metrics** | http://localhost:8080/metrics | Prometheus metrics |
| **OpenAPI** | http://localhost:8080/api-docs/openapi.json | OpenAPI spec |
| **Swagger UI** | http://localhost:8080/swagger-ui/ | Interactive API docs |
| **Jaeger** | http://localhost:16686 | Distributed tracing UI |
| **Prometheus** | http://localhost:9090 | Metrics storage & query |
| **OTel Collector** | http://localhost:8889/metrics | OTel metrics endpoint |

### Verifying Observability
```bash
# Check Jaeger for traces
curl -s "http://localhost:16686/api/services" | jq .

# Check Prometheus targets
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets'

# Generate test traffic
curl http://localhost:8080/health
curl http://localhost:8080/users/123
```

### Integration Tests with Observability
When running integration tests with Docker:
1. Start observability stack: `docker-compose -f deployment/docker-compose.yml up -d`
2. Wait for services: `sleep 15`
3. Run integration tests: `cargo test --test integration_test`
4. Verify traces in Jaeger: http://localhost:16686
5. Verify metrics in Prometheus: http://localhost:9090
6. Cleanup: `docker-compose -f deployment/docker-compose.yml down`

## References

### Official Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Actix-web Docs](https://actix.rs/docs/)
- [OpenTelemetry Rust](https://opentelemetry.io/docs/instrumentation/rust/)
- [Serde Documentation](https://serde.rs/)

### Internal References
- See `.ai/Skills.md` for technical skills
- See `.ai/CONTRIBUTING.md` for contribution guidelines
- See `.ai/TDD_GUIDE.md` for TDD enforcement guide
- See `.ai/SECURITY.md` for security best practices
- Check existing code for patterns and conventions

## Troubleshooting

### Common Issues
- **Compilation errors**: Run `cargo check` for details
- **Test failures**: Check test output and logs
- **Performance issues**: Use `cargo flamegraph` for profiling
- **Memory leaks**: Use `valgrind` or `heaptrack`

### Getting Help
1. Check existing documentation
2. Look at similar code in the codebase
3. Run tests to verify behavior
4. Ask for clarification if needed

## TDD Enforcement Protocol

### Automatic Test Discovery
When working on any module, AI agents MUST:
1. **Scan for untested functions**:
   ```bash
   # Find all functions without tests in current module
   grep -n "^\s*pub fn\|^\s*fn [a-z_]" src/path/to/module.rs | grep -v "test_" | grep -v "#\[test\]"
   ```
2. **Check test coverage**:
   ```bash
   # Run coverage analysis on specific module
   cargo tarpaulin --lib --src src/path/to/module.rs
   ```
3. **Add missing tests** before modifying any code

### Test-First Implementation Steps
For EVERY new feature or bug fix:
1. **Create failing test** that describes the desired behavior
2. **Run tests** to confirm they fail (RED)
3. **Implement minimal code** to make tests pass (GREEN)
4. **Refactor** while keeping tests green (REFACTOR)
5. **Verify coverage** meets 80%+ requirement

### Legacy Code Protocol
When encountering untested legacy code:
1. **DO NOT MODIFY** without first adding tests
2. **Write characterization tests** that capture current behavior
3. **Ensure tests pass** with existing implementation
4. **Only then** make changes with confidence

## Quality Checklist

Before submitting changes:
- [ ] **TDD FOLLOWED**: Tests written before implementation
- [ ] **ALL TESTS PASS**: `cargo test --lib` shows 0 failures
- [ ] **COVERAGE MET**: `make test-coverage` shows >=80% coverage
- [ ] **UNTESTED FUNCTIONS**: No new untested functions added
- [ ] **LEGACY COVERAGE**: Tests added for any modified legacy code
- [ ] Code compiles without errors
- [ ] Code follows formatting guidelines (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Documentation updated for public APIs
- [ ] Commit message follows conventions
- [ ] Changes are minimal and focused
- [ ] Backward compatibility considered
- [ ] Security implications reviewed

## Pre-commit Checklist

The pre-commit hook (`.git/hooks/pre-commit`) automatically runs:
1. **Formatting Check**: `cargo fmt -- --check`
2. **Linting**: `cargo clippy` (errors only)
3. **Unit Tests**: `cargo test -- --test-threads=1`
4. **Security Audit**: `cargo audit`
5. **Integration Tests**: `cargo test --test integration_test` (with Docker)
6. **Observability Checks**:
   - Jaeger traces available at http://localhost:16686
   - Prometheus metrics available at http://localhost:8889/metrics
   - OpenAPI docs available at http://localhost:8080/api-docs/openapi.json

### Skipping Pre-commit Hook
If needed, you can skip the pre-commit hook:
```bash
git commit --no-verify -m "your commit message"
```

### Running Checks Manually
```bash
# Full pre-commit checks
.git/hooks/pre-commit

# Just unit tests
cargo test -- --test-threads=1

# Integration tests with Docker
cd deployment && docker-compose up -d && sleep 15 && cargo test --test integration_test && docker-compose down

# Security audit
cargo audit

# Coverage check
make test-coverage
```