---
description: Debug build errors, clippy warnings, test failures, or runtime panics in MiniRust.
---

Follow the `debug-minirust` skill.

Steps:
1. Identify the error category: compile error / clippy warning / test failure / runtime panic / wrong HTTP response.
2. Read the full error output — Rust errors include the fix in `help:` notes.
3. Fix using the layering rules: fix types at the call site, propagate with `?`, use `tracing` not `println!`.
4. Rebuild to confirm the fix:
   ```powershell
   cargo build --workspace 2>&1
   ```
5. Run the `validate` command to ensure the full suite passes.

Do not `#[allow(...)]` warnings. Do not `unwrap()` on production paths. If stuck after 3 attempts, report the full error context and ask the user.
