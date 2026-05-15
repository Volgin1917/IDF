.PHONY: build check lint test clean db-up db-down dev dev-mcp

build:
	cargo build --workspace

check:
	cargo check --workspace

lint:
	cargo clippy --workspace -- -D warnings

test:
	cargo test --workspace

clean:
	cargo clean
	rm -rf chatbot/.venv chatbot/__pycache__ dashboard/node_modules dashboard/dist

db-up:
	docker compose up -d postgres n8n

stack-up:
	docker compose up -d

stack-down:
	docker compose down

db-down:
	docker compose down

db-reset:
	docker compose down -v
	docker compose up -d postgres
	@sleep 3
	cargo run --bin ice-data-migrate

dev:
	cargo run --bin ice-data-api

dev-mcp:
	cargo run --bin ice-data-mcp

bot-install:
	pip install -r chatbot/requirements.txt

bot-dev:
	cd chatbot && python -m src.main

dashboard-dev:
	cd dashboard && npx vite --host

dashboard-build:
	cd dashboard && npx vite build

docker-build:
	docker build -t ice-data-forge:latest .

docker-up:
	docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

docker-deploy:
	pwsh -NoProfile -File scripts/deploy.ps1

docker-deploy-sh:
	bash scripts/deploy.sh
