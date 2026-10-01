use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json};
#[cfg(feature = "hydrate")]
use crate::types::{PremiumResponse, UserListResponse};
use crate::types::UserResponse;

const PAGE_SIZE: usize = 10;

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

    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(
        &format!("{value}T23:59:59Z"),
    ));
    let milliseconds = date.get_time();
    if milliseconds.is_finite() {
        Some((milliseconds / 1000.0) as i64)
    } else {
        None
    }
}

#[component]
#[allow(unused_variables)]
pub fn AdminPage() -> impl IntoView {
    let (users, set_users) = signal(Vec::<UserResponse>::new());
    let (current_user_id, set_current_user_id) = signal(None::<String>);
    let (status, set_status) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());

    let (selected, set_selected) = signal(None::<String>);
    let (edit_email, set_edit_email) = signal(String::new());
    let (edit_role, set_edit_role) = signal("none".to_owned());
    let (premium_active, set_premium_active) = signal(false);
    let (premium_expires, set_premium_expires) = signal(String::new());

    let (search, set_search) = signal(String::new());
    let (role_filter, set_role_filter) = signal("all".to_owned());
    let (premium_filter, set_premium_filter) = signal("all".to_owned());
    let (status_filter, set_status_filter) = signal("all".to_owned());
    let (page, set_page) = signal(1usize);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local({
            async move {
                if let Ok(current_user) =
                    api_json::<UserResponse>(gloo_net::http::Method::GET, "/api/v1/auth/me", None)
                        .await
                {
                    set_current_user_id.set(Some(current_user.id));
                }

                match api_json::<UserListResponse>(
                    gloo_net::http::Method::GET,
                    "/api/v1/admin/users",
                    None,
                )
                .await
                {
                    Ok(response) => set_users.set(response.users),
                    Err(error) => set_status.set(error),
                }
            }
        });
    }

    let reload_users = move || {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local({
            async move {
                match api_json::<UserListResponse>(
                    gloo_net::http::Method::GET,
                    "/api/v1/admin/users",
                    None,
                )
                .await
                {
                    Ok(response) => set_users.set(response.users),
                    Err(error) => set_status.set(error),
                }
            }
        });
    };

    let create_user = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let email = new_email.get();
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_json::<UserResponse>(
                gloo_net::http::Method::POST,
                "/api/v1/admin/users",
                Some(serde_json::json!({ "email": email }).to_string()),
            )
            .await
            {
                Ok(_) => {
                    set_new_email.set(String::new());
                    set_status.set("User created.".to_owned());
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
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
                    set_status.set("User deleted.".to_owned());
                    if selected.get().as_deref() == Some(user_id.as_str()) {
                        set_selected.set(None);
                    }
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    let unlock_user = move |user_id: String| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_json::<UserResponse>(
                gloo_net::http::Method::POST,
                &format!("/api/v1/admin/users/{user_id}/unlock"),
                None,
            )
            .await
            {
                Ok(_) => {
                    set_status.set("User unlocked.".to_owned());
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    let save_user = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(user_id) = selected.get() else {
            return;
        };
        let email = edit_email.get();
        let role = edit_role.get();

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let email_result = api_json::<UserResponse>(
                gloo_net::http::Method::PATCH,
                &format!("/api/v1/admin/users/{user_id}"),
                Some(serde_json::json!({ "email": email }).to_string()),
            )
            .await;

            if let Err(error) = email_result {
                set_status.set(error);
                return;
            }

            match api_json::<UserResponse>(
                gloo_net::http::Method::PUT,
                &format!("/api/v1/admin/users/{user_id}/role"),
                Some(serde_json::json!({ "role": role }).to_string()),
            )
            .await
            {
                Ok(_) => {
                    set_status.set("User changes saved.".to_owned());
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    let save_premium = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(user_id) = selected.get() else {
            return;
        };
        let active = premium_active.get();
        let expiry = premium_expires.get();

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let expires_at = date_to_timestamp(&expiry);
            match api_json::<UserResponse>(
                gloo_net::http::Method::PUT,
                &format!("/api/v1/admin/users/{user_id}/entitlements/premium"),
                Some(
                    serde_json::json!({
                        "active": active,
                        "expires_at": expires_at
                    })
                    .to_string(),
                ),
            )
            .await
            {
                Ok(_) => {
                    set_status.set("Premium entitlement saved.".to_owned());
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    let revoke_premium = move |_| {
        let Some(user_id) = selected.get() else {
            return;
        };

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_json::<UserResponse>(
                gloo_net::http::Method::DELETE,
                &format!("/api/v1/admin/users/{user_id}/entitlements/premium"),
                None,
            )
            .await
            {
                Ok(_) => {
                    set_premium_active.set(false);
                    set_premium_expires.set(String::new());
                    set_status.set("Premium entitlement revoked.".to_owned());
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    let open_edit = move |user: UserResponse| {
        set_selected.set(Some(user.id.clone()));
        set_edit_email.set(user.email);
        set_edit_role.set(if user.is_admin {
            "admin".to_owned()
        } else {
            "none".to_owned()
        });
        set_premium_active.set(user.is_premium);
        set_premium_expires.set(String::new());

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
                Err(error) => set_status.set(error),
            }
        });
    };

    let filtered_users = move || {
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
    };

    let total_pages = move || filtered_users().len().div_ceil(PAGE_SIZE).max(1);

    let page_users = move || {
        let total = total_pages();
        let current_page = page.get().min(total).max(1);
        filtered_users()
            .into_iter()
            .skip((current_page - 1) * PAGE_SIZE)
            .take(PAGE_SIZE)
            .collect::<Vec<_>>()
    };

    view! {
        <div class="mx-auto max-w-7xl space-y-8 px-5 py-12 sm:px-8 lg:px-10">
            <section>
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Administration"</p>
                <h1 class="mt-3 text-4xl font-black text-white">"Users"</h1>
                <p class="mt-3 text-sm text-slate-400">{status}</p>
            </section>

            <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                <div class="flex flex-col gap-4 lg:flex-row lg:items-end">
                    <div class="min-w-0 flex-1">
                        <h2 class="text-xl font-bold text-white">"Create user"</h2>
                        <form on:submit=create_user class="mt-4 flex flex-col gap-3 sm:flex-row">
                            <input
                                type="email"
                                required
                                placeholder="user@example.com"
                                prop:value=new_email
                                on:input=move |ev| set_new_email.set(event_target_value(&ev))
                                class="min-w-0 flex-1 rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"
                            />
                            <button class="rounded-xl bg-cyan-300 px-5 py-3 font-bold text-slate-950" type="submit">
                                "Create"
                            </button>
                        </form>
                    </div>
                </div>
            </section>

            <section class="space-y-4">
                <div class="grid gap-3 md:grid-cols-2 lg:grid-cols-4">
                    <input
                        type="search"
                        placeholder="Search email or name..."
                        prop:value=search
                        on:input=move |ev| {
                            set_search.set(event_target_value(&ev));
                            set_page.set(1);
                        }
                        class="rounded-xl border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-white"
                    />
                    <select
                        prop:value=role_filter
                        on:change=move |ev| {
                            set_role_filter.set(event_target_value(&ev));
                            set_page.set(1);
                        }
                        class="rounded-xl border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-white"
                    >
                        <option value="all">"All roles"</option>
                        <option value="admin">"Admin"</option>
                        <option value="user">"User"</option>
                    </select>
                    <select
                        prop:value=premium_filter
                        on:change=move |ev| {
                            set_premium_filter.set(event_target_value(&ev));
                            set_page.set(1);
                        }
                        class="rounded-xl border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-white"
                    >
                        <option value="all">"All premium states"</option>
                        <option value="active">"Premium"</option>
                        <option value="inactive">"Not premium"</option>
                    </select>
                    <select
                        prop:value=status_filter
                        on:change=move |ev| {
                            set_status_filter.set(event_target_value(&ev));
                            set_page.set(1);
                        }
                        class="rounded-xl border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-white"
                    >
                        <option value="all">"All account states"</option>
                        <option value="active">"Active"</option>
                        <option value="locked">"Locked"</option>
                    </select>
                </div>

                <section class="overflow-x-auto rounded-3xl border border-white/10 bg-white/[0.03]">
                    <table class="w-full min-w-[760px] text-left">
                        <thead>
                            <tr class="text-xs uppercase tracking-widest text-slate-500">
                                <th class="px-4 py-4">"Email"</th>
                                <th class="px-4 py-4">"Role"</th>
                                <th class="px-4 py-4">"Premium"</th>
                                <th class="px-4 py-4">"Status"</th>
                                <th class="px-4 py-4">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || page_users()
                                key=|user| user.id.clone()
                                children=move |user| {
                                    let is_self = current_user_id
                                        .get()
                                        .is_some_and(|current_id| current_id == user.id);

                                    view! {
                                        <tr class="border-t border-white/10">
                                            <td class="px-4 py-4 text-sm text-white">
                                                <div>{user.email.clone()}</div>
                                                {user.full_name.clone().map(|name| view! {
                                                    <div class="mt-1 text-xs text-slate-500">{name}</div>
                                                })}
                                            </td>
                                            <td class="px-4 py-4 text-sm text-slate-400">
                                                {if user.is_admin { "Admin" } else { "User" }}
                                            </td>
                                            <td class="px-4 py-4 text-sm text-slate-400">
                                                {if user.is_premium { "Active" } else { "Inactive" }}
                                            </td>
                                            <td class="px-4 py-4 text-sm text-slate-400">
                                                {if user.is_locked { "Locked" } else { "Active" }}
                                            </td>
                                            <td class="px-4 py-4 text-sm">
                                                <div class="flex flex-wrap gap-2">
                                                    <button
                                                        type="button"
                                                        on:click=move |_| open_edit(user.clone())
                                                        class="rounded-lg border border-white/10 px-3 py-2 text-slate-300 hover:bg-white/10"
                                                    >
                                                        "Edit"
                                                    </button>
                                                    <Show when=move || !is_self>
                                                        <button
                                                            type="button"
                                                            on:click=move |_| delete_user(user.id.clone())
                                                            class="rounded-lg bg-red-300/10 px-3 py-2 text-red-200 hover:bg-red-300/20"
                                                        >
                                                            "Delete"
                                                        </button>
                                                    </Show>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                    <div class="flex items-center justify-between border-t border-white/10 px-4 py-4 text-sm text-slate-400">
                        <span>
                            {move || format!("Page {} of {}", page.get().min(total_pages()), total_pages())}
                        </span>
                        <div class="flex gap-2">
                            <button
                                type="button"
                                disabled=move || page.get() <= 1
                                on:click=move |_| set_page.update(|value| *value = value.saturating_sub(1).max(1))
                                class="rounded-lg border border-white/10 px-3 py-2 disabled:opacity-40"
                            >
                                "Previous"
                            </button>
                            <button
                                type="button"
                                disabled=move || page.get() >= total_pages()
                                on:click=move |_| set_page.update(|value| *value += 1)
                                class="rounded-lg border border-white/10 px-3 py-2 disabled:opacity-40"
                            >
                                "Next"
                            </button>
                        </div>
                    </div>
                </section>
            </section>

            <Show when=move || selected.get().is_some()>
                <section class="rounded-3xl border border-cyan-300/20 bg-white/[0.03] p-6">
                    <div class="flex items-center justify-between gap-4">
                        <div>
                            <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Edit user"</p>
                            <h2 class="mt-2 text-xl font-bold text-white">{move || edit_email.get()}</h2>
                        </div>
                        <button
                            type="button"
                            on:click=move |_| set_selected.set(None)
                            class="rounded-lg border border-white/10 px-3 py-2 text-slate-300"
                        >
                            "Close"
                        </button>
                    </div>

                    <form on:submit=save_user class="mt-6 space-y-5">
                        <div class="grid gap-4 md:grid-cols-2">
                            <label class="block">
                                <span class="text-sm font-medium text-slate-300">"Email"</span>
                                <input
                                    type="email"
                                    required
                                    prop:value=edit_email
                                    on:input=move |ev| set_edit_email.set(event_target_value(&ev))
                                    class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"
                                />
                            </label>
                            <label class="block">
                                <span class="text-sm font-medium text-slate-300">"Role"</span>
                                <select
                                    prop:value=edit_role
                                    on:change=move |ev| set_edit_role.set(event_target_value(&ev))
                                    class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"
                                >
                                    <option value="none">"User"</option>
                                    <option value="admin">"Admin"</option>
                                </select>
                            </label>
                        </div>
                        <button type="submit" class="rounded-xl bg-cyan-300 px-5 py-3 font-bold text-slate-950">
                            "Save user changes"
                        </button>
                    </form>

                    <div class="mt-8 border-t border-white/10 pt-6">
                        <div class="flex items-center justify-between gap-4">
                            <div>
                                <h3 class="text-lg font-bold text-white">"Premium"</h3>
                                <p class="mt-1 text-sm text-slate-500">"Choose an optional expiry date. The date is applied through the existing Premium entitlement API."</p>
                            </div>
                            <span class="text-sm text-slate-400">
                                {move || if premium_active.get() { "Active" } else { "Inactive" }}
                            </span>
                        </div>

                        <form on:submit=save_premium class="mt-4 flex flex-col gap-3 sm:flex-row sm:items-end">
                            <label class="block flex-1">
                                <span class="text-sm font-medium text-slate-300">"Premium expiry"</span>
                                <input
                                    type="date"
                                    prop:value=premium_expires
                                    on:input=move |ev| set_premium_expires.set(event_target_value(&ev))
                                    class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"
                                />
                            </label>
                            <label class="flex items-center gap-2 rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-sm text-slate-300">
                                <input
                                    type="checkbox"
                                    prop:checked=premium_active
                                    on:change=move |ev| set_premium_active.set(event_target_checked(&ev))
                                />
                                "Premium active"
                            </label>
                            <button type="submit" class="rounded-xl bg-cyan-300 px-5 py-3 font-bold text-slate-950">
                                "Save Premium"
                            </button>
                            <button
                                type="button"
                                on:click=revoke_premium
                                class="rounded-xl bg-red-300/10 px-5 py-3 text-red-200"
                            >
                                "Revoke"
                            </button>
                        </form>
                    </div>

                    <Show when=move || selected.get().and_then(|id| users.get().into_iter().find(|user| user.id == id)).is_some_and(|user| user.is_locked)>
                        <div class="mt-8 border-t border-white/10 pt-6">
                            <h3 class="text-lg font-bold text-white">"Account status"</h3>
                            <button
                                type="button"
                                on:click=move |_| {
                                    if let Some(user_id) = selected.get() {
                                        unlock_user(user_id);
                                    }
                                }
                                class="mt-3 rounded-xl bg-amber-300/10 px-5 py-3 text-amber-200"
                            >
                                "Unlock user"
                            </button>
                        </div>
                    </Show>
                </section>
            </Show>
        </div>
    }
}
