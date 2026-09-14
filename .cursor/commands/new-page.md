---
description: Add a new Leptos SSR page to apps/web following the MiniRust view pattern
---

Follow the implement-minirust-feature skill (Web path).

1. Add a `#[component]` in `apps/web/src/lib.rs`.
2. Add a `render_<page>()` function — `.to_html()` prefixed with `<!DOCTYPE html>`.
3. Add an Axum handler returning `(StatusCode::OK, [(CONTENT_TYPE, "text/html; charset=utf-8")], html)`.
4. Register route in `router()`.
5. Style with Tailwind CSS utility classes only — no Bootstrap, no inline styles.
6. Add `#[test]` for render and `#[tokio::test]` for the route.
7. Run the validate command.

SSR only. Call services from `crates/services`. No hydration, no WASM.
