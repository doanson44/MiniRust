---
name: minirust-tailwind-styling
description: >-
  Applies Tailwind CSS v4 in apps/web: utility conventions, mobile-first
  breakpoints, dark theme, custom variants, and safe use of arbitrary values.
  Use when styling Leptos components, fixing responsive behaviour, or adding
  custom utilities to tailwind.css.
argument-hint: "[component, layout, or responsive problem]"
---

# MiniRust Tailwind Styling

Tailwind CSS is the only styling system in `apps/web`. This skill covers how to
apply it in this repository — not a general Tailwind tutorial.

## Pipeline

| Step | Command |
|---|---|
| Install the CLI | `cd apps/web; npm install` |
| Build CSS | `npm run build:css` |
| Entry file | `apps/web/tailwind.css` |
| Output | `apps/web/src/generated.css` (committed, inlined by `include_str!`) |

`tailwind.css` is intentionally minimal:

```css
@import "tailwindcss";
@source "./src";
```

`@source "./src"` is what makes class detection work for `.rs` files — Tailwind
scans them as plain text. Do not remove it, and do not add a `content` array
(a v3 concept) anywhere.

## Utility-first rules

1. Class names must be **static strings** in the source. Tailwind scans text, so
   `format!("text-{size}")` produces a class that is never generated. Select from
   a fixed set of complete class strings instead.
2. No inline `style=` attributes. Use utilities, or an arbitrary value.
3. Style through variants, not duplication: `hover:`, `focus-visible:`,
   `disabled:`, `md:`, `dark:`.
4. Extract a Leptos `#[component]` when a class cluster repeats three or more
   times or carries behaviour — not on the second use.
5. Keep the class list ordered: layout → spacing → typography → colour → state.

## Responsive

Mobile-first: an unprefixed utility is the phone case, and variants add capability
as the viewport grows.

| Prefix | Min width | Typical use here |
|---|---|---|
| (none) | 0 | Single column, stacked actions |
| `sm:` | 640px | Two-up grids, inline actions |
| `md:` | 768px | Show table instead of cards, desktop nav |
| `lg:` | 1024px | Sidebars, multi-column page splits |
| `xl:` | 1280px | Wider content measures |

Rules:

- Never design desktop-first and then patch phones.
- No fixed pixel widths on containers. Use `max-w-*` plus `w-full`.
- Avoid `overflow-x-auto` as a substitute for a responsive layout; the table/card
  split is the house pattern.
- Verify at 375 / 768 / 1280 before calling the work done.
- Responsive class lists stay readable — split a long list across lines rather
  than one 400-character attribute.

## Common patterns

```rust
// Page container
"mx-auto max-w-7xl space-y-8 px-5 py-12 sm:px-8 lg:px-10"

// Raised panel
"rounded-3xl border border-white/10 bg-white/[0.03] p-6"

// Primary action
"inline-flex items-center justify-center rounded-xl bg-cyan-300 px-5 py-3 text-sm font-bold text-slate-950 transition hover:bg-cyan-200 focus:outline-none focus:ring-2 focus:ring-cyan-300 focus:ring-offset-2 focus:ring-offset-slate-950"

// Secondary action
"inline-flex items-center justify-center rounded-xl border border-white/10 px-5 py-3 text-sm font-semibold text-slate-200 transition hover:bg-white/10 focus:outline-none focus:ring-2 focus:ring-cyan-400"

// Icon-only control (label is mandatory)
"rounded-lg p-2 text-slate-300 hover:bg-white/10 focus:outline-none focus:ring-2 focus:ring-cyan-400"

// Table/card split
"hidden md:block"   // table wrapper
"space-y-3 md:hidden" // card list wrapper
```

## Colour and opacity

- The app is dark-only. There is no light mode and no `dark:` variant to write.
- Translucent surfaces are the depth mechanism: `bg-white/[0.03]`,
  `border-white/10`, `bg-white/[0.04]`.
- Use the `/` opacity syntax (`bg-cyan-300/10`) rather than pre-mixed colours.

## Arbitrary values

Allowed when no token fits and the value is genuinely one-off:

```rust
// radial atmosphere behind a hero
"bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.16),transparent_40%)]"
```

Not allowed as a habit:

- Repeating the same arbitrary value in several places — promote it to `@theme`
  (see `minirust-design-system`).
- Arbitrary values for spacing, radius, or type that the default scale already covers.
- Arbitrary colour values that bypass the semantic palette.

## Custom variants and utilities

Prefer `@theme` tokens; they generate real utilities and stay discoverable:

```css
@theme {
  --color-accent: oklch(0.87 0.09 195);
}
/* now usable as bg-accent, text-accent, border-accent, ring-accent */
```

Write `@custom-variant` / `@utility` only for a pattern that cannot be expressed
with tokens plus existing variants, and document why next to the definition.

## Checklist

- [ ] All classes are static strings (no interpolation into class names)
- [ ] No inline `style=` attributes
- [ ] Layout verified at 375 / 768 / 1280
- [ ] Interactive elements have `hover:` and `focus-visible:` states
- [ ] No `dark:` variants (the app is dark-only)
- [ ] No arbitrary values duplicating what a token already provides
- [ ] `npm run build:css` run when `tailwind.css` changed
- [ ] `generated.css` rebuilt and committed
