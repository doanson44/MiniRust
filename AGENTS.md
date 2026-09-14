# MiniRust agent guide

This repository is a **runnable Rust baseline**, not the full long-term platform.

Read and follow:

- Rules in `.cursor/rules/`
- Skills in `.cursor/skills/` when adding crates, implementing features, or validating
- GitHub workflow in `.github/workflows/rust.yml` (same checks as local validation)

Frontend standard:

- `apps/web` uses Leptos SSR with Tailwind CSS.
- Tailwind CSS is the only CSS framework for the frontend.
- Do not add Bootstrap or another CSS/UI framework, and do not preserve Bootstrap compatibility.

Do not implement authentication, CMS, MariaDB persistence, Redis, Telegram, AI, or other product features unless the user explicitly asks. Inspect the tree before changing it. Prefer the smallest change that preserves the workspace layout (`apps/*`, `crates/*`).
