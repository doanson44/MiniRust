//! MiniRust Leptos SSR application.

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
#[cfg(feature = "ssr")]
use axum::http::StatusCode;
#[cfg(feature = "ssr")]
use axum::response::IntoResponse;
#[cfg(feature = "ssr")]
use axum::routing::get;
#[cfg(feature = "ssr")]
use axum::Router;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_router::{components::{Route, Router as LeptosRouter, Routes}, path};
use minirust_core::APP_NAME;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tower_http::trace::TraceLayer;

const CSS: &str = include_str!("generated.css");

/// Shared web application state.
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn new() -> Self {
        Self {
            leptos_options: LeptosOptions::builder()
                .output_name("minirust-web")
                .site_root("target/site")
                .site_pkg_dir("pkg")
                .site_addr("127.0.0.1:3001")
                .build(),
        }
    }
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self { state.leptos_options.clone() }
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn with_leptos_options(mut self, options: LeptosOptions) -> Self {
        self.leptos_options = options;
        self
    }
}

#[cfg(feature = "ssr")]
impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
fn HomePage(message: String) -> impl IntoView {
    view! {
                <header class="border-b border-white/10 bg-slate-950/90 backdrop-blur">
                    <nav class="mx-auto flex max-w-7xl items-center justify-between px-5 py-5 sm:px-8 lg:px-10" aria-label="Main navigation">
                        <a href="#top" class="flex items-center gap-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-cyan-400" aria-label="MiniRust home">
                            <span class="grid size-9 place-items-center rounded-xl bg-cyan-400 font-black text-slate-950">M</span>
                            <span class="text-lg font-bold tracking-tight">{APP_NAME}</span>
                        </a>
                        <div class="hidden items-center gap-8 text-sm text-slate-300 md:flex">
                            <a href="#capabilities" class="transition hover:text-white">Capabilities</a>
                            <a href="#architecture" class="transition hover:text-white">Architecture</a>
                            <a href="#principles" class="transition hover:text-white">Principles</a>
                        </div>
                        <div class="flex items-center gap-3"><a href="/login" class="rounded-full border border-white/10 px-4 py-2 text-sm font-semibold text-slate-300 transition hover:bg-white/10 hover:text-white focus:outline-none focus:ring-2 focus:ring-cyan-400">Sign in</a><a href="/register" class="rounded-full border border-cyan-300/30 bg-cyan-300/10 px-4 py-2 text-sm font-semibold text-cyan-200 transition hover:bg-cyan-300/20 focus:outline-none focus:ring-2 focus:ring-cyan-400">Get started</a></div>
                    </nav>
                </header>

                <main id="top">
                    <section class="relative isolate overflow-hidden">
                        <div class="absolute inset-x-0 top-0 -z-10 h-[32rem] bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.16),transparent_40%),radial-gradient(circle_at_top_left,rgba(59,130,246,0.14),transparent_35%)]"></div>
                        <div class="mx-auto grid max-w-7xl gap-14 px-5 pb-24 pt-20 sm:px-8 sm:pt-28 lg:grid-cols-[1.15fr_0.85fr] lg:items-center lg:px-10 lg:pb-32">
                            <div>
                                <div class="mb-7 inline-flex items-center gap-2 rounded-full border border-cyan-300/20 bg-cyan-300/5 px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.18em] text-cyan-200">
                                    <span class="size-1.5 rounded-full bg-cyan-300"></span>
                                    Rust-first full-stack foundation
                                </div>
                                <h1 class="max-w-4xl text-4xl font-black tracking-tight text-white sm:text-6xl lg:text-7xl">Build the platform once. <span class="text-cyan-300">Extend it for years.</span></h1>
                                <p class="mt-7 max-w-2xl text-base leading-8 text-slate-300 sm:text-lg">{message} MiniRust is structured as a reusable foundation for production-minded Rust applications, with SSR, REST APIs, CQRS-oriented services, persistence, testing, and containerized infrastructure.</p>
                                <div class="mt-9 flex flex-col gap-3 sm:flex-row">
                                    <a href="/register" class="inline-flex items-center justify-center rounded-xl bg-cyan-300 px-5 py-3.5 text-sm font-bold text-slate-950 shadow-lg shadow-cyan-950/30 transition hover:bg-cyan-200 focus:outline-none focus:ring-2 focus:ring-cyan-300 focus:ring-offset-2 focus:ring-offset-slate-950">Create an account <span class="ml-2">"->"</span></a>
                                    <a href="#architecture" class="inline-flex items-center justify-center rounded-xl border border-white/15 bg-white/5 px-5 py-3.5 text-sm font-semibold text-white transition hover:bg-white/10 focus:outline-none focus:ring-2 focus:ring-white/50">See the architecture</a>
                                </div>
                                <div class="mt-10 grid max-w-xl grid-cols-1 gap-4 text-sm text-slate-400 sm:grid-cols-3">
                                    <div><div class="font-bold text-white">Rust</div><div>Type-safe core</div></div>
                                    <div><div class="font-bold text-white">SSR</div><div>Fast server rendering</div></div>
                                    <div><div class="font-bold text-white">CQRS</div><div>Clear application boundaries</div></div>
                                </div>
                            </div>

                            <div class="relative mx-auto w-full max-w-xl">
                                <div class="absolute -inset-4 rounded-[2rem] bg-cyan-400/10 blur-3xl"></div>
                                <div class="relative overflow-hidden rounded-[1.75rem] border border-white/10 bg-white/[0.04] p-5 shadow-2xl shadow-black/30 sm:p-7">
                                    <div class="flex items-center justify-between border-b border-white/10 pb-4">
                                        <div class="flex gap-1.5" aria-hidden="true"><span class="size-2.5 rounded-full bg-red-400"></span><span class="size-2.5 rounded-full bg-amber-400"></span><span class="size-2.5 rounded-full bg-emerald-400"></span></div>
                                        <span class="font-mono text-xs text-slate-500">minirust::platform</span>
                                    </div>
                                    <div class="space-y-4 pt-6 font-mono text-xs leading-6 sm:text-sm">
                                        <div class="text-slate-500">{"// one foundation, many capabilities"}</div>
                                        <div><span class="text-cyan-300">web</span> <span class="text-slate-500">"->"</span> <span class="text-white">SSR presentation</span></div>
                                        <div><span class="text-cyan-300">api</span> <span class="text-slate-500">"->"</span> <span class="text-white">REST transport</span></div>
                                        <div><span class="text-cyan-300">services</span> <span class="text-slate-500">"->"</span> <span class="text-white">commands + queries</span></div>
                                        <div><span class="text-cyan-300">database</span> <span class="text-slate-500">"->"</span> <span class="text-white">persistent state</span></div>
                                        <div><span class="text-cyan-300">tests</span> <span class="text-slate-500">"->"</span> <span class="text-white">real endpoint coverage</span></div>
                                        <div class="pt-2 text-emerald-300">status: foundation ready to evolve</div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </section>

                    <section id="capabilities" class="border-y border-white/10 bg-slate-900/50">
                        <div class="mx-auto max-w-7xl px-5 py-20 sm:px-8 lg:px-10 lg:py-24">
                            <div class="max-w-2xl">
                                <p class="text-sm font-bold uppercase tracking-[0.18em] text-cyan-300">Capabilities</p>
                                <h2 class="mt-3 text-3xl font-bold tracking-tight text-white sm:text-4xl">Designed to grow without losing structure.</h2>
                                <p class="mt-4 text-base leading-7 text-slate-400">The landing surface stays simple while the platform underneath can expand through well-defined application and infrastructure boundaries.</p>
                            </div>
                            <div class="mt-12 grid gap-5 md:grid-cols-2 lg:grid-cols-4">
                                <div class="rounded-2xl border border-white/10 bg-slate-950/70 p-6"><div class="text-2xl">01</div><h3 class="mt-5 font-bold text-white">Rust core</h3><p class="mt-3 text-sm leading-6 text-slate-400">Explicit ownership, strong types, and focused modules form the foundation for long-lived services.</p></div>
                                <div class="rounded-2xl border border-white/10 bg-slate-950/70 p-6"><div class="text-2xl">02</div><h3 class="mt-5 font-bold text-white">SSR web</h3><p class="mt-3 text-sm leading-6 text-slate-400">Leptos renders the web experience on the server and keeps presentation separate from business rules.</p></div>
                                <div class="rounded-2xl border border-white/10 bg-slate-950/70 p-6"><div class="text-2xl">03</div><h3 class="mt-5 font-bold text-white">CQRS services</h3><p class="mt-3 text-sm leading-6 text-slate-400">Commands and queries establish explicit application boundaries that can scale with feature count.</p></div>
                                <div class="rounded-2xl border border-white/10 bg-slate-950/70 p-6"><div class="text-2xl">04</div><h3 class="mt-5 font-bold text-white">Integration tests</h3><p class="mt-3 text-sm leading-6 text-slate-400">HTTP endpoints are exercised through the real router, with real infrastructure where required.</p></div>
                            </div>
                        </div>
                    </section>

                    <section id="architecture" class="mx-auto max-w-7xl px-5 py-20 sm:px-8 lg:px-10 lg:py-28">
                        <div class="grid gap-12 lg:grid-cols-[0.8fr_1.2fr] lg:items-center">
                            <div>
                                <p class="text-sm font-bold uppercase tracking-[0.18em] text-cyan-300">Architecture</p>
                                <h2 class="mt-3 text-3xl font-bold tracking-tight text-white sm:text-4xl">Keep responsibilities visible.</h2>
                                <p class="mt-5 text-base leading-7 text-slate-400">Presentation, transport, application services, domain logic, and infrastructure evolve independently while remaining connected through explicit contracts.</p>
                            </div>
                            <div class="rounded-3xl border border-white/10 bg-white/[0.03] p-5 sm:p-7">
                                <div class="space-y-3">
                                    <div class="rounded-xl border border-cyan-300/20 bg-cyan-300/5 p-4"><div class="text-xs font-bold uppercase tracking-widest text-cyan-200">Presentation</div><div class="mt-1 font-semibold text-white">Leptos SSR</div></div>
                                    <div class="mx-auto h-4 w-px bg-white/15"></div>
                                    <div class="rounded-xl border border-white/10 bg-white/[0.03] p-4"><div class="text-xs font-bold uppercase tracking-widest text-slate-400">Transport</div><div class="mt-1 font-semibold text-white">Axum routes and handlers</div></div>
                                    <div class="mx-auto h-4 w-px bg-white/15"></div>
                                    <div class="grid gap-3 sm:grid-cols-2"><div class="rounded-xl border border-white/10 bg-white/[0.03] p-4"><div class="text-xs font-bold uppercase tracking-widest text-slate-400">Application</div><div class="mt-1 font-semibold text-white">Commands + Queries</div></div><div class="rounded-xl border border-white/10 bg-white/[0.03] p-4"><div class="text-xs font-bold uppercase tracking-widest text-slate-400">Domain</div><div class="mt-1 font-semibold text-white">Business rules</div></div></div>
                                    <div class="mx-auto h-4 w-px bg-white/15"></div>
                                    <div class="rounded-xl border border-white/10 bg-white/[0.03] p-4"><div class="text-xs font-bold uppercase tracking-widest text-slate-400">Infrastructure</div><div class="mt-1 font-semibold text-white">Repositories, database, external adapters</div></div>
                                </div>
                            </div>
                        </div>
                    </section>

                    <section id="principles" class="bg-cyan-300 text-slate-950">
                        <div class="mx-auto grid max-w-7xl gap-10 px-5 py-16 sm:px-8 lg:grid-cols-[1fr_auto] lg:items-center lg:px-10 lg:py-20">
                            <div><p class="text-sm font-bold uppercase tracking-[0.18em] text-slate-700">Engineering principles</p><h2 class="mt-3 max-w-3xl text-3xl font-black tracking-tight sm:text-4xl">Correctness first. Simplicity where possible. Explicit boundaries everywhere.</h2></div>
                            <div id="get-started" class="rounded-2xl bg-slate-950 px-6 py-5 text-white shadow-xl"><div class="text-sm font-bold">Ready to build?</div><div class="mt-1 text-sm text-slate-400">Start with the next vertical slice.</div></div>
                        </div>
                    </section>
                </main>

                <footer class="border-t border-white/10 bg-slate-950">
                    <div class="mx-auto flex max-w-7xl flex-col gap-3 px-5 py-8 text-sm text-slate-500 sm:px-8 md:flex-row md:items-center md:justify-between lg:px-10">
                        <div><span class="font-semibold text-slate-300">{APP_NAME}</span> <span class="mx-2">"."</span> Rust-first platform foundation</div>
                        <div>Built with Rust, Axum, Leptos, and a CQRS-oriented service layer.</div>
                    </div>
                </footer>
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct UserResponse {
    id: String,
    email: String,
    is_admin: bool,
    is_premium: bool,
    full_name: Option<String>,
    avatar_url: Option<String>,
    is_locked: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct ApiEnvelope<T> { data: T }

#[derive(Clone, Debug, Deserialize)]
struct AdminUsers { users: Vec<UserResponse> }

#[derive(Clone, Debug, Deserialize)]
struct ApiProblem { detail: String }

#[cfg(feature = "hydrate")]
async fn api_post_json(path: &str, body: String) -> Result<(), String> {
    let response = gloo_net::http::Request::post(path)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|error| error.to_string())?
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if response.ok() {
        Ok(())
    } else {
        response.json::<ApiProblem>().await.map(|p| Err(p.detail)).unwrap_or_else(|e| Err(e.to_string()))
    }
}

#[component]
fn LoginPage() -> impl IntoView {
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
            if requested.get_untracked() {
                match api_post_json("/api/v1/auth/login/verify-code", serde_json::json!({"email": email_value, "code": code_value}).to_string()).await {
                    Ok(()) => { if let Some(window) = web_sys::window() { let _ = window.location().set_href("/app"); } }
                    Err(error) => set_status.set(error),
                }
            } else {
                match api_post_json("/api/v1/auth/login/request-code", serde_json::json!({"email": email_value}).to_string()).await {
                    Ok(()) => { set_requested.set(true); set_status.set("Verification code requested.".to_owned()); }
                    Err(error) => set_status.set(error),
                }
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
fn RegisterPage() -> impl IntoView {
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
            let endpoint = if verifying { "/api/v1/auth/register/verify-code" } else { "/api/v1/auth/register/request-code" };
            let body = if verifying { serde_json::json!({"email": email_value, "code": code_value}) } else { serde_json::json!({"email": email_value}) };
            match api_post_json(endpoint, body.to_string()).await {
                Ok(()) if verifying => { if let Some(window) = web_sys::window() { let _ = window.location().set_href("/app"); } }
                Ok(()) => { set_requested.set(true); set_status.set("Verification code requested.".to_owned()); }
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

#[component]
fn AppPage() -> impl IntoView {
    let (user, set_user) = signal(None::<UserResponse>);
    let (status, set_status) = signal(String::new());

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            match gloo_net::http::Request::get("/api/v1/auth/me").send().await {
                Ok(response) if response.status() == 401 => { if let Some(window) = web_sys::window() { let _ = window.location().set_href("/login"); } }
                Ok(response) if response.ok() => match response.json::<ApiEnvelope<UserResponse>>().await {
                    Ok(envelope) => set_user.set(Some(envelope.data)),
                    Err(error) => set_status.set(error.to_string()),
                },
                Ok(response) => set_status.set(format!("Unable to load account ({}).", response.status())),
                Err(error) => set_status.set(error.to_string()),
            }
        });
    });

    let logout = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let _ = gloo_net::http::Request::post("/api/v1/auth/logout").send().await;
            if let Some(window) = web_sys::window() { let _ = window.location().set_href("/"); }
        });
    };

    view! {
        <div class="min-h-screen">
            <header class="border-b border-white/10"><nav class="mx-auto flex max-w-7xl items-center justify-between px-5 py-5 sm:px-8 lg:px-10">
                <a href="/" class="font-black text-white">"MiniRust"</a>
                <div class="flex gap-3">
                    <Show when=move || user.get().map(|u| u.is_admin).unwrap_or(false)><a href="/admin" class="rounded-xl border border-white/10 px-4 py-2 text-sm text-slate-300">"Admin"</a></Show>
                    <button on:click=logout class="rounded-xl bg-white/10 px-4 py-2 text-sm font-semibold text-white">"Sign out"</button>
                </div>
            </nav></header>
            <main class="mx-auto max-w-7xl px-5 py-12 sm:px-8 lg:px-10">
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Account"</p>
                <h1 class="mt-3 text-4xl font-black text-white">"Your workspace"</h1>
                <p class="mt-3 text-slate-400">{status}</p>
                <Show when=move || user.get().is_some() fallback=|| view! { <p class="mt-8 text-slate-400">"Loading account..."</p> }>
                    <p class="mt-8 text-white">{move || user.get().map(|u| u.email).unwrap_or_default()}</p>
                </Show>
            </main>
        </div>
    }
}

