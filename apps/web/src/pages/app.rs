use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::api_json;
use crate::types::UserResponse;

/// Landing page shown at `/app`: an introduction to the platform.
#[component]
#[allow(unused_variables)]
pub fn AppPage() -> impl IntoView {
    let (user, set_user) = signal(None::<UserResponse>);
    let locale = use_context::<ReadSignal<String>>().unwrap_or_else(|| signal("vi".to_owned()).0);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(current_user) =
                api_json::<UserResponse>(gloo_net::http::Method::GET, "/api/v1/auth/me", None).await
            {
                set_user.set(Some(current_user));
            }
        });
    }

    let text = move |vi: &'static str, en: &'static str| {
        move || if locale.get() == "vi" { vi } else { en }
    };

    view! {
        <div class="mx-auto max-w-7xl space-y-8 px-5 py-12 sm:px-8 lg:px-10">
            <section class="rounded-3xl border border-white/10 bg-gradient-to-br from-cyan-400/10 via-white/[0.03] to-transparent p-6 sm:p-10">
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">{text("Tổng quan", "Overview")}</p>
                <h1 class="mt-3 text-4xl font-black text-white sm:text-5xl">"MiniRust"</h1>
                <p class="mt-4 max-w-3xl text-sm leading-6 text-slate-300 sm:text-base">
                    {text(
                        "Nền tảng web viết bằng Rust: API Axum, giao diện Leptos SSR kèm hydration và dữ liệu MariaDB — tổ chức theo kiến trúc CQRS modular monolith.",
                        "A Rust web platform: an Axum API, a Leptos SSR front end with hydration, and MariaDB storage — organized as a CQRS-oriented modular monolith.",
                    )}
                </p>
                <Show when=move || user.get().is_some()>
                    <p class="mt-6 text-sm text-slate-400">
                        {text("Xin chào, ", "Welcome, ")}
                        <span class="font-semibold text-white">
                            {move || user.get().and_then(|u| u.full_name.or(Some(u.email))).unwrap_or_default()}
                        </span>
                    </p>
                </Show>
            </section>

            <section>
                <h2 class="text-xl font-bold text-white">{text("Thành phần hệ thống", "System components")}</h2>
                <div class="mt-4 grid gap-4 sm:grid-cols-2">
                    <article class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <p class="text-2xl">"🛠️"</p>
                        <h3 class="mt-3 text-lg font-bold text-white">{text("Transport", "Transport")}</h3>
                        <p class="mt-2 text-sm leading-6 text-slate-400">
                            {text(
                                "API Axum REST kèm health check, Swagger UI và middleware kiểm tra phiên đăng nhập.",
                                "Axum REST API with health checks, Swagger UI, and session auth middleware.",
                            )}
                        </p>
                    </article>
                    <article class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <p class="text-2xl">"🖥️"</p>
                        <h3 class="mt-3 text-lg font-bold text-white">{text("Giao diện", "Front end")}</h3>
                        <p class="mt-2 text-sm leading-6 text-slate-400">
                            {text(
                                "Leptos SSR kèm hydration, điều hướng phía client và Tailwind CSS.",
                                "Leptos SSR with hydration, client-side navigation, and Tailwind CSS.",
                            )}
                        </p>
                    </article>
                    <article class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <p class="text-2xl">"🧩"</p>
                        <h3 class="mt-3 text-lg font-bold text-white">{text("Lớp ứng dụng", "Application layer")}</h3>
                        <p class="mt-2 text-sm leading-6 text-slate-400">
                            {text(
                                "CQRS commands/queries tách biệt transport, domain và truy cập dữ liệu đọc/ghi.",
                                "CQRS commands and queries separating transport, domain, and read/write data access.",
                            )}
                        </p>
                    </article>
                    <article class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <p class="text-2xl">"🗄️"</p>
                        <h3 class="mt-3 text-lg font-bold text-white">{text("Dữ liệu và vận hành", "Data and operations")}</h3>
                        <p class="mt-2 text-sm leading-6 text-slate-400">
                            {text(
                                "MariaDB với migration SQLx, log tập trung và Docker Compose cho toàn bộ hệ thống.",
                                "MariaDB with SQLx migrations, centralized logging, and Docker Compose for the whole stack.",
                            )}
                        </p>
                    </article>
                </div>
            </section>

            <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                <h2 class="text-xl font-bold text-white">{text("Bắt đầu", "Get started")}</h2>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a href="/profile" class="rounded-xl bg-cyan-300 px-5 py-3 text-sm font-bold text-slate-950">
                        {text("Thông tin tài khoản", "Account information")}
                    </a>
                    <Show when=move || user.get().map(|u| u.is_admin).unwrap_or(false)>
                        <a href="/admin" class="rounded-xl border border-white/10 px-5 py-3 text-sm font-semibold text-slate-200 hover:bg-white/10">
                            {text("Quản trị người dùng", "User administration")}
                        </a>
                    </Show>
                </div>
            </section>
        </div>
    }
}
