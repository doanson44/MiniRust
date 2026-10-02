---
description: Environment configuration and secrets handling conventions. Applied when editing crates/config, .env*, docker-compose.yml files.
trigger: model_decision
globs: ["crates/config/**", ".env*", "docker-compose.yml"]
---

# Config & Secrets — MiniRust

## Loading

- `Config::load()` reads from environment variables via `dotenvy`.
- Missing `.env` file is allowed — the binary must still start with defaults.
- Development defaults are safe. Production (`MINIRUST_ENV=production`) requires explicit host/port.

## Variables reference

| Variable | Description | Required |
|---|---|---|
| `MINIRUST_ENV` | `development` / `production` | No (defaults dev) |
| `MINIRUST_API_HOST` | API bind host | No |
| `MINIRUST_API_PORT` | API bind port | No |
| `MINIRUST_WEB_HOST` | Web bind host | No |
| `MINIRUST_WEB_PORT` | Web bind port | No |
| `MINIRUST_DB_HOST` | MariaDB host | No (defaults `127.0.0.1`) |
| `MINIRUST_DB_PORT` | MariaDB port | No (defaults `3306`) |
| `MINIRUST_DB_NAME` | MariaDB database name | No (defaults `minirust`) |
| `MINIRUST_DB_USER` | MariaDB user | No (defaults `minirust`) |
| `MINIRUST_DB_PASSWORD` | MariaDB password | No (defaults `minirust`) |
| `MINIRUST_DB_MAX_CONNECTIONS` | MariaDB pool size | No (defaults `10`) |
| `MINIRUST_UPLOAD_DIR` | Directory for uploaded files | No (defaults `uploaded`) |
| `MINIRUST_DATABASE_URL` | Full SQLx URL; overrides all `MINIRUST_DB_*` | No — optional |
| `MINIRUST_REDIS_URL` | Redis URL | No — optional |

## Secret rules

- **Never** commit `.env` with real values.
- **Always** update `.env.example` with placeholder values when adding a new variable.
- Log `database_configured = true/false` and `redis_configured = true/false` booleans — never log connection strings or passwords.
- `.env` values containing `$` must be wrapped in single quotes — dotenvy expands `$VAR` in unquoted and double-quoted values.
- The connection URL is built with percent-encoded credentials because SQLx percent-decodes the user, password, and database components.

## Docker compose

- `infra` profile is optional. Do not make `docker compose up` required to compile or start apps.
- DB and Redis containers are for local development convenience only.
