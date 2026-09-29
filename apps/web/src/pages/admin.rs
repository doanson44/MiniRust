use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json};
use crate::types::{UserListResponse, UserResponse};

#[component]
#[allow(unused_variables)]
pub fn AdminPage() -> impl IntoView {
    let (users, set_users) = signal(Vec::<UserResponse>::new());
    let (status, set_status) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());

    let (selected, set_selected) = signal(None::<String>);
    let (edit_email, set_edit_email) = signal(String::new());
    let (premium_expires, set_premium_expires) = signal(String::new());

    #[cfg(feature = "hydrate")]
    {
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

    let action_user = move |email: String, action: &'static str| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let user = users.get_untracked().into_iter().find(|u| u.email == email);
            let Some(user) = user else { return };
            let (method, path) = match action {
                "unlock" => (
                    gloo_net::http::Method::POST,
                    format!("/api/v1/admin/users/{}/unlock", user.id),
                ),
                "delete" => (
                    gloo_net::http::Method::DELETE,
                    format!("/api/v1/admin/users/{}", user.id),
                ),
                _ => return,
            };
            match api_empty(method, &path, None).await {
                Ok(()) => {
                    set_status.set(format!("Action {} succeeded.", action));
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    let role_user = move |email: String, role: String| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let user = users.get_untracked().into_iter().find(|u| u.email == email);
            let Some(user) = user else { return };
            match api_json::<UserResponse>(
                gloo_net::http::Method::PUT,
                &format!("/api/v1/admin/users/{}/role", user.id),
                Some(serde_json::json!({ "role": role }).to_string()),
            )
            .await
            {
                Ok(_) => reload_users(),
                Err(error) => set_status.set(error),
            }
        });
    };

    let premium_user = move |email: String, active: bool, expires: Option<i64>| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let user = users.get_untracked().into_iter().find(|u| u.email == email);
            let Some(user) = user else { return };
            let result = if active {
                api_json::<UserResponse>(
                    gloo_net::http::Method::PUT,
                    &format!("/api/v1/admin/users/{}/entitlements/premium", user.id),
                    Some(serde_json::json!({ "active": true, "expires_at": expires }).to_string()),
                )
                .await
            } else {
                api_json::<UserResponse>(
                    gloo_net::http::Method::DELETE,
                    &format!("/api/v1/admin/users/{}/entitlements/premium", user.id),
                    None,
                )
                .await
            };
            match result {
                Ok(_) => {
                    set_premium_expires.set(String::new());
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    let update_user = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let old_email = selected.get();
        let new_email = edit_email.get();
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let Some(old_email) = old_email else { return };
            let user = users
                .get_untracked()
                .into_iter()
                .find(|u| u.email == old_email);
            let Some(user) = user else { return };

            match api_json::<UserResponse>(
                gloo_net::http::Method::PATCH,
                &format!("/api/v1/admin/users/{}", user.id),
                Some(serde_json::json!({ "email": new_email }).to_string()),
            )
            .await
            {
                Ok(_) => {
                    set_selected.set(None);
                    set_status.set("User updated.".to_owned());
                    reload_users();
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    view! {
        <div class="mx-auto max-w-7xl space-y-8 px-5 py-12 sm:px-8 lg:px-10">
                <section>
                    <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Administration"</p>
                    <h1 class="mt-3 text-4xl font-black text-white">"Users"</h1>
                    <p class="mt-3 text-sm text-slate-400">{status}</p>
                </section>

                <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                    <h2 class="text-xl font-bold text-white">"Create user"</h2>
                    <form on:submit=create_user class="mt-4 flex flex-col gap-3 sm:flex-row">
                        <input type="email" required placeholder="user@example.com" prop:value=new_email on:input=move |ev| set_new_email.set(event_target_value(&ev)) class="min-w-0 flex-1 rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/>
                        <button class="rounded-xl bg-cyan-300 px-5 py-3 font-bold text-slate-950" type="submit">"Create"</button>
                    </form>
                </section>

                <section class="overflow-x-auto rounded-3xl border border-white/10 bg-white/[0.03]">
                    <table class="w-full min-w-[980px] text-left">
                        <thead><tr class="text-xs uppercase tracking-widest text-slate-500">
                            <th class="px-4 py-4">"Email"</th><th class="px-4 py-4">"Role"</th><th class="px-4 py-4">"Premium"</th><th class="px-4 py-4">"Status"</th><th class="px-4 py-4">"Actions"</th>
                        </tr></thead>
                        <tbody>
                            <For
                                each=move || users.get()
                                key=|user| user.id.clone()
                                children=move |user| {
                                    let email = StoredValue::new(user.email.clone());
                                    let is_admin = user.is_admin;
                                    let is_premium = user.is_premium;
                                    let is_locked = user.is_locked;

                                    view! {
                                        <tr class="border-t border-white/10 align-top">
                                            <td class="px-4 py-4 text-sm text-white">{email.with_value(|value| value.clone())}</td>
                                            <td class="px-4 py-4 text-sm text-slate-400">
                                                <select on:change=move |ev| {
                                                    let role = event_target_value(&ev);
                                                    role_user(email.with_value(|value| value.clone()), role);
                                                } class="rounded-lg border border-white/10 bg-slate-950 px-2 py-2">
                                                    <option value="user" selected=move || !is_admin>"User"</option>
                                                    <option value="admin" selected=move || is_admin>"Admin"</option>
                                                </select>
                                            </td>
                                            <td class="px-4 py-4 text-sm text-slate-400">
                                                <div>{if is_premium {"Active"} else {"Inactive"}}</div>
                                                <div class="mt-2 flex gap-2">
                                                    <input type="number" placeholder="Unix expiry timestamp (optional)" class="w-44 rounded-lg border border-white/10 bg-slate-950 px-2 py-2 text-xs text-white"
                                                        on:input=move |ev| set_premium_expires.set(event_target_value(&ev))/>
                                                    <button on:click=move |_| {
                                                        let value = premium_expires.get();
                                                        let timestamp = if value.is_empty() { None } else { value.parse::<i64>().ok() };
                                                        premium_user(email.with_value(|value| value.clone()), true, timestamp);
                                                    } class="rounded-lg bg-cyan-300/10 px-2 py-2 text-xs text-cyan-200">"Grant"</button>
                                                    <button on:click=move |_| premium_user(email.with_value(|value| value.clone()), false, None) class="rounded-lg bg-red-300/10 px-2 py-2 text-red-200">"Revoke"</button>
                                                </div>
                                            </td>
                                            <td class="px-4 py-4 text-sm text-slate-400">{if is_locked {"Locked"} else {"Active"}}</td>
                                            <td class="px-4 py-4 text-sm">
                                                <div class="flex flex-wrap gap-2">
                                                    <button on:click=move |_| {
                                                        let email = email.with_value(|value| value.clone());
                                                        set_selected.set(Some(email.clone()));
                                                        set_edit_email.set(email);
                                                    } class="rounded-lg border border-white/10 px-3 py-2 text-slate-300">"Edit"</button>
                                                    <Show when=move || is_locked>
                                                        <button on:click=move |_| action_user(email.with_value(|value| value.clone()), "unlock") class="rounded-lg bg-amber-300/10 px-3 py-2 text-amber-200">"Unlock"</button>
                                                    </Show>
                                                    <button on:click=move |_| action_user(email.with_value(|value| value.clone()), "delete") class="rounded-lg bg-red-300/10 px-3 py-2 text-red-200">"Delete"</button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }
                            />                    </tbody>
                    </table>
                </section>

                <Show when=move || selected.get().is_some()>
                    <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <h2 class="text-xl font-bold text-white">"Edit user email"</h2>
                        <form on:submit=update_user class="mt-4 flex flex-col gap-3 sm:flex-row">
                            <input type="email" required prop:value=edit_email on:input=move |ev| set_edit_email.set(event_target_value(&ev)) class="min-w-0 flex-1 rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/>
                            <button class="rounded-xl bg-cyan-300 px-5 py-3 font-bold text-slate-950" type="submit">"Save"</button>
                            <button type="button" on:click=move |_| set_selected.set(None) class="rounded-xl border border-white/10 px-5 py-3 text-white">"Cancel"</button>
                        </form>
                    </section>
                </Show>
        </div>
    }
}
