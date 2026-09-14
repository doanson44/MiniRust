---
name: add-minirust-crate
description: >-
  Adds a new workspace crate or app the correct way (cargo new, workspace deps,
  no empty speculative packages). Use when creating a new crate, adding
  apps/bot, apps/worker, or a new crates/* package, or when scaffolding a new
  module.
---

# Add a MiniRust Crate

## Before creating anything

1. Run `Get-ChildItem -Recurse -Filter "Cargo.toml"` and inspect the workspace tree.
2. Only create a crate if the current feature genuinely needs it.
3. Prefer extending `crates/services` with a new service over adding a new crate.

## Crate types

| Kind | Path pattern | Package name | Role |
|---|---|---|---|
| Binary (app) | `apps/<name>/` | `minirust-<name>` | Process entry point |
| Library | `crates/<name>/` | `minirust-<name>` | Shared code / adapters |

**Allowed future names** (create only when implementing, not before):
`ai`, `auth`, `cache`, `files`, `market`, `monitoring`, `notification`, `resume`, `scheduler`, `tools`, `ui`, `websocket`

## Steps

1. **Create the crate**
   ```powershell
   # Library
   cargo new --lib crates/<name> --name minirust-<name>
   # Binary
   cargo new --bin apps/<name> --name minirust-<name>
   ```

2. **Set workspace inheritance** in the new `Cargo.toml`:
   ```toml
   [package]
   name = "minirust-<name>"
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   ```

3. **Add to workspace** — append to `members` in root `Cargo.toml`:
   ```toml
   "apps/<name>",   # or "crates/<name>"
   ```

4. **Add dependencies** using workspace versions:
   ```toml
   [dependencies]
   tokio.workspace = true
   ```
   Use `cargo add --package minirust-<name> <dep>` when the dep is already in workspace.
   For new deps, add the version to `[workspace.dependencies]` in the root first.

5. **Respect layering**:
   - Domain types → `crates/core`
   - Business rules → `crates/services`
   - I/O, framework code → the new crate or app

6. **Never** import Axum, Leptos, SQLx, or Teloxide into `crates/core` or `crates/services`.

7. **Run `validate-minirust`** skill after creation.

## After creation

Follow the `implement-minirust-feature` skill for adding actual code.
