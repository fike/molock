.PHONY: build test test-coverage run docker-build docker-run docker-down clean lint fmt help benchmark benchmark-all benchmark-health benchmark-users benchmark-delay benchmark-post benchmark-docker observability-up observability-down

help:
	@echo "Available targets:"
	@echo "  build        - Build release binary"
	@echo "  test         - Run all tests"
	@echo "  test-coverage - Run tests with coverage report"
	@echo "  run          - Run the application"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run with docker-compose"
	@echo "  docker-down  - Stop docker-compose services"
	@echo "  clean        - Clean build artifacts"
	@echo "  lint         - Run clippy linter"
	@echo "  fmt          - Format code with rustfmt"
	@echo "  check        - Check code without building"
	@echo "  dev          - Run in development mode"
	@echo "  benchmark    - Run comprehensive benchmark suite"
	@echo "  benchmark-all - Run all benchmark scenarios"
	@echo "  benchmark-health - Benchmark health endpoints"
	@echo "  benchmark-users - Benchmark user endpoints"
	@echo "  benchmark-delay - Benchmark delayed responses"
	@echo "  benchmark-post - Benchmark POST endpoints"
	@echo "  benchmark-docker - Run benchmarks against Docker Compose stack"
	@echo "  observability-up - Start observability stack only"
	@echo "  observability-down - Stop observability stack"

build:
	cargo build --release

test:
	cargo test --features otel

test-coverage:
	cargo tarpaulin --features otel --out Html --skip-clean --ignore-tests

run:
	cargo run --release -- --config config/molock-config.yaml

dev:
	cargo run --features otel -- --config config/molock-config.yaml

benchmark-run:
	cargo run --release -- --config config/benchmark-config.yaml

docker-build:
	@echo "Building Molock Docker image (multi-stage build)..."
	docker build -f deployment/Dockerfile -t molock .

docker-run:
	docker-compose -f deployment/docker-compose.yml up

clean:
	cargo clean
	rm -rf target/ coverage/

lint:
	cargo clippy --features otel -- -D warnings

fmt:
	cargo fmt --all

check:
	cargo check --features otel

bench:
	cargo bench

doc:
	cargo doc --no-deps --open

benchmark: benchmark-all

benchmark-all:
	@echo "Running comprehensive benchmark suite..."
	@chmod +x benchmarks/benchmark.sh
	@./benchmarks/benchmark.sh

benchmark-health:
	@echo "Running health endpoint benchmarks..."
	@chmod +x benchmarks/health_benchmark.sh
	@./benchmarks/health_benchmark.sh

benchmark-users:
	@echo "Running user endpoint benchmarks..."
	@chmod +x benchmarks/users_benchmark.sh
	@./benchmarks/users_benchmark.sh

benchmark-delay:
	@echo "Running delayed response benchmarks..."
	@chmod +x benchmarks/delay_benchmark.sh
	@./benchmarks/delay_benchmark.sh

benchmark-post:
	@echo "Running POST endpoint benchmarks..."
	@chmod +x benchmarks/post_benchmark.sh
	@./benchmarks/post_benchmark.sh

benchmark-docker:
	@echo "Starting observability stack and Molock in Docker..."
	@echo "Building Docker image and starting services..."
	docker-compose -f deployment/docker-compose.yml up -d --build
	@echo "Waiting for services to start (10 seconds)..."
	@sleep 10
	@echo "Running benchmarks against Dockerized Molock..."
	@chmod +x benchmarks/benchmark.sh
	./benchmarks/benchmark.sh --docker
	@echo ""
	@echo "Benchmarks completed!"
	@echo "Observability UIs:"
	@echo "- Jaeger (traces): http://localhost:16686"
	@echo "- Grafana (metrics): http://localhost:3000 (admin/admin)"
	@echo "- Prometheus: http://localhost:9090"
	@echo ""
	@echo "To stop services: make docker-down"
	@echo "To view logs: docker-compose -f deployment/docker-compose.yml logs -f"

docker-down:
	docker-compose -f deployment/docker-compose.yml down

observability-up:
	@echo "Starting observability stack only..."
	@echo "Note: Molock service will not be started"
	docker-compose -f deployment/docker-compose.yml up -d otel-collector jaeger prometheus grafana
	@echo "Observability stack started:"
	@echo "- Jaeger (traces): http://localhost:16686"
	@echo "- Grafana (metrics): http://localhost:3000 (admin/admin)"
	@echo "- Prometheus: http://localhost:9090"
	@echo ""
	@echo "Now start Molock with: make docker-run"
	@echo "Or run benchmarks with: make benchmark-docker"

observability-down:
	docker-compose -f deployment/docker-compose.yml down