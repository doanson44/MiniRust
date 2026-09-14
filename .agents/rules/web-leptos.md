---
description: Leptos SSR web app conventions for apps/web. Applied when editing apps/web files.
trigger: model_decision
globs: ["apps/web/**"]
---

# Web — `minirust-web` (Leptos SSR)

## Configuration

- SSR-only: `features = ["ssr"]`. Do **not** add hydration, `cargo-leptos`, or WASM unless explicitly asked.
- Served by an internal Axum server. Bind from `minirust_config` (`ServerKind::Web`). Default: `127.0.0.1:3001`.

## Baseline routes

| Method | Path | Response |
|---|---|---|
| GET | `/` | `<!DOCTYPE html>` full HTML page |
| GET | `/health` | JSON or plain health status |

## Rendering pattern

```rust
pub fn render_home_page(message: &str) -> String {
    let html = view! { <HomePage message=message.to_owned()/> }.to_html();
    format!("<!DOCTYPE html>{html}")
}
```

- Prefix every rendered page with `<!DOCTYPE html>`.
- Components go in `src/components/`. Pages go in `src/pages/`.
- Call `GreetingService` / `HealthService` from `crates/services`. Do not duplicate business logic.

## CSS — Tailwind CSS only

- Tailwind CSS is the **only** CSS framework.
- Do not add Bootstrap, DaisyUI, or other CSS/UI frameworks.
- Use Tailwind utility classes directly in `view!` macros.
- Small reusable Leptos components over large CSS abstractions.

## What not to build (yet)

Do not create CMS, auth UI, dashboards, admin panels, or resume tools unless the user explicitly requests them.
