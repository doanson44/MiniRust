---
description: Run the exact CI pipeline locally with --locked flags — the definitive pre-commit check.
---

This mirrors `.github/workflows/rust.yml` exactly. Use this before pushing or opening a PR.

Run from the workspace root:

```powershell
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked --all-targets
cargo build --workspace --locked
```

Report each step with PASS or FAIL. If `--locked` fails due to Cargo.lock drift, run `cargo update` and commit the updated `Cargo.lock` first.

Do not push unless all 5 steps pass.
