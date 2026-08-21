//! MiniRust REST API application.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use minirust_core::{AppError, EchoInput};
use minirust_services::{EchoService, GreetingService, HealthService};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

/// Shared API state. Services are cheap and stateless in the baseline.
#[derive(Clone)]
pub struct AppState {
    pub health: HealthService,
    pub greetings: GreetingService,
    pub echo: EchoService,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            health: HealthService,
            greetings: GreetingService,
            echo: EchoService,
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

/// Request body for `POST /api/v1/echo`.
#[derive(Deserialize)]
struct EchoRequest {
    message: String,
}

/// Successful response body for `POST /api/v1/echo`.
#[derive(Serialize)]
struct EchoResponse {
    echo: String,
}

/// JSON error envelope returned on application errors.
///
/// Used by any endpoint that maps [`AppError::Validation`] to `422`.
/// Future endpoints should reuse this type so API error responses stay consistent.
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Map an [`AppError`] to an Axum-compatible HTTP response.
///
/// - `Validation` → 422 Unprocessable Entity with a JSON error body (safe to show callers).
/// - Other variants → 500 Internal Server Error with a generic message (never expose internals).
fn app_error_response(error: AppError) -> Response {
    match error {
        AppError::Validation(message) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse { error: message }),
        )
            .into_response(),
        other => {
            tracing::error!(%other, "unexpected application error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "internal server error".to_owned(),
                }),
            )
                .into_response()
        }
    }
}

/// HTTP router used by the binary and by tests.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/hello", get(hello))
        .route("/api/v1/echo", post(echo))
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

async fn echo(State(state): State<AppState>, Json(body): Json<EchoRequest>) -> impl IntoResponse {
    match EchoInput::parse(body.message) {
        Ok(input) => {
            let result = state.echo.echo(input);
            (StatusCode::OK, Json(EchoResponse { echo: result.echo })).into_response()
        }
        Err(error) => app_error_response(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, Request};
    use tower::ServiceExt;

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn json_body(json: &str) -> Body {
        Body::from(json.to_owned())
    }

    fn post_json(uri: &str, body: &str) -> Request<Body> {
        Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json_body(body))
            .unwrap()
    }

    // ── existing baseline tests ──────────────────────────────────────────────

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

    // ── echo endpoint tests ──────────────────────────────────────────────────

    #[tokio::test]
    async fn echo_returns_the_message() {
        let response = router(AppState::new())
            .oneshot(post_json("/api/v1/echo", r#"{"message":"Hello World"}"#))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert_eq!(body, r#"{"echo":"Hello World"}"#);
    }

    #[tokio::test]
    async fn echo_trims_surrounding_whitespace() {
        let response = router(AppState::new())
            .oneshot(post_json("/api/v1/echo", r#"{"message":"  hi  "}"#))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert_eq!(body, r#"{"echo":"hi"}"#);
    }

    #[tokio::test]
    async fn echo_returns_422_for_empty_message() {
        let response = router(AppState::new())
            .oneshot(post_json("/api/v1/echo", r#"{"message":""}"#))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_string(response).await;
        assert!(body.contains("empty"), "body was: {body}");
    }

    #[tokio::test]
    async fn echo_returns_422_for_oversized_message() {
        let big = "a".repeat(501);
        let payload = format!(r#"{{"message":"{big}"}}"#);
        let response = router(AppState::new())
            .oneshot(post_json("/api/v1/echo", &payload))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_string(response).await;
        assert!(body.contains("500"), "body was: {body}");
    }

    #[tokio::test]
    async fn echo_returns_error_for_missing_json_body() {
        // Axum's JSON extractor rejects a missing body before the handler runs.
        let response = router(AppState::new())
            .oneshot(Request::post("/api/v1/echo").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // 400 Bad Request or 415 Unsupported Media Type — either is acceptable;
        // the important thing is that it is not a success.
        assert!(
            response.status().is_client_error(),
            "expected a 4xx, got {}",
            response.status()
        );
    }
}
