#![recursion_limit = "256"]
#![cfg_attr(not(feature = "hydrate"), allow(unused_variables, dead_code))]

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
#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_router::{
    components::{ParentRoute, Route, Router as LeptosRouter, Routes},
    path,
};
use minirust_core::APP_NAME;
#[cfg(feature = "ssr")]
use std::net::SocketAddr;
#[cfg(feature = "ssr")]
use tower_http::trace::TraceLayer;

mod api;
mod pages;
pub mod types;

use pages::{AdminPage, AppLayout, AppPage, HomePage, LoginPage, RegisterPage};

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
                .site_addr(SocketAddr::from(([127, 0, 0, 1], 3001)))
                .build(),
        }
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
        <LeptosRouter>
            <Routes fallback=|| view! { <main class="min-h-screen bg-slate-950 p-10 text-white"><h1>"Not found"</h1></main> }>
                <Route path=path!("") view=|| view! { <HomePage message="Hello from MiniRust".to_owned()/> }/>
                <Route path=path!("/login") view=LoginPage/>
                <Route path=path!("/register") view=RegisterPage/>
                <ParentRoute path=path!("/app") view=AppLayout>
                    <Route path=path!("") view=AppPage/>
                </ParentRoute>
                <ParentRoute path=path!("/admin") view=AppLayout>
                    <Route path=path!("") view=AdminPage/>
                </ParentRoute>
            </Routes>
        </LeptosRouter>
    }
}

#[cfg(feature = "ssr")]
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
    Router::<AppState>::new()
        .route("/health", get(health))
        .leptos_routes(&state, routes, {
            let options = state.leptos_options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
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

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn account_pages_are_server_rendered() {
        let app = router(AppState::new());
        for path in ["/", "/login", "/register", "/app", "/admin"] {
            let response = app
                .clone()
                .oneshot(Request::get(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = body_string(response).await;
            assert!(body.contains("MiniRust"));
            assert!(body.contains("pkg"));

            if path == "/app" || path == "/admin" {
                assert!(body.contains("Application navigation"));
                assert!(body.contains("Sign out"));
                assert!(body.contains("Rust-first platform foundation"));
            }

            if path == "/login" {
                assert!(body.contains("Access your account"));
            }

            if path == "/register" {
                assert!(body.contains("Start with your email"));
            }
        }
    }

    #[cfg(feature = "ssr")]
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
