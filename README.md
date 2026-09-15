# MiniRust

Rust workspace for a large, extensible web platform.

The current baseline provides a runnable Axum API, Leptos SSR web application, MariaDB connectivity, health checks, Swagger UI, and Docker Compose orchestration. The application layer is being structured around CQRS so the repository can scale to many bounded contexts and a large number of independent features.

## Architecture

MiniRust follows a **CQRS-oriented modular monolith** with domain boundaries designed for future service extraction.

```text
Leptos SSR / Axum
       ↓
Transport handlers
       ↓
Commands ─────────────── Queries
       ↓                      ↓
Command handlers        Query handlers
       ↓                      ↓
Domain / write model   Read models / DTOs
       ↓                      ↓
Write repositories      Read repositories
       └──────────┬───────────┘
                  ↓
               MariaDB
```

CQRS is applied at the application boundary. The initial implementation intentionally uses one MariaDB database; separate read storage and asynchronous messaging are evolutionary steps, not prerequisites.

See:

- [`docs/architecture/README.md`](docs/architecture/README.md) — architecture blueprint and evolution path
- [`docs/architecture/cqrs.md`](docs/architecture/cqrs.md) — command/query, repository, transaction, event, and consistency rules
- [`docs/architecture/team-development.md`](docs/architecture/team-development.md) — team ownership and bounded-context rules

## Current workspace

```text
MiniRust/
├── apps/
│   ├── api/              # Axum REST transport
│   └── web/              # Leptos SSR + Tailwind CSS
└── crates/
    ├── config/           # environment configuration
    ├── core/             # domain primitives and application errors
    ├── database/         # MariaDB / SQLx infrastructure
    └── services/         # current application-layer package; CQRS commands/queries
```

The `services` package name is retained temporarily for Cargo workspace compatibility during the CQRS migration. Its internal structure is no longer a generic service collection: application behavior is organized under `commands/` and `queries/`.

## CQRS baseline

The current baseline demonstrates both sides of CQRS:

- `EchoCommand` → `EchoCommandHandler` for a write-side operation.
- `GreetingQuery` → `GreetingQueryHandler` for a read-side operation.
- Axum handlers translate HTTP into commands/queries and map results into HTTP responses.
- Leptos uses the query handler for SSR data.
- Database health remains infrastructure health logic and is intentionally not forced through CQRS.

## Frontend policy

- Leptos SSR is the web rendering model.
- Tailwind CSS is the only CSS framework.
- Bootstrap must not be introduced or preserved as a compatibility layer.

## API endpoints

API (`http://127.0.0.1:3000`):

- `GET /health` — live MariaDB connectivity check. Returns `200` when the database is reachable and `503` otherwise.
- `GET /api/v1/hello` — greeting query.
- `POST /api/v1/echo` — echo command.
- `GET /api/v1/openapi.json` — OpenAPI 3.0 document.
- `GET /swagger` — Swagger UI.

Web (`http://127.0.0.1:3001`):

- `GET /` — SSR index page.
- `GET /health` — web process health.

## Database

MariaDB is the selected relational database. SQLx uses its `mysql` driver for MariaDB connectivity.

The API requires `MINIRUST_DATABASE_URL` because its current startup contract establishes a live database connection before serving requests.

## Integration tests

Database integration tests use Docker Compose to start an isolated MariaDB instance for the test run. The test allocates an ephemeral host port, waits for the database healthcheck, connects through the same `minirust-database` abstraction used by the application, and removes the container and its volumes after the test.

Run the integration test together with the workspace tests:

```bash
cargo test --workspace --all-targets
```

Docker must be running when the database integration test executes.

The dedicated test environment is defined in [`docker-compose.integration.yml`](docker-compose.integration.yml) and is intentionally separate from the runtime `docker-compose.yml` stack.

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

Start MariaDB, create the database, and set `MINIRUST_DATABASE_URL` in `.env`:

```bash
cargo run -p minirust-api
cargo run -p minirust-web
```

## Docker Compose

```bash
docker compose up -d --build
```

Check service state and database health:

```bash
docker compose ps
curl http://127.0.0.1:3000/health
```
