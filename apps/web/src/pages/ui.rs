//! Shared UI primitives for the MiniRust web app.
//!
//! Class constants and small components encode the component state matrices from
//! the `minirust-design-system` skill. Every interactive element carries a
//! hover, focus-visible, and disabled treatment, and every control that renders
//! a glyph carries an accessible name.
//!
//! Tailwind scans this file as plain text, so the class strings below must stay
//! static literals — never build a class name from a `format!` argument.

use leptos::prelude::*;

// ── Layout ───────────────────────────────────────────────────────────────────

/// Standard page container: max width, gutter, vertical rhythm.
pub const PAGE_SHELL: &str = "mx-auto max-w-7xl space-y-8 px-5 py-12 sm:px-8 lg:px-10";

/// Small uppercase label that introduces a page or section.
pub const EYEBROW: &str = "text-sm font-bold uppercase tracking-widest text-accent";

/// Page title.
pub const PAGE_TITLE: &str = "mt-3 text-4xl font-black text-foreground";

// ── Buttons ──────────────────────────────────────────────────────────────────
//
// Buttons are class constants rather than a component: the app renders them as
// `<button>`, `<a>`, and submit buttons, and each case needs different attributes.
// The focus treatment is baked into every constant so it cannot drift per call site.

pub const BTN_PRIMARY: &str = "inline-flex items-center justify-center gap-2 rounded-xl text-sm font-bold transition disabled:cursor-not-allowed disabled:opacity-40 bg-accent text-canvas px-5 py-3 hover:bg-accent-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

/// Primary action, full-width (form submit buttons).
pub const BTN_PRIMARY_FULL: &str = "flex w-full items-center justify-center gap-2 rounded-xl bg-accent px-4 py-3 text-sm font-bold text-canvas transition hover:bg-accent-strong disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

/// Secondary action, default size.
pub const BTN_SECONDARY: &str = "inline-flex items-center justify-center gap-2 rounded-xl text-sm font-semibold transition disabled:cursor-not-allowed disabled:opacity-40 border border-line text-muted-foreground px-5 py-3 hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

/// Secondary action, compact size (table rows, pagination).
pub const BTN_SECONDARY_SM: &str = "inline-flex items-center justify-center gap-2 rounded-xl text-sm font-semibold transition disabled:cursor-not-allowed disabled:opacity-40 border border-line text-muted-foreground px-3 py-2 hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

/// Destructive action, default size.
pub const BTN_DANGER: &str = "inline-flex items-center justify-center gap-2 rounded-xl text-sm font-bold transition disabled:cursor-not-allowed disabled:opacity-40 bg-danger text-canvas px-5 py-3 hover:bg-danger/80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-danger focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

/// Destructive action, compact size.
pub const BTN_DANGER_SM: &str = "inline-flex items-center justify-center gap-2 rounded-xl text-sm font-semibold transition disabled:cursor-not-allowed disabled:opacity-40 border border-transparent bg-danger/10 text-danger px-3 py-2 hover:bg-danger/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-danger focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

/// Header icon-only control (no border).
pub const BTN_ICON_PLAIN: &str = "inline-grid size-10 shrink-0 place-items-center rounded-xl text-muted-foreground transition hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

/// Row in a dropdown / side menu.
pub const MENU_ITEM: &str = "block rounded-xl px-3 py-2.5 text-sm text-muted-foreground transition hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset";

/// Destructive row in a dropdown / side menu.
pub const MENU_ITEM_DANGER: &str = "block w-full rounded-xl px-3 py-2.5 text-left text-sm text-danger transition hover:bg-danger/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-danger focus-visible:ring-inset";

/// Inline text link.
pub const LINK: &str = "rounded font-semibold text-accent transition hover:text-accent-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

// ── Form controls ────────────────────────────────────────────────────────────

/// Text-like input.
pub const INPUT: &str = "w-full rounded-xl border border-line bg-canvas px-4 py-3 text-sm text-foreground transition placeholder:text-faint-foreground focus-visible:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40";

/// Select. Pair with `SELECT_CHEVRON` for the affordance.
pub const SELECT: &str = "w-full appearance-none rounded-xl border border-line bg-canvas-raised py-3 pl-4 pr-10 text-sm text-foreground transition focus-visible:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40";

