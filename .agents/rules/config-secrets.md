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
| `MINIRUST_DATABASE_URL` | MariaDB connection (mysql driver) | No — optional |
| `MINIRUST_REDIS_URL` | Redis URL | No — optional |

## Secret rules

- **Never** commit `.env` with real values.
- **Always** update `.env.example` with placeholder values when adding a new variable.
- Log `database_configured = true/false` and `redis_configured = true/false` booleans — never log connection strings or passwords.

## Docker compose

- `infra` profile is optional. Do not make `docker compose up` required to compile or start apps.
- DB and Redis containers are for local development convenience only.
