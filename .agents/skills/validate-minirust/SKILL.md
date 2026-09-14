---
name: validate-minirust
description: >-
  Runs MiniRust workspace validation (fmt, check, clippy, test, build) matching
  the CI pipeline in .github/workflows/rust.yml. Use when finishing a change,
  validating before commit, or the user asks to format, lint, clippy, test, or
  verify the baseline.
---

# Validate MiniRust

Run all checks from the **workspace root** (`d:\GIT\MiniRust`). Fix every failure before continuing.

## Full CI checklist

Copy this checklist and mark each step PASS/FAIL with the actual command output:

```
[ ] 1. cargo fmt --all -- --check
[ ] 2. cargo check --workspace --locked
[ ] 3. cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
[ ] 4. cargo test --workspace --locked --all-targets
[ ] 5. cargo build --workspace --locked
```

This is the exact sequence run by `.github/workflows/rust.yml`.

> **Important**: Do not claim PASS unless the command actually ran and produced zero errors. Show the command output.

## Quick local check (no --locked)

When iterating during development and Cargo.lock has changed:

```
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

## Optional runtime smoke test

Run only when the user wants to verify runtime behavior:

1. `cargo run -p minirust-api` → test `GET http://127.0.0.1:3000/health` and `GET http://127.0.0.1:3000/api/v1/hello`
2. `cargo run -p minirust-web` → test `GET http://127.0.0.1:3001/` (HTML) and `GET http://127.0.0.1:3001/health`
3. Stop all processes when done. Do not leave servers running unless the user asks.

## Notes

- MariaDB and Redis are **not required** for validation. Do not start `docker compose --profile infra`.
- If a test requires a database, it must use an in-process mock or be behind `#[cfg(feature = "integration")]`.
- Fix `clippy` warnings before moving on — CI treats them as errors (`-D warnings`).