/// Compact select used inside table toolbars.
pub const SELECT_SM: &str = "appearance-none rounded-lg border border-line bg-canvas-raised py-1 pl-2.5 pr-6 text-xs text-foreground transition focus-visible:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40";

/// Checkbox sized for touch.
pub const CHECKBOX: &str = "size-5 shrink-0 cursor-pointer rounded border-line bg-canvas text-accent transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";

/// Absolute chevron overlay for a select.
pub const SELECT_CHEVRON: &str =
    "pointer-events-none absolute inset-y-0 right-3 flex items-center text-faint-foreground";

// ── Switch ───────────────────────────────────────────────────────────────────

const SWITCH_TRACK: &str = "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";
const SWITCH_KNOB: &str =
    "pointer-events-none inline-block size-5 rounded-full bg-white shadow transition";

// ── Surfaces ─────────────────────────────────────────────────────────────────

/// Table wrapper that keeps the border and rounding consistent.
pub const TABLE_SHELL: &str = "overflow-hidden rounded-3xl border border-line bg-surface";

/// Table header cell.
pub const TH: &str = "px-4 py-4";

/// Table body row.
pub const TR: &str = "border-t border-line";

// ── Components ───────────────────────────────────────────────────────────────

/// A labelled form control. Wrapping the control inside `<label>` gives it an
/// accessible name without an explicit `for`/`id` pair.
#[component]
pub fn Field(
    label: String,
    #[prop(optional)] description: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <label class="block">
            <span class="text-sm font-medium text-muted-foreground">{label}</span>
            {description.map(|text| {
                view! { <span class="mt-1 block text-xs text-faint-foreground">{text}</span> }
            })}
            <div class="mt-2">{children()}</div>
        </label>
    }
}

/// An on/off switch. Exposes `role="switch"` and `aria-checked`, and always
/// carries an accessible name — the track renders no text of its own.
#[component]
pub fn ToggleSwitch(
    #[prop(into)] checked: Signal<bool>,
    name: String,
    #[prop(into)] on_toggle: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="switch"
            aria-label=name
            aria-checked=move || checked.get().to_string()
            on:click=move |_| on_toggle.run(())
            class=move || {
                format!(
                    "{} {}",
                    SWITCH_TRACK,
                    if checked.get() { "bg-accent" } else { "bg-slate-700" },
                )
            }
        >
            <span class=move || {
                format!(
                    "{} {}",
                    SWITCH_KNOB,
                    if checked.get() { "translate-x-5" } else { "translate-x-0" },
                )
            }></span>
        </button>
    }
}

/// A switch with a title and description on the left. Use when the switch is a
/// standalone setting rather than a row inside a form.
#[component]
pub fn ToggleRow(
    title: String,
    #[prop(optional)] description: Option<String>,
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
) -> impl IntoView {
    let name = title.clone();
    view! {
        <div class="flex items-center justify-between gap-4 rounded-xl border border-line bg-canvas px-4 py-3">
            <div class="min-w-0">
                <p class="text-sm font-medium text-muted-foreground">{title}</p>
                {description.map(|text| {
                    view! { <p class="text-xs text-faint-foreground">{text}</p> }
                })}
            </div>
            <ToggleSwitch name=name checked=checked on_toggle=on_toggle/>
        </div>
    }
}

/// A modal dialog. Renders nothing while closed, traps nothing but is announced
/// as a dialog, and always offers a labelled dismiss target.
#[component]
pub fn Modal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    close_label: String,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-canvas/80 p-4 backdrop-blur-sm">
                <button
                    type="button"
                    aria-label=close_label.clone()
                    class="absolute inset-0 cursor-default"
                    on:click=move |_| on_close.run(())
                ></button>
                <section
                    role="dialog"
                    aria-modal="true"
                    class="relative z-10 max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-3xl border border-line bg-canvas-raised p-6 shadow-2xl"
                >
                    {children()}
                </section>
            </div>
        </Show>
    }
}

