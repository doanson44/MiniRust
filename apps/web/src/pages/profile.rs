use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json};
use crate::types::UserResponse;

#[component]
#[allow(unused_variables)]
pub fn ProfilePage() -> impl IntoView {
    let (user, set_user) = signal(None::<UserResponse>);
    let (status, set_status) = signal(String::new());
    let (full_name, set_full_name) = signal(String::new());
    let (avatar_url, set_avatar_url) = signal(String::new());
    let locale = use_context::<ReadSignal<String>>().unwrap_or_else(|| signal("vi".to_owned()).0);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local({
            async move {
                match api_json::<UserResponse>(gloo_net::http::Method::GET, "/api/v1/auth/me", None)
                    .await
                {
                    Ok(user) => {
                        set_full_name.set(user.full_name.clone().unwrap_or_default());
                        set_avatar_url.set(user.avatar_url.clone().unwrap_or_default());
                        set_user.set(Some(user));
                    }
                    Err(error) => set_status.set(error),
                }
            }
        });
    }

    let update_profile = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let name = full_name.get();
        let avatar = avatar_url.get();
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_json::<UserResponse>(
                gloo_net::http::Method::PATCH,
                "/api/v1/users/me",
                Some(
                    serde_json::json!({
                        "full_name": if name.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(name) },
                        "avatar_url": if avatar.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(avatar) }
                    })
                    .to_string(),
                ),
            )
            .await
            {
                Ok(user) => {
                    set_user.set(Some(user));
                    set_status.set(if locale.get() == "vi" { "Đã cập nhật hồ sơ." } else { "Profile updated." }.to_owned());
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    let lock_account = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_empty(gloo_net::http::Method::POST, "/api/v1/users/me/lock", None).await {
                Ok(()) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    view! {
        <div class="mx-auto max-w-7xl space-y-8 px-5 py-12 sm:px-8 lg:px-10">
                <section>
                    <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">{move || if locale.get() == "vi" { "Không gian làm việc" } else { "Workspace" }}</p>
                    <h1 class="mt-3 text-4xl font-black text-white">{move || if locale.get() == "vi" { "Tài khoản của bạn" } else { "Your account" }}</h1>
                    <p class="mt-3 text-sm text-slate-400">{status}</p>
                </section>

                <Show when=move || user.get().is_some()>
                    {move || {
                        let u = user.get().unwrap();
                        view! {
                        <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                            <h2 class="text-xl font-bold text-white">{move || if locale.get() == "vi" { "Thông tin tài khoản" } else { "Account info" }}</h2>
                            <dl class="mt-4 space-y-2 text-sm">
                                <div class="flex gap-4"><dt class="w-28 text-slate-500">"Email"</dt><dd class="text-white">{u.email.clone()}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-slate-500">{move || if locale.get() == "vi" { "Vai trò" } else { "Role" }}</dt><dd class="text-white">{if u.is_admin { "Admin" } else { "User" }}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-slate-500">"Premium"</dt><dd class="text-white">{if u.is_premium { "Active" } else { "Inactive" }}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-slate-500">{move || if locale.get() == "vi" { "Trạng thái" } else { "Status" }}</dt><dd class="text-white">{if u.is_locked { "Locked" } else { "Active" }}</dd></div>
                            </dl>
                        </section>
                        }
                    }}
                </Show>

                <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                    <h2 class="text-xl font-bold text-white">{move || if locale.get() == "vi" { "Hồ sơ" } else { "Profile" }}</h2>
                    <form on:submit=update_profile class="mt-4 space-y-4">
                        <label class="block text-sm font-semibold text-slate-200">{move || if locale.get() == "vi" { "Họ tên" } else { "Full name" }}
                            <input type="text" prop:value=full_name on:input=move |ev| set_full_name.set(event_target_value(&ev)) class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/>
                        </label>
                        <label class="block text-sm font-semibold text-slate-200">"Avatar URL"
                            <input type="url" prop:value=avatar_url on:input=move |ev| set_avatar_url.set(event_target_value(&ev)) class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/>
                        </label>
                        <button class="rounded-xl bg-cyan-300 px-5 py-3 font-bold text-slate-950" type="submit">{move || if locale.get() == "vi" { "Lưu hồ sơ" } else { "Save profile" }}</button>
                    </form>
                </section>

                <section class="rounded-3xl border border-red-500/20 bg-red-500/5 p-6">
                    <h2 class="text-xl font-bold text-red-300">{move || if locale.get() == "vi" { "Khu vực nguy hiểm" } else { "Danger zone" }}</h2>
                    <p class="mt-2 text-sm text-slate-400">{move || if locale.get() == "vi" { "Các thao tác này có thể gây mất dữ liệu." } else { "These actions are destructive." }}</p>
                    <div class="mt-4 flex flex-wrap gap-3">
                        <button on:click=lock_account class="rounded-xl bg-amber-300/10 px-4 py-3 text-sm font-semibold text-amber-200">{move || if locale.get() == "vi" { "Khóa tài khoản" } else { "Lock my account" }}</button>
                    </div>
                </section>
        </div>
    }
}
