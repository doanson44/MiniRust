---
name: implement-minirust-feature
description: >-
  Implements a MiniRust feature correctly through the UI/handler/service/domain/
  adapter layers without expanding into unrequested product work. Use when adding
  an endpoint, page, service, repository, or infrastructure adapter.
---

# Implement a MiniRust Feature

## Phase 1 — Understand before coding

1. **Inspect the real tree** — docs describe intent, the actual code is the source of truth.
   ```powershell
   Get-ChildItem -Recurse -Name | Where-Object { $_ -match "\.rs$" }
   ```
2. **Clarify scope** — implement only what was asked. Auth, CMS, AI, bot, workers, market, resume, WebSocket are out of scope until requested.
3. **Decide placement** using the table below.

## Phase 2 — Decide placement

```
User request type               → Target location
─────────────────────────────────────────────────
New HTML page                   → apps/web (handler + Leptos view)
New REST endpoint               → apps/api (handler)
Business rule, reusable logic   → crates/services
Shared domain type / AppError   → crates/core
DB / Redis / Telegram adapter   → new adapter crate (only if needed)
```

- Prefer **extending** `crates/services` + a thin handler over creating a new crate.
- If a new crate is needed, follow the `add-minirust-crate` skill first.

## Phase 3 — Implement

### Domain types (`crates/core`)
- Pure Rust structs/enums. No Axum, no Leptos, no SQLx.
- `AppError` is the single error type. Add variants as needed.

### Service (`crates/services`)
- `pub struct <Name>Service { ... }` with `impl <Name>Service { pub fn <action>(&self) -> Result<..., AppError> { ... } }`.
- No `async` unless genuinely async. No `println!`. Use `tracing::info!` etc.

### API handler (`apps/api`)
- Extract `State`, call service, map result to `StatusCode` + `Json`.
- Register route in `router()`.
- Add `AppState` field for the new service.

### Web handler (`apps/web`)
- Call service. Pass data to a Leptos `view!` macro. Return `format!("<!DOCTYPE html>{html}")`.
- Tailwind CSS only for styling.

### Configuration (`crates/config`)
- New env var → add to `Config` struct, document in `.env.example`.
- Never add defaults for production secrets.

### Database (`crates/database`)
- SQLx `mysql` driver. Pool from `MINIRUST_DATABASE_URL`.
- Binary must still start when `MINIRUST_DATABASE_URL` is unset.
- Use SQLx macros (`query!`, `query_as!`) for compile-time checked queries.

## Phase 4 — Tests

- Service unit test: assert the returned domain value, not stdout.
- API test: `router().oneshot(request)` — check `status` and deserialized JSON body.
- Web test: `render_*()` — check that the HTML contains expected strings.

## Phase 5 — Validate

Run the `validate-minirust` skill. All five steps must PASS before calling the work done.
