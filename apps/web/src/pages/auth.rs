use crate::pages::AuthLayout;
use leptos::prelude::*;
use minirust_locales::{text as translate, Key, Locale};

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct RegistrationRequestResponse {
    email_exists: bool,
    verification_url: Option<String>,
}

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json};

use super::ui::{Field, BTN_PRIMARY_FULL, EYEBROW, INPUT, LINK};

#[component]
#[allow(unused_variables)]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (requested, set_requested) = signal(false);
    let (status, set_status) = signal(String::new());
    let locale = use_context::<ReadSignal<Locale>>().unwrap_or_else(|| signal(Locale::DEFAULT).0);
    let text = move |key: Key| move || translate(locale.get(), key);

    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let email_value = email.get();
        let code_value = code.get();
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let (path, body) = if requested.get_untracked() {
                (
                    "/api/v1/auth/login/verify-code",
                    serde_json::json!({"email": email_value, "code": code_value}).to_string(),
                )
            } else {
                (
                    "/api/v1/auth/login/request-code",
                    serde_json::json!({"email": email_value}).to_string(),
                )
            };
            match api_empty(gloo_net::http::Method::POST, path, Some(body)).await {
                Ok(()) if requested.get_untracked() => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/app");
                    }
                }
                Ok(()) => {
                    set_requested.set(true);
                    set_status
                        .set(translate(locale.get_untracked(), Key::AuthCodeRequested).to_owned());
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    view! {
        <AuthLayout>
            <a href="/login" class="mb-8 inline-block text-sm font-bold text-accent transition hover:text-accent-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas">"<- MiniRust"</a>
            <section class="rounded-3xl border border-line bg-surface p-6 sm:p-8">
                <p class=EYEBROW>{text(Key::AuthLoginEyebrow)}</p>
                <h1 class="mt-3 text-3xl font-black text-foreground">{text(Key::AuthLoginTitle)}</h1>
                <p class="mt-3 text-sm leading-6 text-subtle-foreground">
                    {text(Key::AuthLoginHint)}
                </p>
                <Show
                    when=move || !requested.get()
                    fallback=move || view! {
                        <form on:submit=submit class="mt-8 space-y-4">
                            <label class="block">
                                <span class="text-sm font-medium text-muted-foreground">{text(Key::AuthVerificationCode)}</span>
                                <div class="mt-2">
                                    <input
                                        type="text"
                                        inputmode="numeric"
                                        maxlength="6"
                                        required
                                        prop:value=code
                                        on:input=move |ev| set_code.set(event_target_value(&ev))
                                        class=INPUT
                                    />
                                </div>
                            </label>
                            <button type="submit" class=BTN_PRIMARY_FULL>{text(Key::AuthVerifyContinue)}</button>
                        </form>
                    }
                >
                    <form on:submit=submit class="mt-8 space-y-4">
                        <Field label="Email".to_owned()>
                            <input
                                type="email"
                                required
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                class=INPUT
                            />
                        </Field>
                        <button type="submit" class=BTN_PRIMARY_FULL>{text(Key::AuthSendCode)}</button>
                    </form>
                </Show>
                <p class="mt-4 text-sm text-subtle-foreground">{status}</p>
                <p class="mt-8 text-sm text-faint-foreground">
                    {text(Key::AuthNewHere)}
                    <a href="/register" class=LINK>{text(Key::AuthCreateAccount)}</a>
                </p>
            </section>
        </AuthLayout>
    }
}

