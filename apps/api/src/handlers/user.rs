use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use minirust_services::cqrs::AsyncCommandHandler;
use minirust_services::{UserAdminCommand, UserAdminCommandResult, UserLocale};
use serde::{Deserialize, Serialize};

use crate::handlers::auth::{current_authenticated_user, SESSION_COOKIE};
use crate::response::{ApiResponse, Locale, ProblemDetails};
use crate::{auth_user_response, json_rejection_response, AppState};
use axum::extract::rejection::JsonRejection;

#[derive(Deserialize)]
pub(crate) struct UserProfileUpdateRequest {
    full_name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct AccountActionResponse {
    success: bool,
}

pub async fn profile_update(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    body: Result<Json<UserProfileUpdateRequest>, JsonRejection>,
) -> impl IntoResponse {
    let request_locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, request_locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let locale = Locale::from_user_locale(user.locale);

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            return json_rejection_response(rejection, locale).into_response()
        }
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

pub async fn language_update(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    body: Result<Json<UserLanguageUpdateRequest>, JsonRejection>,
) -> impl IntoResponse {
    let request_locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, request_locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            return json_rejection_response(rejection, request_locale).into_response()
        }
    };

    let Some(locale) = UserLocale::parse(&body.locale) else {
        return ProblemDetails::user_admin(
            &minirust_services::UserAdminError::InvalidLocale,
            request_locale,
        )
        .into_response();
    };

    match state
        .user_commands
        .handle(UserAdminCommand::SetLocale {
            user_id: user.id,
            locale,
        })
        .await
    {
        Ok(UserAdminCommandResult::User(user)) => (
            StatusCode::OK,
            Json(ApiResponse::new(auth_user_response(user))),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, request_locale).into_response(),
        Ok(_) => ProblemDetails::internal(request_locale).into_response(),
    }
}

pub async fn lock(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let request_locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, request_locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let locale = Locale::from_user_locale(user.locale);

    match state
        .user_commands
        .handle(UserAdminCommand::LockUser { user_id: user.id })
        .await
    {
        Ok(UserAdminCommandResult::Locked) => (
            StatusCode::OK,
            jar.remove(Cookie::build(SESSION_COOKIE).path("/").build()),
            Json(ApiResponse::new(AccountActionResponse { success: true })),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

pub async fn delete(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let request_locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, request_locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let locale = Locale::from_user_locale(user.locale);

    match state
        .user_commands
        .handle(UserAdminCommand::DeleteUser { user_id: user.id })
        .await
    {
        Ok(UserAdminCommandResult::Deleted) => (
            StatusCode::OK,
            jar.remove(Cookie::build(SESSION_COOKIE).path("/").build()),
            Json(ApiResponse::new(AccountActionResponse { success: true })),
        )
            .into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}
