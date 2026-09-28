use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::api_empty;

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
        <div class="mx-auto flex min-h-screen max-w-lg flex-col justify-center px-5 py-12">
            <a href="/" class="mb-8 text-sm font-bold text-cyan-300">"<- MiniRust"</a>
            <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6 sm:p-8">
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Sign in"</p>
                <h1 class="mt-3 text-3xl font-black text-white">"Access your account"</h1>
                <p class="mt-3 text-sm leading-6 text-slate-400">"Passwordless authentication uses a verification code."</p>
                <form on:submit=submit class="mt-8 space-y-4">
                    <label class="block text-sm font-semibold text-slate-200">"Email"
                        <input type="email" required prop:value=email on:input=move |ev| set_email.set(event_target_value(&ev)) class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/>
                    </label>
                    <Show when=move || requested.get()>
                        <label class="block text-sm font-semibold text-slate-200">"Code"
                            <input type="text" inputmode="numeric" maxlength="6" prop:value=code on:input=move |ev| set_code.set(event_target_value(&ev)) class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/>
                        </label>
                    </Show>
                    <button type="submit" class="w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950">{move || if requested.get() { "Verify and continue" } else { "Send code" }}</button>
                </form>
                <p class="mt-4 text-sm text-slate-400">{status}</p>
                <p class="mt-8 text-sm text-slate-500">"New here? " <a href="/register" class="font-semibold text-cyan-300">"Create an account"</a></p>
            </section>
        </div>
    }
}

#[component]
#[allow(unused_variables)]
pub fn RegisterPage() -> impl IntoView {
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
            let verifying = requested.get_untracked();
            let endpoint = if verifying {
                "/api/v1/auth/register/verify-code"
            } else {
                "/api/v1/auth/register/request-code"
            };
            let body = if verifying {
                serde_json::json!({"email": email_value, "code": code_value})
            } else {
                serde_json::json!({"email": email_value})
            };
            match api_empty(
                gloo_net::http::Method::POST,
                endpoint,
                Some(body.to_string()),
            )
            .await
            {
                Ok(()) if verifying => {
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
        <div class="mx-auto flex min-h-screen max-w-lg flex-col justify-center px-5 py-12">
            <a href="/" class="mb-8 text-sm font-bold text-cyan-300">"<- MiniRust"</a>
            <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6 sm:p-8">
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Create account"</p>
                <h1 class="mt-3 text-3xl font-black text-white">"Start with your email"</h1>
                <p class="mt-3 text-sm leading-6 text-slate-400">"We will send a verification code."</p>
                <form on:submit=submit class="mt-8 space-y-4">
                    <label class="block text-sm font-semibold text-slate-200">"Email"
                        <input type="email" required prop:value=email on:input=move |ev| set_email.set(event_target_value(&ev)) class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/>
                    </label>
                    <Show when=move || requested.get()>
                        <label class="block text-sm font-semibold text-slate-200">"Code"
                            <input type="text" inputmode="numeric" maxlength="6" prop:value=code on:input=move |ev| set_code.set(event_target_value(&ev)) class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/>
                        </label>
                    </Show>
                    <button type="submit" class="w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950">{move || if requested.get() { "Verify and continue" } else { "Send code" }}</button>
                </form>
                <p class="mt-4 text-sm text-slate-400">{status}</p>
                <p class="mt-8 text-sm text-slate-500">"Already registered? " <a href="/login" class="font-semibold text-cyan-300">"Sign in"</a></p>
            </section>
        </div>
    }
}
