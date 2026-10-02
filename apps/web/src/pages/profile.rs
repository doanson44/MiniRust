use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json};
use crate::types::UserResponse;

use super::ui::{LoadingState, BTN_PRIMARY, EYEBROW, PAGE_SHELL, PAGE_TITLE};
#[cfg(feature = "hydrate")]
use super::ui::ToastController;

#[component]
#[allow(unused_variables)]
pub fn ProfilePage() -> impl IntoView {
    let (user, set_user) = signal(None::<UserResponse>);
    let (full_name, set_full_name) = signal(String::new());
    let (avatar_url, set_avatar_url) = signal(String::new());
    let locale = use_context::<ReadSignal<String>>().unwrap_or_else(|| signal("vi".to_owned()).0);
    #[cfg(feature = "hydrate")]
    let toast = use_context::<ToastController>().unwrap_or_else(|| ToastController {
        show: Callback::new(|_| {}),
    });

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
                    Err(error) => toast.error(error),
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
                    toast.success(if locale.get() == "vi" { "Đã cập nhật hồ sơ." } else { "Profile updated." });
                }
                Err(error) => toast.error(error),
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
                Err(error) => toast.error(error),
            }
        });
    };

    view! {
        <div class=PAGE_SHELL>
                <section>
                    <p class=EYEBROW>{move || if locale.get() == "vi" { "Không gian làm việc" } else { "Workspace" }}</p>
                    <h1 class=PAGE_TITLE>{move || if locale.get() == "vi" { "Tài khoản của bạn" } else { "Your account" }}</h1>
                </section>

                <Show when=move || user.get().is_none()>
                    <LoadingState label="Loading account".to_owned()/>
                </Show>
                <Show when=move || user.get().is_some()>
                    {move || {
                        let u = user.get().unwrap();
                        view! {
                        <section class="rounded-3xl border border-line bg-surface p-6">
                            <h2 class="text-xl font-bold text-foreground">{move || if locale.get() == "vi" { "Thông tin tài khoản" } else { "Account info" }}</h2>
                            <dl class="mt-4 space-y-2 text-sm">
                                <div class="flex gap-4"><dt class="w-28 text-faint-foreground">"Email"</dt><dd class="break-all text-foreground">{u.email.clone()}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-faint-foreground">{move || if locale.get() == "vi" { "Vai trò" } else { "Role" }}</dt><dd class="text-foreground">{if u.is_admin { "Admin" } else { "User" }}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-faint-foreground">"Premium"</dt><dd class="text-foreground">{if u.is_premium { "Active" } else { "Inactive" }}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-faint-foreground">{move || if locale.get() == "vi" { "Trạng thái" } else { "Status" }}</dt><dd class="text-foreground">{if u.is_locked { "Locked" } else { "Active" }}</dd></div>
                            </dl>
                        </section>
                        }
                    }}
                </Show>

                <section class="rounded-3xl border border-line bg-surface p-6">
                    <h2 class="text-xl font-bold text-foreground">{move || if locale.get() == "vi" { "Hồ sơ" } else { "Profile" }}</h2>
                    <form on:submit=update_profile class="mt-4 space-y-4">
                        <label class="block text-sm font-semibold text-muted-foreground">{move || if locale.get() == "vi" { "Họ tên" } else { "Full name" }}
                            <input type="text" prop:value=full_name on:input=move |ev| set_full_name.set(event_target_value(&ev)) class="mt-2 w-full rounded-xl border border-line bg-canvas px-4 py-3 text-sm text-foreground transition placeholder:text-faint-foreground focus-visible:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40"/>
                        </label>
                        <label class="block text-sm font-semibold text-muted-foreground">"Avatar URL"
                            <input type="url" prop:value=avatar_url on:input=move |ev| set_avatar_url.set(event_target_value(&ev)) class="mt-2 w-full rounded-xl border border-line bg-canvas px-4 py-3 text-sm text-foreground transition placeholder:text-faint-foreground focus-visible:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40"/>
                        </label>
                        <button class=BTN_PRIMARY type="submit">{move || if locale.get() == "vi" { "Lưu hồ sơ" } else { "Save profile" }}</button>
                    </form>
                </section>

                <section class="rounded-3xl border border-danger/20 bg-danger/5 p-6">
                    <h2 class="text-xl font-bold text-danger">{move || if locale.get() == "vi" { "Khu vực nguy hiểm" } else { "Danger zone" }}</h2>
                    <p class="mt-2 text-sm text-subtle-foreground">{move || if locale.get() == "vi" { "Các thao tác này có thể gây mất dữ liệu." } else { "These actions are destructive." }}</p>
                    <div class="mt-4 flex flex-wrap gap-3">
                        <button on:click=lock_account class="rounded-xl bg-amber-300/10 px-4 py-3 text-sm font-semibold text-amber-200 transition hover:bg-amber-300/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-300 focus-visible:ring-offset-2 focus-visible:ring-offset-canvas">{move || if locale.get() == "vi" { "Khóa tài khoản" } else { "Lock my account" }}</button>
                    </div>
                </section>
        </div>
    }
}
