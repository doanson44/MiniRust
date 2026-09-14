---
description: Rust coding style, error handling, logging, and test conventions for MiniRust. Applied to all *.rs files.
trigger: model_decision
globs: ["**/*.rs"]
---

# Rust Style — MiniRust

## Guiding principle

Prefer **explicit, boring Rust**. Avoid extra generics, macros, `unsafe`, excessive `clone`, `Arc`, `Mutex`, or trait objects unless genuinely required.

## Error handling

- Use `Result<T, AppError>` from `minirust-core`. Do not `panic!` or `unwrap()` on normal application paths.
- Map `AppError` to HTTP status in the app crate (handler layer), never inside `core` or `services`.
- Propagate errors with `?`. Add context with `.map_err(|e| AppError::from(e))` when needed.

## Logging

- Use `tracing` macros only (`tracing::info!`, `tracing::warn!`, `tracing::error!`).
- No `println!`, `eprintln!`, or `dbg!` in production code paths.
- Initialize tracing exactly once in `main`. Never re-initialize in library code.
- Log operational facts (startup addresses, feature flags as booleans). Never log secrets, passwords, or connection strings.

## Async

- Use `tokio::spawn` for background tasks. Prefer structured concurrency.
- Keep `async fn` at the handler / I/O boundary; pure domain logic stays `fn`.

## Imports

- Group std, external crates, then internal crates with blank lines separating groups.
- Prefer explicit paths over glob imports (`use std::collections::HashMap` not `use std::*`).

## Code examples

```rust
// BAD - framework and I/O inside a service
pub async fn hello() -> String {
    println!("hello");
    panic!("missing config");
}

// GOOD - pure service, caller maps to HTTP/HTML
pub fn hello(&self) -> Greeting {
    Greeting::new(format!("Hello from {APP_NAME}"))
}
```

## Tests

- Test behavior, not line coverage.
- API tests: use `axum::Router::oneshot()` — not a live process.
- Unit tests live in the same file in `#[cfg(test)]` modules.
- Integration tests go in `tests/` subdirectory of the crate.
