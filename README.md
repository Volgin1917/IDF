<!-- Banner -->
<p align="center">
  <img src="banner.png" alt="IceData Forge Banner" width="100%" />
</p>

<!-- Badges -->
<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75-orange?logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/PostgreSQL-15-blue?logo=postgresql" alt="PostgreSQL" />
  <img src="https://img.shields.io/badge/React-18-61dafb?logo=react" alt="React" />
  <img src="https://img.shields.io/badge/Python-3.11-3776ab?logo=python" alt="Python" />
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License" />
  <img src="https://img.shields.io/badge/Docker-Ready-2496ed?logo=docker" alt="Docker" />
</p>

<!-- Quick Links -->
<p align="center">
  <a href="#quick-start">🚀 Быстрый старт</a> •
  <a href="#features">✨ Возможности</a> •
  <a href="#architecture">🏗️ Архитектура</a> •
  <a href="#компоненты">📦 Компоненты</a> •
  <a href="#deployment">🌐 Деплой</a> •
  <a href="#разработка">🛠️ Разработка</a>
</p>

---

# IceData Forge 🏒

**Hockey Analytics Platform** — автоматизированный анализ игроков НХЛ через public NHL API с продвинутыми метриками (xG, Corsi, Fenwick) и AI-генерацией скаутских отчётов.

> **Статус:** ✅ Все 9 фаз завершены. Production-ready.

---

## О проекте

IceData Forge — это full-stack платформа для хоккейной аналитики, которая объединяет данные NHL API, продвинутые статистические метрики и искусственный интеллект для генерации профессиональных скаутских отчётов.

**Для кого:**
- 🎯 Скауты и аналитики хоккейных клубов
- 📊 Журналисты и блогеры
- 🏒 Фанаты хоккея с глубоким интересом к статистике
- 🤖 Разработчики, использующие MCP для AI-агентов

**Что делает:**
- Автоматически синхронизирует данные игроков НХЛ (60 запросов/мин, rate-limited)
- Рассчитывает продвинутые метрики: Corsi%, Fenwick%, xG, P/60, G/60
- Генерирует AI-отчёты через GPT-4 (JSON mode, кэширование в БД)
- Предоставляет REST API + MCP Server + Telegram бот + React Dashboard
- Автоматизирует пайплайны через n8n (4 workflow)

---

## Features

| 🏒 **NHL API** | 📊 **Аналитика** | 🤖 **AI** |
|----------------|------------------|-----------|
| Rate-limited клиент (Semaphore) | Пер-игрок и пер-60 метрики | GPT-4 интеграция |
| Экспоненциальный backoff | Corsi% / Fenwick% / xG | JSON response format |
| 3 эндпоинта (player/team/news) | Лидеры по 7 категориям | Кэширование анализов |
| 32 команды, актуальные ростеры | Таймлайн статистики | 4 промпт-шаблона |

| 💬 **Telegram Bot** | 📱 **Dashboard** | 🔌 **MCP Server** |
|---------------------|------------------|-------------------|
| 6 команд (/start, /search, /player, /stats, /analyze, /compare) | 7 страниц (Overview, Players, Detail, Teams, Analytics, News, Compare) | 6 JSON-RPC инструментов |
| Inline клавиатуры | 3 типа графиков (Line, Bar, Radar) | SSE транспорт |
| Callback handlers | Тёмная тема | API Key auth |
| Search & compare | Поиск игроков | Инструменты: get_player, search_players, analyze_player, compare_players, get_team_roster, get_player_news |

