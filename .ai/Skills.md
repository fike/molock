# Molock - Required Skills

This document outlines the core technical skills needed to work on the Molock project effectively.

## Rust Fundamentals

### Core Concepts
- **Ownership & Borrowing**: Understand Rust's ownership model, borrowing rules, and lifetimes
- **Async Programming**: Familiarity with async/await, futures, and Tokio runtime
- **Error Handling**: Using `anyhow` for application errors and `thiserror` for library errors
- **Traits & Generics**: Effective use of trait bounds and generic programming
- **Smart Pointers**: Understanding `Arc`, `Mutex`, `RwLock` for shared state

### Key Crates
- **Serde**: For serialization/deserialization of configuration and data
- **Tokio**: Async runtime for high-performance networking
- **Tracing**: Structured logging and instrumentation
- **Actix-web**: Web framework for building HTTP servers

## Actix Web Framework

### Core Components
- **Actors & Addresses**: Understanding Actix actor system (though less used in Actix-web 4.x)
- **Middleware**: Implementing custom middleware for logging, telemetry, etc.
- **Extractors**: Creating and using request extractors
- **Responders**: Implementing custom response types
- **Application State**: Managing shared application state safely

### Best Practices
- Use `web::Data<T>` for shared state
- Implement proper error handling with `actix_web::error`
- Use async handlers for I/O-bound operations
- Follow Actix-web's security recommendations

## OpenTelemetry Integration

### Tracing
- **Spans**: Creating and managing spans for request tracing
- **Attributes**: Adding contextual information to spans
- **Span Events**: Recording important events within spans
- **Context Propagation**: Passing trace context between services

### Metrics
- **Counters**: For tracking request counts, errors, etc.
- **Histograms**: For measuring latency distributions
- **Gauges**: For tracking current values (like active connections)
- **Meters & Instruments**: Setting up metric collection

### Exporters
- **OTLP**: OpenTelemetry Protocol for exporting to collectors
- **Jaeger**: For trace visualization
- **Prometheus**: For metrics collection (optional)

## Configuration Management

### Serde Patterns
- Derive `Serialize` and `Deserialize` for configuration structs
- Use `#[serde(default)]` for optional fields
- Implement custom deserializers for complex types
- Validate configuration on load

### Hot Reload
- Use `notify` crate for file system watching
- Implement atomic configuration updates with `ArcSwap`
- Handle configuration errors gracefully

## Test-Driven Development & Coverage

### TDD Methodology
- **Red-Green-Refactor**: Write failing tests first, then implement, then refactor
- **Test-First Development**: Never write production code without a failing test
- **Behavior-Driven Development**: Write tests that describe expected behavior
- **Regression Prevention**: Tests protect against future breaking changes

### Unit Testing (Mandatory)
- **Test Isolation**: Test individual functions and modules in isolation
- **Mocking**: Use appropriate mocking for external dependencies
- **Edge Cases**: Test boundary conditions, error cases, and invalid inputs
- **Test Organization**: Use `#[cfg(test)]` for test-only code in same file

### Integration Testing
- **End-to-End**: Test the full HTTP stack with `reqwest`
- **Configuration**: Test configuration loading, validation, and hot reload
- **Telemetry**: Test observability integration and signal correctness
- **State Management**: Test stateful behavior and persistence

### Test Automation & Tools
- **Tarpaulin**: For code coverage measurement and gap analysis
- **Criterion**: For benchmarking performance and detecting regressions
- **Test Discovery**: Use `cargo test --lib` to run all unit tests
- **Coverage Enforcement**: Minimum 80% line and branch coverage required

### TDD Workflow Skills
1. **Identify Test Gaps**: Use `cargo tarpaulin` to find untested code
2. **Write Failing Tests**: Create tests that describe desired behavior
3. **Implement Minimal Solution**: Write just enough code to pass tests
4. **Refactor Safely**: Improve code structure while tests remain green
5. **Verify Coverage**: Ensure new code meets 80%+ coverage requirement

## Containerization & Deployment

### Docker
- Multi-stage builds for small images
- Non-root user execution for security
- Proper layer caching for faster builds

### Docker Compose
- Service orchestration for local development
- Integration with OpenTelemetry stack
- Environment variable management

### Production Considerations
- Health checks and readiness probes
- Resource limits and requests
- Security context configuration

## Learning Resources

### Rust
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/)

### Actix-web
- [Actix-web Documentation](https://actix.rs/docs/)
- [Actix-web Examples](https://github.com/actix/examples)

### OpenTelemetry
- [OpenTelemetry Rust](https://opentelemetry.io/docs/instrumentation/rust/)
- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)

### Testing
- [The Rust Testing Guide](https://doc.rust-lang.org/rust-by-example/testing.html)
- [Mockall Documentation](https://docs.rs/mockall/latest/mockall/)

## Common Patterns in Molock

1. **Configuration Loading**: See `src/config/loader.rs`
2. **Rule Matching**: See `src/rules/matcher.rs`
3. **Response Generation**: See `src/rules/executor.rs`
4. **Telemetry Setup**: See `src/telemetry/`
5. **Error Handling**: Consistent use of `anyhow::Result` and `thiserror::Error`

## Code Review Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Coverage remains above 80%
- [ ] Follows existing patterns and conventions
- [ ] Includes appropriate documentation
- [ ] Handles errors gracefully
- [ ] Includes telemetry where appropriate
- [ ] Security considerations addressed