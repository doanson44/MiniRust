---
name: minirust-design-system
description: >-
  Defines MiniRust design tokens and component specs for Tailwind CSS v4:
  three-layer token architecture, naming, dark theme, and component state
  matrices. Use when introducing tokens, scales, or systematic component
  standards in apps/web.
argument-hint: "[token, scale, or component]"
---

# MiniRust Design System

Token architecture and component specification for `apps/web`, expressed in
Tailwind CSS v4 (CSS-first configuration — there is no `tailwind.config.js`).

## Source of truth

| File | Role |
|---|---|
| `apps/web/tailwind.css` | Tailwind entry: `@import "tailwindcss";` + `@source "./src";` + `@theme` tokens |
| `apps/web/src/generated.css` | Build output, inlined into the SSR shell via `include_str!` |
| `apps/web/src/pages/*.rs` | Consumers — utility classes only |

Rebuild after changing tokens:

```powershell
cd apps/web
npm install
npm run build:css
```

`generated.css` is committed, so a token change is only complete once the CSS is rebuilt.

## Three-layer token architecture

| Layer | Purpose | Example |
|---|---|---|
| Primitive | Raw values, never used directly in markup | `--color-brand-500: oklch(0.78 0.14 195)` |
| Semantic | Purpose aliases, the layer pages refer to | `--color-accent: var(--color-brand-500)` |
| Component | Per-component decisions | `--button-bg: var(--color-accent)` |

Rules:

1. Primitives are defined once and never referenced from a page.
2. Semantics express intent (`accent`, `surface`, `muted-foreground`), not hue.
3. Component tokens are only introduced when a component genuinely diverges from
   its semantic default.
4. A layer may only reference the layer above it.

## Tailwind v4 `@theme` wiring

Declare tokens inside `@theme` so they become real utilities
(`bg-accent`, `text-muted-foreground`, `border-surface`):

```css
@import "tailwindcss";
@source "./src";

@theme {
  /* Primitive */
  --color-brand-300: oklch(0.87 0.09 195);
  --color-brand-500: oklch(0.78 0.14 195);
  --color-ink-950: oklch(0.15 0.02 250);

  /* Semantic */
  --color-accent: var(--color-brand-500);
  --color-surface: oklch(1 0 0 / 0.03);
  --color-canvas: var(--color-ink-950);
}
```

Do not hand-write utility classes in `@layer utilities` for something `@theme`
can generate.

## Current palette

The app is dark-only. Keep these semantic meanings; substitute values only with
a deliberate reason.

| Semantic | Current utility | Meaning |
|---|---|---|
| Canvas | `bg-slate-950` | Page background |
| Surface | `bg-white/[0.03]` | Raised panel on the canvas |
| Border | `border-white/10` | Hairline separator |
| Accent | `cyan-300` | Primary action, focus ring, active state |
| Foreground | `text-white` | Headings and primary text |
| Muted foreground | `text-slate-400` | Body and helper text |
| Danger | `red-400` | Destructive and error states |
| Success | `emerald-400` | Confirmation states |

Adding a second accent hue is a design decision, not a convenience — state the
reason or keep the single accent.

## Type and spacing scales

- Type: use Tailwind's scale (`text-xs` → `text-7xl`). Display headings pair
  `font-black tracking-tight`; eyebrows use
  `text-sm font-bold uppercase tracking-widest`.
- Spacing: stay on the 4px scale (`gap-2/3/4/6/8`, `p-4/6/8/10`). Off-scale
  values need a justification.
- Radius: `rounded-xl` for controls, `rounded-2xl` for cards, `rounded-3xl` for
  page-level panels.
- Density: page sections breathe (`space-y-8`, `py-12`); table rows are tighter.

## Component spec pattern

Specify every interactive component as a state matrix before coding it.

| Property | Default | Hover | Focus-visible | Active | Disabled |
|---|---|---|---|---|---|
| Background | accent | `accent` lightened | accent | accent darkened | `white/5` |
| Text | `slate-950` | `slate-950` | `slate-950` | `slate-950` | `slate-500` |
| Border | none | none | none | none | `white/10` |
| Ring | none | none | `ring-2 ring-cyan-400` | none | none |
| Cursor | pointer | pointer | pointer | pointer | `not-allowed` |

Non-negotiables for every interactive element:

- A `focus-visible` treatment that meets contrast on the dark canvas.
- A disabled state that is visually distinct and not focusable
  (`disabled` attribute, not just reduced opacity).
- A touch target of at least 44×44 CSS pixels on mobile.

## Review checklist

- [ ] No raw hex, `rgb()`, or `oklch()` values in a page file
- [ ] No inline `style=` attributes
- [ ] New colour or size appears in `@theme`, not as an arbitrary value
- [ ] Every interactive component has default / hover / focus / disabled states
- [ ] `generated.css` rebuilt and committed after a token change
- [ ] Contrast verified against the dark canvas, not assumed

## References

- `references/token-architecture.md` — layering, naming, and migration order
