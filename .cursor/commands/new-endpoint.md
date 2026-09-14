---
description: Add a new REST endpoint to apps/api following the MiniRust handler pattern
---

Follow the implement-minirust-feature skill (API path).

1. Add domain type to `crates/core` if needed (no frameworks).
2. Add service method to `crates/services` if needed.
3. Add Axum handler in `apps/api/src/lib.rs` — extract State, call service, return `(StatusCode, Json(...))`.
4. Register route in `router()`.
5. Add `AppState` field for new service.
6. Add `#[tokio::test]` using `router().oneshot(request)`.
7. Run the validate command.

No business logic in handlers. No Axum/Leptos in `crates/services` or `crates/core`.
