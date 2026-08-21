//! MiniRust REST API application.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use minirust_services::{GreetingService, HealthService};
use serde::Serialize;
use tower_http::trace::TraceLayer;

/// Shared API state. Services are cheap and stateless in the baseline.
#[derive(Clone)]
pub struct AppState {
    pub health: HealthService,
    pub greetings: GreetingService,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            health: HealthService,
            greetings: GreetingService,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

/// HTTP router used by the binary and by tests.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/hello", get(hello))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.health.status();
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: status.status,
        }),
    )
}

async fn hello(State(state): State<AppState>) -> impl IntoResponse {
    let greeting = state.greetings.hello();
    (
        StatusCode::OK,
        Json(HelloResponse {
            message: greeting.message,
        }),
    )
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

    #[tokio::test]
    async fn health_returns_ok() {
        let response = router(AppState::new())
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert_eq!(body, r#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn hello_returns_greeting() {
        let response = router(AppState::new())
            .oneshot(Request::get("/api/v1/hello").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert_eq!(body, r#"{"message":"Hello from MiniRust"}"#);
    }
}
