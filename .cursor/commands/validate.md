---
description: Format, check, clippy, and test the MiniRust workspace
---

Run the validate-minirust skill from the repository root.

Execute, then report each command and its result:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

Do not start SQL Server or Redis. Do not claim success for a command that did not run.
