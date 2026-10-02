# MiniRust

Rust workspace for a large, extensible web platform.

The current baseline provides a runnable Axum API, Leptos SSR + hydration web application, MariaDB connectivity, health checks, Swagger UI, and Docker Compose orchestration. The application layer is being structured around CQRS so the repository can scale to many bounded contexts and a large number of independent features.

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
    ├── observability/    # shared tracing logger
    └── services/         # current application-layer package; CQRS commands/queries
```

The `services` package name is retained temporarily for Cargo workspace compatibility during the CQRS migration. Its internal structure is no longer a generic service collection: application behavior is organized under `commands/` and `queries/`.

## CQRS baseline

The current baseline demonstrates both sides of CQRS:

- `EchoCommand` → `EchoCommandHandler` for a write-side operation.
- `GreetingQuery` → `GreetingQueryHandler` for a read-side operation.
- Axum handlers translate HTTP into commands/queries and map results into HTTP responses.
- Leptos owns presentation and hydration; application commands/queries remain behind explicit application boundaries.
- Database health remains infrastructure health logic and is intentionally not forced through CQRS.

## Frontend policy

- Leptos full-stack SSR + client-side hydration is the web rendering model.
- The server renders the initial HTML; `cargo-leptos` builds the browser WASM/JS bundle used to hydrate the same component tree.
- Interactive frontend behavior belongs in Leptos components; inline JavaScript is not part of the frontend architecture.
- Tailwind CSS is the only CSS framework.
- Bootstrap must not be introduced or preserved as a compatibility layer.

## Accounts and menu access

Three account dimensions are distinguished. See [`docs/architecture/authentication.md`](docs/architecture/authentication.md) for the full model:

- normal — registered account without the Premium entitlement and without the admin role
- premium — account with an active Premium entitlement
- admin — account with the admin system role; administrators always have full access

Every menu row carries two access flags, `allow_user` and `allow_premium`. The flags are matched exactly against the account, so an active Premium entitlement does not implicitly grant menus marked for normal users. Administrators manage the flags on `/admin/menus`; that page only edits access, because the menu registry is part of the system definition and menus are added or removed through migrations.

## API endpoints

API (`http://127.0.0.1:3000`):

- `GET /health` — live MariaDB connectivity check. Returns `200` when the database is reachable and `503` otherwise.
- `GET /api/v1/hello` — greeting query.
- `POST /api/v1/echo` — echo command.
- `GET /api/v1/auth/me` — current authenticated user and profile.
- `GET /api/v1/menus` — active menus the authenticated account may open.
- `GET /api/v1/admin/menus` — list all system menus; admin only.
- `PATCH /api/v1/admin/menus/{menu_id}` — update a menu row, including its access flags; admin only.
- `PATCH /api/v1/users/me` — update full name and avatar URL.
- `POST /api/v1/users/me/lock` — lock current account.
- `DELETE /api/v1/users/me` — permanently delete current account.
- `GET /api/v1/admin/users` — list users; admin only.
- `POST /api/v1/admin/users` — create user; admin only.
- `GET /api/v1/admin/users/{user_id}` — read user; admin only.
- `PATCH /api/v1/admin/users/{user_id}` — update user; admin only.
- `DELETE /api/v1/admin/users/{user_id}` — delete user; admin only.
- `POST /api/v1/admin/users/{user_id}/unlock` — unlock user; admin only.
- `PUT /api/v1/admin/users/{user_id}/role` — assign/remove admin role.
- `GET /api/v1/admin/users/{user_id}/entitlements/premium` — read premium entitlement.
- `PUT /api/v1/admin/users/{user_id}/entitlements/premium` — grant/update premium entitlement.
- `DELETE /api/v1/admin/users/{user_id}/entitlements/premium` — revoke premium entitlement.
- `GET /api/v1/openapi.json` — OpenAPI 3.0 document.
- `GET /swagger` — Swagger UI.

Web (`http://127.0.0.1:3001`):

