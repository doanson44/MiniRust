# MiniRust — Antigravity Agent Guide

This repository is a **runnable Rust baseline**, not the full long-term product platform.

## Rules (always loaded)

- `.agents/rules/architecture.md` — stack, workspace layout, layering, scope guard, hard rules
- `.agents/rules/rust-style.md` — Rust coding style, error handling, logging, tests
- `.agents/rules/api-axum.md` — Axum handler/routing conventions (`apps/api/**`)
- `.agents/rules/web-leptos.md` — Leptos SSR conventions (`apps/web/**`)
- `.agents/rules/cargo-workspace.md` — Cargo workspace and dependency conventions
- `.agents/rules/config-secrets.md` — Environment configuration and secret handling

## Skills (on-demand)

| Skill | When to activate |
|---|---|
| `validate-minirust` | After any code change; before calling work done |
| `implement-minirust-feature` | Adding endpoints, pages, services, or adapters |
| `add-minirust-crate` | Creating a new crate or app binary |
| `debug-minirust` | Diagnosing build errors, test failures, runtime panics |
| `pull-and-build` | Pull latest code, build, and auto-fix errors until workspace is clean |

## Commands (slash commands)

| Command | Description |
|---|---|
| `pull-and-build` | Pull → build → fix errors → clippy → test |
| `validate` | Full CI validation (fmt, check, clippy, test, build) |
| `check-ci` | Exact CI run with `--locked` — use before push |
| `fmt` | Auto-format with rustfmt + fix clippy warnings |
| `test` | Run test suite only |
| `debug` | Diagnose and fix errors |
| `implement-feature` | Implement a feature through correct layers |
| `new-endpoint` | Add a REST endpoint to `apps/api` |
| `new-page` | Add a Leptos SSR page to `apps/web` |
| `new-service` | Add a service to `crates/services` |
| `add-crate` | Add a workspace crate or app binary |
| `run` | Start API/web servers for smoke testing |
| `inspect` | Inspect workspace layout and public API surface |

## CI pipeline

`.github/workflows/rust.yml` runs on every push/PR to `master`:

```
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked --all-targets
cargo build --workspace --locked
```

Local validation must match CI exactly (use `--locked`).

## Frontend standard

- `apps/web` uses **Leptos SSR** with **Tailwind CSS**.
- Tailwind CSS is the **only** CSS framework. No Bootstrap or other frameworks.

## Scope

Do not implement authentication, CMS, MariaDB persistence, Telegram bot, AI, or other product features unless the user explicitly asks.

Always inspect the actual file tree before making changes. Prefer the smallest change that preserves the workspace layout.
