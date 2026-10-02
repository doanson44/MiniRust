use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::api_json;
use crate::types::UserResponse;
use minirust_locales::{text as translate, Key, Locale};

use super::ui::{BTN_PRIMARY, BTN_SECONDARY, EYEBROW, PAGE_SHELL};

/// Landing page shown at `/app`: an introduction to the platform.
#[component]
#[allow(unused_variables)]
pub fn AppPage() -> impl IntoView {
    let (user, set_user) = signal(None::<UserResponse>);
    let locale = use_context::<ReadSignal<Locale>>().unwrap_or_else(|| signal(Locale::DEFAULT).0);

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

    let text = move |key: Key| move || translate(locale.get(), key);

    view! {
        <div class=PAGE_SHELL>
            <section class="rounded-3xl border border-line bg-gradient-to-br from-cyan-400/10 via-white/[0.03] to-transparent p-6 sm:p-10">
                <p class=EYEBROW>{text(Key::DashboardEyebrow)}</p>
                <h1 class="mt-3 text-4xl font-black text-foreground sm:text-5xl">"MiniRust"</h1>
                <p class="mt-4 max-w-3xl text-sm leading-6 text-muted-foreground sm:text-base">
                    {text(Key::DashboardIntro)}
                </p>
                <Show when=move || user.get().is_none()>
                    <p class="mt-6 text-sm text-faint-foreground">{text(Key::CommonLoading)}</p>
                </Show>
                <Show when=move || user.get().is_some()>
                    <p class="mt-6 text-sm text-subtle-foreground">
                        {text(Key::DashboardWelcome)}
                        <span class="font-semibold text-foreground">
                            {move || user.get().and_then(|u| u.full_name.or(Some(u.email))).unwrap_or_default()}
                        </span>
                    </p>
                </Show>
            </section>

            <section>
                <h2 class="text-xl font-bold text-white">{text(Key::DashboardComponents)}</h2>
                <div class="mt-4 grid gap-4 sm:grid-cols-2">
                    <article class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <p class="text-2xl">"🛠️"</p>
                        <h3 class="mt-3 text-lg font-bold text-white">"Transport"</h3>
                        <p class="mt-2 text-sm leading-6 text-slate-400">
                            {text(Key::DashboardTransportBody)}
                        </p>
                    </article>
                    <article class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <p class="text-2xl">"🖥️"</p>
                        <h3 class="mt-3 text-lg font-bold text-white">{text(Key::DashboardFrontendTitle)}</h3>
                        <p class="mt-2 text-sm leading-6 text-slate-400">
                            {text(Key::DashboardFrontendBody)}
                        </p>
                    </article>
                    <article class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <p class="text-2xl">"🧩"</p>
                        <h3 class="mt-3 text-lg font-bold text-white">{text(Key::DashboardApplicationTitle)}</h3>
                        <p class="mt-2 text-sm leading-6 text-slate-400">
                            {text(Key::DashboardApplicationBody)}
                        </p>
                    </article>
                    <article class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                        <p class="text-2xl">"🗄️"</p>
                        <h3 class="mt-3 text-lg font-bold text-white">{text(Key::DashboardDataTitle)}</h3>
                        <p class="mt-2 text-sm leading-6 text-slate-400">
                            {text(Key::DashboardDataBody)}
                        </p>
                    </article>
                </div>
            </section>

            <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                <h2 class="text-xl font-bold text-white">{text(Key::DashboardGetStarted)}</h2>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a href="/profile" class=BTN_PRIMARY>
                        {text(Key::NavProfile)}
                    </a>
                    <Show when=move || user.get().map(|u| u.is_admin).unwrap_or(false)>
                        <a href="/admin" class=BTN_SECONDARY>
                            {text(Key::DashboardUserAdmin)}
                        </a>
                    </Show>
                </div>
            </section>
        </div>
    }
}
