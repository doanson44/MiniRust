---
name: minirust-ux-review
description: >-
  Reviews MiniRust pages for usability, hierarchy, states, accessibility, and
  responsive behaviour, and reports findings with severity and concrete fixes.
  Use before shipping a page, after a UI complaint, or when a surface feels
  wrong without an obvious cause.
argument-hint: "[page, route, or component to review]"
---

# MiniRust UX Review

Use for evaluation, not generation. Produce findings; do not silently rewrite the
page unless the user asks for fixes.

## How to review

1. Read the page and its layout in `apps/web/src/pages/`.
2. Trace the data path: which endpoints feed it, and what happens on failure.
3. Judge the rendered result at **375 / 768 / 1280**.
4. Report findings grouped by severity, each with a concrete fix.

### Severity

| Level | Meaning |
|---|---|
| Blocker | Unusable, inaccessible, or data-losing |
| Major | Users will be confused, stuck, or misled |
| Minor | Polish that is worth doing but not shipping-blocking |
| Note | Observation or follow-up |

## Checklist

### Structure and hierarchy

- [ ] The purpose of the page is clear from the first screenful
- [ ] One primary action is obvious; secondary actions visibly recede
- [ ] Landmarks are used (`header`, `nav`, `main`, `footer`) rather than nested `<div>`s
- [ ] The page has a real `<h1>` and heading levels descend without skipping
- [ ] Content is grouped meaningfully, not stacked as an even wall

### Navigation and orientation

- [ ] The current location is identifiable from the UI
- [ ] Pages registered in the menu are reachable from navigation
- [ ] A by-URL visit to a page the account cannot open redirects with an
      understandable result (the menu registry decides this — not the page)
- [ ] Back behaviour and browser history are not broken by client-side navigation

### States

Every data surface must handle all four:

- [ ] **Loading** — a skeleton or busy indicator, not an empty frame
- [ ] **Empty** — explains why there is nothing and what to do next
- [ ] **Error** — shows the `ProblemDetails.detail`, with a way to retry
- [ ] **Populated** — the normal case, including a single-row and a many-row case

Additional state checks:

- [ ] Partial failures do not leave the page in a mixed, unexplained state
- [ ] Long values truncate deliberately (`truncate`, `break-words`) instead of
      breaking the layout
- [ ] Disabled controls explain why they are disabled
- [ ] Mutations cannot be double-submitted

### Forms

- [ ] Every field has a visible `<label>` bound with `for`/`id`
- [ ] Required fields are marked in text, not by colour alone
- [ ] Validation errors appear next to the field and in an `role="alert"` region
- [ ] Invalid fields carry `aria-invalid="true"`
- [ ] `autocomplete` and `inputmode` are set appropriately
- [ ] Submitting with the keyboard alone works end to end
- [ ] Error text states how to fix the problem, not just that something failed

### Tables and lists

- [ ] The table does not require horizontal scrolling on a phone (card fallback exists)
- [ ] Column headers are real `<th>` with an accessible table name
- [ ] Row actions have accessible names, not bare icons
- [ ] Sort and page state are reflected in the UI and survive navigation
- [ ] Numbers and dates align consistently

### Accessibility

- [ ] Keyboard: every interactive element is reachable and operable in order
- [ ] Focus is always visible (`focus:ring-2 focus:ring-cyan-400` or equivalent)
- [ ] Icon-only controls have `aria-label`
- [ ] Toggle controls expose state (`aria-expanded`, `aria-pressed`, `aria-current`)
- [ ] Live updates are announced, not only coloured
- [ ] Contrast is checked against the dark canvas for body text and placeholders
- [ ] Touch targets are at least 44×44 CSS pixels on mobile
- [ ] Motion respects `prefers-reduced-motion`

### Responsive

- [ ] No horizontal overflow at 375px
- [ ] Primary actions remain reachable without zooming
- [ ] Content order is sensible when columns collapse
- [ ] Hidden desktop-only affordances have a mobile equivalent
- [ ] Long labels and translated strings do not break the layout

### Content and tone

- [ ] Both Vietnamese and English strings exist for every user-facing label
- [ ] Error and empty-state copy is specific, not "Something went wrong"
- [ ] Placeholder text is not used as a label
- [ ] Dates, numbers, and currency are formatted consistently

## Report format

```text
Blocker
  <page> · <element>
  Problem: what is wrong and who it affects.
  Fix: the concrete change.

Major
  ...

Minor
  ...
```

Close with the single highest-impact change. If nothing is blocking, say so plainly.
