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
5. For web work, explicitly identify the target viewport range and responsive behavior before coding.

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

### Web presentation

- Build mobile-first layouts and progressively enhance them with Tailwind responsive variants.
- Every user-facing page MUST remain usable at mobile, tablet, and desktop widths without horizontal overflow.
- Use responsive typography, spacing, grids, navigation, and action layouts rather than fixed desktop dimensions.
- Preserve readable line lengths and touch-friendly controls on small screens.
- Include the viewport meta tag in SSR documents.
- Use semantic landmarks, accessible names, visible focus states, and sufficient contrast.
- Verify responsive behavior from the rendered HTML by testing for the relevant Tailwind breakpoint classes; use browser/device verification when a browser environment is available.
- Use Tailwind CSS utility classes as the styling mechanism. Do not introduce Bootstrap or ad-hoc inline styles.
- If Tailwind has no build pipeline yet, keep browser/CDN usage explicitly development-oriented and track production asset compilation as infrastructure work rather than silently treating Play CDN as production-ready.

### Infrastructure

- MariaDB access uses SQLx's `mysql` driver.
- Infrastructure code must not leak into the domain.
- If asynchronous integration events are required later, use an outbox-based design rather than publishing directly inside an uncommitted transaction.

## Phase 4 — Tests

- Unit-test domain invariants and command/query handlers.
- **Every HTTP endpoint introduced or modified MUST have an integration test.**
- Endpoint integration tests MUST exercise the real Axum router with `router().oneshot(...)` rather than calling transport handlers directly.
- Cover the endpoint's success contract and meaningful validation/error paths.
- For web pages, assert the rendered HTML contract and the responsive/accessibility markers that are part of the page requirement.
- If an endpoint depends on MariaDB, integration-test it against real MariaDB using the Docker-backed test environment.
- Persistence tests are added when a repository is introduced.
- Test idempotency and concurrency behavior when the command requires it.
- Before completion, compare registered HTTP routes with integration tests and resolve every uncovered endpoint or document a justified exception.

## Phase 5 — Validate

Run the `validate-minirust` skill. Do not claim completion without actual verification.
