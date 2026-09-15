---
description: MiniRust architecture, CQRS boundaries, scope, and technology stack. Loaded always.
trigger: always_on
---

# MiniRust — Architecture & Scope

## Stack

| Layer | Technology |
|---|---|
| API | Axum 0.8, Tokio, tower, tower-http |
| Web | Leptos 0.8 SSR-only, Axum host, **Tailwind CSS** |
| Application | CQRS-oriented command/query modules in the current `crates/services` compatibility package |
| Domain | `crates/core` — pure Rust domain types and `AppError` |
| Configuration | `crates/config` — dotenvy and environment variables |
| Database | SQLx 0.8 `mysql` driver for MariaDB |
| Logging | `tracing` + `tracing-subscriber` |

## Workspace layout

```text
apps/
  api/        Axum REST transport
  web/        Leptos SSR transport
crates/
  config/     configuration
  core/       domain primitives and application errors
  database/   MariaDB / SQLx infrastructure
  services/   current application-layer package
              ├── commands/
              └── queries/
```

## Layering rule

```text
Transport
   ↓
Command / Query
   ↓
Application handler
   ↓
Domain or read model
   ↓
Repository / adapter
   ↓
Infrastructure
```

- Commands represent business intent and may change state.
- Queries never change state and return read DTOs/projections.
- Handlers stay thin: translate transport input, invoke a command/query, map the result.
- Domain code must not depend on Axum, Leptos, SQLx, MariaDB, or vendor SDKs.
- Read-side code must not call command handlers.
- Write-side code must not depend on presentation DTOs.
- Cross-context access must use explicit contracts or integration events.
- Direct access to another context's tables, aggregates, or repositories is forbidden.

## CQRS evolution

The first implementation uses one MariaDB database with logically separated read/write models. Separate read stores, transactional outbox processing, asynchronous integration events, and independent service deployment are later optimization steps.

Do not introduce Event Sourcing or a message broker merely because CQRS exists. Add them when a concrete consistency, integration, audit, or scaling requirement justifies the complexity.

## Frontend

- `apps/web` is presentation only.
- Tailwind CSS is the only CSS framework.
- Bootstrap must not be introduced or preserved for compatibility.

## Database

MariaDB is the selected database. The API currently requires `MINIRUST_DATABASE_URL` during startup and `/health` performs a live `SELECT 1` connectivity check.

## Scope

Do not implement product features such as authentication, CMS, notifications, AI, market functionality, or other future domains unless explicitly requested. When a feature is requested, place it inside the appropriate bounded context rather than creating a global service collection.

## Hard rules

- English identifiers, comments, documentation, and user-facing strings.
- Never commit `.env` or real secrets.
- Do not commit, push, or open a PR unless explicitly requested.
- Do not create speculative empty bounded contexts.
- Always inspect the actual tree before changing it.
- Preserve explicit ownership boundaries so multiple teams can work independently.
