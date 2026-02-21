# Build stage
FROM rust:1.93 AS builder

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY config ./config
COPY tests ./tests

# Build for release
RUN cargo build --release --features otel

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r molock && useradd -r -g molock -m -d /app molock

# Create configuration directory
RUN mkdir -p /etc/molock/config && chown -R molock:molock /etc/molock

# Copy binary from builder
COPY --from=builder --chown=molock:molock /app/target/release/molock /usr/local/bin/molock

# Copy configuration
COPY --chown=molock:molock config /etc/molock/config

# Switch to non-root user
USER molock

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV MOLOCK_CONFIG_PATH=/etc/molock/config/molock-config.yaml

# Run the application
ENTRYPOINT ["molock"]
CMD ["--config", "/etc/molock/config/molock-config.yaml"]