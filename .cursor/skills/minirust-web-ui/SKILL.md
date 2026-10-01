---
name: minirust-web-ui
description: >-
  Routes MiniRust frontend work to the right web/UI skill and enforces apps/web
  constraints (Leptos SSR, Tailwind only, responsive, accessible). Use when
  building or restyling pages, components, or layouts in apps/web, or when
  deciding which frontend skill should handle a request.
argument-hint: "[page, component, or UI request]"
---

# MiniRust Web UI

Entry point for frontend work targeting `apps/web`. When a generic UI or design
skill conflicts with a repository rule, the repository rule wins.

## Mandatory constraints

1. **Leptos SSR** — page components live under `apps/web/src/pages/` and are
   registered in `apps/web/src/lib.rs`. Do not introduce another frontend framework.
2. **Tailwind CSS only** — no Bootstrap, DaisyUI, or any other CSS/UI framework.
   No inline `style=` attributes; use utility classes.
3. **Responsive is not optional** — every user-facing page must hold up at
   375 / 768 / 1280. Mobile-first utilities, `sm:` `md:` `lg:` variants, and no
   fixed widths that create horizontal scrolling.
4. **Accessibility** — semantic landmarks, `aria-label` on icon-only controls,
   visible focus rings, and labelled inputs.
5. **No business rules in the web app** — call `crates/services` for SSR or the
   `/api/v1/...` endpoints on hydrate paths. Never query another context's
   persistence directly and never duplicate a rule that lives in `crates/services`.
6. **Viewport meta tag** — supplied once by the shell in `lib.rs`; do not remove it.
7. **Navigation is data-driven** — a page appears in the sidebar, and is reachable
   by direct URL, only when a migration registers it in the menus tables.

## Routing guide

| Request | Skill |
|---|---|
| New list, table, detail, or create/edit form page | `create-minirust-page` |
| Design tokens, type/spacing scales, component states, Tailwind `@theme` | `minirust-design-system` |
| Tailwind utilities, breakpoints, custom variants, theme wiring | `minirust-tailwind-styling` |
| Raise visual quality so a page stops looking generic | `minirust-frontend-design` |
| Review UX, hierarchy, accessibility, or interaction quality | `minirust-ux-review` |

## Mixed requests

Apply in this order:

1. **Structure** — `create-minirust-page` (route, layout, data, states).
2. **System** — `minirust-design-system`, then `minirust-tailwind-styling`.
3. **Polish** — `minirust-frontend-design`, then `minirust-ux-review`.

Do not start from a generic aesthetic skill when the request is really
"add an admin list page".

## Where things live

| Path | Purpose |
|---|---|
| `apps/web/src/lib.rs` | Shell, route table, auth + menu middleware, SSR tests |
| `apps/web/src/pages/*.rs` | Page components and layouts |
| `apps/web/src/pages/mod.rs` | Page exports consumed by `lib.rs` |
| `apps/web/src/pages/layouts.rs` | `AppLayout` (authenticated shell) and `AuthLayout` |
| `apps/web/src/api.rs` | `api_json`, `api_json_with_meta`, `api_empty` helpers |
| `apps/web/src/types.rs` | DTOs mirroring the API response envelopes |
| `apps/web/src/models/` | View models passed into pages |
| `apps/web/tailwind.css` | Tailwind entry (`@import "tailwindcss"` + `@source "./src"`) |
| `apps/web/src/generated.css` | Built CSS, inlined into the shell via `include_str!` |

## Reference rules

- `.agents/rules/web-leptos.md`
- `.agents/rules/architecture.md`
- `.agents/rules/testing.md`
- `.agents/skills/validate-minirust/SKILL.md`
