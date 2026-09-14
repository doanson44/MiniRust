---
name: debug-minirust
description: >-
  Diagnoses and fixes build errors, clippy warnings, test failures, and runtime
  panics in MiniRust. Use when cargo errors, tests fail, the app crashes at
  startup, or the user asks to fix a specific error.
---

# Debug MiniRust

## Step 1 — Identify the error category

| Symptom | Go to section |
|---|---|
| `cargo check` / `cargo build` compilation error | [Compile errors](#compile-errors) |
| `cargo clippy` warnings treated as errors | [Clippy warnings](#clippy-warnings) |
| `cargo test` test failure | [Test failures](#test-failures) |
| Runtime panic / crash on startup | [Runtime panics](#runtime-panics) |
| Wrong HTTP response (status / body) | [HTTP issues](#http-issues) |

## Compile errors

1. Read the **full error message** including the `help:` note — Rust errors are usually self-contained.
2. Check borrow checker errors: prefer cloning the minimum data needed, not wrapping in `Arc` unless shared state is genuinely needed.
3. Trait bound errors: add the bound to the function, not the struct.
4. Lifetime errors: try naming lifetimes before reaching for `'static`.

## Clippy warnings

Run with full context:
```powershell
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1
```
- Fix each warning, do not `#[allow(...)]` unless it is a known false positive.
- Common fixes: `map_err`, `?` instead of `unwrap`, `if let` instead of `match` with one arm.

## Test failures

```powershell
# Run a single test with output
cargo test --workspace -p minirust-<crate> <test_name> -- --nocapture
```
- Check that the test assertion matches the **actual** domain logic, not a stale snapshot.
- API tests: ensure `AppState` is constructed with a valid `Config` (use `Config::default()` or a test fixture).

## Runtime panics

1. Enable full backtraces:
   ```powershell
   $env:RUST_BACKTRACE = "full"
   cargo run -p minirust-api
   ```
2. Look for `unwrap()` / `expect()` on a `None` or `Err` — replace with proper error handling.
3. Check that required env vars are present or have safe defaults.

## HTTP issues

- Verify the route is registered in `router()`.
- Confirm the handler returns the correct `StatusCode`.
- Use `curl -v http://127.0.0.1:3000/path` to see raw response headers and body.
- Check `TraceLayer` logs for request/response details.

## After fixing

Run the `validate-minirust` skill to confirm all checks pass.
