.PHONY: help dev test build clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

dev: ## Start development environment
	docker-compose up -d
	@echo "Fechatter is running at http://localhost:8080"

test: ## Run all tests
	cargo test --workspace
	cd fechatter_frontend && npm test

build: ## Build for production
	docker-compose -f docker-compose.prod.yml build

clean: ## Clean up containers and volumes
	docker-compose down -v

logs: ## Show service logs
	docker-compose logs -f

restart: ## Restart all services
	docker-compose restart

setup: ## Initial setup
	cp .env.example .env
	@echo "Please edit .env with your configuration" 