use axum::extract::multipart::MultipartRejection;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use minirust_services::cqrs::{AsyncCommandHandler, AsyncQueryHandler};
use minirust_services::{
    LoadAvatarQuery, UploadAvatarCommand, UploadError, UserAdminCommand, UserAdminCommandResult,
    UserLocale,
};
use serde::{Deserialize, Serialize};

use crate::handlers::auth::{current_authenticated_user, SESSION_COOKIE};
use crate::handlers::upload::read_file_field;
use crate::response::{ApiResponse, Locale, ProblemDetails};
use crate::{auth_user_response, json_rejection_response, AppState};
use axum::extract::rejection::JsonRejection;

#[derive(Deserialize)]
pub(crate) struct UserProfileUpdateRequest {
    full_name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UserLanguageUpdateRequest {
    locale: String,
}

#[derive(Serialize)]
pub(crate) struct AccountActionResponse {
    success: bool,
}

#[derive(Serialize)]
pub(crate) struct AvatarResponse {
    avatar_url: String,
}

pub async fn upload_avatar(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    multipart: Result<Multipart, MultipartRejection>,
) -> impl IntoResponse {
    let request_locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, request_locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let locale = Locale::from_user_locale(user.locale);

    let mut multipart = match multipart {
        Ok(multipart) => multipart,
        Err(error) if error.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            return ProblemDetails::payload_too_large(locale).into_response()
        }
        Err(_) => return ProblemDetails::bad_request(locale).into_response(),
    };

    let (file_name, content) = match read_file_field(&mut multipart, locale).await {
        Ok(field) => field,
        Err(response) => return response,
    };

    match state
        .avatars
        .handle(UploadAvatarCommand {
            directory: user.id.as_uuid().to_string(),
            file_name,
            content,
        })
        .await
    {
        Ok(stored) => (
            StatusCode::OK,
            Json(ApiResponse::new(AvatarResponse {
                avatar_url: avatar_url(&stored.extension),
            })),
        )
            .into_response(),
        Err(error) => ProblemDetails::upload(&error, locale).into_response(),
    }
}

pub async fn get_avatar(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(extension): Path<String>,
) -> impl IntoResponse {
    let request_locale = Locale::from_accept_language(&headers);
    let user = match current_authenticated_user(&state, &jar, request_locale).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    match state
        .avatar_queries
        .handle(LoadAvatarQuery {
            directory: user.id.as_uuid().to_string(),
            extension: extension.clone(),
        })
        .await
    {
        Ok(Some(content)) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, avatar_content_type(&extension)),
                (header::CACHE_CONTROL, "private, max-age=86400"),
            ],
            content,
        )
            .into_response(),
        Ok(None)
        | Err(UploadError::AvatarExtensionUnsupported)
        | Err(UploadError::AvatarDirectoryInvalid) => {
            ProblemDetails::not_found(request_locale).into_response()
        }
        Err(_) => ProblemDetails::internal(request_locale).into_response(),
    }
}

fn avatar_url(extension: &str) -> String {
    let version = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    format!("/api/v1/users/me/avatar/{extension}?v={version}")
}

fn avatar_content_type(extension: &str) -> &'static str {
    match extension.trim().to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/jpeg",
    }
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
        .handle(UserAdminCommand::LockOwnAccount { user_id: user.id })
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
