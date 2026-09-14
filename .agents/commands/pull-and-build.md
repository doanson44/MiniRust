---
description: Pull latest code on the current branch, build the workspace, and iteratively fix compile errors, clippy warnings, and test failures until clean.
---

Follow the `pull-and-build` skill exactly.

Execute all 6 phases in order:
1. Check working tree — stash if dirty.
2. `git pull --ff-only` on the current branch.
3. Smart Build loop — `cargo build --workspace`, fix errors, repeat up to 3 times.
4. Clippy — `cargo clippy --workspace --all-targets --all-features -- -D warnings`, fix warnings.
5. Tests — `cargo test --workspace --all-targets`.
6. Print the Pull & Build Report.

Do not push. Do not auto-resolve merge conflicts. Do not suppress clippy warnings with `#[allow(...)]`.
