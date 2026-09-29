use leptos::prelude::*;
use leptos_router::components::Outlet;
use crate::types::{LanguageRequest, UserResponse};

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json};

#[component]
pub fn AppLayout() -> impl IntoView {
    let (locale, set_locale) = signal("vi".to_owned());

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(user) = api_json::<UserResponse>(
                gloo_net::http::Method::GET,
                "/api/v1/auth/me",
                None,
            )
            .await {
                set_locale.set(user.locale);
            }
        });
    }

    let change_locale = move |event: leptos::ev::Event| {
        let next_locale = event_target_value(&event);
        set_locale.set(next_locale.clone());
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let _ = api_json::<UserResponse>(
                gloo_net::http::Method::PUT,
                "/api/v1/users/me/language",
                Some(serde_json::json!({ "locale": next_locale }).to_string()),
            )
            .await;
        });
    };

    provide_context(locale);

    let sign_out = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let _ = api_empty(gloo_net::http::Method::POST, "/api/v1/auth/logout", None).await;
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        });
    };

    view! {
        <div class="flex min-h-screen flex-col">
            <header class="border-b border-white/10 bg-slate-950/95 backdrop-blur">
                <nav class="mx-auto flex w-full max-w-7xl items-center justify-between px-5 py-5 sm:px-8 lg:px-10" aria-label="Application navigation">
                    <a href="/app" class="font-black text-white">"MiniRust"</a>
                    <div class="flex items-center gap-4">
                        <a href="/app" class="text-sm text-slate-400 transition hover:text-white">{move || if locale.get() == "vi" { "Không gian làm việc" } else { "Workspace" }}</a>
                        <a href="/admin" class="text-sm text-slate-400 transition hover:text-white">{move || if locale.get() == "vi" { "Quản trị" } else { "Admin" }}</a>
                        <button
                            type="button"
                            on:click=sign_out
                            class="text-sm text-slate-400 transition hover:text-white"
                        >
                            {move || if locale.get() == "vi" { "Đăng xuất" } else { "Sign out" }}
                        </button>
                    </div>
                </nav>
            </header>

            <main class="w-full flex-1">
                <Outlet/>
            </main>

            <footer class="border-t border-white/10">
                <div class="mx-auto flex w-full max-w-7xl flex-col gap-2 px-5 py-6 text-xs text-slate-500 sm:px-8 md:flex-row md:items-center md:justify-between lg:px-10">
                    <span>"MiniRust"</span>
                    <span>{move || if locale.get() == "vi" { "Nền tảng Rust-first" } else { "Rust-first platform foundation" }}</span>
                </div>
            </footer>
        </div>
    }
}

#[component]
pub fn AuthLayout(children: Children) -> impl IntoView {
    view! {
        <main class="mx-auto flex min-h-screen w-full max-w-lg flex-col justify-center px-5 py-12">
            {children()}
        </main>
    }
}
