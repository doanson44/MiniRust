use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json, api_upload_json};
#[cfg(feature = "hydrate")]
use crate::types::AvatarResponse;
use crate::types::UserResponse;
use minirust_locales::{text as translate, Key, Locale};

#[cfg(feature = "hydrate")]
use super::ui::ToastController;
use super::ui::{LoadingState, BTN_PRIMARY, EYEBROW, INPUT, PAGE_SHELL, PAGE_TITLE};

#[cfg(feature = "hydrate")]
fn picked_avatar(input: &NodeRef<leptos::html::Input>) -> Option<web_sys::File> {
    input
        .get()
        .and_then(|element| element.files())
        .and_then(|files| files.get(0))
}

#[component]
#[allow(unused_variables)]
pub fn ProfilePage() -> impl IntoView {
    let (user, set_user) = signal(None::<UserResponse>);
    let (full_name, set_full_name) = signal(String::new());
    let (avatar_url, set_avatar_url) = signal(String::new());
    let (avatar_preview, set_avatar_preview) = signal(String::new());
    let (saving, set_saving) = signal(false);
    let avatar_input = NodeRef::<leptos::html::Input>::new();
    let locale = use_context::<ReadSignal<Locale>>().unwrap_or_else(|| signal(Locale::DEFAULT).0);
    let text = move |key: Key| move || translate(locale.get(), key);
    let save_label = text(Key::ProfileSave);
    let saving_label = text(Key::ProfileSaving);
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

    let pick_avatar = move |event: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            let input: web_sys::HtmlInputElement = event_target(&event);
            let Some(file) = input.files().and_then(|files| files.get(0)) else {
                set_avatar_preview.set(String::new());
                return;
            };

            let previous = avatar_preview.get_untracked();
            if !previous.is_empty() {
                let _ = web_sys::Url::revoke_object_url(&previous);
            }

            match web_sys::Url::create_object_url_with_blob(&file) {
                Ok(url) => set_avatar_preview.set(url),
                Err(_) => set_avatar_preview.set(String::new()),
            }
        }
    };

    let update_profile = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();

        if saving.get_untracked() {
            return;
        }

        let name = full_name.get();
        set_saving.set(true);

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let avatar = match picked_avatar(&avatar_input) {
                Some(file) => {
                    match api_upload_json::<AvatarResponse>("/api/v1/users/me/avatar", &file).await
                    {
                        Ok(stored) => stored.avatar_url,
                        Err(error) => {
                            toast.error(error);
                            set_saving.set(false);
                            return;
                        }
                    }
                }
                None => avatar_url.get_untracked(),
            };

            set_avatar_url.set(avatar.clone());

            match api_json::<UserResponse>(
                gloo_net::http::Method::PATCH,
                "/api/v1/users/me",
                Some(
                    serde_json::json!({
                        "full_name": if name.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(name) },
                        "avatar_url": serde_json::Value::String(avatar)
                    })
                    .to_string(),
                ),
            )
            .await
            {
                Ok(user) => {
                    set_user.set(Some(user));

                    let preview = avatar_preview.get_untracked();
                    if !preview.is_empty() {
                        let _ = web_sys::Url::revoke_object_url(&preview);
                    }
                    set_avatar_preview.set(String::new());
                    if let Some(input) = avatar_input.get() {
                        input.set_value("");
                    }

                    toast.success(translate(locale.get(), Key::ProfileSaved));
                }
                Err(error) => toast.error(error),
            }

            set_saving.set(false);
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
                    <p class=EYEBROW>{text(Key::ProfileEyebrow)}</p>
                    <h1 class=PAGE_TITLE>{text(Key::ProfileTitle)}</h1>
                </section>

                <Show when=move || user.get().is_none()>
                    <LoadingState label="Loading account".to_owned()/>
                </Show>
                <Show when=move || user.get().is_some()>
                    {move || {
                        let u = user.get().unwrap();
                        view! {
                        <section class="rounded-3xl border border-line bg-surface p-6">
                            <h2 class="text-xl font-bold text-foreground">{text(Key::NavProfile)}</h2>
                            <dl class="mt-4 space-y-2 text-sm">
                                <div class="flex gap-4"><dt class="w-28 text-faint-foreground">"Email"</dt><dd class="break-all text-foreground">{u.email.clone()}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-faint-foreground">{text(Key::ProfileRole)}</dt><dd class="text-foreground">{if u.is_admin { text(Key::CommonRoleAdmin) } else { text(Key::CommonRoleUser) }}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-faint-foreground">"Premium"</dt><dd class="text-foreground">{if u.is_premium { "Active" } else { "Inactive" }}</dd></div>
                                <div class="flex gap-4"><dt class="w-28 text-faint-foreground">{text(Key::ProfileStatus)}</dt><dd class="text-foreground">{if u.is_locked { text(Key::CommonLocked) } else { text(Key::CommonActive) }}</dd></div>
                            </dl>
                        </section>
                        }
                    }}
                </Show>

                <section class="rounded-3xl border border-line bg-surface p-6">
                    <header class="max-w-2xl">
                        <h2 class="text-xl font-bold text-foreground">{text(Key::ProfileSection)}</h2>
                        <p class="mt-1 text-sm text-subtle-foreground">
                            {text(Key::ProfileSectionHint)}
                        </p>
                    </header>

                    <form on:submit=update_profile class="mt-6 max-w-2xl space-y-6">
                        <div class="flex flex-col gap-4 sm:flex-row sm:items-start">
                            <div class="size-20 shrink-0 overflow-hidden rounded-full border border-line bg-canvas">
                                {move || {
                                    let preview = avatar_preview.get();
                                    let url = if preview.is_empty() { avatar_url.get() } else { preview };

                                    if url.is_empty() {
                                        view! {
                                            <span class="flex size-full items-center justify-center text-xl font-black text-faint-foreground" aria-hidden="true">
                                                {user.get().and_then(|u| u.full_name.or(Some(u.email))).and_then(|name| name.chars().next()).map(|letter| letter.to_ascii_uppercase()).unwrap_or('U')}
                                            </span>
                                        }
                                            .into_any()
                                    } else {
                                        view! { <img src=url alt="" class="size-full object-cover"/> }.into_any()
                                    }
                                }}
                            </div>

                            <label class="block min-w-0 flex-1">
                                <span class="text-sm font-medium text-muted-foreground">{text(Key::ProfileAvatar)}</span>
                                <div class="mt-2">
                                    <input
                                        type="file"
                                        accept="image/png,image/jpeg,image/gif,image/webp"
                                        node_ref=avatar_input
                                        on:change=pick_avatar
                                        aria-describedby="avatar-hint"
                                        class="block w-full cursor-pointer text-sm text-subtle-foreground transition file:mr-3 file:cursor-pointer file:rounded-xl file:border-0 file:bg-accent file:px-4 file:py-3 file:text-sm file:font-bold file:text-canvas hover:file:bg-accent-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas"
                                    />
                                </div>
                                <p id="avatar-hint" class="mt-2 text-xs text-faint-foreground">
                                    {text(Key::ProfileAvatarHint)}
                                </p>
                            </label>
                        </div>

                        <label class="block">
                            <span class="text-sm font-medium text-muted-foreground">{text(Key::ProfileFullName)}</span>
                            <div class="mt-2">
                                <input
                                    type="text"
                                    autocomplete="name"
                                    prop:value=full_name
                                    on:input=move |ev| set_full_name.set(event_target_value(&ev))
                                    class=INPUT
                                />
                            </div>
                        </label>

                        <button class=BTN_PRIMARY type="submit" disabled=move || saving.get()>
                            {move || if saving.get() { saving_label() } else { save_label() }}
                        </button>
                    </form>
                </section>

                <section class="rounded-3xl border border-danger/20 bg-danger/5 p-6">
                    <h2 class="text-xl font-bold text-danger">{text(Key::ProfileDangerZone)}</h2>
                    <p class="mt-2 text-sm text-subtle-foreground">{text(Key::ProfileDangerHint)}</p>
                    <div class="mt-4 flex flex-wrap gap-3">
                        <button on:click=lock_account class="rounded-xl bg-amber-300/10 px-4 py-3 text-sm font-semibold text-amber-200 transition hover:bg-amber-300/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-300 focus-visible:ring-offset-2 focus-visible:ring-offset-canvas">{text(Key::ProfileLockAccount)}</button>
                    </div>
                </section>
        </div>
    }
}
