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

Do not implement authentication, CMS, MariaDB, Telegram, AI, or other product features unless the user explicitly asks. Inspect the tree before changing it. Prefer the smallest change that preserves the workspace layout (`apps/*`, `crates/*`).
## Dead-code policy

- Never add, retain, or use `dead_code` allowances to bypass, suppress, or silence compiler warnings.
- When code is reported as dead code, fix the underlying issue if the code is required, or remove the unused code if it is not required.
- Do not use `#[allow(dead_code)]`, `#![allow(dead_code)]`, or `cfg_attr(..., allow(dead_code))` as a workaround.
