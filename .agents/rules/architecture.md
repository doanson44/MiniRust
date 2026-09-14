---
description: MiniRust architecture, scope, and technology stack. Loaded always.
trigger: always_on
---

# MiniRust — Architecture & Scope

## Stack

| Layer | Technology |
|---|---|
| API | Axum 0.8, Tokio, tower, tower-http (TraceLayer) |
| Web | Leptos 0.8 SSR-only, Axum host, **Tailwind CSS** |
| Shared logic | `crates/services` (pure Rust, no framework imports) |
| Domain types | `crates/core` (AppError, domain structs) |
| Configuration | `crates/config` (dotenvy, env vars) |
| Database | SQLx 0.8 mysql driver (MariaDB) — **optional at startup** |
| Logging | `tracing` + `tracing-subscriber` only |

## Workspace layout

```
apps/
  api/        Axum REST   (127.0.0.1:3000)
  web/        Leptos SSR  (127.0.0.1:3001)
crates/
  config/     Config::load(), env, secrets
  core/       domain types, AppError (no Axum/Leptos/SQLx)
  database/   SQLx pool, migrations (optional)
  services/   business logic shared by apps
```

## Layering rule

UI (Leptos + Tailwind) => Handler => Service => Domain (core) => Repository/Adapter => Infrastructure

- **Handlers** stay thin: extract State, call service, return HTTP/HTML.
- **`crates/services`**: all business rules. Never import Axum, Leptos, SQLx, Teloxide here.
- **`crates/core`**: domain types and AppError. Zero framework dependencies.
- **`apps/web`**: presentation only. **Tailwind CSS** is the only CSS framework. No Bootstrap.
- Prefer Tailwind utility classes and small reusable Leptos components.

## Scope guard

Do NOT implement authentication, CMS, MariaDB, Telegram, AI, Redis, WebSocket, or other product features unless the user explicitly asks.

## Hard rules

- English identifiers, comments, and user-facing strings.
- Never commit .env or real secrets. Update .env.example with placeholders only.
- Do not commit, push, or open PRs unless the user explicitly requests it.
- Do not create empty speculative crates. Add new apps/* or crates/* only when implementing that feature.
- MariaDB optional: binary must start without a DB connection.
- Always inspect the actual file tree before making any changes.
