# The Quality Bar

Eight checks that separate a page that was designed from one that was assembled
from defaults. Use this as the acceptance bar for any frontend deliverable in
`apps/web`.

Score honestly. A page that fails three or more of these is not finished, no
matter how many features it has.

---

## 01 · Point of view, not a template

The page commits to a direction — engineering precision, editorial, dark luxury,
utilitarian console — and executes it without flinching. A template has no opinion.

- [ ] The direction can be stated in one sentence
- [ ] Every visual choice reinforces that direction
- [ ] Changing one element would visibly break the cohesion

## 02 · Typography that does work

Scale, weight, and spacing carry the hierarchy. Headings look chosen.

- [ ] Display and eyebrow treatments are distinct and consistent
- [ ] Hierarchy is legible with colour removed
- [ ] Body measure is bounded (`max-w-prose` / `max-w-3xl`), not full-bleed
- [ ] Numbers and technical labels align (monospace where appropriate)
- [ ] No new web font added without a stated reason

## 03 · A restrained colour system

One dominant tone with sharp accents. Premium reads as restraint, not decoration.

- [ ] Uses the semantic palette, not raw hues
- [ ] A single accent, or a second accent with a stated justification
- [ ] Translucency provides depth instead of ad-hoc greys
- [ ] No gradient used as a default panel fill
- [ ] Body text contrast verified against the dark canvas

## 04 · Hierarchy that breathes

The eye is led without effort. Nothing is a wall of equal-weight blocks.

- [ ] The primary action is obvious within one second
- [ ] Secondary and tertiary elements visibly recede
- [ ] Whitespace separates groups instead of borders everywhere
- [ ] Mobile stacking preserves the intended reading order

## 05 · States with intent

Loading, empty, error, and populated are all designed — for every data surface.

- [ ] Loading shows structure (skeleton or busy state), not a blank frame
- [ ] Empty explains why and what to do next
- [ ] Error surfaces a real message and a way to retry
- [ ] Long and single-row extremes do not break the layout
- [ ] Disabled controls explain themselves

## 06 · Motion that whispers

Motion confirms an action or reveals structure. It never delays the user.

- [ ] CSS transitions only, short durations (`duration-200`)
- [ ] `prefers-reduced-motion` respected
- [ ] Nothing animates on a timer
- [ ] Admin surfaces stay still; motion is reserved for landing surfaces
- [ ] No layout shift caused by an entrance animation

## 07 · Interaction craft

Every control behaves the way the user's hands expect.

- [ ] Focus is visible on every interactive element
- [ ] Icon-only controls carry an accessible name
- [ ] Touch targets are at least 44×44 CSS pixels on mobile
- [ ] Keyboard-only operation works end to end
- [ ] Mutations cannot be double-submitted
- [ ] Toggles expose their state (`aria-expanded`, `aria-pressed`, `aria-current`)

## 08 · Nothing is there for no reason

Every element earns its place.

- [ ] No decorative element that carries no information
- [ ] No placeholder copy left in the UI
- [ ] No duplicate affordance doing the same job twice
- [ ] No nesting of card inside card inside card
- [ ] Both Vietnamese and English strings exist for every user-facing label

---

## Applying the bar

Failing checks are findings, not opinions. Report each one as:

```text
<check number> · <page> · <element>
Fails because: <observable reason>
Fix: <concrete change>
```

Close with the single highest-impact fix. A page failing only notes is shippable
with follow-ups tracked.
