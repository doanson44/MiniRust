---
description: Add a new workspace crate or app binary.
---

Follow the `add-minirust-crate` skill.

Steps:
1. Inspect the workspace tree and `Cargo.toml` members first.
2. Only create the crate if the current feature genuinely needs it (prefer extending `crates/services`).
3. `cargo new --lib crates/<name> --name minirust-<name>` or `cargo new --bin apps/<name> --name minirust-<name>`.
4. Set `version.workspace = true`, `edition.workspace = true`, `license.workspace = true` in the new `Cargo.toml`.
5. Add path to `members` in the root `Cargo.toml`.
6. Use `dep.workspace = true` for all shared dependencies.
7. Keep `crates/core` and `crates/services` free of infrastructure SDKs (Axum, Leptos, SQLx, Teloxide).
8. Run the `validate` command.
