use crate::pages::AuthLayout;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct RegistrationRequestResponse {
    email_exists: bool,
    verification_url: Option<String>,
}

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json};

#[component]
#[allow(unused_variables)]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (requested, set_requested) = signal(false);
    let (status, set_status) = signal(String::new());

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
                    set_status.set("Verification code requested.".to_owned());
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    view! {
        <AuthLayout>
            <a href="/login" class="mb-8 text-sm font-bold text-cyan-300">"<- MiniRust"</a>
            <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6 sm:p-8">
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Sign in"</p>
                <h1 class="mt-3 text-3xl font-black text-white">"Access your account"</h1>
                <p class="mt-3 text-sm leading-6 text-slate-400">
                    "Passwordless authentication uses a verification code."
                </p>
                <Show
                    when=move || !requested.get()
                    fallback=move || view! {
                        <form on:submit=submit class="mt-8 space-y-4">
                            <label class="block text-sm font-semibold text-slate-200">
                                "Verification code"
                                <input
                                    type="text"
                                    inputmode="numeric"
                                    maxlength="6"
                                    required
                                    prop:value=code
                                    on:input=move |ev| set_code.set(event_target_value(&ev))
                                    class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"
                                />
                            </label>
                            <button
                                type="submit"
                                class="w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950"
                            >
                                "Verify and continue"
                            </button>
                        </form>
                    }
                >
                    <form on:submit=submit class="mt-8 space-y-4">
                        <label class="block text-sm font-semibold text-slate-200">
                            "Email"
                            <input
                                type="email"
                                required
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"
                            />
                        </label>
                        <button
                            type="submit"
                            class="w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950"
                        >
                            "Send code"
                        </button>
                    </form>
                </Show>
                <p class="mt-4 text-sm text-slate-400">{status}</p>
                <p class="mt-8 text-sm text-slate-500">
                    "New here? "
                    <a href="/register" class="font-semibold text-cyan-300">"Create an account"</a>
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
                        set_status.set("This email is already registered.".to_owned());
                    } else if let Some(url) = result.verification_url {
                        set_status.set(format!("Local test link: {url}"));
                    } else {
                        set_status.set("Check your email for the verification link.".to_owned());
                    }
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    view! {
        <AuthLayout>
            <a href="/" class="mb-8 text-sm font-bold text-cyan-300">"<- MiniRust"</a>
            <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6 sm:p-8">
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Create account"</p>
                <h1 class="mt-3 text-3xl font-black text-white">"Start with your email"</h1>
                <Show
                    when=move || !requested.get()
                    fallback=move || view! {
                        <div class="mt-8 space-y-4">
                            <p class="text-sm leading-6 text-slate-300">
                                "We've sent a verification link to your email. Open it to complete your registration."
                            </p>
                            <p class="text-sm text-slate-400">{status}</p>
                            <Show
                                when=move || status.get() == "This email is already registered."
                            >
                                <a
                                    href="/login"
                                    class="inline-block rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950"
                                >
                                    "Go to login"
                                </a>
                            </Show>
                            <Show
                                when=move || status.get().starts_with("Local test link: ")
                            >
                                <a
                                    href={move || {
                                        status
                                            .get()
                                            .trim_start_matches("Local test link: ")
                                            .to_owned()
                                    }}
                                    class="inline-block rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950"
                                >
                                    "Open verification link"
                                </a>
                            </Show>
                            <p class="text-sm text-slate-500">
                                "If you do not receive the email, check your spam folder or "
                                <button
                                    type="button"
                                    class="font-semibold text-cyan-300"
                                    on:click=move |_| {
                                        set_requested.set(false);
                                        set_status.set(String::new());
                                    }
                                >
                                    "try again"
                                </button>
                                "."
                            </p>
                        </div>
                    }
                >
                    <p class="mt-3 text-sm leading-6 text-slate-400">
                        "We'll send a verification link to confirm that you own this email address."
                    </p>
                    <form on:submit=submit class="mt-8 space-y-4">
                        <label class="block text-sm font-semibold text-slate-200">
                            "Email"
                            <input
                                type="email"
                                required
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"
                            />
                        </label>
                        <button
                            type="submit"
                            class="w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950"
                        >
                            "Register"
                        </button>
                    </form>
                    <p class="mt-4 text-sm text-slate-400">{status}</p>
                </Show>
                <p class="mt-8 text-sm text-slate-500">
                    "Already registered? "
                    <a href="/login" class="font-semibold text-cyan-300">"Sign in"</a>
                </p>
            </section>
        </AuthLayout>
    }
}

#[component]
#[allow(unused_variables)]
pub fn RegisterVerifyPage() -> impl IntoView {
    let (status, set_status) = signal("Verifying your email…".to_owned());

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let token = web_sys::window()
                .and_then(|window| window.location().search().ok())
                .and_then(|search| search.strip_prefix("?token=").map(str::to_owned))
                .and_then(|value| value.split('&').next().map(str::to_owned));

            leptos::task::spawn_local(async move {
                let Some(token) = token else {
                    set_status.set("The verification link is invalid.".to_owned());
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
            <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6 text-center sm:p-8">
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Email verification"</p>
                <h1 class="mt-3 text-3xl font-black text-white">"Complete registration"</h1>
                <p class="mt-4 text-sm leading-6 text-slate-400">{status}</p>
                <a href="/register" class="mt-8 inline-block font-semibold text-cyan-300">"Back to registration"</a>
            </section>
        </AuthLayout>
    }
}
