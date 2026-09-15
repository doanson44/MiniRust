---
name: validate-minirust
description: Runs MiniRust workspace validation and optional API/SSR smoke checks.
---

# Validate MiniRust

Run the complete CI-equivalent checklist from the repository root. Do not claim PASS unless commands actually ran.

```text
- [ ] cargo fmt --all -- --check
- [ ] cargo check --workspace --locked
- [ ] cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
- [ ] cargo test --workspace --locked --all-targets
- [ ] cargo build --workspace --locked
```

## Runtime smoke

When requested:

1. `cargo run -p minirust-api` → `/health`, `/api/v1/hello`, `/api/v1/echo`.
2. `cargo run -p minirust-web` → `/` and `/health`.
3. Stop all processes when verification is complete.

The API currently requires MariaDB through `MINIRUST_DATABASE_URL` at startup.

CI equivalent: `.github/workflows/rust.yml`.
