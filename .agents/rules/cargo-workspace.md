---
description: Cargo workspace and dependency conventions. Applied when editing Cargo.toml files.
trigger: model_decision
globs: ["**/Cargo.toml"]
---

# Cargo Workspace — MiniRust

## Structure

- Root `Cargo.toml` declares all members and is the **single source of shared versions** in `[workspace.dependencies]`.
- All packages use `version.workspace = true`, `edition.workspace = true`, `license.workspace = true`.
- Package names follow the pattern `minirust-<name>`.

## Adding a dependency

1. Add the version to `[workspace.dependencies]` in the root `Cargo.toml`.
2. Reference it in the crate with `dep.workspace = true` (no version in the crate).
3. Prefer `cargo add --package minirust-<name> <dep>` over hand-editing when possible.

## Adding a crate or app

- Apps (binaries): `apps/<name>/` → package name `minirust-<name>`
- Libraries (shared): `crates/<name>/` → package name `minirust-<name>`
- Add the new path to `members` in the root `Cargo.toml`.
- Do not create speculative empty crates. Create only when implementing the feature.

## Forbidden until explicitly in scope

Do **not** add: `tiberius`, `redis`, `teloxide`, JWT crates, AI/ML crates.

## CI compliance

- `Cargo.lock` must stay consistent — CI runs with `--locked`.
- After changing dependencies, run `cargo check --workspace --locked` to verify.
- Edition: **2021**. Keep `resolver = "2"`.
