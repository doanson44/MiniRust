//! MiniRust REST API application.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod email;
mod handlers;
mod response;

use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;

use minirust_core::{AppError, EntityId};
use minirust_database::Database;
use minirust_services::{
    AuthCommandHandler, AuthError, AuthQueryHandler, AuthService, MenuCommandHandler,
    MenuQueryHandler, MenuService, UserAdminCommandHandler, UserAdminQueryHandler,
    UserAdminService,
};
use minirust_services::{EchoCommandHandler, GreetingQueryHandler};
use response::{Locale, ProblemDetails};
use serde::Serialize;
use uuid::Uuid;

use email::SmtpEmailSender;
pub use email::SmtpEmailSender as PublicSmtpEmailSender;

pub use handlers::router;

#[derive(Clone)]
pub struct AppState {
    pub echo: EchoCommandHandler,
    pub greeting: GreetingQueryHandler,
    pub database: Database,
    pub auth_commands: AuthCommandHandler<Database, SmtpEmailSender>,
    pub auth_queries: AuthQueryHandler<Database, SmtpEmailSender>,
    pub user_commands: UserAdminCommandHandler<Database>,
    pub menu_commands: MenuCommandHandler<Database>,
    pub user_queries: UserAdminQueryHandler<Database>,
    pub menu_queries: MenuQueryHandler<Database>,
    pub auth_rate_limiter: AuthRateLimiter,
    pub secure_cookies: bool,
    pub email_enabled: bool,
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
        let email_sender = SmtpEmailSender::disabled();
        Self::with_email_sender(database, auth_secret, secure_cookies, email_sender)
    }

    pub fn with_email_sender(
        database: Database,
        auth_secret: impl Into<Vec<u8>>,
        secure_cookies: bool,
        email_sender: SmtpEmailSender,
    ) -> Result<Self, AuthError> {
        let email_enabled = email_sender.is_enabled();
        let auth = AuthService::new(database.clone(), email_sender, auth_secret)?;
        let users = UserAdminService::new(database.clone());
        let menus = MenuService::new(database.clone());
        Ok(Self {
            echo: EchoCommandHandler,
            greeting: GreetingQueryHandler,
            database,
            auth_commands: AuthCommandHandler::new(auth.clone()),
            auth_queries: AuthQueryHandler::new(auth),
            user_commands: UserAdminCommandHandler::new(users.clone()),
            menu_commands: MenuCommandHandler::new(menus.clone()),
            user_queries: UserAdminQueryHandler::new(users),
            menu_queries: MenuQueryHandler::new(menus),
            auth_rate_limiter: AuthRateLimiter::default(),
            secure_cookies,
            email_enabled,
        })
    }
}

// ---------------------------------------------------------------------------
// Shared response helpers (used by multiple handler modules)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub(crate) struct AuthUserResponse {
    pub id: String,
    pub email: String,
    pub is_admin: bool,
    pub is_premium: bool,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_locked: bool,
    pub locale: &'static str,
}

#[derive(Serialize)]
pub(crate) struct AuthSessionResponse {
    pub user: AuthUserResponse,
    pub expires_at: i64,
}

pub(crate) fn auth_user_response(user: minirust_services::UserAccess) -> AuthUserResponse {
    AuthUserResponse {
        id: user.id.as_uuid().to_string(),
        email: user.email,
        is_admin: user.is_admin,
        is_premium: user.is_premium,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        is_locked: user.is_locked,
        locale: user.locale.as_str(),
    }
}

pub(crate) fn auth_error_response(error: AuthError, locale: Locale) -> ProblemDetails {
    tracing::warn!(error = %error, "authentication request failed");
    ProblemDetails::auth(&error, locale)
}

pub(crate) fn app_error_response(error: AppError, locale: Locale) -> ProblemDetails {
    match error {
        AppError::Validation(error) => ProblemDetails::validation(&error, locale),
        other => {
            tracing::error!(%other, "unexpected application error");
            ProblemDetails::internal(locale)
        }
    }
}

pub(crate) fn json_rejection_response(rejection: JsonRejection, locale: Locale) -> ProblemDetails {
    if rejection.status() == StatusCode::BAD_REQUEST {
        ProblemDetails::bad_request(locale)
    } else {
        tracing::error!(%rejection, "request body extraction failed");
        ProblemDetails::internal(locale)
    }
}

pub(crate) fn parse_user_id(value: &str, locale: Locale) -> Result<EntityId, ProblemDetails> {
    let uuid = match Uuid::parse_str(value) {
        Ok(uuid) => uuid,
        Err(_) => return Err(ProblemDetails::bad_request(locale)),
    };

    EntityId::from_uuid(uuid).ok_or_else(|| ProblemDetails::bad_request(locale))
}
