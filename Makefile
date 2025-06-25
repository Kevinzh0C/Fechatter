# Fechatter Local Build + Docker Deployment Makefile
# Production-ready build strategy

.PHONY: help build test docker-build docker-run docker-push clean deps check fmt clippy

# Configuration
IMAGE_NAME ?= fechatter
IMAGE_TAG ?= latest
REGISTRY ?=
COMPOSE_FILE ?= docker-compose.local.yml

# Colors for output
BLUE = \033[0;34m
GREEN = \033[0;32m
YELLOW = \033[1;33m
RED = \033[0;31m
NC = \033[0m # No Color

# Default target
.DEFAULT_GOAL := help

## Display available commands
help:
	@echo "$(BLUE)Fechatter Build Commands$(NC)"
	@echo "========================"
	@echo ""
	@echo "$(GREEN)Local Development:$(NC)"
	@echo "  deps            Install dependencies and setup environment"
	@echo "  check           Run cargo check for quick validation"
	@echo "  fmt             Format code with rustfmt"
	@echo "  clippy          Run clippy linter"
	@echo "  test            Run test suite"
	@echo ""
	@echo "$(GREEN)Build & Deploy:$(NC)"
	@echo "  build           Build all services locally (optimized)"
	@echo "  docker-build    Build Docker image with pre-built binaries"
	@echo "  docker-run      Run full stack with Docker Compose"
	@echo "  docker-push     Build and push to registry"
	@echo ""
	@echo "$(GREEN)Management:$(NC)"
	@echo "  clean           Clean build artifacts and Docker resources"
	@echo "  clean-all       Deep clean including dependencies"
	@echo "  logs            Show application logs"
	@echo "  status          Show service status"
	@echo ""
	@echo "$(YELLOW)Environment Variables:$(NC)"
	@echo "  IMAGE_NAME      Docker image name (default: fechatter)"
	@echo "  IMAGE_TAG       Docker image tag (default: latest)"
	@echo "  REGISTRY        Docker registry URL (optional)"
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make build                    # Build locally"
	@echo "  make docker-build             # Build Docker image"
	@echo "  make docker-run               # Start full stack"
	@echo "  REGISTRY=hub.docker.com/user make docker-push"

## Install dependencies and setup development environment
deps:
	@echo "$(BLUE)[DEPS]$(NC) Installing dependencies..."
	@if ! command -v cargo > /dev/null 2>&1; then \
		echo "$(RED)[ERROR]$(NC) Rust not found. Please install Rust first: https://rustup.rs/"; \
		exit 1; \
	fi
	@if command -v yarn > /dev/null 2>&1; then \
		echo "$(BLUE)[DEPS]$(NC) Installing frontend dependencies..."; \
		cd fechatter_frontend && yarn install; \
	else \
		echo "$(YELLOW)[WARN]$(NC) Yarn not found, skipping frontend dependencies"; \
	fi
	@echo "$(GREEN)[SUCCESS]$(NC) Dependencies installed"

## Run cargo check for quick validation
check:
	@echo "$(BLUE)[CHECK]$(NC) Running cargo check..."
	@cargo check --workspace
	@echo "$(GREEN)[SUCCESS]$(NC) Cargo check completed"

## Format code with rustfmt
fmt:
	@echo "$(BLUE)[FMT]$(NC) Formatting code..."
	@cargo fmt --all
	@echo "$(GREEN)[SUCCESS]$(NC) Code formatted"

## Run clippy linter
clippy:
	@echo "$(BLUE)[CLIPPY]$(NC) Running clippy..."
	@cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo "$(GREEN)[SUCCESS]$(NC) Clippy check completed"

## Run test suite
test:
	@echo "$(BLUE)[TEST]$(NC) Running test suite..."
	@cargo test --workspace --all-features
	@echo "$(GREEN)[SUCCESS]$(NC) Tests completed"

## Build all services locally with optimizations
build:
	@echo "$(BLUE)[BUILD]$(NC) Building all services locally..."
	@chmod +x scripts/build.sh
	@./scripts/build.sh build
	@echo "$(GREEN)[SUCCESS]$(NC) Local build completed"

## Build Docker image with pre-built binaries
docker-build: build
	@echo "$(BLUE)[DOCKER]$(NC) Building Docker image..."
	@chmod +x scripts/docker-build.sh
	@./scripts/docker-build.sh build \
		--image-name $(IMAGE_NAME) \
		--image-tag $(IMAGE_TAG) \
		$(if $(REGISTRY),--registry $(REGISTRY))
	@echo "$(GREEN)[SUCCESS]$(NC) Docker image built: $(IMAGE_NAME):$(IMAGE_TAG)"

## Run full stack with Docker Compose
docker-run:
	@echo "$(BLUE)[DOCKER]$(NC) Starting full stack..."
	@if [ ! -f "target/build/fechatter-server" ]; then \
		echo "$(YELLOW)[WARN]$(NC) Binaries not found, building first..."; \
		$(MAKE) build; \
	fi
	@docker-compose -f $(COMPOSE_FILE) up -d
	@echo "$(GREEN)[SUCCESS]$(NC) Services started"
	@echo ""
	@echo "$(BLUE)[INFO]$(NC) Service URLs:"
	@echo "  Main API:      http://localhost:8080"
	@echo "  Gateway:       http://localhost:3000"
	@echo "  Analytics:     http://localhost:3001"
	@echo "  Bot Service:   http://localhost:3002"
	@echo "  Notifications: http://localhost:3003"
	@echo "  Database:      postgresql://fechatter:fechatter_password@localhost:5432/fechatter"
	@echo "  Redis:         redis://:redis_password@localhost:6379"
	@echo "  Search:        http://localhost:7700"

## Build and push Docker image to registry
docker-push: 
	@if [ -z "$(REGISTRY)" ]; then \
		echo "$(RED)[ERROR]$(NC) REGISTRY environment variable is required"; \
		echo "$(YELLOW)[HELP]$(NC) Usage: REGISTRY=your-registry.com make docker-push"; \
		exit 1; \
	fi
	@echo "$(BLUE)[DOCKER]$(NC) Building and pushing to $(REGISTRY)..."
	@chmod +x scripts/docker-build.sh
	@./scripts/docker-build.sh push \
		--image-name $(IMAGE_NAME) \
		--image-tag $(IMAGE_TAG) \
		--registry $(REGISTRY)
	@echo "$(GREEN)[SUCCESS]$(NC) Image pushed to $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)"

## Show application logs
logs:
	@echo "$(BLUE)[LOGS]$(NC) Showing application logs..."
	@docker-compose -f $(COMPOSE_FILE) logs -f fechatter

## Show service status
status:
	@echo "$(BLUE)[STATUS]$(NC) Service status:"
	@docker-compose -f $(COMPOSE_FILE) ps

## Stop all services
stop:
	@echo "$(BLUE)[STOP]$(NC) Stopping services..."
	@docker-compose -f $(COMPOSE_FILE) down
	@echo "$(GREEN)[SUCCESS]$(NC) Services stopped"

## Clean build artifacts and Docker resources
clean:
	@echo "$(BLUE)[CLEAN]$(NC) Cleaning build artifacts..."
	@rm -rf target/build target/release
	@docker-compose -f $(COMPOSE_FILE) down -v --remove-orphans 2>/dev/null || true
	@docker image prune -f
	@echo "$(GREEN)[SUCCESS]$(NC) Cleanup completed"

## Deep clean including dependencies
clean-all: clean
	@echo "$(BLUE)[CLEAN-ALL]$(NC) Deep cleaning..."
	@cargo clean
	@docker system prune -af --volumes
	@if [ -d "fechatter_frontend/node_modules" ]; then \
		rm -rf fechatter_frontend/node_modules; \
	fi
	@echo "$(GREEN)[SUCCESS]$(NC) Deep cleanup completed"

## Development workflow: format, check, test, build
dev: fmt clippy test build
	@echo "$(GREEN)[SUCCESS]$(NC) Development workflow completed"

## CI/CD workflow: check, test, build, docker-build
ci: check test build docker-build
	@echo "$(GREEN)[SUCCESS]$(NC) CI/CD workflow completed"

## Quick start: setup deps and run
quick-start: deps build docker-run
	@echo "$(GREEN)[SUCCESS]$(NC) Quick start completed!"
	@echo "$(BLUE)[INFO]$(NC) Your Fechatter instance is running at http://localhost:8080" 