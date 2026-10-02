#[cfg(feature = "hydrate")]
use crate::types::MenuListResponse;
use crate::types::{MenuResponse, UserResponse};
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_location;

use super::ui::{EmptyState, LoadingState, BTN_ICON_PLAIN, MENU_ITEM, MENU_ITEM_DANGER};

#[cfg(feature = "hydrate")]
use crate::api::{api_empty, api_json};

#[component]
pub fn AppLayout() -> impl IntoView {
    let (locale, set_locale) = signal("vi".to_owned());
    let (user, _set_user) = signal(None::<UserResponse>);
    let (menu_open, set_menu_open) = signal(false);
    let (sidebar_open, set_sidebar_open) = signal(false);
    let (menus, _set_menus) = signal(Vec::<MenuResponse>::new());
    let (menus_loaded, _set_menus_loaded) = signal(false);
    let location = use_location();

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(current_user) =
                api_json::<UserResponse>(gloo_net::http::Method::GET, "/api/v1/auth/me", None).await
            {
                set_locale.set(current_user.locale.clone());
                _set_user.set(Some(current_user));

                if let Ok(menu_response) =
                    api_json::<MenuListResponse>(gloo_net::http::Method::GET, "/api/v1/menus", None)
                        .await
                {
                    _set_menus.set(menu_response.menus);
                }

                // Resolve the surface even when the request failed, so a failure
                // shows the empty state instead of an endless spinner.
                _set_menus_loaded.set(true);
            }
        });
    }

    let change_locale = move |event: leptos::ev::Event| {
        let next_locale = event_target_value(&event);
        set_locale.set(next_locale.clone());
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            if let Ok(current_user) = api_json::<UserResponse>(
                gloo_net::http::Method::PUT,
                "/api/v1/users/me/language",
                Some(serde_json::json!({ "locale": next_locale }).to_string()),
            )
            .await
            {
                set_locale.set(current_user.locale.clone());
                _set_user.set(Some(current_user));
            }
        });
    };

    let sign_out = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let _ = api_empty(gloo_net::http::Method::POST, "/api/v1/auth/logout", None).await;
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/login");
            }
        });
    };

    provide_context(locale);

    view! {
        <div class="min-h-screen bg-slate-950 text-slate-100">
            <header class="sticky top-0 z-40 border-b border-white/10 bg-slate-950/95 backdrop-blur">
                <div class="flex h-16 items-center justify-between px-4 sm:px-6">
                    <div class="flex items-center gap-3">
                        <button
                            type="button"
                            class=BTN_ICON_PLAIN
                            aria-label="Toggle navigation"
                            aria-expanded=move || sidebar_open.get()
                            aria-controls="app-sidebar"
                            on:click=move |_| set_sidebar_open.update(|open| *open = !*open)
                        >
                            <span aria-hidden="true">"☰"</span>
                        </button>
                        <a href="/app" class="rounded text-lg font-black tracking-tight text-foreground transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas">"MiniRust"</a>
                    </div>

                    <div class="relative flex items-center gap-3">
                        <select
                            prop:value=move || locale.get()
                            on:change=change_locale
                            class="hidden rounded-lg border border-line bg-canvas-raised px-2 py-1.5 text-xs text-muted-foreground transition focus-visible:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 sm:block"
                            aria-label="Language"
                        >
                            <option value="vi">"Tiếng Việt"</option>
                            <option value="en">"English"</option>
                        </select>

                        <button
                            type="button"
                            class="flex items-center gap-3 rounded-xl px-2 py-1.5 text-left transition hover:bg-white/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-canvas"
                            aria-expanded=move || menu_open.get()
                            aria-haspopup="true"
                            on:click=move |_| set_menu_open.update(|open| *open = !*open)
                        >
                            <span class="flex h-9 w-9 items-center justify-center overflow-hidden rounded-full bg-cyan-300 font-bold text-slate-950">
                                {move || {
                                    user.get()
                                        .and_then(|current_user| current_user.avatar_url)
                                        .map(|url| view! { <img src=url alt="User avatar" class="h-full w-full object-cover"/> }.into_any())
                                        .unwrap_or_else(|| {
                                            view! {
                                                <span>
                                                    {move || user.get().and_then(|u| u.full_name.or(Some(u.email))).and_then(|name| name.chars().next()).unwrap_or('U').to_ascii_uppercase()}
                                                </span>
                                            }.into_any()
                                        })
                                }}
                            </span>
                            <span class="hidden max-w-40 sm:block">
                                <span class="block truncate text-sm font-semibold text-white">
                                    {move || user.get().and_then(|u| u.full_name.or(Some(u.email))).unwrap_or_else(|| "User".to_owned())}
                                </span>
                                <span class="block text-xs text-slate-500">
                                    {move || user.get().map(|u| if u.is_admin { "Admin" } else { "User" }).unwrap_or("Account")}
                                </span>
                            </span>
                            <span class="text-slate-500">"▼"</span>
                        </button>

                        <Show when=move || menu_open.get()>
                            <div class="absolute right-0 top-12 z-50 w-64 space-y-1 rounded-2xl border border-line bg-canvas-raised p-2 shadow-2xl">
                                <div class="border-b border-white/10 px-3 py-3">
                                    <p class="truncate text-sm font-semibold text-white">
                                        {move || user.get().and_then(|u| u.full_name).unwrap_or_else(|| "User".to_owned())}
                                    </p>
                                    <p class="truncate text-xs text-slate-500">
                                        {move || user.get().map(|u| u.email).unwrap_or_default()}
                                    </p>
                                </div>
                                <a href="/profile" class=MENU_ITEM>
                                    {move || if locale.get() == "vi" { "Thông tin tài khoản" } else { "Account information" }}
                                </a>
                                <Show when=move || user.get().map(|u| u.is_admin).unwrap_or(false)>
                                    <a href="/admin/menus" class=MENU_ITEM>
                                        {move || if locale.get() == "vi" { "Phân quyền menu" } else { "Menu permissions" }}
                                    </a>
                                </Show>
                                <button
                                    type="button"
                                    on:click=sign_out
                                    class=MENU_ITEM_DANGER
                                >
                                    {move || if locale.get() == "vi" { "Đăng xuất" } else { "Log out" }}
                                </button>
                            </div>
                        </Show>
                    </div>
                </div>
            </header>

            <div class="flex min-h-[calc(100vh-4rem)]">
                <Show when=move || sidebar_open.get()>
                    <button
                        type="button"
                        class="fixed inset-0 z-20 bg-black/60"
                        aria-label="Close navigation"
                        on:click=move |_| set_sidebar_open.set(false)
                    ></button>
                </Show>

                <aside
                    id="app-sidebar"
                    class=move || format!(
                        "fixed inset-y-0 left-0 z-30 w-64 transform border-r border-line bg-canvas pt-16 transition-transform duration-200 {}",
                        if sidebar_open.get() { "translate-x-0" } else { "-translate-x-full" },
                    )
                >
                    <nav class="flex h-full flex-col gap-1 p-4" aria-label="Application navigation">
                        <Show when=move || menus.get().is_empty() && !menus_loaded.get()>
                            <LoadingState label="Loading navigation".to_owned()/>
                        </Show>
                        <Show when=move || menus.get().is_empty() && menus_loaded.get()>
                            <EmptyState>
                                {move || if locale.get() == "vi" {
                                    "Tài khoản của bạn chưa được cấp menu nào. Quản trị viên có thể cấp trong mục Phân quyền menu."
                                } else {
                                    "Your account has no menu grants yet. An administrator can grant access under Menu permissions."
                                }}
                            </EmptyState>
                        </Show>
                        <For
                            each=move || menus.get()
                            key=|menu| menu.id.clone()
                            children=move |menu| {
                                let path = menu.path.clone();
                                let label = menu.name.clone();
                                let match_path = path.clone();
                                let pathname = location.pathname;
                                let is_active = Memo::new(move |_| {
                                    let current = pathname.get();
                                    current == match_path
                                        || current.starts_with(&format!("{}/", match_path))
                                });

                                view! {
                                    <a
                                        href=path
                                        aria-current=move || {
                                            if is_active.get() { Some("page") } else { None }
                                        }
                                        class=move || {
                                            if is_active.get() {
                                                "block rounded-xl bg-white/15 px-3 py-2.5 text-sm font-semibold text-white transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset"
                                            } else {
                                                "block rounded-xl px-3 py-2.5 text-sm font-semibold text-muted-foreground transition hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset"
                                            }
                                        }
                                        on:click=move |_| set_sidebar_open.set(false)
                                    >
                                        {label}
                                    </a>
                                }
                            }
                        />

                        <div class="mt-auto border-t border-white/10 pt-4">
                            <select
                                prop:value=move || locale.get()
                                on:change=change_locale
                                class="w-full rounded-lg border border-line bg-canvas-raised px-2 py-2 text-xs text-muted-foreground transition focus-visible:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 sm:hidden"
                                aria-label="Language"
                            >
                                <option value="vi">"Tiếng Việt"</option>
                                <option value="en">"English"</option>
                            </select>
                        </div>
                    </nav>
                </aside>

                <main class="min-w-0 flex-1">
                    <Outlet/>
                </main>
            </div>
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
