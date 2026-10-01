use axum::extract::{rejection::JsonRejection, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use minirust_services::cqrs::{AsyncCommandHandler, AsyncQueryHandler};
use minirust_services::{
    PaginationMeta, PaginationRequest, UserAdminCommand, UserAdminCommandResult, UserAdminQuery,
    UserAdminQueryResult,
};
use serde::{Deserialize, Serialize};

use crate::handlers::auth::{authorize_admin, current_authenticated_user};
use crate::response::{ApiResponse, ApiResponseWithMeta, Locale, ProblemDetails};
use crate::{auth_user_response, json_rejection_response, parse_user_id, AppState};

#[derive(Deserialize)]
pub(crate) struct AdminCreateUserRequest {
    email: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminUpdateUserRequest {
    email: String,
    role: String,
    premium_active: bool,
    premium_expires_at: Option<i64>,
}

#[derive(Serialize)]
pub struct AdminUserListResponse {
    pub users: Vec<crate::AuthUserResponse>,
}

#[derive(Deserialize, Default)]
pub(crate) struct AdminUserListQuery {
    page: Option<u32>,
    page_size: Option<i32>,
}

#[derive(Serialize)]
pub struct AdminUserListMeta {
    pub page: u32,
    pub page_size: i32,
    pub total: u64,
    pub total_pages: u32,
}

impl From<PaginationMeta> for AdminUserListMeta {
    fn from(meta: PaginationMeta) -> Self {
        Self {
            page: meta.page,
            page_size: meta.page_size,
            total: meta.total,
            total_pages: meta.total_pages,
        }
    }
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
    Query(query): Query<AdminUserListQuery>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    if let Err(response) = authorize_admin(&state, &jar, locale).await {
        return response;
    }

    let pagination = match (PaginationRequest {
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(20),
    })
    .normalize()
    {
        Ok(pagination) => pagination,
        Err(_) => return ProblemDetails::bad_request(locale).into_response(),
    };

    match state
        .user_queries
        .handle(UserAdminQuery::ListUsers { pagination })
        .await
    {
        Ok(UserAdminQueryResult::Users(users)) => (
            StatusCode::OK,
            Json(ApiResponseWithMeta::new(
                AdminUserListResponse {
                    users: users.items.into_iter().map(auth_user_response).collect(),
                },
                AdminUserListMeta::from(users.meta),
            )),
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

    let role = match minirust_services::AdminUserRole::parse(&body.role) {
        Ok(role) => role,
        Err(error) => return ProblemDetails::user_admin(&error, locale).into_response(),
    };

    match state
        .user_commands
        .handle(UserAdminCommand::UpdateUser {
            user_id,
            new_email: body.email,
            role,
            premium_active: body.premium_active,
            premium_expires_at: body.premium_expires_at,
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
