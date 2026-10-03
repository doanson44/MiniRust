use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json, api_json_with_meta};
use crate::models::query::{ALL_PAGE_SIZE, PAGE_SIZE_OPTIONS};
use crate::types::UserResponse;
#[cfg(feature = "hydrate")]
use crate::types::{PaginationMeta, PremiumResponse, UserListData};
use minirust_locales::{text as translate, Key, Locale};

#[cfg(feature = "hydrate")]
use super::ui::ToastController;
use super::ui::{
    EmptyState, Field, LoadingState, Modal, PageSizeSelect, Pagination, SortHeader, ToggleRow,
    BTN_DANGER, BTN_DANGER_SM, BTN_PRIMARY, BTN_SECONDARY_SM, EYEBROW, INPUT, PAGE_SHELL,
    PAGE_TITLE, SELECT, SELECT_CHEVRON, TABLE_SHELL, TH, TR,
};

#[cfg(feature = "hydrate")]
fn timestamp_to_date(timestamp: i64) -> String {
    js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(timestamp as f64 * 1000.0))
        .to_iso_string()
        .as_string()
        .and_then(|value| value.get(..10).map(str::to_owned))
        .unwrap_or_default()
}

#[cfg(feature = "hydrate")]
fn date_to_timestamp(value: &str) -> Option<i64> {
    if value.is_empty() {
        return None;
    }

    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(&format!(
        "{value}T23:59:59Z"
    )));
    let milliseconds = date.get_time();
    if milliseconds.is_finite() {
        Some((milliseconds / 1000.0) as i64)
    } else {
        None
    }
}

#[cfg(not(feature = "hydrate"))]
fn timestamp_to_date(_timestamp: i64) -> String {
    String::new()
}

fn premium_label(user: &UserResponse, locale: Locale) -> String {
    if !user.is_premium {
        return translate(locale, Key::AdminPremiumInactive).to_owned();
    }

    match user.premium_expires_at {
        Some(timestamp) => format!(
            "{} {}",
            translate(locale, Key::AdminPremiumUntil),
            timestamp_to_date(timestamp)
        ),
        None => translate(locale, Key::AdminPremiumUnlimited).to_owned(),
    }
}

