---
description: Add a new REST endpoint to apps/api — handler, route registration, optional service method, and integration test.
---

Follow the `implement-minirust-feature` skill, API path, and endpoint integration-testing rules.

Steps:
1. Inspect `apps/api/src/lib.rs` — read the existing handler pattern and `AppState`.
2. Add domain type to `crates/core` if needed (plain Rust struct, no frameworks).
3. Add service method to `crates/services` if needed (`pub fn <action>(&self) -> Result<..., AppError>`).
4. Add the Axum handler in `apps/api/src/lib.rs`:
   - Extract `State(state): State<AppState>`
   - Call the service
   - Return the standardized response contract.
5. Register the route in `router()` in `apps/api/src/lib.rs`.
6. Add `AppState` field for any new service.
7. **Add an integration test for the endpoint in the same change.** The test MUST call `router().oneshot(request)` and exercise the real Axum router.
8. Assert the HTTP method/route behavior, status code, response contract, and meaningful validation/error cases.
9. If the endpoint depends on MariaDB, run it against the Docker-backed MariaDB integration environment.
10. Run the `validate` command.
11. Before completion, confirm that every newly registered or modified endpoint has integration-test coverage.

Do not put business logic in the handler. Do not import Axum or Leptos into `crates/services` or `crates/core`.