#[component]
#[allow(unused_variables)]
pub fn RegisterPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (requested, set_requested) = signal(false);
    let (status, set_status) = signal(String::new());
    let (email_exists, set_email_exists) = signal(false);
    let (local_link, set_local_link) = signal(None::<String>);
    let locale = use_context::<ReadSignal<Locale>>().unwrap_or_else(|| signal(Locale::DEFAULT).0);
    let text = move |key: Key| move || translate(locale.get(), key);

    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let email_value = email.get();
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_json::<RegistrationRequestResponse>(
                gloo_net::http::Method::POST,
                "/api/v1/auth/register/request-verification",
                Some(serde_json::json!({"email": email_value}).to_string()),
            )
            .await
            {
                Ok(result) => {
                    set_requested.set(true);
                    if result.email_exists {
                        set_email_exists.set(true);
                        set_status.set(
                            translate(locale.get_untracked(), Key::AuthEmailRegistered).to_owned(),
                        );
                    } else if let Some(url) = result.verification_url {
                        set_local_link.set(Some(url));
                        set_status.set(String::new());
                    } else {
                        set_status
                            .set(translate(locale.get_untracked(), Key::AuthCheckEmail).to_owned());
                    }
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    view! {
        <AuthLayout>
            <a href="/" class="mb-8 inline-block text-sm font-bold text-accent transition hover:text-accent-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas">"<- MiniRust"</a>
            <section class="rounded-3xl border border-line bg-surface p-6 sm:p-8">
                <p class=EYEBROW>{text(Key::AuthRegisterEyebrow)}</p>
                <h1 class="mt-3 text-3xl font-black text-foreground">{text(Key::AuthRegisterTitle)}</h1>
                <Show
                    when=move || !requested.get()
                    fallback=move || view! {
                        <div class="mt-8 space-y-4">
                            <p class="text-sm leading-6 text-muted-foreground">
                                {text(Key::AuthLinkSent)}
                            </p>
                            <p class="text-sm text-subtle-foreground">{status}</p>
                            {move || local_link.get().map(|url| view! {
                                <p class="break-all text-sm text-subtle-foreground">
                                    {translate(locale.get(), Key::AuthLocalTestLink)}{url}
                                </p>
                            })}
                            <Show when=move || email_exists.get()>
                                <a href="/login" class=BTN_PRIMARY_FULL>{text(Key::AuthGoToLogin)}</a>
                            </Show>
                            <Show when=move || local_link.get().is_some()>
                                <a href={move || local_link.get().unwrap_or_default()} class=BTN_PRIMARY_FULL>
                                    {text(Key::AuthOpenVerificationLink)}
                                </a>
                            </Show>
                            <p class="text-sm text-faint-foreground">
                                {text(Key::AuthResendHint)}
                                <button
                                    type="button"
                                    class=LINK
                                    on:click=move |_| {
                                        set_requested.set(false);
                                        set_email_exists.set(false);
                                        set_local_link.set(None);
                                        set_status.set(String::new());
                                    }
                                >
                                    {text(Key::AuthTryAgain)}
                                </button>
                                "."
                            </p>
                        </div>
                    }
                >
                    <p class="mt-3 text-sm leading-6 text-subtle-foreground">
                        {text(Key::AuthRegisterHint)}
                    </p>
                    <form on:submit=submit class="mt-8 space-y-4">
                        <Field label="Email".to_owned()>
                            <input
                                type="email"
                                required
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                class=INPUT
                            />
                        </Field>
                        <button type="submit" class=BTN_PRIMARY_FULL>{text(Key::AuthRegisterSubmit)}</button>
                    </form>
                    <p class="mt-4 text-sm text-subtle-foreground">{status}</p>
                </Show>
                <p class="mt-8 text-sm text-faint-foreground">
                    {text(Key::AuthAlreadyRegistered)}
                    <a href="/login" class=LINK>{text(Key::AuthLoginEyebrow)}</a>
                </p>
            </section>
        </AuthLayout>
    }
}

#[component]
#[allow(unused_variables)]
pub fn InvitationAcceptPage() -> impl IntoView {
    let locale = use_context::<ReadSignal<Locale>>().unwrap_or_else(|| signal(Locale::DEFAULT).0);
    let text = move |key: Key| move || translate(locale.get(), key);
    let (status, set_status) =
        signal(translate(locale.get_untracked(), Key::AuthVerifying).to_owned());

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let token = web_sys::window()
                .and_then(|window| window.location().search().ok())
                .and_then(|search| search.strip_prefix("?token=").map(str::to_owned))
                .and_then(|value| value.split('&').next().map(str::to_owned));

            leptos::task::spawn_local(async move {
                let Some(token) = token else {
                    set_status
                        .set(translate(locale.get_untracked(), Key::AuthInvalidLink).to_owned());
                    return;
                };

                match api_empty(
                    gloo_net::http::Method::POST,
                    "/api/v1/auth/invitation/verify",
                    Some(serde_json::json!({"token": token}).to_string()),
                )
                .await
                {
                    Ok(()) => {
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_href("/app");
                        }
                    }
                    Err(error) => set_status.set(error),
                }
            });
        });
    }

    view! {
        <AuthLayout>
            <section class="rounded-3xl border border-line bg-surface p-6 text-center sm:p-8">
                <p class=EYEBROW>{text(Key::AuthVerifyEyebrow)}</p>
                <h1 class="mt-3 text-3xl font-black text-foreground">{text(Key::AuthVerifyTitle)}</h1>
                <p class="mt-4 text-sm leading-6 text-subtle-foreground">{status}</p>
                <a href="/login" class="mt-8 inline-block font-semibold text-accent transition hover:text-accent-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas">{text(Key::AuthLoginEyebrow)}</a>
            </section>
        </AuthLayout>
    }
}

#[component]
#[allow(unused_variables)]
pub fn RegisterVerifyPage() -> impl IntoView {
    let locale = use_context::<ReadSignal<Locale>>().unwrap_or_else(|| signal(Locale::DEFAULT).0);
    let text = move |key: Key| move || translate(locale.get(), key);
    let (status, set_status) =
        signal(translate(locale.get_untracked(), Key::AuthVerifying).to_owned());

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let token = web_sys::window()
                .and_then(|window| window.location().search().ok())
                .and_then(|search| search.strip_prefix("?token=").map(str::to_owned))
                .and_then(|value| value.split('&').next().map(str::to_owned));

            leptos::task::spawn_local(async move {
                let Some(token) = token else {
                    set_status
                        .set(translate(locale.get_untracked(), Key::AuthInvalidLink).to_owned());
                    return;
                };

                match api_empty(
                    gloo_net::http::Method::POST,
                    "/api/v1/auth/register/verify",
                    Some(serde_json::json!({"token": token}).to_string()),
                )
                .await
                {
                    Ok(()) => {
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_href("/app");
                        }
                    }
                    Err(error) => set_status.set(error),
                }
            });
        });
    }

    view! {
        <AuthLayout>
            <section class="rounded-3xl border border-line bg-surface p-6 text-center sm:p-8">
                <p class=EYEBROW>{text(Key::AuthVerifyEyebrow)}</p>
                <h1 class="mt-3 text-3xl font-black text-foreground">{text(Key::AuthVerifyTitle)}</h1>
                <p class="mt-4 text-sm leading-6 text-subtle-foreground">{status}</p>
                <a href="/register" class="mt-8 inline-block font-semibold text-accent transition hover:text-accent-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas">{text(Key::AuthBackToRegister)}</a>
            </section>
        </AuthLayout>
    }
}
