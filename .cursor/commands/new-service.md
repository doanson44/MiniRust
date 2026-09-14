---
description: Add a new service to crates/services — pure Rust, no framework imports
---

Follow the implement-minirust-feature skill (Services path).

1. Add domain types to `crates/core/src/lib.rs` if needed.
2. Add service struct in `crates/services/src/lib.rs`:
   `#[derive(Debug, Clone, Copy, Default)] pub struct MyService;`
3. Implement methods returning `Result<Output, AppError>`.
4. Write `#[cfg(test)]` unit tests asserting behavior.
5. Wire service into `AppState` in the relevant app crate.
6. Run the validate command.

No async unless truly needed. No println!. No framework imports.