#[component]
#[allow(unused_variables)]
pub fn AdminPage() -> impl IntoView {
    let (users, set_users) = signal(Vec::<UserResponse>::new());
    let (current_user_id, set_current_user_id) = signal(None::<String>);

    // ── Create modal state ───────────────────────────────────────────────
    let (show_create, set_show_create) = signal(false);
    let (new_email, set_new_email) = signal(String::new());
    let (new_role, set_new_role) = signal("none".to_owned());
    let (new_premium_active, set_new_premium_active) = signal(false);
    let (new_premium_expires, set_new_premium_expires) = signal(String::new());
    let new_premium_expires_input = NodeRef::<leptos::html::Input>::new();

    // ── Edit dialog state ────────────────────────────────────────────────
    let (selected, set_selected) = signal(None::<String>);
    let (delete_candidate, set_delete_candidate) = signal(None::<UserResponse>);
    let (edit_email, set_edit_email) = signal(String::new());
    let (edit_role, set_edit_role) = signal("none".to_owned());
    let (premium_active, set_premium_active) = signal(false);
    let (premium_expires, set_premium_expires) = signal(String::new());
    let (edit_locked, set_edit_locked) = signal(false);
    let premium_expires_input = NodeRef::<leptos::html::Input>::new();

    // ── Filter / sort / page state ───────────────────────────────────────
    let (search, set_search) = signal(String::new());
    let (role_filter, set_role_filter) = signal("all".to_owned());
    let (premium_filter, set_premium_filter) = signal("all".to_owned());
    let (status_filter, set_status_filter) = signal("all".to_owned());
    let (sort_column, set_sort_column) = signal("email".to_owned());
    let (sort_desc, set_sort_desc) = signal(false);
    let (page, set_page) = signal(1u32);
    let (page_size, set_page_size) = signal(20i32);
    let (total_pages, set_total_pages) = signal(1u32);
    let (loading, set_loading) = signal(false);
    let locale = use_context::<ReadSignal<Locale>>().unwrap_or_else(|| signal(Locale::DEFAULT).0);
    let text = move |key: Key| move || translate(locale.get(), key);
    #[cfg(feature = "hydrate")]
    let toast = use_context::<ToastController>().unwrap_or_else(|| ToastController {
        show: Callback::new(|_| {}),
    });

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local({
            async move {
                set_loading.set(true);
                if let Ok(current_user) =
                    api_json::<UserResponse>(gloo_net::http::Method::GET, "/api/v1/auth/me", None)
                        .await
                {
                    set_current_user_id.set(Some(current_user.id));
                }

                match api_json_with_meta::<UserListData, PaginationMeta>(
                    gloo_net::http::Method::GET,
                    &format!(
                        "/api/v1/admin/users?page={}&page_size={}",
                        page.get_untracked(),
                        page_size.get_untracked()
                    ),
                    None,
                )
                .await
                {
                    Ok(response) => {
                        set_users.set(response.data.users);
                        set_total_pages.set(response.meta.total_pages.max(1));
                    }
                    Err(error) => toast.error(error),
                }
                set_loading.set(false);
            }
        });
    }

    let reload_users = move || {
        let current_page = page.get_untracked();
        let current_page_size = page_size.get_untracked();
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local({
            async move {
                set_loading.set(true);
                match api_json_with_meta::<UserListData, PaginationMeta>(
                    gloo_net::http::Method::GET,
                    &format!(
                        "/api/v1/admin/users?page={}&page_size={}&_={}",
                        current_page,
                        current_page_size,
                        js_sys::Date::now() as i64
                    ),
                    None,
                )
                .await
                {
                    Ok(response) => {
                        set_users.set(response.data.users);
                        set_total_pages.set(response.meta.total_pages.max(1));
                    }
                    Err(error) => toast.error(error),
                }
                set_loading.set(false);
            }
        });
    };

    let create_user_with_invite = move |send_invite: bool| {
        let email = new_email.get();
        let role = new_role.get();
        let premium_active = new_premium_active.get();
        let premium_expires = new_premium_expires.get();

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let premium_expires_at = if premium_active {
                date_to_timestamp(&premium_expires)
            } else {
                None
            };

            match api_json::<UserResponse>(
                gloo_net::http::Method::POST,
                "/api/v1/admin/users",
                Some(
                    serde_json::json!({
                        "email": email,
                        "role": role,
                        "premium_active": premium_active,
                        "premium_expires_at": premium_expires_at,
                        "send_invite": send_invite
                    })
                    .to_string(),
                ),
            )
            .await
            {
                Ok(_) => {
                    set_new_email.set(String::new());
                    set_new_role.set("none".to_owned());
                    set_new_premium_active.set(false);
                    set_new_premium_expires.set(String::new());
                    set_show_create.set(false);
                    toast.success(if send_invite {
                        translate(locale.get_untracked(), Key::AdminToastCreatedInvite)
                    } else {
                        translate(locale.get_untracked(), Key::AdminToastCreated)
                    });
                    reload_users();
                }
                Err(error) => toast.error(error),
            }
        });
    };

    let create_user = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        create_user_with_invite(false);
    };

    let create_and_send_email = move |_| {
        create_user_with_invite(true);
    };

    let delete_user = move |user_id: String| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_empty(
                gloo_net::http::Method::DELETE,
                &format!("/api/v1/admin/users/{user_id}"),
                None,
            )
            .await
            {
                Ok(()) => {
                    toast.success(translate(locale.get_untracked(), Key::AdminToastDeleted));
                    if selected.get().as_deref() == Some(user_id.as_str()) {
                        set_selected.set(None);
                    }
                    reload_users();
                }
                Err(error) => toast.error(error),
            }
        });
    };

    let save_changes = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(user_id) = selected.get() else {
            return;
        };
        let role = edit_role.get();
        let premium_active = premium_active.get();
        let premium_expires = premium_expires.get();
        let locked_target = edit_locked.get();

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let premium_expires_at = if premium_active {
                date_to_timestamp(&premium_expires)
            } else {
                None
            };
            match api_json::<UserResponse>(
                gloo_net::http::Method::PATCH,
                &format!("/api/v1/admin/users/{user_id}"),
                Some(
                    serde_json::json!({
                        "role": role,
                        "premium_active": premium_active,
                        "premium_expires_at": premium_expires_at,
                        "is_locked": locked_target
                    })
                    .to_string(),
                ),
            )
            .await
            {
                Ok(updated_user) => {
                    set_users.update(|users| {
                        if let Some(user) = users.iter_mut().find(|user| user.id == updated_user.id)
                        {
                            *user = updated_user;
                        }
                    });
                    toast.success(translate(locale.get_untracked(), Key::AdminToastSaved));
                    set_selected.set(None);
                }
                Err(error) => toast.error(error),
            }
        });
    };

    let open_edit = move |user: UserResponse| {
        let user = users
            .get_untracked()
            .into_iter()
            .find(|item| item.id == user.id)
            .unwrap_or(user);

        set_selected.set(Some(user.id.clone()));
        set_edit_email.set(user.email);
        set_edit_role.set(if user.is_admin {
            "admin".to_owned()
        } else {
            "none".to_owned()
        });
        set_premium_active.set(user.is_premium);
        set_premium_expires.set(String::new());
        set_edit_locked.set(user.is_locked);

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_json::<PremiumResponse>(
                gloo_net::http::Method::GET,
                &format!("/api/v1/admin/users/{}/entitlements/premium", user.id),
                None,
            )
            .await
            {
                Ok(premium) => {
                    set_premium_active.set(premium.active);
                    set_premium_expires.set(
                        premium
                            .expires_at
                            .map(timestamp_to_date)
                            .unwrap_or_default(),
                    );
                }
                Err(error) => toast.error(error),
            }
        });
    };

    let filtered_users = Memo::new(move |_| {
        let query = search.get().trim().to_ascii_lowercase();
        let role = role_filter.get();
        let premium = premium_filter.get();
        let status = status_filter.get();

        users
            .get()
            .into_iter()
            .filter(|user| {
                let matches_search = query.is_empty()
                    || user.email.to_ascii_lowercase().contains(&query)
                    || user
                        .full_name
                        .as_deref()
                        .is_some_and(|name| name.to_ascii_lowercase().contains(&query));
                let matches_role = role == "all"
                    || (role == "admin" && user.is_admin)
                    || (role == "user" && !user.is_admin);
                let matches_premium = premium == "all"
                    || (premium == "active" && user.is_premium)
                    || (premium == "inactive" && !user.is_premium);
                let matches_status = status == "all"
                    || (status == "locked" && user.is_locked)
                    || (status == "active" && !user.is_locked);

                matches_search && matches_role && matches_premium && matches_status
            })
            .collect::<Vec<_>>()
    });

    let sort_users = move |column: &str| {
        if sort_column.get().as_str() == column {
            set_sort_desc.update(|descending| *descending = !*descending);
        } else {
            set_sort_column.set(column.to_owned());
            set_sort_desc.set(false);
        }
        set_page.set(1);
    };

    let sorted_users = Memo::new(move |_| {
        let mut users = filtered_users.get();
        let column = sort_column.get();
        let descending = sort_desc.get();

        users.sort_by(|left, right| {
            let ordering = match column.as_str() {
                "role" => left.is_admin.cmp(&right.is_admin),
                "premium" => left.is_premium.cmp(&right.is_premium),
                "status" => left.is_locked.cmp(&right.is_locked),
                _ => left
                    .email
                    .to_ascii_lowercase()
                    .cmp(&right.email.to_ascii_lowercase()),
            };

            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        });

        users
    });

    let page_users = Memo::new(move |_| sorted_users.get());

    // `SortHeader` hands the column back as an owned `String`; `sort_users`
    // takes a borrow. `Callback` is `Copy`, so one value can serve every header.
    let sort_by = Callback::new(move |column: String| sort_users(&column));

    let change_page_size = move |value: String| {
        if let Ok(value) = value.parse::<i32>() {
            if PAGE_SIZE_OPTIONS.contains(&value) {
                set_page_size.set(value);
                set_page.set(1);
                reload_users();
            }
        }
    };

    view! {
        <div class=PAGE_SHELL>
            <section class="flex flex-wrap items-start justify-between gap-4">
                <div>
                    <p class=EYEBROW>{text(Key::AdminEyebrow)}</p>
                    <h1 class=PAGE_TITLE>{text(Key::AdminTitle)}</h1>
                </div>
                <button
                    type="button"
                    id="btn-open-create-user"
                    on:click=move |_| set_show_create.set(true)
                    class=BTN_PRIMARY
                >
                    {text(Key::AdminCreateOpen)}
                </button>
            </section>

            <section class="space-y-3">
                <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
                    <input
                        type="search"
                        aria-label={translate(locale.get_untracked(), Key::AdminSearchLabel)}
                        placeholder={translate(locale.get_untracked(), Key::AdminSearchPlaceholder)}
                        prop:value=search
                        on:input=move |ev| {
                            set_search.set(event_target_value(&ev));
                            set_page.set(1);
                        }
                        class=INPUT
                    />
                    <div class="relative">
                        <select
                            aria-label={translate(locale.get_untracked(), Key::AdminFilterRoleLabel)}
                            prop:value=role_filter
                            on:change=move |ev| {
                                set_role_filter.set(event_target_value(&ev));
                                set_page.set(1);
                            }
                            class=SELECT
                        >
                            <option value="all" class="bg-canvas-raised text-foreground">{text(Key::AdminFilterRoleAll)}</option>
                            <option value="admin" class="bg-canvas-raised text-foreground">{text(Key::CommonRoleAdmin)}</option>
                            <option value="user" class="bg-canvas-raised text-foreground">{text(Key::CommonRoleUser)}</option>
                        </select>
                        <span class=SELECT_CHEVRON aria-hidden="true">"\u{25BE}"</span>
                    </div>
                    <div class="relative">
                        <select
                            aria-label={translate(locale.get_untracked(), Key::AdminFilterPremiumLabel)}
                            prop:value=premium_filter
                            on:change=move |ev| {
                                set_premium_filter.set(event_target_value(&ev));
                                set_page.set(1);
                            }
                            class=SELECT
                        >
                            <option value="all" class="bg-canvas-raised text-foreground">{text(Key::AdminFilterPremiumAll)}</option>
                            <option value="active" class="bg-canvas-raised text-foreground">"Premium"</option>
                            <option value="inactive" class="bg-canvas-raised text-foreground">{text(Key::AdminFilterPremiumOff)}</option>
                        </select>
                        <span class=SELECT_CHEVRON aria-hidden="true">"\u{25BE}"</span>
                    </div>
                    <div class="relative">
                        <select
                            aria-label={translate(locale.get_untracked(), Key::AdminFilterStatusLabel)}
                            prop:value=status_filter
                            on:change=move |ev| {
                                set_status_filter.set(event_target_value(&ev));
                                set_page.set(1);
                            }
                            class=SELECT
                        >
                            <option value="all" class="bg-canvas-raised text-foreground">{text(Key::AdminFilterStatusAll)}</option>
                            <option value="active" class="bg-canvas-raised text-foreground">{text(Key::CommonActive)}</option>
                            <option value="locked" class="bg-canvas-raised text-foreground">{text(Key::AdminSignInDisabled)}</option>
                        </select>
                        <span class=SELECT_CHEVRON aria-hidden="true">"\u{25BE}"</span>
                    </div>
                </div>

                <section class=TABLE_SHELL>
                    <div class="flex flex-wrap items-center justify-end gap-3 border-b border-line px-4 py-2">
                        <PageSizeSelect
                            value=page_size
                            options=PAGE_SIZE_OPTIONS.to_vec()
                            all_value=ALL_PAGE_SIZE
                            on_change=Callback::new(move |size: i32| change_page_size(size.to_string()))
                        />
                    </div>

                    <Show when=move || loading.get()>
                        <LoadingState label={translate(locale.get_untracked(), Key::AdminLoading).to_owned()}/>
                    </Show>
                    <Show when=move || !loading.get() && page_users.get().is_empty()>
                        <EmptyState>
                            {translate(locale.get_untracked(), Key::AdminEmpty).to_owned()}
                        </EmptyState>
                    </Show>

                    // Desktop table. Below `md` the same rows render as cards so the
                    // page never scrolls horizontally at 375px.
                    <div class="hidden md:block">
                    <table class="w-full text-left" aria-label={text(Key::AdminTitle)}>
                        <thead>
                            <tr class="text-xs uppercase tracking-widest text-faint-foreground">
                                <SortHeader label="Email".to_owned() column="email" active_column=sort_column descending=sort_desc on_sort=sort_by/>
                                <SortHeader label={translate(locale.get_untracked(), Key::AdminRole).to_owned()} column="role" active_column=sort_column descending=sort_desc on_sort=sort_by/>
                                <SortHeader label="Premium".to_owned() column="premium" active_column=sort_column descending=sort_desc on_sort=sort_by/>
                                <SortHeader label={translate(locale.get_untracked(), Key::AdminStatus).to_owned()} column="status" active_column=sort_column descending=sort_desc on_sort=sort_by/>
                                <th scope="col" class=TH>{text(Key::AdminActions)}</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || page_users.get()
                                key=|user| user.id.clone()
                                children=move |user| {
                                    let id_for_self = user.id.clone();
                                    let is_self = Signal::derive(move || {
                                        current_user_id
                                            .get()
                                            .is_some_and(|current_id| current_id == id_for_self)
                                    });
                                    let edit_user_id = StoredValue::new(user.id.clone());
                                    let delete_user_id = StoredValue::new(user.id.clone());
                                    let email = user.email.clone();
                                    let full_name = user.full_name.clone();
                                    let id_for_role = user.id.clone();
                                    let id_for_premium = user.id.clone();
                                    let id_for_status = user.id.clone();

                                    view! {
                                        <tr class=TR>
                                            <td class="px-4 py-4 text-sm text-foreground">
                                                <div class="break-words">{email}</div>
                                                {full_name.map(|name| view! {
                                                    <div class="mt-1 break-words text-xs text-faint-foreground">{name}</div>
                                                })}
                                            </td>
                                            <td class="px-4 py-4 text-sm text-subtle-foreground">
                                                {move || {
                                                    let is_admin = users
                                                        .get()
                                                        .into_iter()
                                                        .find(|item| item.id == id_for_role)
                                                        .map(|item| item.is_admin)
                                                        .unwrap_or(false);
                                                    if is_admin {
                                                        translate(locale.get(), Key::CommonRoleAdmin)
                                                    } else {
                                                        translate(locale.get(), Key::CommonRoleUser)
                                                    }
                                                }}
                                            </td>
                                            <td class="px-4 py-4 text-sm text-subtle-foreground">
                                                {move || {
                                                    users
                                                        .get()
                                                        .into_iter()
                                                        .find(|item| item.id == id_for_premium)
                                                        .map(|item| premium_label(&item, locale.get()))
                                                        .unwrap_or_default()
                                                }}
                                            </td>
                                            <td class="px-4 py-4 text-sm text-subtle-foreground">
                                                {move || {
                                                    let is_locked = users
                                                        .get()
                                                        .into_iter()
                                                        .find(|item| item.id == id_for_status)
                                                        .map(|item| item.is_locked)
                                                        .unwrap_or(false);
                                                    if is_locked {
                                                        translate(locale.get(), Key::AdminSignInDisabled)
                                                    } else {
                                                        translate(locale.get(), Key::CommonActive)
                                                    }
                                                }}
                                            </td>
                                            <td class="px-4 py-4 text-sm">
                                                <div class="flex flex-wrap items-center gap-2">
                                                    <Show when=move || !is_self.get()>
                                                        <button
                                                            type="button"
                                                            on:click=move |_| {
                                                                let user_id = edit_user_id.get_value();
                                                                let target = users
                                                                    .get_untracked()
                                                                    .into_iter()
                                                                    .find(|item| item.id == user_id);
                                                                if let Some(target) = target {
                                                                    open_edit(target);
                                                                }
                                                            }
                                                            class=BTN_SECONDARY_SM
                                                        >
                                                            {text(Key::AdminActionEdit)}
                                                        </button>
                                                        <button
                                                            type="button"
                                                            on:click=move |_| {
                                                                let user_id = delete_user_id.get_value();
                                                                let candidate = users
                                                                    .get()
                                                                    .into_iter()
                                                                    .find(|candidate| candidate.id == user_id);
                                                                set_delete_candidate.set(candidate);
                                                            }
                                                            class=BTN_DANGER_SM
                                                        >
                                                            {text(Key::AdminActionDelete)}
                                                        </button>
                                                    </Show>
                                                    <Show when=move || is_self.get()>
                                                        <span class="text-xs text-faint-foreground">{text(Key::AdminActionYou)}</span>
                                                    </Show>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                    </div>

                    <ul class="space-y-3 p-4 md:hidden">
                        <For
                            each=move || page_users.get()
                            key=|user| user.id.clone()
                            children=move |user| {
                                let id_for_self = user.id.clone();
                                let is_self = Signal::derive(move || {
                                    current_user_id
                                        .get()
                                        .is_some_and(|current_id| current_id == id_for_self)
                                });
                                let edit_user_id = StoredValue::new(user.id.clone());
                                let delete_user_id = StoredValue::new(user.id.clone());
                                let email = user.email.clone();
                                let full_name = user.full_name.clone();
                                let id_for_role = user.id.clone();
                                let id_for_premium = user.id.clone();
                                let id_for_status = user.id.clone();

                                view! {
                                    <li class="rounded-2xl border border-line bg-canvas p-4">
                                        <p class="break-words text-sm font-semibold text-foreground">
                                            {email}
                                        </p>
                                        {full_name.map(|name| view! {
                                            <p class="mt-1 break-words text-xs text-faint-foreground">{name}</p>
                                        })}
                                        <dl class="mt-3 grid grid-cols-2 gap-2 text-xs">
                                            <div>
                                                <dt class="text-faint-foreground">{text(Key::AdminRole)}</dt>
                                                <dd class="text-muted-foreground">
                                                    {move || {
                                                        let is_admin = users
                                                            .get()
                                                            .into_iter()
                                                            .find(|item| item.id == id_for_role)
                                                            .map(|item| item.is_admin)
                                                            .unwrap_or(false);
                                                        if is_admin {
                                                            translate(locale.get(), Key::CommonRoleAdmin)
                                                        } else {
                                                            translate(locale.get(), Key::CommonRoleUser)
                                                        }
                                                    }}
                                                </dd>
                                            </div>
                                            <div>
                                                <dt class="text-faint-foreground">"Premium"</dt>
                                                <dd class="text-muted-foreground">
                                                    {move || {
                                                        users
                                                            .get()
                                                            .into_iter()
                                                            .find(|item| item.id == id_for_premium)
                                                            .map(|item| premium_label(&item, locale.get()))
                                                            .unwrap_or_default()
                                                    }}
                                                </dd>
                                            </div>
                                            <div>
                                                <dt class="text-faint-foreground">{text(Key::AdminStatus)}</dt>
                                                <dd class="text-muted-foreground">
                                                    {move || {
                                                        let is_locked = users
                                                            .get()
                                                            .into_iter()
                                                            .find(|item| item.id == id_for_status)
                                                            .map(|item| item.is_locked)
                                                            .unwrap_or(false);
                                                        if is_locked {
                                                            translate(locale.get(), Key::CommonLocked)
                                                        } else {
                                                            translate(locale.get(), Key::CommonActive)
                                                        }
                                                    }}
                                                </dd>
                                            </div>
                                        </dl>
                                        <div class="mt-3 flex flex-wrap items-center gap-2">
                                            <Show when=move || !is_self.get()>
                                                <button
                                                    type="button"
                                                    on:click=move |_| {
                                                        let user_id = edit_user_id.get_value();
                                                        let target = users
                                                            .get_untracked()
                                                            .into_iter()
                                                            .find(|item| item.id == user_id);
                                                        if let Some(target) = target {
                                                            open_edit(target);
                                                        }
                                                    }
                                                    class=BTN_SECONDARY_SM
                                                >
                                                    {text(Key::AdminActionEdit)}
                                                </button>
                                                <button
                                                    type="button"
                                                    on:click=move |_| {
                                                        let user_id = delete_user_id.get_value();
                                                        let candidate = users
                                                            .get()
                                                            .into_iter()
                                                            .find(|candidate| candidate.id == user_id);
                                                        set_delete_candidate.set(candidate);
                                                    }
                                                    class=BTN_DANGER_SM
                                                >
                                                    {text(Key::AdminActionDelete)}
                                                </button>
                                            </Show>
                                            <Show when=move || is_self.get()>
                                                <span class="text-xs text-faint-foreground">{text(Key::AdminActionYou)}</span>
                                            </Show>
                                        </div>
                                    </li>
                                }
                            }
                        />
                    </ul>
                    <Pagination
                        can_go_back=Signal::derive(move || page.get() > 1)
                        can_go_forward=Signal::derive(move || page.get() < total_pages.get())
                        on_previous=Callback::new(move |_| {
                            set_page.update(|value| *value = value.saturating_sub(1).max(1));
                            reload_users();
                        })
                        on_next=Callback::new(move |_| {
                            set_page.update(|value| *value += 1);
                            reload_users();
                        })
                    >
                        {move || if page_size.get() == ALL_PAGE_SIZE {
                            format!(
                                "{} · {} {}",
                                translate(locale.get(), Key::AdminPaginationAll),
                                users.get().len(),
                                translate(locale.get(), Key::AdminPaginationItems)
                            )
                        } else {
                            format!(
                                "{} {} {} {}",
                                translate(locale.get(), Key::AdminPaginationPage),
                                page.get().min(total_pages.get()),
                                translate(locale.get(), Key::AdminPaginationOf),
                                total_pages.get()
                            )
                        }}
                    </Pagination>
                </section>
            </section>

            // CREATE MODAL
            <Modal
                open=show_create
                on_close=Callback::new(move |_| set_show_create.set(false))
                close_label={translate(locale.get_untracked(), Key::AdminCreateCloseLabel).to_owned()}
            >
                <div class="flex items-start justify-between gap-4">
                    <div>
                        <p class="text-xs font-bold uppercase tracking-widest text-accent">{text(Key::AdminCreateEyebrow)}</p>
                        <h2 class="mt-1 text-xl font-bold text-foreground">{text(Key::AdminCreateTitle)}</h2>
                    </div>
                    <button
                        type="button"
                        on:click=move |_| set_show_create.set(false)
                        class=BTN_SECONDARY_SM
                    >
                        <span aria-hidden="true">"\u{2715}"</span>
                        {text(Key::AdminActionClose)}
                    </button>
                </div>
                <form id="create-user-form" on:submit=create_user class="mt-6 space-y-4">
                    <Field label="Email".to_owned()>
                        <input
                            type="email"
                            required
                            placeholder="user@example.com"
                            prop:value=new_email
                            on:input=move |ev| set_new_email.set(event_target_value(&ev))
                            class=INPUT
                        />
                    </Field>
                    <Field label={translate(locale.get_untracked(), Key::AdminRole).to_owned()}>
                        <div class="relative">
                            <select
                                aria-label={translate(locale.get_untracked(), Key::AdminRole)}
                                prop:value=new_role
                                on:change=move |ev| set_new_role.set(event_target_value(&ev))
                                class=SELECT
                            >
                                <option value="none" class="bg-canvas-raised text-foreground">{text(Key::CommonRoleUser)}</option>
                                <option value="admin" class="bg-canvas-raised text-foreground">{text(Key::CommonRoleAdmin)}</option>
                            </select>
                            <span class=SELECT_CHEVRON aria-hidden="true">"\u{25BE}"</span>
                        </div>
                    </Field>
                    <Show when=move || new_role.get() != "admin">
                        <ToggleRow
                            title="Premium".to_owned()
                            description={translate(locale.get_untracked(), Key::AdminPremiumGrantCreate).to_owned()}
                            checked=new_premium_active
                            on_toggle=Callback::new(move |_| set_new_premium_active.update(|v| *v = !*v))
                        />
                    </Show>
                    <Show when=move || new_role.get() != "admin" && new_premium_active.get()>
                        <Field
                            label={translate(locale.get_untracked(), Key::AdminPremiumExpiryLabel).to_owned()}
                            description={translate(locale.get_untracked(), Key::AdminPremiumExpiryHint).to_owned()}
                        >
                            <div class="flex gap-2">
                                <div class="min-w-0 flex-1">
                                    <input
                                        node_ref=new_premium_expires_input
                                        type="date"
                                        aria-label={translate(locale.get_untracked(), Key::AdminPremiumExpiryAria)}
                                        prop:value=new_premium_expires
                                        on:input=move |ev| set_new_premium_expires.set(event_target_value(&ev))
                                        class=INPUT
                                    />
                                </div>
                                <button
                                    type="button"
                                    aria-label={translate(locale.get_untracked(), Key::AdminPremiumExpiryPicker)}
                                    on:click=move |_| {
                                        if let Some(input) = new_premium_expires_input.get() {
                                            let _ = input.show_picker();
                                        }
                                    }
                                    class=BTN_SECONDARY_SM
                                >
                                    <span aria-hidden="true">"📅"</span>
                                </button>
                            </div>
                        </Field>
                    </Show>
                    <p class="text-xs text-faint-foreground">{text(Key::AdminCreateActiveNote)}</p>
                </form>
                <div class="mt-6 flex justify-end gap-3 border-t border-line pt-5">
                    <button
                        type="button"
                        on:click=move |_| set_show_create.set(false)
                        class=BTN_SECONDARY_SM
                    >
                        {text(Key::AdminActionCancel)}
                    </button>
                    <button type="submit" form="create-user-form" class=BTN_SECONDARY_SM>{text(Key::AdminCreateTitle)}</button>
                    <button
                        type="button"
                        on:click=create_and_send_email
                        class=BTN_PRIMARY
                    >
                        {text(Key::AdminCreateSendInvite)}
                    </button>
                </div>
            </Modal>

            // EDIT MODAL
            <Modal
                open=Signal::derive(move || selected.get().is_some())
                on_close=Callback::new(move |_| set_selected.set(None))
                close_label={translate(locale.get_untracked(), Key::AdminEditCloseLabel).to_owned()}
            >
                <div class="flex items-start justify-between gap-4">
                    <div class="min-w-0 flex-1">
                        <p class="text-xs font-bold uppercase tracking-widest text-accent">{text(Key::AdminEditEyebrow)}</p>
                        <h2 class="mt-1 truncate text-xl font-bold text-foreground" title=move || edit_email.get()>{move || edit_email.get()}</h2>
                    </div>
                    <button
                        type="button"
                        on:click=move |_| set_selected.set(None)
                        class=BTN_SECONDARY_SM
                    >
                        <span aria-hidden="true">"\u{2715}"</span>
                        {text(Key::AdminActionClose)}
                    </button>
                </div>
                <div class="mt-5 rounded-xl border border-line bg-surface px-4 py-3">
                    <p class="text-xs font-medium uppercase tracking-widest text-faint-foreground">"Email"</p>
                    <p class="mt-1 truncate text-sm text-muted-foreground" title=move || edit_email.get()>{move || edit_email.get()}</p>
                </div>
                <form id="edit-user-form" on:submit=save_changes class="mt-5 space-y-4">
                    <Field
                        label={translate(locale.get_untracked(), Key::AdminRole).to_owned()}
                        description=if current_user_id
                            .get()
                            .is_some_and(|id| selected.get().as_deref() == Some(id.as_str()))
                        {
                            translate(locale.get_untracked(), Key::AdminEditRoleLockedHint).to_owned()
                        } else {
                            String::new()
                        }
                    >
                        <div class="relative">
                            <select
                                aria-label={translate(locale.get_untracked(), Key::AdminRole)}
                                prop:value=edit_role
                                disabled=move || {
                                    current_user_id
                                        .get()
                                        .is_some_and(|id| selected.get().as_deref() == Some(id.as_str()))
                                }
                                on:change=move |ev| {
                                    let role = event_target_value(&ev);
                                    if role == "admin" {
                                        set_premium_active.set(true);
                                        set_premium_expires.set(String::new());
                                    }
                                    set_edit_role.set(role);
                                }
                                class=SELECT
                            >
                                <option value="none" class="bg-canvas-raised text-foreground">{text(Key::CommonRoleUser)}</option>
                                <option value="admin" class="bg-canvas-raised text-foreground">{text(Key::CommonRoleAdmin)}</option>
                            </select>
                            <span class=SELECT_CHEVRON aria-hidden="true">"\u{25BE}"</span>
                        </div>
                    </Field>
                    <Show when=move || edit_role.get() != "admin">
                        <ToggleRow
                            title="Premium".to_owned()
                            description={translate(locale.get_untracked(), Key::AdminPremiumGrantEdit).to_owned()}
                            checked=premium_active
                            on_toggle=Callback::new(move |_| set_premium_active.update(|v| *v = !*v))
                        />
                    </Show>
                            // Premium expiry
                            <Show when=move || edit_role.get() != "admin" && premium_active.get()>
                                <Field label={translate(locale.get_untracked(), Key::AdminPremiumExpiryLabel).to_owned()} description={translate(locale.get_untracked(), Key::AdminPremiumExpiryHint).to_owned()}>
                                    <div class="flex gap-2">
                                        <div class="min-w-0 flex-1">
                                            <input
                                                node_ref=premium_expires_input
                                                type="date"
                                                aria-label={translate(locale.get_untracked(), Key::AdminPremiumExpiryAria)}
                                                prop:value=premium_expires
                                                on:input=move |ev| set_premium_expires.set(event_target_value(&ev))
                                                class=INPUT
                                            />
                                        </div>
                                        <button
                                            type="button"
                                            aria-label={translate(locale.get_untracked(), Key::AdminPremiumExpiryPicker)}
                                            on:click=move |_| {
                                                if let Some(input) = premium_expires_input.get() {
                                                    let _ = input.show_picker();
                                                }
                                            }
                                            class=BTN_SECONDARY_SM
                                        >
                                            <span aria-hidden="true">"\u{1F4C5}"</span>
                                        </button>
                                    </div>
                                </Field>
                            </Show>
                            // Sign-in access toggle; the current user cannot disable their own account.
                            <Show when=move || {
                                current_user_id
                                    .get()
                                    .is_none_or(|id| selected.get().as_deref() != Some(id.as_str()))
                            }>
                            <div class=move || format!("flex items-center justify-between gap-4 rounded-xl border px-4 py-3 {}", if edit_locked.get() { "border-amber-300/20 bg-amber-300/5" } else { "border-line bg-canvas" })>
                                <div>
                                    <p class=move || format!("text-sm font-medium {}", if edit_locked.get() { "text-amber-200" } else { "text-muted-foreground" })>
                                        {text(Key::AdminSignInAccess)}
                                    </p>
                                    <p class="text-xs text-faint-foreground">
                                        {move || if edit_locked.get() {
                                            translate(locale.get(), Key::AdminEditCannotSignIn)
                                        } else {
                                            translate(locale.get(), Key::AdminEditCanSignIn)
                                        }}
                                    </p>
                                </div>
                                <button
                                    type="button"
                                    role="switch"
                                    aria-label={translate(locale.get_untracked(), Key::AdminEditDisableSignInAria)}
                                    aria-checked=move || edit_locked.get().to_string()
                                    on:click=move |_| set_edit_locked.update(|v| *v = !*v)
                                    class=move || format!("relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas {}", if edit_locked.get() { "bg-amber-400" } else { "bg-accent" })
                                >
                                    <span class=move || format!("pointer-events-none inline-block size-5 rounded-full bg-white shadow transition {}", if edit_locked.get() { "translate-x-5" } else { "translate-x-0" }) />
                                </button>
                            </div>
                            </Show>
                        </form>
                <div class="mt-6 flex justify-end gap-3 border-t border-line pt-5">
                    <button type="button" on:click=move |_| set_selected.set(None) class=BTN_SECONDARY_SM>{text(Key::AdminActionCancel)}</button>
                    <button type="submit" form="edit-user-form" class=BTN_PRIMARY>{text(Key::AdminEditSave)}</button>
                </div>
            </Modal>

            <Modal
                open=Signal::derive(move || delete_candidate.get().is_some())
                on_close=Callback::new(move |_| set_delete_candidate.set(None))
                close_label={translate(locale.get_untracked(), Key::AdminDeleteCloseLabel).to_owned()}
            >
                <p class="text-sm font-bold uppercase tracking-widest text-danger">{text(Key::AdminDeleteTitle)}</p>
                <h2 class="mt-3 text-xl font-bold text-foreground">{text(Key::AdminDeleteConfirm)}</h2>
                <p class="mt-3 text-sm leading-6 text-subtle-foreground">
                    {text(Key::AdminDeletePrefix)}
                    {move || delete_candidate.get().map(|user| user.email)}
                    {text(Key::AdminDeleteSuffix)}
                </p>
                <div class="mt-6 flex justify-end gap-3">
                    <button
                        type="button"
                        on:click=move |_| set_delete_candidate.set(None)
                        class=BTN_SECONDARY_SM
                    >
                        {text(Key::AdminActionCancel)}
                    </button>
                    <button
                        type="button"
                        on:click=move |_| {
                            if let Some(user) = delete_candidate.get() {
                                set_delete_candidate.set(None);
                                delete_user(user.id);
                            }
                        }
                        class=BTN_DANGER
                    >
                        {text(Key::AdminDeleteTitle)}
                    </button>
                </div>
            </Modal>
        </div>
    }
}
