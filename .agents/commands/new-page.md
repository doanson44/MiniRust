---
description: Add a new Leptos SSR page to apps/web — component, render function, route, and test.
---

Follow the `implement-minirust-feature` skill, Web path.

Steps:
1. Inspect `apps/web/src/lib.rs` — read existing component and render patterns.
2. Add a new Leptos `#[component]` in `apps/web/src/lib.rs` (or a new file under `src/`):
   ```rust
   #[component]
   fn MyPage(prop: String) -> impl IntoView {
       view! { <html lang="en">...</html> }
   }
   ```
3. Add a `render_<page>()` function that calls `.to_html()` and prefixes `<!DOCTYPE html>`.
4. Add an Axum handler that calls the render function and returns `(StatusCode::OK, [(CONTENT_TYPE, "text/html; charset=utf-8")], html)`.
5. Register the route in `router()` in `apps/web/src/lib.rs`.
6. Style with **Tailwind CSS utility classes only** — no inline styles, no Bootstrap.
7. Add a `#[test]` for the render function and a `#[tokio::test]` for the route.
8. Run the `validate` command.

SSR only — do not add hydration, WASM, or `cargo-leptos`. Call services from `crates/services`, not duplicated logic.
