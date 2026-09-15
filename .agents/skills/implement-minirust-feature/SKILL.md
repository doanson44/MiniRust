---
name: implement-minirust-feature
description: Implements a MiniRust feature through the CQRS, domain, persistence, and transport boundaries.
---

# Implement a MiniRust feature

## Phase 1 — Understand

1. Inspect the actual repository tree and current implementation.
2. Identify the owning bounded context or confirm that the feature belongs to the current baseline context.
3. Classify the operation as a command, query, infrastructure operation, or presentation-only change.
4. Implement only the requested scope.

## Phase 2 — Placement

```text
Request type                     → Target
REST transport                  → apps/api
SSR presentation                → apps/web
Command                         → application `commands/`
Query                           → application `queries/`
Domain entity/value object     → crates/core or owning context domain
Write persistence               → database/repository boundary
Read persistence                → query/read repository boundary
Cross-context integration       → explicit contract or integration event
```

The current application package is `crates/services`; its internal structure is CQRS-oriented. Do not add new global service types.

## Phase 3 — CQRS implementation

### Commands

- Name commands after business intent.
- Validate input at the application/domain boundary.
- Enforce invariants in the domain model.
- Persist through a write-oriented repository.
- Define transaction scope explicitly.

### Queries

- Queries never mutate state.
- Return purpose-built DTOs or projections.
- Do not expose domain aggregates as transport response models.
- Optimize SQL for the read use case rather than forcing the write model into the query.

### Transport

- Axum/Leptos handlers translate transport data into commands or queries.
- Handlers contain no business rules.
- Map application/domain errors at the transport boundary.

### Infrastructure

- MariaDB access uses SQLx's `mysql` driver.
- Infrastructure code must not leak into the domain.
- If asynchronous integration events are required later, use an outbox-based design rather than publishing directly inside an uncommitted transaction.

## Phase 4 — Tests

- Unit-test domain invariants and command/query handlers.
- API tests use `router().oneshot(...)`.
- SSR tests validate rendered HTML.
- Persistence tests are added when a repository is introduced.
- Test idempotency and concurrency behavior when the command requires it.

## Phase 5 — Validate

Run the `validate-minirust` skill. Do not claim completion without actual verification.
