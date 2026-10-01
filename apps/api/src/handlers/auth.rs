use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use minirust_services::cqrs::AsyncCommandHandler;
use minirust_services::cqrs::AsyncQueryHandler;
use minirust_services::{
    AuthCommand, AuthCommandResult, AuthError, CurrentSessionQuery, RequireAdminQuery, UserAccess,
};
use serde::{Deserialize, Serialize};

use crate::response::{ApiResponse, Locale, ProblemDetails};
use crate::{auth_error_response, auth_user_response, json_rejection_response, AppState};
use axum::extract::rejection::JsonRejection;

pub const SESSION_COOKIE: &str = "minirust_session";

#[derive(Deserialize)]
pub(crate) struct AuthEmailRequest {
    email: String,
}

#[derive(Deserialize)]
pub(crate) struct AuthVerifyRequest {
    email: String,
    code: String,
}

#[derive(Deserialize)]
pub(crate) struct AuthTokenRequest {
    token: String,
}

#[derive(Serialize)]
pub(crate) struct CodeRequestResponse {
    accepted: bool,
    verification_url: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LogoutResponse {
    success: bool,
}

pub fn session_cookie(value: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, value.to_owned()))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .build()
}

#[allow(clippy::result_large_err)]
pub async fn authorize_admin(
    state: &AppState,
    jar: &CookieJar,
    locale: Locale,
) -> Result<UserAccess, axum::response::Response> {
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

#[allow(clippy::result_large_err)]
pub async fn current_authenticated_user(
    state: &AppState,
    jar: &CookieJar,
    locale: Locale,
) -> Result<UserAccess, axum::response::Response> {
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

pub async fn register_request_verification(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: Result<Json<AuthEmailRequest>, JsonRejection>,
) -> impl IntoResponse {
    use std::time::Duration;
    let locale = Locale::from_accept_language(&headers);
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    if !state
        .auth_rate_limiter
        .check("register:global", 60, Duration::from_secs(15 * 60))
    {
        return ProblemDetails::rate_limited(locale).into_response();
    }

    let rate_key = format!("register:{}", body.email.trim().to_ascii_lowercase());
    if !state
        .auth_rate_limiter
        .check(&rate_key, 3, Duration::from_secs(15 * 60))
    {
        return ProblemDetails::rate_limited(locale).into_response();
    }

    match state
        .auth_commands
        .handle(AuthCommand::RequestRegistrationVerification {
            email: body.email.clone(),
        })
        .await
    {
        Ok(AuthCommandResult::CodeRequested(result)) => {
            let verification_url = if !state.secure_cookies {
                result
                    .verification_token
                    .map(|token| format!("/register/verify?token={token}"))
            } else {
                None
            };

            (
                StatusCode::OK,
                Json(ApiResponse::new(CodeRequestResponse {
                    accepted: true,
                    verification_url,
                })),
            )
                .into_response()
        }
        Err(error) => auth_error_response(error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

pub async fn login_request_code(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: Result<Json<AuthEmailRequest>, JsonRejection>,
) -> impl IntoResponse {
    use std::time::Duration;
    let locale = Locale::from_accept_language(&headers);
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    if !state
        .auth_rate_limiter
        .check("login:global", 60, Duration::from_secs(15 * 60))
    {
        return ProblemDetails::rate_limited(locale).into_response();
    }

    let rate_key = format!("login:{}", body.email.trim().to_ascii_lowercase());
    if !state
        .auth_rate_limiter
        .check(&rate_key, 3, Duration::from_secs(15 * 60))
    {
        return ProblemDetails::rate_limited(locale).into_response();
    }

    match state
        .auth_commands
        .handle(AuthCommand::RequestLoginCode {
            email: body.email.clone(),
        })
        .await
    {
        Ok(AuthCommandResult::CodeRequested(_result)) => (
            StatusCode::OK,
            Json(ApiResponse::new(CodeRequestResponse {
                accepted: true,
                verification_url: None,
            })),
        )
            .into_response(),
        Err(error) => auth_error_response(error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

pub async fn register_verify(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    body: Result<Json<AuthTokenRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .auth_commands
        .handle(AuthCommand::VerifyRegistration { token: body.token })
        .await
    {
        Ok(AuthCommandResult::Session(session)) => {
            let response = crate::AuthSessionResponse {
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

pub async fn login_verify_code(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    body: Result<Json<AuthVerifyRequest>, JsonRejection>,
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
            let response = crate::AuthSessionResponse {
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

pub async fn me(
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

pub async fn logout(
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
        jar.add(removal),
        Json(ApiResponse::new(LogoutResponse { success: true })),
    )
        .into_response()
}