- `GET /health` — web process health.
- `GET /` — redirects to `/app` when authenticated and to `/login` otherwise.
- `GET /login` — passwordless sign-in page.
- `GET /register` — passwordless registration page.
- `GET /app` — landing page introducing the platform.
- `GET /profile` — account information, profile editing, and account lifecycle actions.
- `GET /admin` — user administration; admin only.
- `GET /admin/menus` — menu permission grid; admin only.

The authenticated sidebar is populated from `/api/v1/menus`, so server-side filtering is authoritative for visible navigation. The same menu registry is enforced by the web middleware, so an account cannot reach an ungranted page by typing its URL. Because the sidebar is rendered from that registry, a page appears in the navigation only when a migration registers it.

## Database

MariaDB is the selected relational database. SQLx uses its `mysql` driver for MariaDB connectivity.

The API requires MariaDB connection settings because its current startup contract establishes a live database connection before serving requests.

Connection settings come from the `MINIRUST_DB_*` variables. `MINIRUST_DATABASE_URL`, when set, overrides all of them.

Wrap values in single quotes inside `.env` when they contain `$`, because dotenvy expands `$VAR` in double-quoted and unquoted values only.

## Integration tests

Every API endpoint must have integration-test coverage. Tests should exercise the public HTTP contract through the Axum router and the real application/database boundary.

The project intentionally uses **business-rule-focused integration testing** rather than maximizing test count. A new endpoint should add only the scenarios needed to prove its meaningful business invariants, authorization behavior, persistence effects, and important response contract. Do not add redundant tests for trivial permutations or implementation details already covered by stronger scenarios.

Database integration tests use Testcontainers. The test process starts an isolated MariaDB container, waits for readiness, runs migrations and test seeding, then removes the container automatically.

Run all workspace tests normally:

```bash
cargo test --workspace --all-targets
```

Docker must be running when database integration tests execute. No separate test Compose command or pre-created test database is required.

## Environment

| Variable | Default | Purpose |
| --- | --- | --- |
| `MINIRUST_ENV` | `development` | Runtime environment |
| `MINIRUST_LOG` | `info` | Fallback tracing filter |
| `MINIRUST_LOG_DIR` | `logs` | Production log directory |
| `MINIRUST_API_HOST` | `127.0.0.1` | API bind host |
| `MINIRUST_API_PORT` | `3000` | API bind port |
| `MINIRUST_WEB_HOST` | `127.0.0.1` | Web bind host |
| `MINIRUST_WEB_PORT` | `3001` | Web bind port |
| `MINIRUST_DB_HOST` | `127.0.0.1` | MariaDB host |
| `MINIRUST_DB_PORT` | `3306` | MariaDB port |
| `MINIRUST_DB_NAME` | `minirust` | MariaDB database name |
| `MINIRUST_DB_USER` | `minirust` | MariaDB user |
| `MINIRUST_DB_PASSWORD` | `minirust` | MariaDB password |
| `MINIRUST_DB_MAX_CONNECTIONS` | `10` | MariaDB connection pool size |
| `MINIRUST_DATABASE_URL` | — | Full SQLx URL; overrides every `MINIRUST_DB_*` variable |
| `MINIRUST_AUTH_SECRET` | required | Authentication/session secret |
| `MINIRUST_SMTP_HOST` | `smtp.gmail.com` when SMTP is configured | SMTP relay host |
| `MINIRUST_SMTP_PORT` | `587` when SMTP is configured | SMTP STARTTLS port |
| `MINIRUST_SMTP_USERNAME` | — | SMTP username; required to enable SMTP |
| `MINIRUST_SMTP_PASSWORD` | — | SMTP password/app password; required to enable SMTP |
| `MINIRUST_SMTP_FROM_EMAIL` | — | Sender email; required to enable SMTP |
| `MINIRUST_SMTP_FROM_NAME` | — | Optional sender display name |
| `MINIRUST_ADMIN_EMAIL` | `admin@minirust.local` in development | Bootstrap administrator email |
| `MINIRUST_ADMIN_OTP` | `123456` in development | Bootstrap administrator OTP |
| `MINIRUST_DOMAIN` | required by production Compose | Public HTTPS domain for Caddy |

