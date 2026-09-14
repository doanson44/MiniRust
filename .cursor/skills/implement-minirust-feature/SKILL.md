---
name: implement-minirust-feature
description: Implements a MiniRust feature through UI/handler/service/domain/adapter layers without expanding into unrequested product work. Use when adding an endpoint, page, service, repository, or infrastructure adapter.
---

# Implement a MiniRust feature

## Before coding

1. Inspect the real tree. Docs describe intent; code is the source of truth.
2. Implement only what the user asked. Auth, CMS, bot, workers, AI, market, resume, WebSocket stay out of scope until requested.
3. Prefer extending `services` + a thin handler over a new crate.

## Placement

```text
User request
  → apps/web handler + Leptos view     if it is a page
  → apps/api handler                   if it is REST
  → crates/services                    business rule used by more than one app
  → crates/core                        shared types / AppError
  → new adapter crate                  only when talking to MariaDB, Redis, Telegram, etc.
```

## Rules

- Handlers call services. Services do not import Axum or Leptos.
- Persistence goes in a repository/adapter, not in a service.
- If MariaDB or Redis is needed, keep startup working when the service is unset unless the feature cannot run without it.
- MariaDB persistence should use SQLx's `mysql` driver.
- Redis: TTL on every entry; never store permanent business data.
- Configuration: env vars + `.env.example` placeholders, no secrets in git or logs.
- Tests: behavior of the new service and HTTP/HTML path.

## After coding

Follow `.cursor/skills/validate-minirust/SKILL.md`.
