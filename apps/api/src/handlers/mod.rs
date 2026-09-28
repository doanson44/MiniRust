use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, Router};
use minirust_services::cqrs::{CommandHandler, QueryHandler};
use minirust_services::{EchoCommand, GreetingQuery};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

use crate::response::ApiResponse;
use crate::{app_error_response, json_rejection_response, AppState};

mod admin;
pub(crate) mod auth;
mod health;
mod openapi;
mod user;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/hello", get(hello))
        .route("/api/v1/echo", post(echo))
        .route(
            "/api/v1/auth/register/request-code",
            post(auth::register_request_code),
        )
        .route(
            "/api/v1/auth/register/verify-code",
            post(auth::register_verify_code),
        )
        .route(
            "/api/v1/auth/login/request-code",
            post(auth::login_request_code),
        )
        .route(
            "/api/v1/auth/login/verify-code",
            post(auth::login_verify_code),
        )
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/me", get(auth::me))
        .route("/api/v1/users/me", patch(user::profile_update))
        .route("/api/v1/users/me/lock", post(user::lock))
        .route("/api/v1/users/me", delete(user::delete))
        .route("/api/v1/admin/users", get(admin::list).post(admin::create))
        .route(
            "/api/v1/admin/users/{user_id}",
            get(admin::get)
                .patch(admin::update)
                .delete(admin::delete_user),
        )
        .route("/api/v1/admin/users/{user_id}/unlock", post(admin::unlock))
        .route(
            "/api/v1/admin/users/{user_id}/role",
            put(admin::assign_role),
        )
        .route(
            "/api/v1/admin/users/{user_id}/entitlements/premium",
            get(admin::get_premium)
                .put(admin::set_premium)
                .delete(admin::revoke_premium),
        )
        .route("/api/v1/openapi.json", get(openapi::openapi))
        .route("/swagger", get(openapi::swagger_ui))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Simple handlers that don't warrant their own file
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

#[derive(Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Serialize)]
struct EchoResponse {
    echo: String,
}

async fn hello(State(state): State<AppState>) -> impl IntoResponse {
    let greeting = state.greeting.handle(GreetingQuery);
    (
        StatusCode::OK,
        Json(ApiResponse::new(HelloResponse {
            message: greeting.message,
        })),
    )
}

async fn echo(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: Result<Json<EchoRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = crate::response::Locale::from_accept_language(&headers);

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state.echo.handle(EchoCommand {
        message: body.message,
    }) {
        Ok(result) => (
            StatusCode::OK,
            Json(ApiResponse::new(EchoResponse { echo: result.echo })),
        )
            .into_response(),
        Err(error) => app_error_response(error, locale).into_response(),
    }
}
