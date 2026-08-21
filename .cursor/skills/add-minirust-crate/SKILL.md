---
name: add-minirust-crate
description: Adds a MiniRust workspace crate or app the correct way (cargo new, workspace deps, no empty speculative packages). Use when creating a crate, adding apps/bot, apps/worker, or a new crates/* package, or when the user asks to scaffold a module.
---

# Add a MiniRust crate

Inspect `Cargo.toml` members and the tree first. Create a crate only if this change needs it.

## Apps vs crates

| Kind | Path | Name | Role |
| --- | --- | --- | --- |
| Binary | `apps/<name>` | `minirust-<name>` | process entry (`api`, `web`, later `bot`, `worker`) |
| Library | `crates/<name>` | `minirust-<name>` | shared code |

Allowed future crate names (create when implementing, not before): `ai`, `auth`, `cache`, `database`, `files`, `market`, `monitoring`, `notification`, `resume`, `scheduler`, `tools`, `ui`, `websocket`.

## Steps

1. `cargo new --lib crates/<name> --name minirust-<name>` or `cargo new --bin apps/<name> --name minirust-<name>`
2. Set `version.workspace`, `edition.workspace`, `license.workspace` in the package `Cargo.toml`
3. Depend with `crate.workspace = true`; add versions only in the root `[workspace.dependencies]`
4. Use `cargo add` from that package directory when possible
5. Put domain types in `core`, business rules in `services`, I/O in the new crate or app
6. `core` / `services` must stay free of Axum, Leptos, Tiberius, Redis, Teloxide, and vendor SDKs
7. Run the `validate-minirust` skill

Do not add database/Redis/Telegram connections to `main` unless that process cannot start without them.
