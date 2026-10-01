---
name: create-minirust-page
description: >-
  Creates a MiniRust Leptos SSR page end to end: page component, layout choice,
  route registration, data loading, responsive list/form UI, locale text, menu
  migration, and router integration test. Use when adding a list, table, detail,
  or create/edit form page to apps/web.
argument-hint: "Page name and shape, e.g. 'Invoices list' or 'Create invoice form'"
---

# Create a MiniRust Page

Follows `.agents/rules/web-leptos.md`. If the page also needs transport or
application-layer work, run `implement-minirust-feature` first.

## 1. Decide the shape

| Shape | When | Mechanism |
|---|---|---|
| Public | Reachable without a session (`/login`, `/register`) | `AuthLayout`, listed in `is_auth_page` |
| Authenticated | Requires a session (`/app`, `/profile`) | `<ParentRoute>` with `AppLayout` |
| Admin | Requires `actor.is_admin` | Path under `/admin/...`; middleware enforces the role |

Route access is decided by `auth_guard` in `apps/web/src/lib.rs`. A new path is
only protected if it is added there — the default for an unknown path is "pass
through".

## 2. Register the page

1. Create `apps/web/src/pages/<name>.rs` with one `#[component]`.
2. Export it from `apps/web/src/pages/mod.rs`.
3. Register the route in `App()` in `apps/web/src/lib.rs`:
   ```rust
   <ParentRoute path=path!("/invoices") view=AppLayout>
       <Route path=path!("") view=InvoiceListPage/>
   </ParentRoute>
   ```
4. Add the new protected prefix to `auth_guard` when the page requires a session.

## 3. Load data

The SSR shell renders the page; page data arrives from the API.

```rust
let (items, set_items) = signal(Vec::<InvoiceResponse>::new());

#[cfg(feature = "hydrate")]
{
    leptos::task::spawn_local(async move {
        if let Ok(response) =
            api_json::<InvoiceListResponse>(gloo_net::http::Method::GET, "/api/v1/invoices", None).await
        {
            set_items.set(response.items);
        }
    });
}
```

Rules:

- Every hydrate-only block is wrapped in `#[cfg(feature = "hydrate")]` so the SSR
  build stays free of `gloo_net`.
- Use `api_json` / `api_json_with_meta` / `api_empty` from `apps/web/src/api.rs`.
  They unwrap the `{ "data": ... }` envelope and surface `ProblemDetails.detail`.
- Never call SQLx or a repository from a page.

## 4. Localized copy

`AppLayout` provides the active locale as context. Read it and pair both languages
at the call site:

```rust
let locale = use_context::<ReadSignal<String>>().unwrap_or_else(|| signal("vi".to_owned()).0);
let text = move |vi: &'static str, en: &'static str| {
    move || if locale.get() == "vi" { vi } else { en }
};
```

- Both languages are user-facing strings — this is allowed and expected.
- Code comments, identifiers, and log messages stay English.

## 5. List page pattern

- Render one collection twice: a table for `md:` and up, a stacked card list below
  it. Do not rely on a horizontally scrolling table on phones.
  ```rust
  <div class="hidden md:block">/* table */</div>
  <div class="space-y-3 md:hidden">/* cards */</div>
  ```
- Give the table a `<caption class="sr-only">` or an `aria-label`.
- Cover four states explicitly: loading, empty, error, populated.
- Empty and error states are designed surfaces, not blank space.
- Pagination: surface total counts from the query meta rather than guessing.
- Row actions are real links or buttons with accessible names — never
  clickable `<div>`s.

## 6. Form page pattern

- A real `<form>` with `on:submit`, and `event.prevent_default()` when the
  request is sent with `api_json`.
- Every input has a `<label for>` (or `aria-label`) and a `name`.
- Guard double submission with a busy signal; the submit button is
  `disabled=move || busy.get()` and reflects `aria-busy`.
- On failure, show the `ProblemDetails.detail` text in a live region
  (`role="alert"`), and mark the offending inputs `aria-invalid="true"`.
- Reset or reload state after a successful mutation instead of leaving stale values.
- Validate on the server; client checks are a convenience, not the rule.

## 7. Styling baseline

Match the existing surfaces so a new page does not look bolted on:

| Element | Classes |
|---|---|
| Page container | `mx-auto max-w-7xl space-y-8 px-5 py-12 sm:px-8 lg:px-10` |
| Card / panel | `rounded-3xl border border-white/10 bg-white/[0.03] p-6` |
| Section heading | `text-xl font-bold text-white` |
| Eyebrow | `text-sm font-bold uppercase tracking-widest text-cyan-300` |
| Body copy | `text-sm leading-6 text-slate-400` |
| Primary button | `rounded-xl bg-cyan-300 px-5 py-3 text-sm font-bold text-slate-950` |
| Secondary button | `rounded-xl border border-white/10 px-5 py-3 text-sm font-semibold text-slate-200 hover:bg-white/10` |
| Focus ring | `focus:outline-none focus:ring-2 focus:ring-cyan-400` |

Detailed scales, states, and token wiring: use `minirust-design-system` and
`minirust-tailwind-styling`.

## 8. Add the menu entry

A page does not appear in navigation until it is registered in the database.

1. Add a migration under `crates/database/migrations/` that inserts the menu row
   (route, label keys, ordering) and its access grants.
2. Follow the shape of `0005_menus.sql`, `0006_menu_access.sql`, and
   `0009_user_management_menu_route.sql`.
3. Never edit an already-applied migration; add a new one.

## 9. Test the route

Add a test in `mod tests` of `apps/web/src/lib.rs` that exercises the real router:

```rust
let response = router(AppState::new())
    .oneshot(Request::get("/invoices").body(Body::empty()).unwrap())
    .await
    .unwrap();
```

Assert the redirect/status contract and at least one representative responsive
marker (for example that both the `md:` table and the `md:hidden` card list exist).
Follow `.agents/rules/testing.md` for what is worth asserting.

## 10. Finish

Run the `validate-minirust` skill. Then smoke test with `cargo run -p minirust-web`
(`/` and the new path) and stop the process afterwards.
