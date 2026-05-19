# Use Actix-web for High-Performance Mocking

*   Status: accepted
*   Date: 2026-05-19

## Context and Problem Statement

Molock aims to be a high-performance mock server capable of handling tens of thousands of requests per second with minimal latency. We need a web framework that is fast, reliable, and supports asynchronous operations efficiently.

## Decision Drivers

- Extreme performance and low latency.
- Robust asynchronous support (Tokio-based).
- Type safety and developer experience.
- Ecosystem maturity and community support.

## Considered Options

- **Option 1: Rocket**: Easy to use but was historically slower and lagged in async support (though improved in recent versions).
- **Option 2: Axum**: Modern, built on Tower, and very popular, but Actix-web has a longer track record of winning performance benchmarks.
- **Option 3: Actix-web**: Consistently one of the fastest frameworks in benchmarks, with a powerful actor model (though we use it mostly for HTTP) and great async support.

## Decision Outcome

Chosen option: **Option 3: Actix-web**, because it provides the best performance profile for a high-throughput mock server. Its maturity and the fine-grained control it offers over the HTTP lifecycle align with Molock's goals of being "production-ready" for stress testing.

### Consequences

- **Good**: Near-native performance for HTTP handling.
- **Good**: Excellent integration with the `tracing` and `opentelemetry` ecosystems.
- **Bad**: Slightly steeper learning curve compared to more minimalist frameworks like Axum.
