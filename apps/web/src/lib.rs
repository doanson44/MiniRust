//! MiniRust Leptos SSR application.

use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use leptos::prelude::*;
use minirust_core::APP_NAME;
use minirust_services::{GreetingQuery, GreetingQueryHandler};
use tower_http::trace::TraceLayer;

/// Shared web application state.
#[derive(Clone)]
pub struct AppState {
    pub greeting: GreetingQueryHandler,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            greeting: GreetingQueryHandler,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
fn HomePage(message: String) -> impl IntoView {
    view! {
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>{APP_NAME}</title>
            </head>
            <body>
                <main class="mx-auto max-w-3xl p-8">
                    <h1 class="text-3xl font-bold">{APP_NAME}</h1>
                    <p class="mt-4">{message}</p>
                </main>
            </body>
        </html>
    }
}

/// Render the baseline home page to an HTML string.
pub fn render_home_page(message: &str) -> String {
    let html = view! { <HomePage message=message.to_owned()/> }.to_html();
    format!("<!DOCTYPE html>{html}")
}

/// HTTP router used by the binary and by tests.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn home(State(state): State<AppState>) -> impl IntoResponse {
    let greeting = state.greeting.handle(GreetingQuery);
    let html = render_home_page(&greeting.message);
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        )],
        html,
    )
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[cfg(test)]
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

    #[test]
    fn home_page_renders_the_greeting() {
        let html = render_home_page("Hello from MiniRust");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<h1>MiniRust</h1>"));
        assert!(html.contains("Hello from MiniRust"));
    }

    #[tokio::test]
    async fn get_root_returns_html() {
        let response = router(AppState::new())
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.starts_with("text/html"));

        let body = body_string(response).await;
        assert!(body.contains("MiniRust"));
        assert!(body.contains("Hello from MiniRust"));
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
