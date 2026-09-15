# MiniRust — Agent Guide

MiniRust is a runnable Rust baseline evolving toward a **CQRS-oriented modular monolith** designed for many bounded contexts and a large engineering team.

## Rules

- `.agents/rules/architecture.md` — architecture, CQRS boundaries, technology stack, and scope
- `.agents/rules/rust-style.md` — Rust style, errors, logging, and tests
- `.agents/rules/api-axum.md` — Axum transport conventions
- `.agents/rules/web-leptos.md` — Leptos SSR conventions
- `.agents/rules/cargo-workspace.md` — workspace and dependency conventions
- `.agents/rules/config-secrets.md` — configuration and secret handling

## Architecture documents

- `docs/architecture/README.md` — architecture blueprint and evolution path
- `docs/architecture/cqrs.md` — CQRS rules
- `docs/architecture/team-development.md` — team and bounded-context rules

## Skills

| Skill | When to activate |
|---|---|
| `validate-minirust` | After code changes and before calling work done |
| `implement-minirust-feature` | Adding a feature, endpoint, page, command, query, repository, or adapter |
| `add-minirust-crate` | Creating a new workspace crate or app |
| `debug-minirust` | Diagnosing build or runtime failures |
| `pull-and-build` | Pulling latest code and validating the workspace |

## Commands

| Command | Description |
|---|---|
| `pull-and-build` | Pull → build → fix → clippy → test |
| `validate` | Full CI validation |
| `check-ci` | Exact CI validation with `--locked` |
| `fmt` | Format and address clippy warnings |
| `test` | Run tests |
| `debug` | Diagnose failures |
| `implement-feature` | Implement a feature through the correct CQRS/domain layers |
| `new-endpoint` | Add an Axum transport endpoint |
| `new-page` | Add a Leptos SSR page |
| `add-crate` | Add a workspace crate or app |
| `run` | Run API/web smoke tests |
| `inspect` | Inspect workspace and architecture boundaries |

## Team-scale rules

- Identify the owning bounded context before implementing a feature.
- Classify application operations as commands or queries.
- Keep transport handlers thin.
- Do not access another context's database tables, repositories, or domain internals.
- Prefer explicit contracts over shared business models.
- Do not add a broker, separate read database, or Event Sourcing without a concrete requirement.

## CI

`.github/workflows/rust.yml` is authoritative for validation:

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked --all-targets
cargo build --workspace --locked
```

Never claim PASS unless the commands actually ran.

## Frontend

- Leptos SSR + Tailwind CSS.
- Tailwind CSS is the only frontend CSS framework.
- No Bootstrap compatibility layer.

Always inspect the actual repository before changing it. GitHub `master` is the source of truth.
