//! MiniRust REST API application.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod response;

use axum::extract::{rejection::JsonRejection, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use minirust_core::AppError;
use minirust_database::Database;
use minirust_services::cqrs::{
    AsyncCommandHandler, AsyncQueryHandler, CommandHandler, QueryHandler,
};
use minirust_services::{
    AuthCommand, AuthCommandHandler, AuthCommandResult, AuthError, AuthQueryHandler, AuthService,
    CurrentSessionQuery, RequireAdminQuery, UnavailableEmailSender, UserAdminCommand,
    UserAdminCommandHandler, UserAdminCommandResult, UserAdminQuery, UserAdminQueryHandler,
    UserAdminQueryResult, UserAdminService,
};
use minirust_services::{EchoCommand, EchoCommandHandler, GreetingQuery, GreetingQueryHandler};
use response::{ApiResponse, Locale, ProblemDetails};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub echo: EchoCommandHandler,
    pub greeting: GreetingQueryHandler,
    pub database: Database,
    pub auth_commands: AuthCommandHandler<Database, UnavailableEmailSender>,
    pub auth_queries: AuthQueryHandler<Database, UnavailableEmailSender>,
    pub user_commands: UserAdminCommandHandler<Database>,
    pub user_queries: UserAdminQueryHandler<Database>,
    pub auth_rate_limiter: AuthRateLimiter,
    pub secure_cookies: bool,
}

