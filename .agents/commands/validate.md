---
description: Run full local validation — fmt, check, clippy, test, build — matching the CI pipeline exactly.
---

Follow the `validate-minirust` skill.

Run each command from the workspace root (`d:\GIT\MiniRust`). Report PASS or FAIL with output for each step.

```powershell
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked --all-targets
cargo build --workspace --locked
```

Do not skip any step. Do not start MariaDB or Redis. Do not claim PASS unless the command actually ran.
