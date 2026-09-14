---
description: Add a new service to crates/services — pure Rust business logic, no framework imports.
---

Follow the `implement-minirust-feature` skill, Services path.

Steps:
1. Inspect `crates/services/src/lib.rs` — read existing service patterns (`GreetingService`, `HealthService`, `EchoService`).
2. Add domain types to `crates/core/src/lib.rs` if needed:
   - Plain Rust struct/enum.
   - No Axum, Leptos, SQLx, or Teloxide.
   - Follow the `AppError` pattern for validation.
3. Add the service struct in `crates/services/src/lib.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, Default)]
   pub struct MyService;

   impl MyService {
       pub fn action(&self, input: MyInput) -> Result<MyOutput, AppError> {
           // business logic here
       }
   }
   ```
4. Export the new service from `crates/services/src/lib.rs` (it is `pub` by default if in the module root).
5. Add `#[cfg(test)]` unit tests in the same file asserting behavior.
6. Wire the service into `AppState` in `apps/api/src/lib.rs` and/or `apps/web/src/lib.rs` if needed.
7. Run the `validate` command.

No `async` in services unless genuinely async. No `println!` — use `tracing::info!`. No framework imports.
