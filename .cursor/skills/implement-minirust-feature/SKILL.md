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
5. For web work, identify the responsive viewport range and behavior before coding.

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

## Web implementation

- Build mobile-first layouts and progressively enhance them with Tailwind responsive variants.
- Every user-facing page MUST remain usable at mobile, tablet, and desktop widths without horizontal overflow.
- Use responsive typography, spacing, grids, navigation, and action layouts rather than fixed desktop dimensions.
- Preserve readable line lengths and touch-friendly controls on small screens.
- Include the viewport meta tag in SSR documents.
- Use semantic landmarks, accessible names, visible focus states, and sufficient contrast.
- Verify responsive behavior from rendered HTML and use browser/device verification when available.
- If Tailwind has no build pipeline yet, browser/CDN usage is development-only and must not be described as production-ready.

## Testing

- Unit-test domain invariants and command/query handlers.
- Every HTTP endpoint introduced or modified MUST have an integration test.
- Endpoint integration tests MUST exercise the real Axum router with `router().oneshot(...)` rather than calling transport handlers directly.
- Web page tests MUST assert the rendered HTML contract and meaningful responsive/accessibility markers.
- Database-dependent endpoint tests use the Docker-backed MariaDB environment.
- Before completion, compare registered HTTP routes with integration tests and resolve every uncovered endpoint or document a justified exception.

## After coding

Follow the validation skill and report exactly what was inspected, formatted, compiled, tested, linted, committed, pushed, and merged.
