# ICE DATA FORGE — Agent Guide

**Status:** Active development (Phase 0: Scaffolding done). The spec at
`plan IceData Forge.txt` is the **vector** (direction), not dogma.

## Project

Hockey analytics platform: automated NHL player analysis via public NHL API,
advanced metrics (xG, Corsi, Fenwick), AI-generated scouting reports.

## Stack (decided)

| Layer | Technology |
|-------|-----------|
| Backend | Rust 1.75+, Axum, Tokio — `crates/ice-data-api` |
| Database | PostgreSQL 15+, sqlx — `crates/ice-data-db` |
| AI | OpenAI API (GPT-4) |
| MCP | Rust on `/mcp/sse` — `crates/ice-data-mcp` |
| Chatbot | Python + aiogram — `chatbot/` |
| Dashboard | React 18+ + TypeScript + Tailwind + Recharts — `dashboard/` |
| Workflows | n8n (self-hosted) — exports in `n8n/` |
| Infra | Docker + Compose → Docker Hub |

## Repository structure

```
Cargo.toml              # workspace root — 5 member crates
crates/
  ice-data-core/        # shared types, config, errors (no deps on siblings)
  ice-data-nhl/         # NHL API client (rate-limited, retry)
  ice-data-db/          # sqlx pool, queries, migrations
  ice-data-api/         # Axum REST API + analytics engine
  ice-data-mcp/         # MCP Server (SSE transport)
chatbot/                # Python + aiogram
dashboard/              # React 18+ (Vite, Tailwind, Recharts)
n8n/                    # 4 n8n workflow exports (JSON)
scripts/                # deploy.ps1, deploy.sh, production.env.example
.github/workflows/      # CI/CD: deploy.yml (build → push → ssh deploy)
migrations/             # symlink / reference for sqlx
```

## Architecture

- **Monorepo + Cargo workspace** — all Rust crates in one repo
- **API-First:** REST (Axum, `/v1`) + MCP (SSE/JSON-RPC 2.0)
- **Analytics engine** is inlined into `ice-data-api` crate (not a separate service)
- **NHL API:** `https://api-web.nhle.com/v1` — 60 req/min, exponential backoff
- **Auth:** JWT (REST) / API Key (MCP)
- **Cache:** in-memory (Tokio Mutex), TTL 300s fast / 7d stable / 24h young players
- **Config:** all via env vars, `AppConfig::from_env()` in `ice-data-core`

## Build order (phases completed and next)

| Phase | Status | What |
|-------|--------|------|
| 0 | ✅ | Workspace, Docker Compose, configs, Makefile, structure |
| 1 | ✅ | DB migrations, NHL client, data ingestion |
| 2 | ✅ | Axum REST endpoints, middleware, cache |
| 3 | ✅ | Analytics engine (inlined in API — done with Phase 2) |
| 4 | ✅ | MCP Server — SSE, 6 tools (before AI) |
| 5 | ✅ | OpenAI GPT-4 integration |
| 6 | ✅ | n8n workflows — 4 pipeline exports, webhook endpoints |
| 7 | ✅ | Chatbot — Python + aiogram, 6 commands, inline keyboards |
| 8 | ✅ | Dashboard — React 18+ + Vite + Tailwind + Recharts, 6 pages |
| 9 | ✅ | Docker Hub deployment |

## Developer commands

```sh
make check    # cargo check --workspace
make lint     # cargo clippy --workspace -- -D warnings
make test     # cargo test --workspace
make db-up    # docker compose up -d postgres n8n
make db-down  # docker compose down
make dev      # cargo run --bin ice-data-api
make dev-mcp  # cargo run --bin ice-data-mcp
make dashboard-dev  # vite dev server (port 5173)
```

## Key spec references (use as compass, not cage)

- **§7** — DB schema: `players`, `player_season_stats`, `ai_analysis`, `news_events`, `system_config`, `audit_log`
- **§6** — REST API: base `/v1`, endpoints `/players`, `/teams`, `/news`, `/analytics`
- **§4.3** — MCP tools: 6 JSON-RPC tool schemas
- **§4.4** — n8n: 4 workflow pipelines
- **§4.1** — Config template (env vars)
