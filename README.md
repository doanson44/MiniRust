# MiniRust

Rust workspace for a small, extensible web platform.

The current repository is a runnable baseline: configuration, structured logging, a REST API, and a Leptos SSR page. It is not the full long-term product.

## Current scope

Working today:

- Cargo workspace
- Shared `core`, `config`, and `services` crates
- Axum REST API (`apps/api`)
- Leptos SSR web app (`apps/web`)
- Environment configuration via `dotenvy`
- Structured logging via `tracing`
- Health and hello endpoints
- Optional Docker Compose services for later SQL Server and Redis work

Not implemented yet:

- Authentication
- CMS, blog, user center, file manager
- SQL Server persistence
- Redis cache, sessions, or rate limiting
- Telegram bot
- Background workers
- AI, market data, resume builder, WebSocket, monitoring, and other product features

## Architecture

```text
UI (Leptos SSR)
 ↓
Handler (Axum)
 ↓
Application Service
 ↓
Domain types (core)
```

Infrastructure adapters (SQL Server, Redis, Telegram, AI vendors) are not wired into startup. Future crates can be added under `crates/` without moving the existing apps.

```text
MiniRust/
├── apps/
│   ├── api     # Axum REST API
│   └── web     # Leptos SSR
└── crates/
    ├── config  # environment configuration
    ├── core    # shared types and errors
    └── services
```

SQL Server and Redis are planned infrastructure. They are optional and unused by this baseline. When those features exist, persistence belongs in repositories, cache entries must have a TTL, and Redis must not be the source of truth for permanent business data.

## Prerequisites

- Rust stable (edition 2021)
- Optional: Docker, only for local SQL Server or Redis

## Setup

```bash
git clone https://github.com/doanson44/MiniRust.git
cd MiniRust
cp .env.example .env
```

`.env` is loaded automatically when present. It is not required for development defaults.

## Environment configuration

| Variable | Default (development) | Purpose |
| --- | --- | --- |
| `MINIRUST_ENV` | `development` | `development` or `production` |
| `MINIRUST_LOG` | `info` | Fallback tracing filter when `RUST_LOG` is unset |
| `MINIRUST_API_HOST` | `127.0.0.1` | API bind host |
| `MINIRUST_API_PORT` | `3000` | API bind port |
| `MINIRUST_WEB_HOST` | `127.0.0.1` | Web bind host |
| `MINIRUST_WEB_PORT` | `3001` | Web bind port |
| `MINIRUST_DATABASE_URL` | unset | Reserved for SQL Server. Unused today. |
| `MINIRUST_REDIS_URL` | unset | Reserved for Redis. Unused today. |

In production (`MINIRUST_ENV=production`), the process being started requires explicit host and port variables. The API requires `MINIRUST_API_HOST` and `MINIRUST_API_PORT`. The web app requires `MINIRUST_WEB_HOST` and `MINIRUST_WEB_PORT`.

Do not put secrets in logs or in committed files. `RUST_LOG` overrides `MINIRUST_LOG` when set.

## Run

API:

```bash
cargo run -p minirust-api
```

- `GET http://127.0.0.1:3000/health`
- `GET http://127.0.0.1:3000/api/v1/hello`

Web:

```bash
cargo run -p minirust-web
```

- `GET http://127.0.0.1:3001/` renders the SSR page
- `GET http://127.0.0.1:3001/health`

Press `Ctrl+C` to shut down either process.

## Test, format, and lint

```bash
cargo test --workspace --all-targets
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

SQL Server and Redis are not required for these commands.

## Docker

`docker-compose.yml` defines optional Redis and SQL Server containers behind the `infra` profile. They are for later features, not for the default developer workflow.

```bash
docker compose --profile infra up -d
docker compose --profile infra down
```

SQL Server will not start unless `MSSQL_SA_PASSWORD` is set. The baseline applications still start without Docker.
