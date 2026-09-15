---
description: Add a new Leptos SSR page to apps/web following the MiniRust view pattern
---

Follow the implement-minirust-feature skill (Web path).

1. Inspect the existing page and web rules before coding.
2. Add a `#[component]` in `apps/web/src/lib.rs` or a focused presentation module.
3. Add a `render_<page>()` function — `.to_html()` prefixed with `<!DOCTYPE html>`.
4. Add an Axum handler returning `(StatusCode::OK, [(CONTENT_TYPE, "text/html; charset=utf-8")], html)`.
5. Register the route in `router()`.
6. Style with Tailwind CSS utility classes only — no Bootstrap and no inline styles.
7. Use a mobile-first responsive layout with `sm:`, `md:`, `lg:`, or `xl:` variants where appropriate. Avoid fixed desktop-only widths and horizontal overflow.
8. Include viewport metadata, semantic landmarks, accessible labels, and visible focus states.
9. Add `#[test]` for render and `#[tokio::test]` for the real router using `oneshot(...)`; assert the page contract and responsive markers.
10. Run the validate command.

SSR only. Call services from `crates/services`. No hydration, no WASM.

Tailwind Play CDN is development-only until the project has a production CSS build pipeline.
