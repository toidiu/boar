.DEFAULT_GOAL := help

help: ## Show this help
	@awk 'BEGIN{FS=":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---- host (ns_s1..ns_c2 chain via scripts/virt_*.sh; needs sudo) ----

test: ## cargo test
	cargo test

build: ## cargo build
	cargo build

run: build ## Run boar on the host (sets up netns; takes ARGS="-d 50mb -c 5")
	sudo ./target/debug/boar $(ARGS)

# ---- docker (two containers on a bridge; no sudo) ----

docker-build: ## Build the boar image (cargo cache mounts kick in)
	docker compose build

docker-up: ## One-shot: bring server up, run client, tear down
	docker compose up --build --abort-on-container-exit

docker-server: ## Start boar-server detached, for iterative client runs
	docker compose up -d --build boar-server

docker-run: ## Run client with extra args, e.g. make docker-run ARGS="-d 50mb -c 5"
	docker compose run --rm boar-client boar --mode docker $(ARGS)

docker-down: ## Stop and remove containers + network
	docker compose down

.PHONY: help test build run docker-build docker-up docker-server docker-run docker-down
