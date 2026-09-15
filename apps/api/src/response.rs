use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Standard success envelope for JSON API responses.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

/// RFC 9457 problem details returned by the API for failures.
#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub detail: String,
}

impl ProblemDetails {
    pub fn validation(detail: impl Into<String>) -> Self {
        Self {
            problem_type: "https://minirust.dev/problems/validation-error",
            title: "Validation error",
            status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
            detail: detail.into(),
        }
    }

    pub fn internal() -> Self {
        Self {
            problem_type: "about:blank",
            title: "Internal Server Error",
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            detail: "An unexpected error occurred".to_owned(),
        }
    }

    pub fn service_unavailable(detail: impl Into<String>) -> Self {
        Self {
            problem_type: "about:blank",
            title: "Service Unavailable",
            status: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
            detail: detail.into(),
        }
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (status, Json(self)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}
