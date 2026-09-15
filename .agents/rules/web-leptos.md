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
- All public pages must include the viewport meta tag.

## Responsive UI

- All user-facing pages MUST be responsive across mobile, tablet, and desktop.
- Use mobile-first Tailwind utilities and breakpoint variants (`sm:`, `md:`, `lg:`, `xl:`) where needed.
- Avoid fixed widths that create horizontal scrolling on narrow screens.
- Responsive navigation, grids, typography, spacing, and CTA layouts must be intentional rather than relying on browser wrapping by accident.
- Keep interactive controls touch-friendly and provide visible keyboard focus states.
- Prefer semantic HTML and accessible labels/landmarks.
- Verify responsive behavior at representative small, medium, and large viewport widths when browser tooling is available.

## CSS — Tailwind CSS only

- Tailwind CSS is the only CSS framework.
- Do not add Bootstrap, DaisyUI, or another CSS/UI framework.
- Prefer utility classes and small reusable Leptos components.
- Tailwind Play CDN is acceptable only while the web app has no production CSS build pipeline; it is not a production deployment strategy.

## Routes

| Method | Path | Purpose |
|---|---|---|
| GET | `/` | SSR landing page |
| GET | `/health` | Web process health |
