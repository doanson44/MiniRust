---
description: Add a new REST endpoint to apps/api — handler, route registration, optional service method, and test.
---

Follow the `implement-minirust-feature` skill, API path.

Steps:
1. Inspect `apps/api/src/lib.rs` — read the existing handler pattern and `AppState`.
2. Add domain type to `crates/core` if needed (plain Rust struct, no frameworks).
3. Add service method to `crates/services` if needed (`pub fn <action>(&self) -> Result<..., AppError>`).
4. Add the Axum handler in `apps/api/src/lib.rs`:
   - Extract `State(state): State<AppState>`
   - Call the service
   - Return `(StatusCode, Json(ResponseStruct))`
5. Register the route in `router()` in `apps/api/src/lib.rs`.
6. Add `AppState` field for any new service.
7. Add a `#[tokio::test]` using `router().oneshot(request)` that asserts the status code and JSON body.
8. Run the `validate` command.

Do not put business logic in the handler. Do not import Axum or Leptos into `crates/services` or `crates/core`.
