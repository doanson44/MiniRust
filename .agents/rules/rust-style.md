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

## Comments

- Default is no comments. A comment is allowed only when it is genuinely necessary — it records a
  non-obvious constraint, trade-off, or bug workaround that the code itself cannot express — or when
  the user explicitly asks for comments.
- Never restate what the code does, narrate a change, or explain obvious control flow.
- Never keep commented-out code; delete it, git keeps the history.
- No banner or section-divider comments.
- Comments and doc comments are English.

## Imports

Group std, external crates, then internal crates with blank lines.

## Tests

- Test behavior, not line coverage.
- API tests use `Router::oneshot()`.
- Command/query unit tests live with their handlers.
- Integration tests belong in the crate `tests/` directory.
## Dead code

- Never use `dead_code` allowances to bypass, suppress, or silence compiler warnings.
- If code is required, fix the underlying reason it is reported as dead code; if it is not required, remove it.
- Never introduce `#[allow(dead_code)]`, `#![allow(dead_code)]`, or `cfg_attr(..., allow(dead_code))` as a workaround.
