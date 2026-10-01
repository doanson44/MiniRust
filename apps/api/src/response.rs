use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use minirust_core::ValidationError;
use minirust_services::{AuthError, MenuError, UserAdminError, UserLocale};
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
        let detail = match (error, locale) {
            (ValidationError::MessageRequired, Locale::Vi) => {
                "Message không được để trống.".to_owned()
            }
            (ValidationError::MessageRequired, Locale::En) => {
                "Message must not be empty.".to_owned()
            }
            (ValidationError::MessageTooLong { max }, Locale::Vi) => {
                format!("Message không được vượt quá {max} ký tự.")
            }
            (ValidationError::MessageTooLong { max }, Locale::En) => {
                format!("Message must not exceed {max} characters.")
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
        Self {
            problem_type: "https://minirust.dev/problems/bad-request",
            title: "Bad request",
            status: StatusCode::BAD_REQUEST.as_u16(),
            code: "BAD_REQUEST",
            message_key: "errors.request.bad_request",
            locale: locale.as_str(),
            detail: match locale {
                Locale::Vi => "Yêu cầu không hợp lệ.".to_owned(),
                Locale::En => "The request is invalid.".to_owned(),
            },
        }
    }

    pub fn internal(locale: Locale) -> Self {
        Self {
            problem_type: "https://minirust.dev/problems/internal-error",
            title: "Internal server error",
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            code: "INTERNAL_ERROR",
            message_key: "errors.internal.unexpected",
            locale: locale.as_str(),
            detail: match locale {
                Locale::Vi => "Đã xảy ra lỗi không mong muốn.".to_owned(),
                Locale::En => "An unexpected error occurred.".to_owned(),
            },
        }
    }

    pub fn auth(error: &AuthError, locale: Locale) -> Self {
        let (status, code, message_key, detail) = match error {
            AuthError::InvalidEmail => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_EMAIL",
                "errors.auth.invalid_email",
                match locale {
                    Locale::Vi => "Email không hợp lệ.".to_owned(),
                    Locale::En => "The email address is invalid.".to_owned(),
                },
            ),
            AuthError::InvalidCode => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_VERIFICATION_CODE",
                "errors.auth.invalid_code",
                match locale {
                    Locale::Vi => "Mã xác thực không hợp lệ.".to_owned(),
                    Locale::En => "The verification code is invalid.".to_owned(),
                },
            ),
            AuthError::InvalidVerificationToken => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_VERIFICATION_TOKEN",
                "errors.auth.invalid_verification_token",
                match locale {
                    Locale::Vi => "Liên kết xác thực không hợp lệ.".to_owned(),
                    Locale::En => "The verification link is invalid.".to_owned(),
                },
            ),
            AuthError::CodeExpired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VERIFICATION_CODE_EXPIRED",
                "errors.auth.code_expired",
                match locale {
                    Locale::Vi => "Mã xác thực đã hết hạn.".to_owned(),
                    Locale::En => "The verification code has expired.".to_owned(),
                },
            ),
            AuthError::VerificationTokenExpired => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VERIFICATION_TOKEN_EXPIRED",
                "errors.auth.verification_token_expired",
                match locale {
                    Locale::Vi => "Liên kết xác thực đã hết hạn.".to_owned(),
                    Locale::En => "The verification link has expired.".to_owned(),
                },
            ),
            AuthError::CodeAttemptsExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "VERIFICATION_ATTEMPTS_EXCEEDED",
                "errors.auth.attempts_exceeded",
                match locale {
                    Locale::Vi => "Đã vượt quá số lần nhập mã cho phép.".to_owned(),
                    Locale::En => {
                        "The maximum number of verification attempts was exceeded.".to_owned()
                    }
                },
            ),
            AuthError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "errors.authorization.forbidden",
                match locale {
                    Locale::Vi => "Bạn không có quyền thực hiện thao tác này.".to_owned(),
                    Locale::En => "You are not authorized to perform this operation.".to_owned(),
                },
            ),
            AuthError::AccountLocked => (
                StatusCode::FORBIDDEN,
                "ACCOUNT_LOCKED",
                "errors.auth.account_locked",
                match locale {
                    Locale::Vi => "Tài khoản đã bị khóa.".to_owned(),
                    Locale::En => "The account is locked.".to_owned(),
                },
            ),
            AuthError::SessionInvalid => (
                StatusCode::UNAUTHORIZED,
                "SESSION_INVALID",
                "errors.auth.session_invalid",
                match locale {
                    Locale::Vi => "Phiên đăng nhập không hợp lệ hoặc đã hết hạn.".to_owned(),
                    Locale::En => "The session is invalid or has expired.".to_owned(),
                },
            ),
            AuthError::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "EMAIL_ALREADY_EXISTS",
                "errors.auth.email_already_exists",
                match locale {
                    Locale::Vi => "Email đã được sử dụng.".to_owned(),
                    Locale::En => "The email address is already in use.".to_owned(),
                },
            ),
            AuthError::BootstrapAdminConflict => (
                StatusCode::CONFLICT,
                "BOOTSTRAP_ADMIN_CONFLICT",
                "errors.auth.bootstrap_admin_conflict",
                match locale {
                    Locale::Vi => {
                        "Email bootstrap admin đang trỏ tới một tài khoản thường.".to_owned()
                    }
                    Locale::En => {
                        "The bootstrap admin email points to an existing non-bootstrap account."
                            .to_owned()
                    }
                },
            ),
            AuthError::EmailDeliveryUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "EMAIL_DELIVERY_UNAVAILABLE",
                "errors.auth.email_delivery_unavailable",
                match locale {
                    Locale::Vi => "Dịch vụ email hiện chưa khả dụng.".to_owned(),
                    Locale::En => "Email delivery is currently unavailable.".to_owned(),
                },
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
            message_key,
            locale: locale.as_str(),
            detail,
        }
    }

    pub fn user_admin(error: &UserAdminError, locale: Locale) -> Self {
        let (status, code, message_key, detail) = match error {
            UserAdminError::InvalidEmail => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_EMAIL",
                "errors.user.invalid_email",
                match locale {
                    Locale::Vi => "Email không hợp lệ.".to_owned(),
                    Locale::En => "The email address is invalid.".to_owned(),
                },
            ),
            UserAdminError::InvalidFullName => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_FULL_NAME",
                "errors.user.invalid_full_name",
                match locale {
                    Locale::Vi => "Họ tên không hợp lệ.".to_owned(),
                    Locale::En => "The full name is invalid.".to_owned(),
                },
            ),
            UserAdminError::InvalidLocale => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_LOCALE",
                "errors.user.invalid_locale",
                match locale {
                    Locale::Vi => "Ngôn ngữ không được hỗ trợ.".to_owned(),
                    Locale::En => "The language is not supported.".to_owned(),
                },
            ),
            UserAdminError::InvalidAvatarUrl => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_AVATAR_URL",
                "errors.user.invalid_avatar_url",
                match locale {
                    Locale::Vi => "Avatar URL không hợp lệ.".to_owned(),
                    Locale::En => "The avatar URL is invalid.".to_owned(),
                },
            ),
            UserAdminError::InvalidPremiumExpiry => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_PREMIUM_EXPIRY",
                "errors.user.invalid_premium_expiry",
                match locale {
                    Locale::Vi => "Thời hạn Premium phải nằm trong tương lai.".to_owned(),
                    Locale::En => "The Premium expiry must be in the future.".to_owned(),
                },
            ),
            UserAdminError::InvalidRole => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_ROLE",
                "errors.user.invalid_role",
                match locale {
                    Locale::Vi => "Role không hợp lệ. Chỉ hỗ trợ admin hoặc none.".to_owned(),
                    Locale::En => {
                        "The role is invalid. Only admin or none is supported.".to_owned()
                    }
                },
            ),
            UserAdminError::NotFound => (
                StatusCode::NOT_FOUND,
                "USER_NOT_FOUND",
                "errors.user.not_found",
                match locale {
                    Locale::Vi => "Không tìm thấy người dùng.".to_owned(),
                    Locale::En => "User was not found.".to_owned(),
                },
            ),
            UserAdminError::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "EMAIL_ALREADY_EXISTS",
                "errors.user.email_already_exists",
                match locale {
                    Locale::Vi => "Email đã được sử dụng.".to_owned(),
                    Locale::En => "The email address is already in use.".to_owned(),
                },
            ),
            UserAdminError::ProtectedUser => (
                StatusCode::CONFLICT,
                "PROTECTED_USER",
                "errors.user.protected",
                match locale {
                    Locale::Vi => "Tài khoản hệ thống này được bảo vệ.".to_owned(),
                    Locale::En => "This system account is protected.".to_owned(),
                },
            ),
            UserAdminError::Persistence => return Self::internal(locale),
        };

        Self {
            problem_type: "https://minirust.dev/problems/user-management",
            title: "User management error",
            status: status.as_u16(),
            code,
            message_key,
            locale: locale.as_str(),
            detail,
        }
    }

    pub fn menu(error: &MenuError, locale: Locale) -> Self {
        let (status, code, message_key, detail) = match error {
            MenuError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "errors.authorization.forbidden",
                match locale {
                    Locale::Vi => "Bạn không có quyền quản lý menu.".to_owned(),
                    Locale::En => "You are not authorized to manage menus.".to_owned(),
                },
            ),
            MenuError::NotFound => (
                StatusCode::NOT_FOUND,
                "MENU_NOT_FOUND",
                "errors.menu.not_found",
                match locale {
                    Locale::Vi => "Không tìm thấy menu.".to_owned(),
                    Locale::En => "Menu was not found.".to_owned(),
                },
            ),
            MenuError::InvalidName => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MENU_NAME",
                "errors.menu.invalid_name",
                match locale {
                    Locale::Vi => "Tên menu không hợp lệ.".to_owned(),
                    Locale::En => "The menu name is invalid.".to_owned(),
                },
            ),
            MenuError::InvalidPath => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MENU_PATH",
                "errors.menu.invalid_path",
                match locale {
                    Locale::Vi => "Đường dẫn menu không hợp lệ.".to_owned(),
                    Locale::En => "The menu path is invalid.".to_owned(),
                },
            ),
            MenuError::InvalidIcon => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MENU_ICON",
                "errors.menu.invalid_icon",
                match locale {
                    Locale::Vi => "Icon menu không hợp lệ.".to_owned(),
                    Locale::En => "The menu icon is invalid.".to_owned(),
                },
            ),

            MenuError::InvalidParent => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MENU_PARENT",
                "errors.menu.invalid_parent",
                match locale {
                    Locale::Vi => "Menu cha không hợp lệ.".to_owned(),
                    Locale::En => "The menu parent is invalid.".to_owned(),
                },
            ),
            MenuError::Persistence => return Self::internal(locale),
        };

        Self {
            problem_type: "https://minirust.dev/problems/menu-management",
            title: "Menu management error",
            status: status.as_u16(),
            code,
            message_key,
            locale: locale.as_str(),
            detail,
        }
    }

    pub fn rate_limited(locale: Locale) -> Self {
        Self {
            problem_type: "https://minirust.dev/problems/rate-limit",
            title: "Too many requests",
            status: StatusCode::TOO_MANY_REQUESTS.as_u16(),
            code: "RATE_LIMITED",
            message_key: "errors.rate_limit.exceeded",
            locale: locale.as_str(),
            detail: match locale {
                Locale::Vi => "Quá nhiều yêu cầu. Vui lòng thử lại sau.".to_owned(),
                Locale::En => "Too many requests. Please try again later.".to_owned(),
            },
        }
    }

    pub fn service_unavailable(locale: Locale) -> Self {
        Self {
            problem_type: "https://minirust.dev/problems/service-unavailable",
            title: "Service unavailable",
            status: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
            code: "DEPENDENCY_UNAVAILABLE",
            message_key: "errors.dependency.unavailable",
            locale: locale.as_str(),
            detail: match locale {
                Locale::Vi => "Một dịch vụ phụ thuộc hiện không khả dụng.".to_owned(),
                Locale::En => "A required dependency is currently unavailable.".to_owned(),
            },
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
