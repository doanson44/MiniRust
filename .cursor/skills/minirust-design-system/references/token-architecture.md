# Token Architecture

Three-layer token system for `apps/web`. Tailwind CSS v4 is CSS-first: tokens are
declared in `@theme` inside `apps/web/tailwind.css`, and each declaration
generates real utilities.

## Layer overview

```text
┌──────────────────────────────────────────────┐
│  Component tokens   --button-bg              │  Per-component decisions
├──────────────────────────────────────────────┤
│  Semantic tokens    --color-accent           │  Purpose aliases
├──────────────────────────────────────────────┤
│  Primitive tokens   --color-brand-500        │  Raw values
└──────────────────────────────────────────────┘
```

## Why three layers

| Layer | Purpose | How often it changes |
|---|---|---|
| Primitive | Raw values, no meaning attached | Rarely — foundational |
| Semantic | Assigns meaning to raw values | When the theme or brand shifts |
| Component | Per-component divergence | Only for genuine exceptions |

A page must only ever reference semantic or component tokens. Referencing a
primitive from a page couples markup to a raw value and defeats the middle layer.

## Layer 1 — primitives

```css
@theme {
  /* Colour */
  --color-brand-300: oklch(0.87 0.09 195);
  --color-brand-500: oklch(0.78 0.14 195);
  --color-ink-900: oklch(0.22 0.02 250);
  --color-ink-950: oklch(0.15 0.02 250);

  /* Spacing (4px base) */
  --spacing-1: 0.25rem;
  --spacing-2: 0.5rem;
  --spacing-4: 1rem;
  --spacing-6: 1.5rem;

  /* Type */
  --text-sm: 0.875rem;
  --text-base: 1rem;
  --text-lg: 1.125rem;

  /* Radius */
  --radius-control: 0.75rem;
  --radius-card: 1rem;
  --radius-panel: 1.5rem;
}
```

Notes for this repository:

- Tailwind already ships the full default scale. Only declare a primitive when the
  value is genuinely outside it — otherwise you are duplicating the framework.
- Prefer `oklch()` for new colour values; it keeps perceived lightness stable when
  the hue shifts.
- Do not declare primitives that nothing references.

## Layer 2 — semantic

```css
@theme {
  --color-canvas: var(--color-ink-950);
  --color-surface: oklch(1 0 0 / 0.03);
  --color-border-subtle: oklch(1 0 0 / 0.10);
  --color-foreground: oklch(1 0 0);
  --color-muted-foreground: oklch(0.70 0.02 250);

  --color-accent: var(--color-brand-300);
  --color-accent-hover: var(--color-brand-500);
  --color-danger: oklch(0.70 0.19 22);
  --color-success: oklch(0.77 0.15 163);
}
```

These names are the contract. `bg-accent` survives a brand-colour change;
`bg-cyan-300` does not.

## Layer 3 — component

Introduce a component token only when a component genuinely diverges:

```css
@theme {
  --button-bg: var(--color-accent);
  --button-fg: var(--color-canvas);
  --button-radius: var(--radius-control);
  --input-border: var(--color-border-subtle);
  --input-focus-ring: var(--color-accent);
  --card-bg: var(--color-surface);
  --card-border: var(--color-border-subtle);
  --card-radius: var(--radius-card);
}
```

If a component token resolves to exactly its semantic default, delete it — it is
an alias, not a decision.

## Theme switching

`apps/web` is dark-only. There is no `.dark` class and no `@custom-variant dark`.
Semantic tokens point directly at the dark primitives.

If light mode is ever required, the change is confined to layer 2 — override the
semantic tokens under a variant and leave every page untouched. That is the whole
point of the middle layer, and it is why pages must not use primitives.

## Naming

```text
--{category}-{item}-{state}

--color-accent              category-item
--color-accent-hover        category-item-state
--button-bg                 component-property
--button-bg-hover           component-property-state
```

| Category | Examples |
|---|---|
| `color` | canvas, surface, accent, muted-foreground, danger |
| `spacing` | 1, 2, 4, 6, section, component |
| `text` | sm, base, lg, xl (Tailwind scale names) |
| `radius` | control, card, panel |
| `shadow` | sm, default, lg |
| `duration` | fast, normal, slow |

## File organisation

`apps/web/tailwind.css` is a single file with layer comment banners. Split it into
`tokens/primitives.css` etc. only if it exceeds roughly 150 lines.

```css
@import "tailwindcss";
@source "./src";

/* === PRIMITIVES === */
@theme { /* ... */ }

/* === SEMANTIC === */
@theme { /* ... */ }

/* === COMPONENT === */
@theme { /* ... */ }
```

## Migration order

Do not migrate everything at once. Colour, spacing, and radius tokens each need
their own pass and their own `npm run build:css`.

1. Add the primitive and semantic tokens for one category.
2. Rebuild CSS (`cd apps/web; npm run build:css`).
3. Replace usages of the raw utility in that category, one page at a time.
4. Verify at 375 / 768 / 1280 — a token swap must be visually identical.
5. Commit the rebuilt `generated.css` with the change.

A token refactor that changes how the app looks is a redesign, not a refactor.
Do one or the other.