| ⚙️ **n8n Workflows** | 🗄️ **Database** |
|----------------------|-----------------|
| 4 экспортированных workflow | PostgreSQL 15+ |
| NHL Sync (ежедневный) | 8 таблиц + индексы + триггеры |
| AI Analysis (по расписанию) | sqlx миграции |
| Player Monitoring | Views + seed данные |
| News Collection | Audit log |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         IceData Forge Architecture                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
│  │   Dashboard  │    │  Telegram    │    │     n8n      │               │
│  │   React SPA  │    │     Bot      │    │  Workflows   │               │
│  │   :5173/:80  │    │   Python     │    │   :5678      │               │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘               │
│         │                   │                   │                        │
│         └───────────────────┼───────────────────┘                        │
│                             │                                            │
│                             ▼                                            │
│              ┌──────────────────────────────┐                           │
│              │      REST API (Axum)         │                           │
│              │         Port 8080            │                           │
│              │  /v1/players, /teams, /news  │                           │
│              │  /v1/analytics, /ai, /webhook│                           │
│              └──────────────┬───────────────┘                           │
│                             │                                            │
│         ┌───────────────────┼───────────────────┐                       │
│         │                   │                   │                       │
│         ▼                   ▼                   ▼                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │ Analytics    │  │   AI         │  │   NHL        │                  │
│  │ Engine       │  │   Service    │  │   Client     │                  │
│  │ (inlined)    │  │  (GPT-4)     │  │  (rate-limit)│                  │
│  └──────────────┘  └──────────────┘  └──────────────┘                  │
│                             │                                            │
│                             ▼                                            │
│              ┌──────────────────────────────┐                           │
│              │   PostgreSQL 15              │                           │
│              │   players, stats, ai_analysis│                           │
│              │   news_events, audit_log     │                           │
│              └──────────────────────────────┘                           │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────┐         │
│  │              MCP Server (Port 3001)                        │         │
│  │  SSE /mcp/sse  +  POST /mcp/message (JSON-RPC 2.0)        │         │
│  └────────────────────────────────────────────────────────────┘         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Tech Stack

| Layer | Technology | Version | Crate/Package |
|-------|------------|---------|---------------|
| **Backend** | Rust + Axum + Tokio | 1.75+ | `ice-data-api`, `ice-data-core`, `ice-data-db`, `ice-data-nhl`, `ice-data-mcp`, `ice-data-ai` |
| **Database** | PostgreSQL + sqlx | 15+ | `ice-data-db` |
| **AI** | OpenAI API (GPT-4) | — | `ice-data-ai` |
| **MCP** | Rust + SSE + JSON-RPC 2.0 | — | `ice-data-mcp` |
| **Chatbot** | Python + aiogram | 3.x | `chatbot/` |
| **Dashboard** | React + TypeScript + Vite + Tailwind + Recharts | 18+ | `dashboard/` |
| **Workflows** | n8n (self-hosted) | latest | `n8n/` |
| **Infra** | Docker + Compose | — | `docker-compose.yml`, `docker-compose.prod.yml` |
| **CI/CD** | GitHub Actions | — | `.github/workflows/deploy.yml` |

---

## Quick Start

### Предусловия

```bash
# Docker + Docker Compose
docker --version      # 24+
docker compose version # 2.20+

# (Опционально) Rust для разработки
rustc --version       # 1.75+

# (Опционально) Node.js для dashboard dev
node --version        # 20+
```

### Запуск за 1 минуту

```bash
# 1. Клонировать репозиторий
git clone https://github.com/Volgin1917/IDF.git
cd IDF

# 2. Создать .env из примера
cp .env.example .env
# Отредактировать .env (минимум: JWT_SECRET, OPENAI_API_KEY)

# 3. Запустить все сервисы
docker compose up -d

# 4. Проверить статус
docker compose ps
```

### Доступ к сервисам

| Сервис | URL | Описание |
|--------|-----|----------|
| **Dashboard** | http://localhost:5173 | React SPA (dev) / http://localhost:80 (prod) |
| **REST API** | http://localhost:8080/v1 | 12+ эндпоинтов |
| **MCP Server** | http://localhost:3001/mcp/sse | SSE транспорт |
| **n8n** | http://localhost:5678 | Workflow editor |
| **PostgreSQL** | localhost:5432 | DB (icedata / icedata) |

### Тестовые запросы

