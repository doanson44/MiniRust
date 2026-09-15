---
name: implement-minirust-feature
description: Implements a MiniRust feature through CQRS, domain, persistence, and transport boundaries.
---

# Implement a MiniRust feature

## Before coding

1. Inspect the real tree. Code is the source of truth.
2. Identify the owning bounded context.
3. Classify the operation as a command, query, infrastructure operation, or presentation-only change.
4. Implement only the requested scope.

## Placement

```text
REST transport              → apps/api
SSR presentation            → apps/web
Command                     → crates/services/commands
Query                       → crates/services/queries
Domain primitive            → crates/core or owning context
Write persistence           → repository / database boundary
Read persistence            → query repository / projection
Cross-context integration   → explicit contract or integration event
```

## Rules

- Commands express business intent and may change state.
- Queries never change state and return purpose-built read DTOs.
- Transport handlers contain no business rules.
- Persistence belongs behind repositories/adapters.
- Domain code must not depend on Axum, Leptos, SQLx, MariaDB, or vendor SDKs.
- Do not access another context's tables, repositories, or domain internals directly.
- MariaDB persistence uses SQLx's `mysql` driver.
- Tailwind CSS is the only frontend CSS framework.
- Do not add Bootstrap compatibility.
- Configuration uses environment variables and `.env.example` placeholders.
- Never commit secrets or log credentials.

## After coding

Follow `.cursor/skills/validate-minirust/SKILL.md`.