#[component]
fn AdminPage() -> impl IntoView {
    let (users, set_users) = signal(Vec::<UserResponse>::new());
    let (status, set_status) = signal(String::from("Loading..."));

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            let me = gloo_net::http::Request::get("/api/v1/auth/me").send().await;
            match me {
                Ok(response) if response.status() == 401 => { if let Some(window) = web_sys::window() { let _ = window.location().set_href("/login"); } }
                Ok(response) if response.ok() => match response.json::<ApiEnvelope<UserResponse>>().await {
                    Ok(envelope) if envelope.data.is_admin => {
                        match gloo_net::http::Request::get("/api/v1/admin/users").send().await {
                            Ok(response) if response.ok() => match response.json::<ApiEnvelope<AdminUsers>>().await {
                                Ok(envelope) => { let count = envelope.data.users.len(); set_users.set(envelope.data.users); set_status.set(format!("{count} users loaded.")); }
                                Err(error) => set_status.set(error.to_string()),
                            },
                            Ok(response) => set_status.set(format!("Unable to load users ({}).", response.status())),
                            Err(error) => set_status.set(error.to_string()),
                        }
                    }
                    Ok(_) => { if let Some(window) = web_sys::window() { let _ = window.location().set_href("/app"); } }
                    Err(error) => set_status.set(error.to_string()),
                },
                Ok(response) => set_status.set(format!("Unable to load session ({}).", response.status())),
                Err(error) => set_status.set(error.to_string()),
            }
        });
    });

    view! {
        <div class="min-h-screen">
            <header class="border-b border-white/10"><nav class="mx-auto flex max-w-7xl items-center justify-between px-5 py-5 sm:px-8 lg:px-10">
                <a href="/app" class="font-black text-white">"MiniRust"</a><a href="/app" class="text-sm text-cyan-300">"Back to workspace"</a>
            </nav></header>
            <main class="mx-auto max-w-7xl px-5 py-12 sm:px-8 lg:px-10">
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Administration"</p><h1 class="mt-3 text-4xl font-black text-white">"Users"</h1>
                <p class="mt-3 text-sm text-slate-400">{status}</p>
                <div class="mt-8 overflow-x-auto rounded-3xl border border-white/10 bg-white/[0.03]"><table class="w-full min-w-[680px] text-left">
                    <thead><tr class="text-xs uppercase tracking-widest text-slate-500"><th class="px-3 py-4">"Email"</th><th class="px-3 py-4">"Role"</th><th class="px-3 py-4">"Entitlement"</th><th class="px-3 py-4 text-right">"Status"</th></tr></thead>
                    <tbody><For each=move || users.get() key=|user| user.id.clone() let:user>
                        <tr class="border-t border-white/10"><td class="px-3 py-3 text-sm text-white">{user.email.clone()}</td><td class="px-3 py-3 text-sm text-slate-400">{if user.is_admin {"Admin"} else {"User"}}</td><td class="px-3 py-3 text-sm text-slate-400">{if user.is_premium {"Premium"} else {"—"}}</td><td class="px-3 py-3 text-right text-sm text-slate-400">{if user.is_locked {"Locked"} else {"Active"}}</td></tr>
                    </For></tbody>
                </table></div>
            </main>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <LeptosRouter>
            <Routes fallback=|| view! { <main class="min-h-screen bg-slate-950 p-10 text-white"><h1>"Not found"</h1></main> }>
                <Route path=path!("") view=|| view! { <HomePage message="Hello from MiniRust".to_owned()/> }/>
                <Route path=path!("/login") view=LoginPage/>
                <Route path=path!("/register") view=RegisterPage/>
                <Route path=path!("/app") view=AppPage/>
                <Route path=path!("/admin") view=AdminPage/>
            </Routes>
        </LeptosRouter>
    }
}

fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <html lang="en" class="scroll-smooth bg-slate-950 text-slate-100"><head>
            <meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
            <meta name="description" content="MiniRust is a Rust-first full-stack platform foundation built for long-term growth."/>
            <meta name="theme-color" content="#020617"/><style>{CSS}</style>
            <leptos::hydration::HydrationScripts options=options.clone()/><title>{APP_NAME} {" - Rust-first platform foundation"}</title>
        </head><body class="min-h-screen overflow-x-hidden bg-slate-950 antialiased"><App/></body></html>
    }
}

/// Render the MiniRust landing page to an HTML string.
#[cfg(feature = "ssr")]
pub fn render_home_page(message: &str) -> String {
    let html = view! { <HomePage message=message.to_owned()/> }.to_html();
    format!("<!DOCTYPE html>{html}")
}

#[cfg(feature = "ssr")]
pub fn router(state: AppState) -> Router {
    let routes = generate_route_list(App);
    Router::new()
        .route("/health", get(health))
        .leptos_routes(&state.leptos_options, routes, {
            let options = state.leptos_options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(feature = "ssr")]
async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_body(App);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn account_pages_are_server_rendered() {
        let app = router(AppState::new());
        for path in ["/", "/login", "/register", "/app", "/admin"] {
            let response = app.clone().oneshot(Request::get(path).body(Body::empty()).unwrap()).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = body_string(response).await;
            assert!(body.contains("MiniRust"));
            assert!(body.contains("pkg"));
        }
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn get_health_returns_ok() {
        let response = router(AppState::new()).oneshot(Request::get("/health").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_string(response).await, "ok");
    }
}
