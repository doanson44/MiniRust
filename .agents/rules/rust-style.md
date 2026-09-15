---
description: Rust coding style, error handling, logging, and test conventions for MiniRust.
trigger: model_decision
globs: ["**/*.rs"]
---

# Rust Style — MiniRust

## Guiding principle

Prefer explicit, boring Rust. Avoid extra generics, macros, `unsafe`, excessive `clone`, `Arc`, `Mutex`, or trait objects unless genuinely required.

## Error handling

- Use `Result<T, AppError>` from `minirust-core` where application/domain errors are appropriate.
- Map application/domain errors to HTTP or UI responses at the transport boundary.
- Propagate errors with `?`.
- Do not expose infrastructure details such as SQL errors to external callers.

## CQRS

- Command and query handlers are explicit types.
- Commands express intent; queries describe information needs.
- Keep pure domain logic synchronous unless it genuinely requires asynchronous I/O.
- Do not mix command mutation with query projection logic.

## Logging

- Use `tracing` macros only.
- No `println!`, `eprintln!`, or `dbg!` in production paths.
- Initialize tracing exactly once in application binaries.
- Never log secrets, credentials, or connection strings.

## Imports

Group std, external crates, then internal crates with blank lines.

## Tests

- Test behavior, not line coverage.
- API tests use `Router::oneshot()`.
- Command/query unit tests live with their handlers.
- Integration tests belong in the crate `tests/` directory.
