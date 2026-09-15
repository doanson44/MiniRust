---
name: validate-minirust
description: Runs MiniRust workspace validation matching CI.
---

# Validate MiniRust

Run every check from the workspace root. Do not claim PASS unless the command actually ran.

```text
- [ ] cargo fmt --all -- --check
- [ ] cargo check --workspace --locked
- [ ] cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
- [ ] cargo test --workspace --locked --all-targets
- [ ] cargo build --workspace --locked
```

The exact CI sequence is defined by `.github/workflows/rust.yml`.

## Runtime smoke test

Only when runtime verification is requested:

1. `cargo run -p minirust-api` → `/health`, `/api/v1/hello`, and `/api/v1/echo`.
2. `cargo run -p minirust-web` → `/` and `/health`.
3. Stop the processes when verification is complete.

The API currently requires MariaDB through `MINIRUST_DATABASE_URL` at startup. Use Docker Compose when a real database is required.
