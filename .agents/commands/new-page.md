---
description: Add a new Leptos SSR page to apps/web — component, render function, route, and integration test.
---

Follow the `implement-minirust-feature` skill, Web path.

Steps:
1. Inspect `apps/web/src/lib.rs` and the current web rule before changing the page.
2. Define the page structure with semantic landmarks and accessible labels.
3. Add a new Leptos `#[component]` in `apps/web/src/lib.rs` or a focused presentation module.
4. Add a `render_<page>()` function that calls `.to_html()` and prefixes `<!DOCTYPE html>`.
5. Add an Axum handler that calls the render function and returns `(StatusCode::OK, [(CONTENT_TYPE, "text/html; charset=utf-8")], html)`.
6. Register the route in `router()`.
7. Build the layout mobile-first with Tailwind utility classes. Use responsive breakpoint variants for typography, spacing, grids, navigation, and actions. Do not introduce Bootstrap, inline styles, or fixed desktop-only layouts.
8. Include the viewport meta tag and visible keyboard focus states. Avoid horizontal overflow at narrow widths.
9. Add a `#[test]` for the render function and a `#[tokio::test]` that exercises the real router with `oneshot(...)`. Assert the page contract and representative responsive markers.
10. Run the `validate` command.

SSR only — do not add hydration, WASM, or `cargo-leptos`. Call services from `crates/services`, not duplicated logic.

If Tailwind does not yet have a production asset pipeline, browser/CDN Tailwind may be used for development only; do not describe it as production-ready.