Do not use development defaults for production secrets or credentials.

Email delivery is optional. When SMTP username, password, and sender email are not configured, MiniRust keeps email delivery disabled and authentication requests that require email return the existing `EMAIL_DELIVERY_UNAVAILABLE` behavior. When SMTP is configured, the API uses Gmail-compatible SMTP STARTTLS by default.

For Gmail, use an App Password rather than the normal account password. Google requires 2-Step Verification for App Passwords.

### Logging

The application uses one shared global `tracing` subscriber for the API and Leptos SSR processes:

- Development: logs are written to the console.
- Production: logs are written to daily-rotated files under `MINIRUST_LOG_DIR`.
- Docker Compose mounts `/var/log/minirust` to the persistent `minirust_logs` volume in production, so container restarts do not remove application logs.

The API and web processes use separate file prefixes: `minirust-api.log.*` and `minirust-web.log.*`.

## Run locally

Start MariaDB, create the database, and set the `MINIRUST_DB_*` variables in `.env`:

```bash
cargo run -p minirust-api
cd apps/web
cargo leptos watch
```

## Docker Compose

The base Compose file is suitable for local development:

```bash
docker compose up -d --build
```

Check service state and database health:

```bash
docker compose ps
curl http://127.0.0.1:3000/health
```

The API waits for MariaDB to become healthy before starting. Its `/health` endpoint executes `SELECT 1`, so it verifies actual database connectivity.

### Single-server production deployment

Production is designed for **one server**. Caddy is the only service exposed to the public network; it terminates HTTPS and routes the web and API traffic over the private Compose network.

```text
Internet
   |
 HTTPS :443
   |
 Caddy
  /api/* -> api:3000
  /*     -> web:3001
   |
 MariaDB (private)
```

Set the public DNS name on the server and provide the domain through `MINIRUST_DOMAIN`:

```bash
export MINIRUST_DOMAIN=example.com
docker compose -f docker-compose.yml -f docker-compose.production.yml up -d --build
```

The production overlay removes host port publishing for MariaDB, API, and Web. Only ports `80` and `443` are published by Caddy.

Caddy automatically provisions and renews the public TLS certificate when the domain points to this server and ports `80` and `443` are reachable from the Internet.

Check the stack:

```bash
docker compose -f docker-compose.yml -f docker-compose.production.yml ps
```

The application remains split into independent API and SSR Web processes inside the same server. This preserves the current architecture without requiring separate hosting.

Stop the production stack:

```bash
docker compose -f docker-compose.yml -f docker-compose.production.yml down
```

Remove the MariaDB volume as well:

```bash
docker compose -f docker-compose.yml -f docker-compose.production.yml down -v
```

## Verification

The repository currently has two GitHub Actions workflows. The main `CI` workflow also installs Rust 1.90, the WASM target, `cargo-leptos`, and verifies the SSR + hydration build. A second `Rust CI` workflow performs locked workspace check, clippy, tests, and build.

The verification commands include:

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked --all-targets
cargo build --workspace --locked
```

Do not claim verification unless the commands were actually executed.
## Current verification state

Documentation describes the repository state but does not imply that the latest commit has passed CI. Build, lint, test, formatting, and frontend verification claims must be based on actual command output or a GitHub Actions result for the exact commit.
- `POST /api/v1/auth/register/request-verification` — request registration code.
- `POST /api/v1/auth/register/verify` — verify registration code and create session.
- `POST /api/v1/auth/login/request-code` — request login code.
- `POST /api/v1/auth/login/verify-code` — verify login code and create session.
- `POST /api/v1/auth/logout` — clear current session.

### API integration-test assertion policy

Integration tests validate stable API behavior rather than message wording. Error scenarios must assert the HTTP status and, where defined by the API contract, the stable application error `code`. They should not assert localized `detail` or human-readable error text unless that exact presentation is the behavior under test. Error bodies and internal diagnostics must not be embedded in assertion messages.