```bash
# Получить список игроков
curl http://localhost:8080/v1/players?limit=5

# Поиск игроков
curl "http://localhost:8080/v1/players/search?q=Ovechkin"

# AI анализ игрока
curl http://localhost:8080/v1/players/8478550/ai-analysis

# Лидеры по очкам
curl http://localhost:8080/v1/analytics/leaders?metric=points&limit=10

# Новости
curl http://localhost:8080/v1/news?limit=5
```

---

## Компоненты

### REST API (`ice-data-api`)

**Base URL:** `/v1`

| Endpoint | Method | Описание |
|----------|--------|----------|
| `/players` | GET | Список игроков (пагинация, фильтры) |
| `/players/:id` | GET | Профиль игрока |
| `/players/:id/stats` | GET | Статистика по сезонам |
| `/players/:id/ai-analysis` | GET | AI скаутский отчёт |
| `/players/search` | GET | Поиск по имени |
| `/teams` | GET | Список команд (32) |
| `/teams/:id/roster` | GET | Ростер команды |
| `/news` | GET | Новости и события |
| `/analytics/leaders` | GET | Лидеры по метрикам |
| `/analytics/timeline/:id` | GET | Таймлайн статистики |
| `/webhook/nhl-sync` | POST | Webhook для n8n sync |
| `/webhook/analyze-players` | POST | Webhook для AI анализа |
| `/webhook/collect-news` | POST | Webhook для сбора новостей |

**Auth:** JWT Bearer token (кроме webhook — там N8N_WEBHOOK_SECRET)

---

### MCP Server (`ice-data-mcp`)

**Transport:** SSE (`GET /mcp/sse`) + POST (`/mcp/message`)

**6 инструментов:**

```json
{
  "tools": [
    "get_player",
    "search_players",
    "analyze_player",
    "compare_players",
    "get_team_roster",
    "get_player_news"
  ]
}
```

**Auth:** API Key через заголовок `X-API-Key`

---

### Dashboard (`dashboard/`)

**7 страниц:**

| Страница | Маршрут | Описание |
|----------|---------|----------|
| Overview | `/` | Статистика лиги, топ-5, новости |
| Players | `/players` | Поиск, grid карточек |
| Player Detail | `/players/:id` | Профиль + табы Stats/AI + графики |
| Teams | `/teams` | 32 команды по конференциям |
| Analytics | `/analytics` | Лидеры по 7 метрикам |
| News | `/news` | Лента с сентимент-иконками |
| Compare | `/compare` | Сравнение игроков (BarChart) |

**3 chart компонента:**
- `PointsChart` — LineChart (points/goals/assists по сезонам)
- `AdvancedMetricsChart` — RadarChart (PPG, G/60, Corsi%, Fenwick%, xG)
- `ComparisonChart` — BarChart (сравнение нескольких игроков)

---

### Telegram Bot (`chatbot/`)

**6 команд:**

| Команда | Описание |
|---------|----------|
| `/start` | Приветствие + меню |
| `/search` | Поиск игрока (inline keyboard) |
| `/player` | Профиль игрока |
| `/stats` | Статистика по сезонам |
| `/analyze` | AI скаутский отчёт |
| `/compare` | Сравнение двух игроков |
| `/news` | Последние новости |

**Запуск:**
```bash
cd chatbot
pip install -r requirements.txt
python -m src.main
```

---

### n8n Workflows (`n8n/`)

**4 workflow:**

| Workflow | Файл | Описание |
|----------|------|----------|
| **NHL Sync** | `nhl-sync-workflow.json` | Ежедневная синхронизация игроков и команд |
| **AI Analysis** | `ai-analysis-workflow.json` | Генерация отчётов для топ-100 игроков |
| **Player Monitoring** | `player-monitoring-workflow.json` | Отслеживание изменений статистики |
| **News Collection** | `news-collection-workflow.json` | Сбор новостей из NHL API |

**Webhook endpoints:**
- `POST /v1/webhook/nhl-sync`
- `POST /v1/webhook/analyze-players`
- `POST /v1/webhook/collect-news`

**Auth:** Bearer token (`N8N_WEBHOOK_SECRET`)

---

## Configuration

### Переменные окружения

