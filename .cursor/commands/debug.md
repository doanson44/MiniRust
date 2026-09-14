---
description: Debug build errors, clippy warnings, test failures, or runtime panics
---

Follow the debug-minirust skill.

1. Identify error category: compile / clippy / test failure / runtime panic / wrong HTTP response.
2. Read the full Rust error — the `help:` note usually contains the fix.
3. Fix: propagate with `?`, use `tracing` not `println!`, fix types at call sites.
4. Confirm: `cargo build --workspace 2>&1`
5. Run the validate command to ensure the full suite passes.

Do not `#[allow(...)]` warnings. Do not `unwrap()` on production paths. Report full error context after 3 failed attempts.
