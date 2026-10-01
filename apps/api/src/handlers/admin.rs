use axum::extract::{rejection::JsonRejection, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use minirust_services::cqrs::{AsyncCommandHandler, AsyncQueryHandler};
use minirust_services::{
    UserAdminCommand, UserAdminCommandResult, UserAdminQuery, UserAdminQueryResult,
};
use serde::{Deserialize, Serialize};

use crate::handlers::auth::authorize_admin;
use crate::response::{ApiResponse, Locale, ProblemDetails};
use crate::{auth_user_response, json_rejection_response, parse_user_id, AppState};

#[derive(Deserialize)]
pub(crate) struct AdminCreateUserRequest {
    email: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminUpdateUserRequest {
    email: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminAssignRoleRequest {
    role: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminPremiumRequest {
    active: bool,
    expires_at: Option<i64>,
}

#[derive(Serialize)]
pub struct AdminUserListResponse {
    pub users: Vec<crate::AuthUserResponse>,
}

#[derive(Serialize)]
pub(crate) struct AdminPremiumResponse {
    active: bool,
    expires_at: Option<i64>,
}

pub async fn list(
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

pub async fn create(
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

pub async fn get(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id_value): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let user_id = match parse_user_id(&user_id_value, locale) {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match state
        .user_queries
        .handle(UserAdminQuery::GetUser { user_id })
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

pub async fn update(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id_value): Path<String>,
    body: Result<Json<AdminUpdateUserRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let user_id = match parse_user_id(&user_id_value, locale) {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::UpdateUserEmail {
            user_id,
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

pub async fn unlock(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id_value): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let user_id = match parse_user_id(&user_id_value, locale) {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::UnlockUser { user_id })
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

pub async fn delete_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id_value): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let user_id = match parse_user_id(&user_id_value, locale) {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match current_authenticated_user(&state, &jar, locale).await {
        Ok(current_user) if current_user.id == user_id => {
            return ProblemDetails::user_admin(
                &minirust_services::UserAdminError::ProtectedUser,
                locale,
            )
            .into_response();
        }
        Ok(_) => {}
        Err(response) => return response,
    }

    match state
        .user_commands
        .handle(UserAdminCommand::DeleteUser { user_id })
        .await
    {
        Ok(UserAdminCommandResult::Deleted) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => ProblemDetails::user_admin(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

pub async fn get_premium(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id_value): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let user_id = match parse_user_id(&user_id_value, locale) {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match state
        .user_queries
        .handle(UserAdminQuery::GetPremium { user_id })
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

pub async fn revoke_premium(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id_value): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let user_id = match parse_user_id(&user_id_value, locale) {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::RevokePremium { user_id })
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

pub async fn set_premium(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id_value): Path<String>,
    body: Result<Json<AdminPremiumRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let user_id = match parse_user_id(&user_id_value, locale) {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::SetPremium {
            user_id,
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

pub async fn assign_role(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id_value): Path<String>,
    body: Result<Json<AdminAssignRoleRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let user_id = match parse_user_id(&user_id_value, locale) {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

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
        .handle(UserAdminCommand::AssignRole { user_id, role })
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