| Variable | Required | Default | Описание |
|----------|----------|---------|----------|
| `DATABASE_URL` | ✅ | — | PostgreSQL connection string |
| `JWT_SECRET` | ✅ | — | Secret для JWT токенов (мин. 32 chars) |
| `OPENAI_API_KEY` | ✅ | — | OpenAI API key (GPT-4) |
| `N8N_WEBHOOK_SECRET` | ✅ | — | Secret для webhook auth |
| `MCP_API_KEY` | ✅ | — | API key для MCP Server |
| `BOT_TOKEN` | ❌ | — | Telegram bot token (BotFather) |
| `POSTGRES_PASSWORD` | ✅ | — | DB password (prod) |
| `N8N_ENCRYPTION_KEY` | ✅ | — | n8n encryption key (prod) |
| `DOCKER_HUB_USER` | ❌ | `icedataforge` | Docker Hub username |
| `TAG` | ❌ | `latest` | Docker image tag |
| `RUST_LOG` | ❌ | `info` | Log level (trace/debug/info/warn/error) |

**Пример `.env`:**
```bash
# Server
HOST=0.0.0.0
PORT=8080

# Database
DATABASE_URL=postgresql://icedata:icedata@localhost:5432/ice_data_forge

# JWT
JWT_SECRET=super-secret-jwt-key-change-in-production
JWT_EXPIRATION_HOURS=24

# NHL API
NHL_API_BASE_URL=https://api-web.nhle.com/v1
NHL_API_RATE_LIMIT=60

# OpenAI
OPENAI_API_KEY=sk-...
OPENAI_MODEL=gpt-4

# n8n
N8N_URL=http://localhost:5678
N8N_WEBHOOK_SECRET=n8n-webhook-secret-change-me

# MCP Server
MCP_ADDR=0.0.0.0:3001
MCP_API_KEY=mcp-api-key-change-me

# Telegram
BOT_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz

# Docker
DOCKER_HUB_USER=icedataforge
TAG=latest
```

---

## Deployment

### Docker Hub

**4 образа:**
- `icedataforge/ice-data-api:latest`
- `icedataforge/ice-data-mcp:latest`
- `icedataforge/ice-data-bot:latest`
- `icedataforge/ice-data-dashboard:latest`

**Ручной деплой:**
```bash
# PowerShell
pwsh -NoProfile -File scripts/deploy.ps1 -Tag 2026.05.15

# Bash
bash scripts/deploy.sh 2026.05.15
```

**Auto-deploy (GitHub Actions):**
- Пуш в `main` → CI/CD → build → push → SSH deploy
- Нужны secrets: `DOCKER_HUB_TOKEN`, `SSH_HOST`, `SSH_USER`, `SSH_KEY`, `SSH_PATH`

### Production Compose

```bash
# 1. Скопировать production env
cp scripts/production.env.example .env

# 2. Заполнить секреты (обязательно!)
# POSTGRES_PASSWORD, JWT_SECRET, OPENAI_API_KEY, MCP_API_KEY, N8N_ENCRYPTION_KEY, N8N_WEBHOOK_SECRET

# 3. Запустить production стек
docker compose -f docker-compose.prod.yml --env-file .env up -d

# 4. Проверить логи
docker compose -f docker-compose.prod.yml logs -f
```

**Production features:**
- `restart: always` — автоперезапуск
- `pull_policy: always` — всегда тянуть свежие образы
- `deploy.resources.limits` — лимиты памяти
- Healthcheck для PostgreSQL
- Resource limits для всех сервисов

---

## Разработка

### Make команды

```bash
make check          # cargo check --workspace
make lint           # cargo clippy --workspace -- -D warnings
make test           # cargo test --workspace
make clean          # Очистка target, node_modules, .venv

make db-up          # docker compose up -d postgres n8n
make db-down        # docker compose down
make db-reset       # Полная пересоздание БД + миграции

make dev            # cargo run --bin ice-data-api
make dev-mcp        # cargo run --bin ice-data-mcp

make bot-install    # pip install -r chatbot/requirements.txt
make bot-dev        # cd chatbot && python -m src.main

make dashboard-dev  # cd dashboard && npx vite --host
make dashboard-build # cd dashboard && npx vite build

make docker-build   # docker build -t ice-data-forge:latest .
make docker-up      # docker compose up -d
make docker-deploy  # pwsh scripts/deploy.ps1
```

