# Makefile for Raugupatis Log development and deployment

.PHONY: help build run test clean dev docker-build docker-run setup lint format

help: ## Show this help message
	@echo "Raugupatis Log - Available commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

setup: ## Set up development environment
	@./scripts/setup-dev.sh

build: ## Build the application
	cargo build

build-release: ## Build the application in release mode
	cargo build --release

run: ## Run the application in development mode
	cargo run

dev: ## Run the application with hot-reload (requires cargo-watch)
	cargo watch -x run

test: ## Run all tests
	cargo test

test-watch: ## Run tests with hot-reload
	cargo watch -x test

lint: ## Run clippy linter
	cargo clippy --all-targets -- -D warnings

format: ## Format code with rustfmt
	cargo fmt

format-check: ## Check code formatting
	cargo fmt -- --check

clean: ## Clean build artifacts
	cargo clean
	docker image prune -f

docker-build: ## Build Docker image
	docker build -t raugupatis-log:latest .

docker-run: ## Run Docker container
	docker run -p 3000:3000 -v $$(pwd)/data:/app/data raugupatis-log:latest

docker-compose-up: ## Start application with Docker Compose
	docker-compose up --build

docker-compose-down: ## Stop Docker Compose services
	docker-compose down

docker-compose-logs: ## View Docker Compose logs
	docker-compose logs -f

deploy: ## Deploy using the deployment script
	@./scripts/deploy.sh

check-health: ## Check application health
	@curl -f http://localhost:3000/health && echo " ✅ Health check passed" || echo " ❌ Health check failed"

# Development workflow
dev-full: clean build test lint format ## Run full development workflow

# CI/CD workflow  
ci: build test lint format-check ## Run CI pipeline