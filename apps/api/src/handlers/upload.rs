use axum::extract::multipart::MultipartRejection;
use axum::extract::{Multipart, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use minirust_services::cqrs::AsyncCommandHandler;
use minirust_services::{UploadCommand, MAX_UPLOAD_BYTES};
use serde::Serialize;

use super::auth::current_authenticated_user;
use crate::response::{ApiResponse, Locale, ProblemDetails};
use crate::AppState;

pub(crate) const MAX_REQUEST_BYTES: usize = MAX_UPLOAD_BYTES + 1024 * 1024;

#[derive(Serialize)]
struct UploadResponse {
    name: String,
    original_name: String,
    size: u64,
}

pub async fn upload(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    multipart: Result<Multipart, MultipartRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);

    if let Err(response) = current_authenticated_user(&state, &jar, locale).await {
        return response;
    }

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
        .uploads
        .handle(UploadCommand { file_name, content })
        .await
    {
        Ok(stored) => (
            StatusCode::CREATED,
            Json(ApiResponse::new(UploadResponse {
                name: stored.name,
                original_name: stored.original_name,
                size: stored.size,
            })),
        )
            .into_response(),
        Err(error) => ProblemDetails::upload(&error, locale).into_response(),
    }
}

#[allow(clippy::result_large_err)]
pub(crate) async fn read_file_field(
    multipart: &mut Multipart,
    locale: Locale,
) -> Result<(String, Vec<u8>), axum::response::Response> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ProblemDetails::bad_request(locale).into_response())?
    {
        if field.name() != Some("file") {
            continue;
        }

        let file_name = field.file_name().unwrap_or_default().to_owned();
        let content = field.bytes().await.map_err(|error| {
            if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
                ProblemDetails::payload_too_large(locale).into_response()
            } else {
                ProblemDetails::bad_request(locale).into_response()
            }
        })?;

        return Ok((file_name, content.to_vec()));
    }

    Err(ProblemDetails::bad_request(locale).into_response())
}