### Тесты

```bash
# Все тесты
cargo test --workspace

# Только analytics engine
cargo test --package ice-data-api -- analytics_engine

# С выводим логов
cargo test --package ice-data-api -- --nocapture
```

### Структура проекта

```
IDF/
├── Cargo.toml              # workspace (6 crates)
├── docker-compose.yml      # dev compose
├── docker-compose.prod.yml # prod compose
├── Dockerfile              # multi-stage (api, mcp)
├── Makefile                # developer commands
├── .env.example            # env template
├── .github/workflows/deploy.yml  # CI/CD
│
├── crates/
│   ├── ice-data-core/      # shared types, config, errors
│   ├── ice-data-nhl/       # NHL API client (rate-limited)
│   ├── ice-data-db/        # sqlx pool, queries, migrations
│   ├── ice-data-api/       # Axum REST API + analytics engine
│   ├── ice-data-mcp/       # MCP Server (SSE/JSON-RPC)
│   └── ice-data-ai/        # OpenAI GPT-4 integration
│
├── chatbot/                # Python + aiogram
│   ├── src/
│   │   ├── main.py
│   │   ├── config.py
│   │   ├── api_client.py
│   │   ├── keyboards.py
│   │   └── handlers/       # 6 command handlers
│   ├── Dockerfile
│   └── requirements.txt
│
├── dashboard/              # React 18+ + Vite + TypeScript
│   ├── src/
│   │   ├── App.tsx
│   │   ├── pages/          # 7 pages
│   │   ├── components/     # 7 components + 3 charts
│   │   ├── api/client.ts
│   │   └── types/index.ts
│   ├── Dockerfile
│   └── nginx.conf
│
├── n8n/                    # 4 workflow exports (JSON)
│
├── migrations/             # SQL migrations (symlink for sqlx)
│   └── 20240515000001_initial_schema.sql
│
└── scripts/
    ├── deploy.ps1          # PowerShell deploy script
    ├── deploy.sh           # Bash deploy script
    └── production.env.example
```

---

## Database Schema

**8 таблиц:**

| Таблица | Описание |
|---------|----------|
| `players` | Профили игроков (NHL ID, имя, позиция, команда) |
| `player_season_stats` | Статистика по сезонам (JSONB payload) |
| `teams` | 32 команды НХЛ (seed) |
| `ai_analysis` | AI скаутские отчёты (кэш) |
| `news_events` | Новости и события |
| `system_config` | Конфигурация системы |
| `audit_log` | Лог аудита (все изменения) |
| `user_preferences` | Пользовательские настройки |

**Индексы:**
- `players(nhl_id)`, `players(last_name)`, `players(team_id)`
- `player_season_stats(player_id, season)`
- `ai_analysis(player_id, analysis_type)`
- `news_events(event_date, event_type)`

**Views:**
- `player_career_summary` — карьерная сводка
- `team_roster_summary` — ростер команды

---

## API Reference

Полная спецификация OpenAPI доступна по адресу:
```
http://localhost:8080/openapi.json
```

Или используйте Swagger UI (если включён):
```
http://localhost:8080/docs
```

---

## License

MIT License — см. [LICENSE](LICENSE)

---

## Контакты

- **GitHub:** https://github.com/Volgin1917/IDF
- **Docker Hub:** https://hub.docker.com/u/icedataforge
- **Документация:** (TBD)
- **Roadmap:** (TBD)

---

<p align="center">
  <strong>IceData Forge</strong> — Built with ❤️ for hockey analytics
</p>

<p align="center">
  <img src="IDF-icon/IDF_64x64.png" alt="IceData Forge Icon" width="64" />
</p>
