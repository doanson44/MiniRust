# MiniRust

Rust workspace for a small, extensible web platform.

The repository provides a runnable API, Leptos SSR web page, MariaDB connectivity, health checks, Swagger UI, and Docker Compose orchestration.

## Current scope

Working today:

- Cargo workspace
- Shared `core`, `config`, `database`, and `services` crates
- Axum REST API (`apps/api`)
- Leptos SSR web app (`apps/web`)
- Tailwind CSS as the frontend CSS framework
- MariaDB access through SQLx
- `/health` with a live MariaDB connectivity check
- OpenAPI document at `/api/v1/openapi.json`
- Swagger UI at `/swagger`
- Docker Compose for MariaDB, API, and web

Frontend styling policy:

- Tailwind CSS is the only CSS framework used by MiniRust.
- Bootstrap is not used and must not be introduced.
- UI components should use Leptos and Tailwind utility classes.

## Architecture

```text
UI (Leptos SSR + Tailwind CSS)
 ↓
Handler (Axum)
 ↓
Application Service
 ↓
Domain types (core)
 ↓
Database adapter
 ↓
MariaDB
```

```text
MiniRust/
├── apps/
│   ├── api     # Axum REST API
│   └── web     # Leptos SSR + Tailwind CSS
└── crates/
    ├── config    # environment configuration
    ├── core      # shared types and errors
    ├── database  # MariaDB/SQLx adapter
    └── services  # application services
```

Business logic remains independent from Axum, Leptos, SQLx, and MariaDB. Database access is isolated in `crates/database`.

## Endpoints

API (`http://127.0.0.1:3000`):

- `GET /health` — application and MariaDB health. Returns `200` when both are healthy and `503` when MariaDB is unavailable.
- `GET /api/v1/hello` — greeting endpoint.
- `POST /api/v1/echo` — validated echo endpoint.
- `GET /api/v1/openapi.json` — OpenAPI 3.0 document.
- `GET /swagger` — Swagger UI.

Web (`http://127.0.0.1:3001`):

- `GET /` — SSR index page.
- `GET /health` — web process health.

## Environment

| Variable | Default | Purpose |
| --- | --- | --- |
| `MINIRUST_ENV` | `development` | Runtime environment |
| `MINIRUST_LOG` | `info` | Fallback tracing filter |
| `MINIRUST_API_HOST` | `127.0.0.1` | API bind host |
| `MINIRUST_API_PORT` | `3000` | API bind port |
| `MINIRUST_WEB_HOST` | `127.0.0.1` | Web bind host |
| `MINIRUST_WEB_PORT` | `3001` | Web bind port |
| `MINIRUST_DATABASE_URL` | required by API | MariaDB SQLx connection URL |

## Run locally

Start MariaDB, create the database, and set `MINIRUST_DATABASE_URL` in `.env`, then run:

```bash
cargo run -p minirust-api
cargo run -p minirust-web
```

## Docker Compose

The complete local stack is MariaDB + API + web:

```bash
docker compose up -d --build
```

Check service state:

```bash
docker compose ps
curl http://127.0.0.1:3000/health
```

The API waits for MariaDB to become healthy before starting. Its `/health` endpoint then performs `SELECT 1` against MariaDB, so the endpoint represents actual database connectivity rather than process liveness alone.

Stop the stack:

```bash
docker compose down
```

Remove the MariaDB volume as well:

```bash
docker compose down -v
```

## Verification

```bash
cargo test --workspace --all-targets
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