#[derive(Clone, Default)]
pub struct AuthRateLimiter {
    entries: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl AuthRateLimiter {
    pub fn check(&self, key: &str, max_requests: usize, window: Duration) -> bool {
        let now = Instant::now();
        let mut entries = match self.entries.lock() {
            Ok(entries) => entries,
            Err(poisoned) => poisoned.into_inner(),
        };
        if !entries.contains_key(key) && entries.len() >= 10_000 {
            entries.retain(|_, timestamps| !timestamps.is_empty());
            if entries.len() >= 10_000 {
                if let Some(oldest_key) = entries.keys().next().cloned() {
                    entries.remove(&oldest_key);
                }
            }
        }
        let timestamps = entries.entry(key.to_owned()).or_default();
        while timestamps
            .front()
            .is_some_and(|timestamp| now.duration_since(*timestamp) >= window)
        {
            timestamps.pop_front();
        }
        if timestamps.len() >= max_requests {
            return false;
        }
        timestamps.push_back(now);
        true
    }
}

impl AppState {
    pub fn new(
        database: Database,
        auth_secret: impl Into<Vec<u8>>,
        secure_cookies: bool,
    ) -> Result<Self, AuthError> {
        let auth = AuthService::new(database.clone(), UnavailableEmailSender, auth_secret)?;
        let users = UserAdminService::new(database.clone());
        Ok(Self {
            echo: EchoCommandHandler,
            greeting: GreetingQueryHandler,
            database,
            auth_commands: AuthCommandHandler::new(auth.clone()),
            auth_queries: AuthQueryHandler::new(auth),
            user_commands: UserAdminCommandHandler::new(users.clone()),
            user_queries: UserAdminQueryHandler::new(users),
            auth_rate_limiter: AuthRateLimiter::default(),
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
        .route(
            "/api/v1/auth/register/request-code",
            post(auth_register_request_code),
        )
        .route(
            "/api/v1/auth/register/verify-code",
            post(auth_register_verify_code),
        )
        .route(
            "/api/v1/auth/login/request-code",
            post(auth_login_request_code),
        )
        .route(
            "/api/v1/auth/login/verify-code",
            post(auth_login_verify_code),
        )
        .route("/api/v1/auth/logout", post(auth_logout))
        .route("/api/v1/auth/me", get(auth_me))
        .route("/api/v1/users/me", patch(user_profile_update))
        .route("/api/v1/users/me/lock", post(user_lock))
        .route("/api/v1/users/me", delete(user_delete))
        .route(
            "/api/v1/admin/users",
            get(admin_users_list).post(admin_users_create),
        )
        .route(
            "/api/v1/admin/users/{email}",
            get(admin_user_get)
                .patch(admin_user_update)
                .delete(admin_user_delete),
        )
        .route(
            "/api/v1/admin/users/{email}/unlock",
            post(admin_user_unlock),
        )
        .route(
            "/api/v1/admin/users/{email}/role",
            put(admin_user_assign_role),
        )
        .route(
            "/api/v1/admin/users/{email}/entitlements/premium",
            get(admin_user_get_premium)
                .put(admin_user_set_premium)
                .delete(admin_user_revoke_premium),
        )
        .route("/api/v1/openapi.json", get(openapi))
        .route("/swagger", get(swagger_ui))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
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
    full_name: Option<String>,
    avatar_url: Option<String>,
    is_locked: bool,
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

#[derive(Deserialize)]
struct UserProfileUpdateRequest {
    full_name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
struct AccountActionResponse {
    success: bool,
}

fn auth_user_response(user: minirust_services::UserAccess) -> AuthUserResponse {
    AuthUserResponse {
        id: user.id.as_uuid().to_string(),
        email: user.email,
        is_admin: user.is_admin,
        is_premium: user.is_premium,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        is_locked: user.is_locked,
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
    body: Result<Json<AuthEmailRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    if !state
        .auth_rate_limiter
        .check("register:global", 60, Duration::from_secs(15 * 60))
    {
        return response::ProblemDetails::rate_limited(locale).into_response();
    }

    let rate_key = format!("register:{}", body.email.trim().to_ascii_lowercase());
    if !state
        .auth_rate_limiter
        .check(&rate_key, 3, Duration::from_secs(15 * 60))
    {
        return response::ProblemDetails::rate_limited(locale).into_response();
    }

    match state
        .auth_commands
        .handle(AuthCommand::RequestRegistrationCode {
            email: body.email.clone(),
        })
        .await
    {
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
    body: Result<Json<AuthEmailRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    if !state
        .auth_rate_limiter
        .check("login:global", 60, Duration::from_secs(15 * 60))
    {
        return response::ProblemDetails::rate_limited(locale).into_response();
    }

    let rate_key = format!("login:{}", body.email.trim().to_ascii_lowercase());
    if !state
        .auth_rate_limiter
        .check(&rate_key, 3, Duration::from_secs(15 * 60))
    {
        return response::ProblemDetails::rate_limited(locale).into_response();
    }

    match state
        .auth_commands
        .handle(AuthCommand::RequestLoginCode {
            email: body.email.clone(),
        })
        .await
    {
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
    body: Result<Json<AuthVerifyRequest>, JsonRejection>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .auth_commands
        .handle(AuthCommand::VerifyRegistrationCode {
            email: body.email.clone(),
            code: body.code.clone(),
        })
        .await
    {
        Ok(AuthCommandResult::Session(session)) => {
            let response = AuthSessionResponse {
                user: auth_user_response(session.user),
                expires_at: session.expires_at,
            };
            (
                StatusCode::OK,
                jar.add(session_cookie(&session.token, state.secure_cookies)),
                Json(ApiResponse::new(response)),
            )
                .into_response()
        }
        Err(error) => auth_error_response(error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn auth_login_verify_code(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: Result<Json<AuthVerifyRequest>, JsonRejection>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .auth_commands
        .handle(AuthCommand::VerifyLoginCode {
            email: body.email.clone(),
            code: body.code.clone(),
        })
        .await
    {
        Ok(AuthCommandResult::Session(session)) => {
            let response = AuthSessionResponse {
                user: auth_user_response(session.user),
                expires_at: session.expires_at,
            };
            (
                StatusCode::OK,
                jar.add(session_cookie(&session.token, state.secure_cookies)),
                Json(ApiResponse::new(response)),
            )
                .into_response()
        }
        Err(error) => auth_error_response(error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
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

    match state
        .auth_queries
        .handle(CurrentSessionQuery {
            token: cookie.value().to_owned(),
        })
        .await
    {
        Ok(user) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => auth_error_response(error, locale).into_response(),
    }
}

#[derive(Deserialize)]
struct AdminCreateUserRequest {
    email: String,
}

#[derive(Deserialize)]
struct AdminUpdateUserRequest {
    email: String,
}

#[derive(Deserialize)]
struct AdminAssignRoleRequest {
    role: String,
}

#[derive(Deserialize)]
struct AdminPremiumRequest {
    active: bool,
    expires_at: Option<i64>,
}

#[derive(Serialize)]
struct AdminUserListResponse {
    users: Vec<AuthUserResponse>,
}

#[derive(Serialize)]
struct AdminPremiumResponse {
    active: bool,
    expires_at: Option<i64>,
}

async fn authorize_admin(
    state: &AppState,
    jar: &CookieJar,
    locale: Locale,
) -> Result<minirust_services::UserAccess, axum::response::Response> {
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return Err(auth_error_response(AuthError::SessionInvalid, locale).into_response());
    };
    state
        .auth_queries
        .handle(RequireAdminQuery {
            token: cookie.value().to_owned(),
        })
        .await
        .map_err(|error| auth_error_response(error, locale).into_response())
}

async fn admin_users_list(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    match state.user_queries.handle(UserAdminQuery::ListUsers).await {
        Ok(UserAdminQueryResult::Users(users)) => (
            StatusCode::OK,
            Json(ApiResponse::new(AdminUserListResponse {
                users: users.into_iter().map(auth_user_response).collect(),
            })),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_users_create(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    body: Result<Json<AdminCreateUserRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::CreateUser {
            email: body.email.clone(),
        })
        .await
    {
        Ok(UserAdminCommandResult::User(user)) => (
            StatusCode::CREATED,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_user_get(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(email): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    match state
        .user_queries
        .handle(UserAdminQuery::GetUser {
            email: email.clone(),
        })
        .await
    {
        Ok(UserAdminQueryResult::User(user)) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_user_update(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(email): Path<String>,
    body: Result<Json<AdminUpdateUserRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::UpdateUserEmail {
            current_email: email.clone(),
            new_email: body.email.clone(),
        })
        .await
    {
        Ok(UserAdminCommandResult::User(user)) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_user_unlock(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(email): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    match state
        .user_commands
        .handle(UserAdminCommand::UnlockUser {
            email: email.clone(),
        })
        .await
    {
        Ok(UserAdminCommandResult::User(user)) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_user_delete(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(email): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    match state
        .user_commands
        .handle(UserAdminCommand::DeleteUser {
            email: email.clone(),
        })
        .await
    {
        Ok(UserAdminCommandResult::Deleted) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_user_get_premium(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(email): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    match state
        .user_queries
        .handle(UserAdminQuery::GetPremium {
            email: email.clone(),
        })
        .await
    {
        Ok(UserAdminQueryResult::Premium(entitlement)) => (
            StatusCode::OK,
            Json(ApiResponse::new(AdminPremiumResponse {
                active: entitlement.active,
                expires_at: entitlement.expires_at,
            })),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_user_revoke_premium(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(email): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    match state
        .user_commands
        .handle(UserAdminCommand::RevokePremium {
            email: email.clone(),
        })
        .await
    {
        Ok(UserAdminCommandResult::User(user)) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_user_set_premium(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(email): Path<String>,
    body: Result<Json<AdminPremiumRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::SetPremium {
            email: email.clone(),
            active: body.active,
            expires_at: body.expires_at,
        })
        .await
    {
        Ok(UserAdminCommandResult::User(user)) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn admin_user_assign_role(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(email): Path<String>,
    body: Result<Json<AdminAssignRoleRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    let role = match minirust_services::AdminUserRole::parse(&body.role) {
        Ok(role) => role,
        Err(error) => return ProblemDetails::user_admin(&error, locale).into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::AssignRole {
            email: email.clone(),
            role,
        })
        .await
    {
        Ok(UserAdminCommandResult::User(user)) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn user_profile_update(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    body: Result<Json<UserProfileUpdateRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::UpdateProfile {
            user_id: user.id,
            full_name: body.full_name.clone(),
            avatar_url: body.avatar_url.clone(),
        })
        .await
    {
        Ok(UserAdminCommandResult::User(user)) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn user_lock(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    match state
        .user_commands
        .handle(UserAdminCommand::LockUser { user_id: user.id })
        .await
    {
        Ok(UserAdminCommandResult::Locked) => (
            jar.remove(Cookie::build(SESSION_COOKIE).path("/").removal().build()),
            StatusCode::OK,
            Json(ApiResponse::new(AccountActionResponse { success: true })),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn user_delete(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    match state
        .user_commands
        .handle(UserAdminCommand::DeleteUserById { user_id: user.id })
        .await
    {
        Ok(UserAdminCommandResult::Deleted) => (
            jar.remove(Cookie::build(SESSION_COOKIE).path("/").removal().build()),
            StatusCode::OK,
            Json(ApiResponse::new(AccountActionResponse { success: true })),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn current_authenticated_user(
    state: &AppState,
    jar: &CookieJar,
    locale: Locale,
) -> Result<minirust_services::UserAccess, axum::response::Response> {
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return Err(auth_error_response(AuthError::SessionInvalid, locale).into_response());
    };

    state
        .auth_queries
        .handle(CurrentSessionQuery {
            token: cookie.value().to_owned(),
        })
        .await
        .map_err(|error| auth_error_response(error, locale).into_response())
}

async fn auth_logout(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);

    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        if let Err(error) = state
            .auth_commands
            .handle(AuthCommand::Logout {
                token: cookie.value().to_owned(),
            })
            .await
        {
            return auth_error_response(error, locale).into_response();
        }
    }

    let removal = Cookie::build(SESSION_COOKIE).path("/").removal().build();

    (
        StatusCode::OK,
        jar.remove(removal),
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
                "/api/v1/echo": { "post": { "summary": "Echo command", "requestBody": { "required": true, "content": { "application/json": { "schema": { "type": "object", "required": ["message"], "properties": { "message": { "type": "string" } } } } } }, "responses": { "200": { "description": "Echo response" }, "400": { "description": "Malformed JSON or invalid content type" }, "422": { "description": "Validation error" }, "500": { "description": "Unexpected server failure" } } } },
                "/api/v1/auth/register/request-code": { "post": { "summary": "Request registration verification code", "responses": { "200": { "description": "Request accepted" } } } },
                "/api/v1/auth/register/verify-code": { "post": { "summary": "Verify registration code and create session", "responses": { "200": { "description": "Authenticated session" } } } },
                "/api/v1/auth/login/request-code": { "post": { "summary": "Request login verification code", "responses": { "200": { "description": "Request accepted" } } } },
                "/api/v1/auth/login/verify-code": { "post": { "summary": "Verify login code and create session", "responses": { "200": { "description": "Authenticated session" } } } },
                "/api/v1/auth/logout": { "post": { "summary": "Revoke current session", "responses": { "200": { "description": "Session revoked" } } } },
                "/api/v1/auth/me": { "get": { "summary": "Get current authenticated user", "responses": { "200": { "description": "Current user" }, "401": { "description": "Invalid or expired session" } } } },
                "/api/v1/users/me": { "patch": { "summary": "Update current user profile", "responses": { "200": { "description": "Profile updated" }, "401": { "description": "Authentication required" }, "422": { "description": "Invalid profile data" } } }, "delete": { "summary": "Delete current user account", "responses": { "200": { "description": "Account deleted" }, "401": { "description": "Authentication required" } } } },
                "/api/v1/users/me/lock": { "post": { "summary": "Lock current user account", "responses": { "200": { "description": "Account locked" }, "401": { "description": "Authentication required" } } } },
                "/api/v1/admin/users": { "get": { "summary": "List users (admin only)", "responses": { "200": { "description": "Users" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" } } }, "post": { "summary": "Create user by email (admin only)", "responses": { "201": { "description": "User created" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" } } } },
                "/api/v1/admin/users/{email}": { "get": { "summary": "Get user by email (admin only)", "responses": { "200": { "description": "User" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } }, "patch": { "summary": "Update user email (admin only)", "responses": { "200": { "description": "User updated" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" }, "409": { "description": "Email already exists or protected user" } } }, "delete": { "summary": "Delete user by email (admin only)", "responses": { "204": { "description": "User deleted" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } } },
                "/api/v1/admin/users/{email}/unlock": { "post": { "summary": "Unlock user account (admin only)", "responses": { "200": { "description": "User unlocked" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } } },
                "/api/v1/admin/users/{email}/role": { "put": { "summary": "Assign or remove admin role (admin only)", "responses": { "200": { "description": "User role updated" }, "403": { "description": "Admin role required" }, "422": { "description": "Invalid role" } } } },
                "/api/v1/admin/users/{email}/entitlements/premium": { "get": { "summary": "Get premium entitlement (admin only)", "responses": { "200": { "description": "Premium entitlement" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } }, "put": { "summary": "Assign premium entitlement (admin only)", "responses": { "200": { "description": "Premium entitlement updated" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" }, "422": { "description": "Invalid premium expiry" } } }, "delete": { "summary": "Revoke premium entitlement (admin only)", "responses": { "200": { "description": "Premium entitlement revoked" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } } }
            }
        })),
    )
}

async fn swagger_ui() -> impl IntoResponse {
    let html = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MiniRust API — OpenAPI</title>
<style>
body{margin:0;font:15px/1.5 system-ui,sans-serif;background:#0f172a;color:#e2e8f0}
main{max-width:1100px;margin:auto;padding:32px 20px}h1{margin:0 0 8px;color:#fff}p{color:#94a3b8}
.card{margin:14px 0;padding:16px;border:1px solid #334155;border-radius:12px;background:#111827}
.method{display:inline-block;padding:3px 8px;border-radius:6px;background:#22d3ee;color:#082f49;font-weight:700;margin-right:10px}
.path{font-family:ui-monospace,monospace;color:#fff}.summary{margin:8px 0;color:#94a3b8}
pre{white-space:pre-wrap;background:#020617;padding:16px;border-radius:10px;overflow:auto}
a{color:#67e8f9}
</style>
</head>
<body><main><h1>MiniRust API</h1><p>OpenAPI 3.0.3 documentation. The viewer is bundled with the application; no external CDN is required.</p><div id="docs">Loading…</div></main>
<script>
fetch('/api/v1/openapi.json').then(r=>r.json()).then(spec=>{
 const root=document.querySelector('#docs'); const paths=spec.paths||{};
 root.innerHTML=Object.entries(paths).flatMap(([path,item])=>Object.entries(item).map(([method,op])=>'<section class="card"><div><span class="method">'+method.toUpperCase()+'</span><span class="path">'+path+'</span></div><div class="summary">'+(op.summary||'')+'</div></section>')).join('')+'<section class="card"><details><summary>Raw OpenAPI document</summary><pre>'+JSON.stringify(spec,null,2).replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]))+'</pre></details></section>';
}).catch(()=>{document.querySelector('#docs').textContent='Unable to load OpenAPI document.'});
</script></body></html>"#;
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
}