/// Status strip shown while a data surface is loading. The label is exposed to
/// assistive technology only, so no visible text has to be localized twice.
#[component]
pub fn LoadingState(#[prop(optional)] label: Option<String>) -> impl IntoView {
    let label = label.unwrap_or_else(|| "Loading".to_owned());
    view! {
        <div role="status" aria-label=label class="flex items-center justify-center gap-3 px-4 py-6">
            <span
                class="size-4 animate-spin rounded-full border-2 border-line border-t-accent"
                aria-hidden="true"
            ></span>
        </div>
    }
}

/// Empty state. The caller composes the copy so it stays localized.
#[component]
pub fn EmptyState(children: Children) -> impl IntoView {
    view! {
        <div class="px-4 py-10 text-center">
            <p class="mx-auto max-w-md text-sm leading-6 text-faint-foreground">{children()}</p>
        </div>
    }
}

/// Error state. Announced as an alert so it is not missed by screen readers.
#[allow(dead_code)]
#[component]
pub fn ErrorState(message: String) -> impl IntoView {
    view! {
        <p
            role="alert"
            class="rounded-xl border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger"
        >
            {message}
        </p>
    }
}

/// Sortable table header cell. Renders the sort direction as text so the state
/// is available without colour or glyph recognition alone.
#[component]
pub fn SortHeader(
    label: String,
    column: &'static str,
    #[prop(into)] active_column: Signal<String>,
    #[prop(into)] descending: Signal<bool>,
    #[prop(into)] on_sort: Callback<String>,
) -> impl IntoView {
    let aria_label = label.clone();

    view! {
        <th scope="col" class=TH>
            <button
                type="button"
                on:click=move |_| on_sort.run(column.to_owned())
                class="inline-flex items-center gap-2 rounded text-xs uppercase tracking-widest transition hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas"
                aria-label=move || {
                    let direction = if active_column.get() == column {
                        if descending.get() { "descending" } else { "ascending" }
                    } else {
                        "none"
                    };
                    format!("Sort by {aria_label}, currently {direction}")
                }
            >
                {label}
                <span class="text-sm" aria-hidden="true">
                    {move || {
                        if active_column.get() == column {
                            if descending.get() { "\u{2193}" } else { "\u{2191}" }
                        } else {
                            "\u{2195}"
                        }
                    }}
                </span>
            </button>
        </th>
    }
}

/// Rows-per-page selector shared by paged tables.
#[component]
pub fn PageSizeSelect(
    #[prop(into)] value: Signal<i32>,
    options: Vec<i32>,
    all_value: i32,
    #[prop(into)] on_change: Callback<i32>,
) -> impl IntoView {
    view! {
        <label class="flex items-center gap-2 text-xs text-faint-foreground">
            "Rows per page"
            <select
                prop:value=move || value.get().to_string()
                on:change=move |event| {
                    if let Ok(size) = event_target_value(&event).parse::<i32>() {
                        on_change.run(size);
                    }
                }
                class=SELECT_SM
            >
                {options
                    .into_iter()
                    .map(|size| {
                        let label = if size == all_value { "All".to_owned() } else { size.to_string() };
                        view! { <option value=size.to_string()>{label}</option> }
                    })
                    .collect_view()}
            </select>
        </label>
    }
}

/// Previous/next pager. The summary is supplied by the caller so each table can
/// describe its own paging semantics.
#[component]
pub fn Pagination(
    #[prop(into)] can_go_back: Signal<bool>,
    #[prop(into)] can_go_forward: Signal<bool>,
    #[prop(into)] on_previous: Callback<()>,
    #[prop(into)] on_next: Callback<()>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center justify-between gap-4 border-t border-line px-4 py-4 text-sm text-subtle-foreground">
            <span>{children()}</span>
            <div class="flex gap-2">
                <button
                    type="button"
                    disabled=move || !can_go_back.get()
                    on:click=move |_| on_previous.run(())
                    class=BTN_SECONDARY_SM
                >
                    "Previous"
                </button>
                <button
                    type="button"
                    disabled=move || !can_go_forward.get()
                    on:click=move |_| on_next.run(())
                    class=BTN_SECONDARY_SM
                >
                    "Next"
                </button>
            </div>
        </div>
    }
}
