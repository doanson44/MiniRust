---
description: Pull latest code on the current branch, build the workspace, and iteratively fix any compile errors or clippy warnings until clean
---

Follow the pull-and-build skill in `.agents/skills/pull-and-build/SKILL.md`.

Phase 1 — Check working tree (`git status`). Stash dirty changes if any.
Phase 2 — Pull: `git pull --ff-only`. Stop on merge conflicts and report to user.
Phase 3 — Format & Check: `cargo fmt --all` and `cargo check --workspace`.
Phase 4 — Smart Build: `cargo build --workspace`. Fix compile errors iteratively (max 3 rounds). Escalate if still broken.
Phase 5 — Clippy: `cargo clippy --workspace --all-targets --all-features -- -D warnings`. Fix warnings.
Phase 6 — Tests: `cargo test --workspace --all-targets`. Report failures.
Phase 7 — Print the Pull & Build Report summary.

Do NOT push. Do NOT auto-resolve merge conflicts. Do NOT add `#[allow(clippy::...)]` to suppress warnings.
