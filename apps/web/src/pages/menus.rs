use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::{api_json, api_json_with_meta};
use crate::models::query::{ALL_PAGE_SIZE, PAGE_SIZE_OPTIONS};
use crate::types::MenuResponse;
#[cfg(feature = "hydrate")]
use crate::types::{MenuListData, PaginationMeta};

use super::ui::{
    EmptyState, LoadingState, PageSizeSelect, Pagination, CHECKBOX, EYEBROW, PAGE_SHELL,
    PAGE_TITLE, TABLE_SHELL, TH, TR,
};

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
    let (loading, set_loading) = signal(false);

    let locale = use_context::<ReadSignal<String>>().unwrap_or_else(|| signal("vi".to_owned()).0);
    let text = move |vi: &'static str, en: &'static str| {
        move || if locale.get() == "vi" { vi } else { en }
    };

    let reload = move || {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            set_loading.set(true);
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
            set_loading.set(false);
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
        <div class=PAGE_SHELL>
            <section>
                <p class=EYEBROW>"Administration"</p>
                <h1 class=PAGE_TITLE>{text("Phân quyền menu", "Menu permissions")}</h1>
                <p class="mt-3 text-sm text-subtle-foreground">{status}</p>
            </section>

            <section class="space-y-2 rounded-3xl border border-line bg-surface p-6">
                <p class="text-sm leading-6 text-subtle-foreground">
                    {text(
                        "Đây là toàn bộ menu của hệ thống. Đánh dấu để cho phép người dùng thường hoặc người dùng Premium mở menu tương ứng.",
                        "These are all menus of the system. Tick a column to allow normal users or Premium users to open that menu.",
                    )}
                </p>
                <p class="text-sm leading-6 text-subtle-foreground">
                    {text(
                        "Quản trị viên luôn có toàn bộ quyền. Tài khoản không được cấp sẽ không thấy menu và cũng không mở được bằng URL trực tiếp.",
                        "Administrators always have full access. An account without a grant does not see the menu and cannot open it by direct URL either.",
                    )}
                </p>
            </section>

            <section class=TABLE_SHELL>
                <div class="flex flex-wrap items-center justify-end gap-3 border-b border-line px-4 py-2">
                    <PageSizeSelect
                        value=page_size
                        options=PAGE_SIZE_OPTIONS.to_vec()
                        all_value=ALL_PAGE_SIZE
                        on_change=Callback::new(move |size: i32| change_page_size(size.to_string()))
                    />
                </div>

                <Show when=move || loading.get()>
                    <LoadingState label="Loading menus".to_owned()/>
                </Show>
                <Show when=move || !loading.get() && menus.get().is_empty()>
                    <EmptyState>
                        {text(
                            "Chưa có menu nào được đăng ký. Menu được thêm bằng migration trong crates/database.",
                            "No menus are registered yet. Menus are added by migration in crates/database.",
                        )}
                    </EmptyState>
                </Show>

                <div class="hidden md:block">
                <table class="w-full text-left" aria-label={text("Phân quyền menu", "Menu permissions")}>
                    <thead>
                        <tr class="text-xs uppercase tracking-widest text-faint-foreground">
                            <th scope="col" class=TH>"Menu"</th>
                            <th scope="col" class=TH>{text("Đường dẫn", "Path")}</th>
                            <th scope="col" class=TH>{text("Người dùng thường", "Normal users")}</th>
                            <th scope="col" class=TH>"Premium"</th>
                            <th scope="col" class=TH>{text("Quản trị viên", "Administrators")}</th>
                        </tr>
                    </thead>
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
                                let user_label = name.clone();
                                let premium_label = name.clone();

                                view! {
                                    <tr class=TR>
                                        <td class="px-4 py-4 text-sm text-foreground">
                                            <span class="break-words">{name}</span>
                                            <Show when=move || !is_active>
                                                <span class="ml-2 rounded-md bg-amber-300/10 px-2 py-0.5 text-xs text-amber-200">
                                                    {text("đang ẩn", "hidden")}
                                                </span>
                                            </Show>
                                        </td>
                                        <td class="break-all px-4 py-4 text-sm text-subtle-foreground">{path}</td>
                                        <td class="px-4 py-4">
                                            <label class="flex min-h-8 items-center">
                                                <input
                                                    type="checkbox"
                                                    aria-label=move || format!("Allow normal users for {}", user_label)
                                                    prop:checked=allow_user
                                                    on:change=move |ev| update_access(menu_for_user.clone(), event_target_checked(&ev), allow_premium)
                                                    class=CHECKBOX
                                                />
                                            </label>
                                        </td>
                                        <td class="px-4 py-4">
                                            <label class="flex min-h-8 items-center">
                                                <input
                                                    type="checkbox"
                                                    aria-label=move || format!("Allow premium for {}", premium_label)
                                                    prop:checked=allow_premium
                                                    on:change=move |ev| update_access(menu_for_premium.clone(), allow_user, event_target_checked(&ev))
                                                    class=CHECKBOX
                                                />
                                            </label>
                                        </td>
                                        <td class="px-4 py-4 text-sm text-subtle-foreground">
                                            {text("Toàn quyền", "Full access")}
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
                </div>

                <ul class="space-y-3 p-4 md:hidden">
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
                            let user_label = name.clone();
                            let premium_label = name.clone();

                            view! {
                                <li class="rounded-2xl border border-line bg-canvas p-4">
                                    <div class="flex items-start justify-between gap-2">
                                        <p class="break-words text-sm font-semibold text-foreground">{name}</p>
                                        <Show when=move || !is_active>
                                            <span class="shrink-0 rounded-md bg-amber-300/10 px-2 py-0.5 text-xs text-amber-200">
                                                {text("đang ẩn", "hidden")}
                                            </span>
                                        </Show>
                                    </div>
                                    <p class="mt-1 break-all text-xs text-faint-foreground">{path}</p>
                                    <div class="mt-3 grid grid-cols-2 gap-3">
                                        <label class="flex min-h-11 items-center gap-2 rounded-lg border border-line px-3">
                                            <input
                                                type="checkbox"
                                                aria-label=move || format!("Allow normal users for {}", user_label)
                                                prop:checked=allow_user
                                                on:change=move |ev| update_access(menu_for_user.clone(), event_target_checked(&ev), allow_premium)
                                                class=CHECKBOX
                                            />
                                            <span class="text-xs text-muted-foreground">{text("Người dùng thường", "Normal users")}</span>
                                        </label>
                                        <label class="flex min-h-11 items-center gap-2 rounded-lg border border-line px-3">
                                            <input
                                                type="checkbox"
                                                aria-label=move || format!("Allow premium for {}", premium_label)
                                                prop:checked=allow_premium
                                                on:change=move |ev| update_access(menu_for_premium.clone(), allow_user, event_target_checked(&ev))
                                                class=CHECKBOX
                                            />
                                            <span class="text-xs text-muted-foreground>Premium"</span>
                                        </label>
                                    </div>
                                </li>
                            }
                        }
                    />
                </ul>

                <Pagination
                    can_go_back=Signal::derive(move || page.get() > 1)
                    can_go_forward=Signal::derive(move || page.get() < total_pages.get())
                    on_previous=Callback::new(move |_| {
                        set_page.update(|value| *value = value.saturating_sub(1).max(1));
                        reload();
                    })
                    on_next=Callback::new(move |_| {
                        set_page.update(|value| *value += 1);
                        reload();
                    })
                >
                    {move || if page_size.get() == ALL_PAGE_SIZE {
                        format!("All · {} items", menus.get().len())
                    } else {
                        format!("Page {} of {}", page.get().min(total_pages.get()), total_pages.get())
                    }}
                </Pagination>
            </section>

        </div>
    }
}
