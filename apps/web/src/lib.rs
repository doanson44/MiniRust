#![recursion_limit = "256"]

//! MiniRust Leptos SSR application.

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
#[cfg(feature = "ssr")]
use axum::http::StatusCode;
#[cfg(feature = "ssr")]
use axum::middleware;
#[cfg(feature = "ssr")]
use axum::response::{Html, IntoResponse, Redirect};
#[cfg(feature = "ssr")]
use axum::routing::get;
#[cfg(feature = "ssr")]
use axum::Router;
#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_router::{
    components::{ParentRoute, Route, Router as LeptosRouter, Routes},
    path,
};
#[cfg(feature = "ssr")]
use minirust_core::APP_NAME;
#[cfg(feature = "ssr")]
use minirust_database::Database;
#[cfg(feature = "ssr")]
use minirust_services::{AuthService, MenuAccessDecision, MenuService, UnavailableEmailSender};
#[cfg(feature = "ssr")]
use std::net::SocketAddr;
#[cfg(feature = "ssr")]
use tower_http::trace::TraceLayer;

mod api;
pub mod models;
mod pages;
pub mod types;

use pages::ui::GlobalToast;
use pages::{
    AdminPage, AppLayout, AppPage, AuthLayout, InvitationAcceptPage, LoginPage, MenuAdminPage,
    ProfilePage, RegisterPage, RegisterVerifyPage,
};

#[cfg(feature = "ssr")]
const CSS: &str = include_str!("generated.css");

#[cfg(feature = "ssr")]
type WebAuthService = AuthService<Database, UnavailableEmailSender>;

#[cfg(feature = "ssr")]
type WebMenuService = MenuService<Database>;

/// Shared web application state.
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub auth: Option<WebAuthService>,
    pub menus: Option<WebMenuService>,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn new() -> Self {
        Self {
            leptos_options: LeptosOptions::builder()
                .output_name("minirust-web")
                .site_root("target/site")
                .site_pkg_dir("pkg")
                .site_addr(SocketAddr::from(([127, 0, 0, 1], 3001)))
                .build(),
            auth: None,
            menus: None,
        }
    }

    pub fn with_auth(mut self, auth: WebAuthService) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_menus(mut self, menus: WebMenuService) -> Self {
        self.menus = Some(menus);
        self
    }
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
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
fn App() -> impl IntoView {
    view! {
        <GlobalToast>
            <LeptosRouter>
                <Routes fallback=|| view! { <main class="min-h-screen bg-slate-950 p-10 text-white"><h1>"Not found"</h1></main> }>
                <Route path=path!("") view=|| {
                    #[cfg(feature = "hydrate")]
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().set_href("/login");
                    }
                    view! { <AuthLayout><p class="text-center text-slate-400">"Redirecting to sign in…"</p></AuthLayout> }
                }/>
                <Route path=path!("/login") view=LoginPage/>
                <Route path=path!("/register") view=RegisterPage/>
                <Route path=path!("/register/verify") view=RegisterVerifyPage/>
                <Route path=path!("/invite/accept") view=InvitationAcceptPage/>
                <ParentRoute path=path!("/app") view=AppLayout>
                    <Route path=path!("") view=AppPage/>
                </ParentRoute>
                <ParentRoute path=path!("/profile") view=AppLayout>
                    <Route path=path!("") view=ProfilePage/>
                </ParentRoute>
                <ParentRoute path=path!("/admin/users") view=AppLayout>
                    <Route path=path!("") view=AdminPage/>
                </ParentRoute>
                <ParentRoute path=path!("/admin/menus") view=AppLayout>
                    <Route path=path!("") view=MenuAdminPage/>
                </ParentRoute>
                </Routes>
            </LeptosRouter>
        </GlobalToast>
    }
}

#[cfg(feature = "ssr")]
fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <html lang="vi" class="scroll-smooth bg-slate-950 text-slate-100"><head>
            <meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
            <meta name="description" content="MiniRust closed application."/>
            <meta name="theme-color" content="#020617"/><style>{CSS}</style>
            <leptos::hydration::HydrationScripts options=options.clone/><title>{APP_NAME} {" - Application"}</title>
        </head><body class="min-h-screen overflow-x-hidden bg-slate-950 antialiased"><App/></body></html>
    }
}

