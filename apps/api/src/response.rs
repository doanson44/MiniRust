use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use minirust_core::ValidationError;
use minirust_locales::{self, Key};
use minirust_services::{AuthError, MenuError, UploadError, UserAdminError, UserLocale};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Vi,
    En,
}

impl Locale {
    pub const DEFAULT: Self = Self::Vi;

    pub fn from_accept_language(headers: &HeaderMap) -> Self {
        let value = headers
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();

        for language in value.split(',').map(str::trim) {
            let language = language
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();

            if language == "vi" || language.starts_with("vi-") {
                return Self::Vi;
            }

            if language == "en" || language.starts_with("en-") {
                return Self::En;
            }
        }

        Self::DEFAULT
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vi => "vi",
            Self::En => "en",
        }
    }

    pub const fn from_user_locale(locale: UserLocale) -> Self {
        match locale {
            UserLocale::Vi => Self::Vi,
            UserLocale::En => Self::En,
        }
    }
}

impl From<Locale> for minirust_locales::Locale {
    fn from(locale: Locale) -> Self {
        match locale {
            Locale::Vi => minirust_locales::Locale::Vi,
            Locale::En => minirust_locales::Locale::En,
        }
    }
}

fn validation_key(error: &ValidationError) -> Key {
    match error {
        ValidationError::MessageRequired => Key::ErrorsValidationMessageRequired,
        ValidationError::MessageTooLong { .. } => Key::ErrorsValidationMessageTooLong,
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponseWithMeta<T, M> {
    pub data: T,
    pub meta: M,
}

impl<T, M> ApiResponseWithMeta<T, M> {
    pub fn new(data: T, meta: M) -> Self {
        Self { data, meta }
    }
}

#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub code: &'static str,
    pub message_key: &'static str,
    pub locale: &'static str,
    pub detail: String,
}

impl ProblemDetails {
    pub fn validation(error: &ValidationError, locale: Locale) -> Self {
        let key = validation_key(error);
        let detail = match error {
            ValidationError::MessageTooLong { max } => {
                minirust_locales::text(locale.into(), key).replace("{max}", &max.to_string())
            }
            ValidationError::MessageRequired => {
                minirust_locales::text(locale.into(), key).to_owned()
            }
        };

        Self {
            problem_type: "https://minirust.dev/problems/validation-error",
            title: "Validation error",
            status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
            code: error.code(),
            message_key: error.message_key(),
            locale: locale.as_str(),
            detail,
        }
    }

    pub fn bad_request(locale: Locale) -> Self {
        let key = Key::ErrorsRequestBadRequest;

        Self {
            problem_type: "https://minirust.dev/problems/bad-request",
            title: "Bad request",
            status: StatusCode::BAD_REQUEST.as_u16(),
            code: "BAD_REQUEST",
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
        }
    }

    pub fn payload_too_large(locale: Locale) -> Self {
        let key = Key::ErrorsRequestPayloadTooLarge;

        Self {
            problem_type: "https://minirust.dev/problems/payload-too-large",
            title: "Payload too large",
            status: StatusCode::PAYLOAD_TOO_LARGE.as_u16(),
            code: "PAYLOAD_TOO_LARGE",
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
        }
    }

    pub fn not_found(locale: Locale) -> Self {
        let key = Key::ErrorsRequestNotFound;

        Self {
            problem_type: "https://minirust.dev/problems/not-found",
            title: "Not found",
            status: StatusCode::NOT_FOUND.as_u16(),
            code: "NOT_FOUND",
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
        }
    }

    pub fn internal(locale: Locale) -> Self {
        let key = Key::ErrorsInternalUnexpected;

        Self {
            problem_type: "https://minirust.dev/problems/internal-error",
            title: "Internal server error",
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            code: "INTERNAL_ERROR",
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
        }
    }

    pub fn auth(error: &AuthError, locale: Locale) -> Self {
        let (status, code, key) = match error {
            AuthError::InvalidEmail => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_EMAIL",
                Key::ErrorsAuthInvalidEmail,
            ),
            AuthError::InvalidCode => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_VERIFICATION_CODE",
                Key::ErrorsAuthInvalidCode,
            ),
            AuthError::InvalidVerificationToken => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_VERIFICATION_TOKEN",
                Key::ErrorsAuthInvalidVerificationToken,
            ),
            AuthError::CodeExpired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VERIFICATION_CODE_EXPIRED",
                Key::ErrorsAuthCodeExpired,
            ),
            AuthError::VerificationTokenExpired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VERIFICATION_TOKEN_EXPIRED",
                Key::ErrorsAuthVerificationTokenExpired,
            ),
            AuthError::CodeAttemptsExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "VERIFICATION_ATTEMPTS_EXCEEDED",
                Key::ErrorsAuthAttemptsExceeded,
            ),
            AuthError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                Key::ErrorsAuthorizationForbidden,
            ),
            AuthError::AccountLocked => (
                StatusCode::FORBIDDEN,
                "ACCOUNT_LOCKED",
                Key::ErrorsAuthAccountLocked,
            ),
            AuthError::SessionInvalid => (
                StatusCode::UNAUTHORIZED,
                "SESSION_INVALID",
                Key::ErrorsAuthSessionInvalid,
            ),
            AuthError::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "EMAIL_ALREADY_EXISTS",
                Key::ErrorsAuthEmailAlreadyExists,
            ),
            AuthError::BootstrapAdminConflict => (
                StatusCode::CONFLICT,
                "BOOTSTRAP_ADMIN_CONFLICT",
                Key::ErrorsAuthBootstrapAdminConflict,
            ),
            AuthError::EmailDeliveryUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "EMAIL_DELIVERY_UNAVAILABLE",
                Key::ErrorsAuthEmailDeliveryUnavailable,
            ),
            AuthError::Persistence | AuthError::InvalidSecret | AuthError::Randomness => {
                return Self::internal(locale);
            }
        };

        Self {
            problem_type: "https://minirust.dev/problems/authentication",
            title: "Authentication error",
            status: status.as_u16(),
            code,
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
        }
    }

    pub fn user_admin(error: &UserAdminError, locale: Locale) -> Self {
        let (status, code, key) = match error {
            UserAdminError::InvalidEmail => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_EMAIL",
                Key::ErrorsUserInvalidEmail,
            ),
            UserAdminError::InvalidFullName => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_FULL_NAME",
                Key::ErrorsUserInvalidFullName,
            ),
            UserAdminError::InvalidLocale => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_LOCALE",
                Key::ErrorsUserInvalidLocale,
            ),
            UserAdminError::InvalidAvatarUrl => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_AVATAR_URL",
                Key::ErrorsUserInvalidAvatarUrl,
            ),
            UserAdminError::InvalidPremiumExpiry => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_PREMIUM_EXPIRY",
                Key::ErrorsUserInvalidPremiumExpiry,
            ),
            UserAdminError::InvalidRole => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_ROLE",
                Key::ErrorsUserInvalidRole,
            ),
            UserAdminError::NotFound => (
                StatusCode::NOT_FOUND,
                "USER_NOT_FOUND",
                Key::ErrorsUserNotFound,
            ),
            UserAdminError::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "EMAIL_ALREADY_EXISTS",
                Key::ErrorsUserEmailAlreadyExists,
            ),
            UserAdminError::ProtectedUser => (
                StatusCode::CONFLICT,
                "PROTECTED_USER",
                Key::ErrorsUserProtected,
            ),
            UserAdminError::CannotChangeOwnRole => (
                StatusCode::CONFLICT,
                "CANNOT_CHANGE_OWN_ROLE",
                Key::ErrorsUserCannotChangeOwnRole,
            ),
            UserAdminError::Persistence => return Self::internal(locale),
        };

        Self {
            problem_type: "https://minirust.dev/problems/user-management",
            title: "User management error",
            status: status.as_u16(),
            code,
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
        }
    }

    pub fn menu(error: &MenuError, locale: Locale) -> Self {
        let (status, code, key) = match error {
            MenuError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                Key::ErrorsAuthorizationForbidden,
            ),
            MenuError::NotFound => (
                StatusCode::NOT_FOUND,
                "MENU_NOT_FOUND",
                Key::ErrorsMenuNotFound,
            ),
            MenuError::InvalidName => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MENU_NAME",
                Key::ErrorsMenuInvalidName,
            ),
            MenuError::InvalidPath => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MENU_PATH",
                Key::ErrorsMenuInvalidPath,
            ),
            MenuError::InvalidIcon => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MENU_ICON",
                Key::ErrorsMenuInvalidIcon,
            ),
            MenuError::InvalidParent => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MENU_PARENT",
                Key::ErrorsMenuInvalidParent,
            ),
            MenuError::Persistence => return Self::internal(locale),
        };

        Self {
            problem_type: "https://minirust.dev/problems/menu-management",
            title: "Menu management error",
            status: status.as_u16(),
            code,
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
        }
    }

    pub fn upload(error: &UploadError, locale: Locale) -> Self {
        let (status, code, key) = match error {
            UploadError::FileNameRequired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UPLOAD_FILE_NAME_REQUIRED",
                Key::ErrorsUploadFileNameRequired,
            ),
            UploadError::FileTooLarge { .. } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UPLOAD_FILE_TOO_LARGE",
                Key::ErrorsUploadFileTooLarge,
            ),
            UploadError::AvatarDirectoryInvalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "AVATAR_DIRECTORY_INVALID",
                Key::ErrorsUploadAvatarDirectoryInvalid,
            ),
            UploadError::AvatarExtensionUnsupported => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UNSUPPORTED_AVATAR_FORMAT",
                Key::ErrorsUploadUnsupportedAvatarFormat,
            ),
            UploadError::Storage => return Self::internal(locale),
        };

        let detail = match error {
            UploadError::FileTooLarge { max } => {
                minirust_locales::text(locale.into(), key).replace("{max}", &max.to_string())
            }
            _ => minirust_locales::text(locale.into(), key).to_owned(),
        };

        Self {
            problem_type: "https://minirust.dev/problems/upload",
            title: "Upload error",
            status: status.as_u16(),
            code,
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail,
        }
    }

    pub fn rate_limited(locale: Locale) -> Self {
        let key = Key::ErrorsRateLimitExceeded;

        Self {
            problem_type: "https://minirust.dev/problems/rate-limit",
            title: "Too many requests",
            status: StatusCode::TOO_MANY_REQUESTS.as_u16(),
            code: "RATE_LIMITED",
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
        }
    }

    pub fn service_unavailable(locale: Locale) -> Self {
        let key = Key::ErrorsDependencyUnavailable;

        Self {
            problem_type: "https://minirust.dev/problems/service-unavailable",
            title: "Service unavailable",
            status: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
            code: "DEPENDENCY_UNAVAILABLE",
            message_key: key.as_str(),
            locale: locale.as_str(),
            detail: minirust_locales::text(locale.into(), key).to_owned(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_prefers_supported_language() {
        let headers = HeaderMap::from_iter([(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-US,en;q=0.9,vi;q=0.8"),
        )]);

        assert_eq!(Locale::from_accept_language(&headers), Locale::En);
    }

    #[test]
    fn locale_falls_back_to_vietnamese() {
        let headers =
            HeaderMap::from_iter([(header::ACCEPT_LANGUAGE, HeaderValue::from_static("fr-FR"))]);

        assert_eq!(Locale::from_accept_language(&headers), Locale::Vi);
    }

    #[test]
    fn validation_message_is_localized() {
        let error = ValidationError::MessageRequired;
        let problem = ProblemDetails::validation(&error, Locale::Vi);

        assert_eq!(problem.code, "MESSAGE_REQUIRED");
        assert_eq!(problem.message_key, "errors.validation.message_required");
        assert_eq!(problem.locale, "vi");
        assert_eq!(problem.detail, "Message không được để trống.");
    }
}
