---
description: Run the exact CI pipeline with --locked — definitive pre-commit check
---

Mirrors `.github/workflows/rust.yml` exactly:

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked --all-targets
cargo build --workspace --locked
```

All 5 steps must PASS before pushing. If --locked fails due to Cargo.lock drift, run `cargo update` and commit the lock file first.
