---
description: Leptos SSR web app conventions for apps/web.
trigger: model_decision
globs: ["apps/web/**"]
---

# Web — `minirust-web` (Leptos SSR)

## Configuration

- SSR-only: `features = ["ssr"]`.
- Do not add hydration, `cargo-leptos`, or WASM unless explicitly requested.
- Served by an internal Axum server on `ServerKind::Web`, default `127.0.0.1:3001`.

## Data flow

```text
SSR handler → Query → Query handler → read DTO → Leptos view
```

SSR code must not duplicate business rules or query another context's persistence directly.

## Rendering

- Prefix rendered pages with `<!DOCTYPE html>`.
- Components belong in presentation modules.
- Query results are passed into views as presentation data.

## CSS — Tailwind CSS only

- Tailwind CSS is the only CSS framework.
- Do not add Bootstrap, DaisyUI, or another CSS/UI framework.
- Prefer utility classes and small reusable Leptos components.

## Routes

| Method | Path | Purpose |
|---|---|---|
| GET | `/` | SSR home page |
| GET | `/health` | Web process health |
