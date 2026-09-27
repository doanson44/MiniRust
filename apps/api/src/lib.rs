//! MiniRust REST API application.

mod response;

use axum::extract::{rejection::JsonRejection, State};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use minirust_core::AppError;
use minirust_database::Database;
use minirust_services::cqrs::{CommandHandler, QueryHandler};
use minirust_services::{AuthError, AuthService, UnavailableEmailSender};
use minirust_services::{EchoCommand, EchoCommandHandler, GreetingQuery, GreetingQueryHandler};
use response::{ApiResponse, Locale, ProblemDetails};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub echo: EchoCommandHandler,
    pub greeting: GreetingQueryHandler,
    pub database: Database,
    pub auth: AuthService<Database, UnavailableEmailSender>,
    pub secure_cookies: bool,
}

impl AppState {
    pub fn new(database: Database, auth_secret: impl Into<Vec<u8>>, secure_cookies: bool) -> Result<Self, AuthError> {
        let auth = AuthService::new(database.clone(), UnavailableEmailSender, auth_secret)?;
        Ok(Self {
            echo: EchoCommandHandler,
            greeting: GreetingQueryHandler,
            database,
            auth,
            secure_cookies,
        })
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

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

fn app_error_response(error: AppError, locale: Locale) -> ProblemDetails {
    match error {
        AppError::Validation(error) => ProblemDetails::validation(&error, locale),
        other => {
            tracing::error!(%other, "unexpected application error");
            ProblemDetails::internal(locale)
        }
    }
}

fn json_rejection_response(rejection: JsonRejection, locale: Locale) -> ProblemDetails {
    if rejection.status() == StatusCode::BAD_REQUEST {
        ProblemDetails::bad_request(locale)
    } else {
        tracing::error!(%rejection, "request body extraction failed");
        ProblemDetails::internal(locale)
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/hello", get(hello))
        .route("/api/v1/echo", post(echo))
        .route("/api/v1/auth/register/request-code", post(auth_register_request_code))
        .route("/api/v1/auth/register/verify-code", post(auth_register_verify_code))
        .route("/api/v1/auth/login/request-code", post(auth_login_request_code))
        .route("/api/v1/auth/login/verify-code", post(auth_login_verify_code))
        .route("/api/v1/auth/logout", post(auth_logout))
        .route("/api/v1/auth/me", get(auth_me))
        .route("/api/v1/openapi.json", get(openapi))
        .route("/swagger", get(swagger_ui))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);

    match state.database.health().await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::new(HealthResponse {
                status: "ok",
                database: "ok",
            })),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "database health check failed");
            ProblemDetails::service_unavailable(locale).into_response()
        }
    }
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
    let locale = Locale::from_accept_language(&headers);

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


const SESSION_COOKIE: &str = "minirust_session";

#[derive(Deserialize)]
struct AuthEmailRequest {
    email: String,
}

#[derive(Deserialize)]
struct AuthVerifyRequest {
    email: String,
    code: String,
}

#[derive(Serialize)]
struct CodeRequestResponse {
    accepted: bool,
}

#[derive(Serialize)]
struct AuthUserResponse {
    id: String,
    email: String,
    is_admin: bool,
    is_premium: bool,
}

#[derive(Serialize)]
struct AuthSessionResponse {
    user: AuthUserResponse,
    expires_at: i64,
}

#[derive(Serialize)]
struct LogoutResponse {
    success: bool,
}

fn auth_user_response(user: minirust_services::UserAccess) -> AuthUserResponse {
    AuthUserResponse {
        id: user.id.as_uuid().to_string(),
        email: user.email,
        is_admin: user.is_admin,
        is_premium: user.is_premium,
    }
}

fn auth_error_response(error: AuthError, locale: Locale) -> ProblemDetails {
    tracing::warn!(error = %error, "authentication request failed");
    ProblemDetails::auth(&error, locale)
}

fn session_cookie(value: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, value.to_owned()))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .build()
}

async fn auth_register_request_code(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<AuthEmailRequest>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);

    match state.auth.request_registration_code(&body.email).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::new(CodeRequestResponse { accepted: true })),
        )
            .into_response(),
        Err(error) => auth_error_response(error, locale).into_response(),
    }
}

async fn auth_login_request_code(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<AuthEmailRequest>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);

    match state.auth.request_login_code(&body.email).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::new(CodeRequestResponse { accepted: true })),
        )
            .into_response(),
        Err(error) => auth_error_response(error, locale).into_response(),
    }
}

async fn auth_register_verify_code(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<AuthVerifyRequest>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);

    match state.auth.verify_registration_code(&body.email, &body.code).await {
        Ok(session) => {
            let response = AuthSessionResponse {
                user: auth_user_response(session.user),
                expires_at: session.expires_at,
            };
            (
                jar.add(session_cookie(&session.token, state.secure_cookies)),
                StatusCode::OK,
                Json(ApiResponse::new(response)),
            )
                .into_response()
        }
        Err(error) => auth_error_response(error, locale).into_response(),
    }
}

async fn auth_login_verify_code(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<AuthVerifyRequest>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);

    match state.auth.verify_login_code(&body.email, &body.code).await {
        Ok(session) => {
            let response = AuthSessionResponse {
                user: auth_user_response(session.user),
                expires_at: session.expires_at,
            };
            (
                jar.add(session_cookie(&session.token, state.secure_cookies)),
                StatusCode::OK,
                Json(ApiResponse::new(response)),
            )
                .into_response()
        }
        Err(error) => auth_error_response(error, locale).into_response(),
    }
}

async fn auth_me(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return auth_error_response(AuthError::SessionInvalid, locale).into_response();
    };

    match state.auth.current_session(cookie.value()).await {
        Ok(user) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => auth_error_response(error, locale).into_response(),
    }
}

async fn auth_logout(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);

    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        if let Err(error) = state.auth.logout(cookie.value()).await {
            return auth_error_response(error, locale).into_response();
        }
    }

    let removal = Cookie::build(SESSION_COOKIE)
        .path("/")
        .removal()
        .build();

    (
        jar.remove(removal),
        StatusCode::OK,
        Json(ApiResponse::new(LogoutResponse { success: true })),
    )
        .into_response()
}

async fn openapi() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "openapi": "3.0.3",
            "info": { "title": "MiniRust API", "version": "0.1.0" },
            "paths": {
                "/health": { "get": { "summary": "Health and MariaDB connectivity", "responses": { "200": { "description": "Application and database are healthy" }, "503": { "description": "Database is unavailable" } } } },
                "/api/v1/hello": { "get": { "summary": "Hello query", "responses": { "200": { "description": "Greeting" } } } },
                "/api/v1/echo": { "post": { "summary": "Echo command", "requestBody": { "required": true, "content": { "application/json": { "schema": { "type": "object", "required": ["message"], "properties": { "message": { "type": "string" } } } } } }, "responses": { "200": { "description": "Echo response" }, "400": { "description": "Malformed JSON or invalid content type" }, "422": { "description": "Validation error" }, "500": { "description": "Unexpected server failure" } } } }
            }
        })),
    )
}

async fn swagger_ui() -> impl IntoResponse {
    let html = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>MiniRust API</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head><body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script><script>window.onload=()=>SwaggerUIBundle({url:'/api/v1/openapi.json',dom_id:'#swagger-ui'});</script></body></html>"#;
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
}