#[cfg(feature = "ssr")]
pub fn router(state: AppState) -> Router {
    let routes = generate_route_list(App);
    Router::<AppState>::new()
        .route("/health", get(health))
        .leptos_routes(&state, routes, {
            let options = state.leptos_options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(middleware::from_fn_with_state(state.clone(), auth_guard))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(feature = "ssr")]
const MENU_FORBIDDEN_HTML: &str = r#"<!doctype html><html lang="vi"><head><meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/><title>403 - MiniRust</title>
<style>body{margin:0;min-height:100vh;display:flex;align-items:center;justify-content:center;background:#020617;color:#e2e8f0;font-family:ui-sans-serif,system-ui,sans-serif}main{max-width:34rem;padding:2rem;text-align:center}h1{margin:0;font-size:3rem}p{color:#94a3b8;line-height:1.6}</style>
</head><body><main><h1>403</h1>
<p>Tài khoản của bạn không được phép mở trang này.</p>
<p>Your account is not allowed to open this page.</p>
</main></body></html>"#;

#[cfg(feature = "ssr")]
async fn auth_guard(
    axum::extract::State(state): axum::extract::State<AppState>,
    jar: CookieJar,
    request: axum::extract::Request,
    next: middleware::Next,
) -> impl IntoResponse {
    let path = request.uri().path().to_owned();
    let is_auth_page = matches!(
        path.as_str(),
        "/login" | "/register" | "/register/verify" | "/invite/accept"
    );
    let is_root = path == "/";
    let is_admin_area = path == "/admin/users"
        || path.starts_with("/admin/users/")
        || path == "/admin/menus"
        || path.starts_with("/admin/menus/");
    let is_protected = is_root || path == "/app" || path.starts_with("/profile") || is_admin_area;

    if !is_auth_page && !is_root && !is_protected {
        return next.run(request).await;
    }

    let actor = match (state.auth.as_ref(), jar.get("minirust_session")) {
        (Some(auth), Some(cookie)) => auth.current_session(cookie.value()).await.ok(),
        _ => None,
    };
    let authenticated = actor.is_some();

    if is_auth_page {
        if authenticated {
            return Redirect::to("/app").into_response();
        }
        return next.run(request).await;
    }

    if is_root {
        return if authenticated {
            Redirect::to("/app").into_response()
        } else {
            Redirect::to("/login").into_response()
        };
    }

    let Some(actor) = actor else {
        return Redirect::to("/login").into_response();
    };

    if is_admin_area && !actor.is_admin {
        return Redirect::to("/app").into_response();
    }

    // The menu registry also governs direct URL access: opening a menu the
    // account is not entitled to redirects to the first menu it may open.
    if let Some(menus) = state.menus.as_ref() {
        match menus.decide_access(&actor, &path).await {
            Ok(MenuAccessDecision::Denied {
                fallback: Some(fallback),
            }) => return Redirect::to(&fallback).into_response(),
            Ok(MenuAccessDecision::Denied { fallback: None }) => {
                return (StatusCode::FORBIDDEN, Html(MENU_FORBIDDEN_HTML)).into_response();
            }
            Ok(_) => {}
            Err(error) => {
                tracing::error!(%error, "failed to evaluate menu access");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "menu access check failed",
                )
                    .into_response();
            }
        }
    }

    next.run(request).await
}

#[cfg(feature = "ssr")]
async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!("PANIC: {info}")));
    }));
    leptos::mount::hydrate_body(App);
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn unauthenticated_root_redirects_to_login() {
        let response = router(AppState::new())
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/login");
    }

    #[tokio::test]
    async fn unauthenticated_app_redirects_to_login() {
        let response = router(AppState::new())
            .oneshot(Request::get("/app").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/login");
    }

    #[tokio::test]
    async fn login_page_is_server_rendered() {
        let response = router(AppState::new())
            .oneshot(Request::get("/login").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("Truy cập tài khoản của bạn"));
    }

    #[tokio::test]
    async fn register_page_is_server_rendered() {
        let response = router(AppState::new())
            .oneshot(Request::get("/register").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("Bắt đầu với email của bạn"));
    }

    #[tokio::test]
    async fn get_health_returns_ok() {
        let response = router(AppState::new())
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_string(response).await, "ok");
    }
}
