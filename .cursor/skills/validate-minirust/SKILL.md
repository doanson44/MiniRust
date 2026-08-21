---
name: validate-minirust
description: Runs MiniRust workspace validation (fmt, check, clippy, test) and optional API/SSR smoke checks. Use when validating, linting, checking CI locally, finishing a change, or the user asks to format, clippy, test, or verify the baseline.
---

# Validate MiniRust

Copy this checklist and complete it. Do not claim PASS unless the command actually ran.

```text
- [ ] cargo fmt --all -- --check
- [ ] cargo check --workspace
- [ ] cargo clippy --workspace --all-targets --all-features -- -D warnings
- [ ] cargo test --workspace --all-targets
```

Run from the repository root. Fix failures before continuing.

## Optional runtime smoke (when the user wants run verification)

1. `cargo run -p minirust-api` → `GET http://127.0.0.1:3000/health` and `/api/v1/hello`
2. `cargo run -p minirust-web` → `GET http://127.0.0.1:3001/` (HTML) and `/health`
3. Stop the processes when done. Do not leave servers running unless asked.

SQL Server and Redis are not required. Do not start `docker compose --profile infra` for this workflow.

CI equivalent: `.github/workflows/rust.yml` (adds `--locked` and `cargo build`).
