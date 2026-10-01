use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::{api_json, api_json_with_meta};
use crate::models::query::{ALL_PAGE_SIZE, PAGE_SIZE_OPTIONS};
use crate::types::MenuResponse;
#[cfg(feature = "hydrate")]
use crate::types::{MenuListData, PaginationMeta};

/// Admin page that lists every system menu and marks which account tiers may open it.
///
/// The menu registry itself is part of the system definition, so this page only
/// edits the access flags: normal users, premium users, and (implicitly) admins.
#[component]
#[allow(unused_variables)]
pub fn MenuAdminPage() -> impl IntoView {
    let (menus, set_menus) = signal(Vec::<MenuResponse>::new());
    let (status, set_status) = signal(String::new());
    let (page, set_page) = signal(1u32);
    let (page_size, set_page_size) = signal(20i32);
    let (total_pages, set_total_pages) = signal(1u32);

    let locale = use_context::<ReadSignal<String>>().unwrap_or_else(|| signal("vi".to_owned()).0);
    let text = move |vi: &'static str, en: &'static str| {
        move || if locale.get() == "vi" { vi } else { en }
    };

    let reload = move || {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            match api_json_with_meta::<MenuListData, PaginationMeta>(
                gloo_net::http::Method::GET,
                &format!(
                    "/api/v1/admin/menus?page={}&page_size={}",
                    page.get_untracked(),
                    page_size.get_untracked()
                ),
                None,
            )
            .await
            {
                Ok(response) => {
                    set_menus.set(response.data.menus);
                    set_total_pages.set(response.meta.total_pages.max(1));
                }
                Err(error) => set_status.set(error),
            }
        });
    };

    #[cfg(feature = "hydrate")]
    reload();

    let change_page_size = move |value: String| {
        if let Ok(value) = value.parse::<i32>() {
            if PAGE_SIZE_OPTIONS.contains(&value) {
                set_page_size.set(value);
                set_page.set(1);
                reload();
            }
        }
    };

    let update_access = move |menu: MenuResponse, allow_user: bool, allow_premium: bool| {
        let saved_message = if locale.get_untracked() == "vi" {
            "Đã cập nhật quyền truy cập menu."
        } else {
            "Menu access updated."
        };

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let request_path = format!("/api/v1/admin/menus/{}", menu.id);
            let body = serde_json::json!({
                "parent_id": menu.parent_id,
                "name": menu.name,
                "path": menu.path,
                "icon": menu.icon,
                "allow_user": allow_user,
                "allow_premium": allow_premium,
                "sort_order": menu.sort_order,
                "is_active": menu.is_active,
            })
            .to_string();

            match api_json::<MenuResponse>(gloo_net::http::Method::PATCH, &request_path, Some(body))
                .await
            {
                Ok(_) => set_status.set(saved_message.to_owned()),
                Err(error) => set_status.set(error),
            }

            // Reloading also restores the checkbox when the update was rejected.
            reload();
        });
    };

    view! {
        <div class="mx-auto max-w-7xl space-y-8 px-5 py-12 sm:px-8 lg:px-10">
            <section>
                <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">"Administration"</p>
                <h1 class="mt-3 text-4xl font-black text-white">{text("Phân quyền menu", "Menu permissions")}</h1>
                <p class="mt-3 text-sm text-slate-400">{status}</p>
            </section>

            <section class="space-y-2 rounded-3xl border border-white/10 bg-white/[0.03] p-6">
                <p class="text-sm leading-6 text-slate-400">
                    {text(
                        "Đây là toàn bộ menu của hệ thống. Đánh dấu để cho phép người dùng thường hoặc người dùng Premium mở menu tương ứng.",
                        "These are all menus of the system. Tick a column to allow normal users or Premium users to open that menu.",
                    )}
                </p>
                <p class="text-sm leading-6 text-slate-400">
                    {text(
                        "Quản trị viên luôn có toàn bộ quyền. Tài khoản không được cấp sẽ không thấy menu và cũng không mở được bằng URL trực tiếp.",
                        "Administrators always have full access. An account without a grant does not see the menu and cannot open it by direct URL either.",
                    )}
                </p>
            </section>

            <section class="overflow-x-auto rounded-3xl border border-white/10 bg-white/[0.03]">
                <table class="w-full min-w-[820px] text-left">
                    <thead><tr class="text-xs uppercase tracking-widest text-slate-500">
                        <th class="px-4 py-4">"Menu"</th>
                        <th class="px-4 py-4">{text("Đường dẫn", "Path")}</th>
                        <th class="px-4 py-4">{text("Người dùng thường", "Normal users")}</th>
                        <th class="px-4 py-4">"Premium"</th>
                        <th class="px-4 py-4">{text("Quản trị viên", "Administrators")}</th>
                    </tr></thead>
                    <tbody>
                        <For
                            each=move || menus.get()
                            key=|menu| (menu.id.clone(), menu.name.clone(), menu.allow_user, menu.allow_premium)
                            children=move |menu| {
                                let name = menu.name.clone();
                                let path = menu.path.clone();
                                let is_active = menu.is_active;
                                let allow_user = menu.allow_user;
                                let allow_premium = menu.allow_premium;
                                let menu_for_user = menu.clone();
                                let menu_for_premium = menu.clone();

                                view! {
                                    <tr class="border-t border-white/10">
                                        <td class="px-4 py-4 text-sm text-white">
                                            {name}
                                            <Show when=move || !is_active>
                                                <span class="ml-2 rounded-md bg-amber-300/10 px-2 py-0.5 text-xs text-amber-200">
                                                    {text("đang ẩn", "hidden")}
                                                </span>
                                            </Show>
                                        </td>
                                        <td class="px-4 py-4 text-sm text-slate-400">{path}</td>
                                        <td class="px-4 py-4">
                                            <input
                                                type="checkbox"
                                                prop:checked=allow_user
                                                on:change=move |ev| update_access(menu_for_user.clone(), event_target_checked(&ev), allow_premium)
                                                class="h-5 w-5 rounded border-white/10 bg-slate-950"
                                            />
                                        </td>
                                        <td class="px-4 py-4">
                                            <input
                                                type="checkbox"
                                                prop:checked=allow_premium
                                                on:change=move |ev| update_access(menu_for_premium.clone(), allow_user, event_target_checked(&ev))
                                                class="h-5 w-5 rounded border-white/10 bg-slate-950"
                                            />
                                        </td>
                                        <td class="px-4 py-4 text-sm text-slate-400">
                                            {text("Toàn quyền", "Full access")}
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </section>

            <div class="flex items-center justify-between gap-4 border-t border-white/10 px-4 py-4 text-sm text-slate-400">
                <label class="flex items-center gap-2">
                    <span>"Items per page"</span>
                    <select
                        prop:value=move || page_size.get().to_string()
                        on:change=move |ev| change_page_size(event_target_value(&ev))
                        class="rounded-lg border border-white/10 bg-white/[0.03] px-3 py-2 text-white"
                    >
                        {PAGE_SIZE_OPTIONS.into_iter().map(|size| {
                            let label = if size == ALL_PAGE_SIZE { "All".to_owned() } else { size.to_string() };
                            view! { <option value=size.to_string()>{label}</option> }
                        }).collect_view()}
                    </select>
                </label>
                <span>{move || format!("Page {} of {}", page.get().min(total_pages.get()), total_pages.get())}</span>
                <div class="flex gap-2">
                    <button type="button" disabled=move || page.get() <= 1
                        on:click=move |_| {
                            let next_page = page.get().saturating_sub(1).max(1);
                            set_page.set(next_page);
                            reload();
                        }
                        class="rounded-lg border border-white/10 px-3 py-2 disabled:opacity-40">"Previous"</button>
                    <button type="button" disabled=move || page.get() >= total_pages.get()
                        on:click=move |_| {
                            let next_page = page.get() + 1;
                            set_page.set(next_page);
                            reload();
                        }
                        class="rounded-lg border border-white/10 px-3 py-2 disabled:opacity-40">"Next"</button>
                </div>
            </div>
        </div>
    }
}
