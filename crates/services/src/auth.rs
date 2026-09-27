//! Authentication application services and contracts.
//!
//! Transport and persistence adapters implement the traits in this module.
//! The application layer owns the passwordless email-code workflow.

use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use getrandom::fill;
use hmac::{Hmac, KeyInit, Mac};
use minirust_core::EntityId;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const OTP_DIGITS: u32 = 1_000_000;
const OTP_MAX_ATTEMPTS: u8 = 5;
const OTP_TTL_SECONDS: i64 = 10 * 60;
const SESSION_TTL_SECONDS: i64 = 30 * 24 * 60 * 60;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengePurpose {
    Registration,
    Login,
}

impl ChallengePurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Registration => "registration",
            Self::Login => "login",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAccess {
    pub id: EntityId,
    pub email: String,
    pub is_admin: bool,
    pub is_premium: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChallengeRef {
    pub id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub user: UserAccess,
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeRequestAccepted;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    InvalidEmail,
    InvalidCode,
    CodeExpired,
    CodeAttemptsExceeded,
    SessionInvalid,
    EmailDeliveryUnavailable,
    Persistence,
    InvalidSecret,
    Randomness,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEmail => f.write_str("invalid email"),
            Self::InvalidCode => f.write_str("invalid verification code"),
            Self::CodeExpired => f.write_str("verification code expired"),
            Self::CodeAttemptsExceeded => f.write_str("verification attempts exceeded"),
            Self::SessionInvalid => f.write_str("invalid session"),
            Self::EmailDeliveryUnavailable => f.write_str("email delivery unavailable"),
            Self::Persistence => f.write_str("authentication persistence failed"),
            Self::InvalidSecret => f.write_str("authentication secret is invalid"),
            Self::Randomness => f.write_str("secure randomness is unavailable"),
        }
    }
}

impl std::error::Error for AuthError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Challenge {
    pub id: EntityId,
    pub expires_at: i64,
    pub max_attempts: u8,
}

pub trait AuthRepository: Clone + Send + Sync + 'static {
    async fn user_exists(&self, email: &str) -> Result<bool, AuthError>;
    async fn is_bootstrap_admin(&self, email: &str) -> Result<bool, AuthError>;

    async fn create_challenge(
        &self,
        challenge: Challenge,
        email: &str,
        purpose: ChallengePurpose,
        code_hash: [u8; 32],
        created_at: i64,
    ) -> Result<(), AuthError>;

    async fn latest_challenge(
        &self,
        email: &str,
        purpose: ChallengePurpose,
        now: i64,
    ) -> Result<Option<ChallengeRef>, AuthError>;

    async fn consume_registration_code(
        &self,
        challenge_id: EntityId,
        email: &str,
        code_hash: [u8; 32],
        user_id: EntityId,
        now: i64,
        session_token_hash: [u8; 32],
        session_expires_at: i64,
    ) -> Result<UserAccess, AuthError>;

    async fn consume_login_code(
        &self,
        challenge_id: EntityId,
        email: &str,
        code_hash: [u8; 32],
        now: i64,
        session_token_hash: [u8; 32],
        session_expires_at: i64,
    ) -> Result<UserAccess, AuthError>;

    async fn find_session(
        &self,
        session_token_hash: [u8; 32],
        now: i64,
    ) -> Result<Option<UserAccess>, AuthError>;

    async fn revoke_session(&self, session_token_hash: [u8; 32]) -> Result<(), AuthError>;
}

pub trait EmailSender: Clone + Send + Sync + 'static {
    async fn send_verification_code(
        &self,
        email: &str,
        purpose: ChallengePurpose,
        code: &str,
    ) -> Result<(), AuthError>;
}

/// Adapter used until a concrete email provider is configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnavailableEmailSender;

impl EmailSender for UnavailableEmailSender {
    async fn send_verification_code(
        &self,
        _email: &str,
        _purpose: ChallengePurpose,
        _code: &str,
    ) -> Result<(), AuthError> {
        Err(AuthError::EmailDeliveryUnavailable)
    }
}

#[derive(Clone)]
pub struct AuthService<R, E> {
    repository: R,
    email_sender: E,
    secret: Vec<u8>,
}

impl<R, E> AuthService<R, E>
where
    R: AuthRepository,
    E: EmailSender,
{
    pub fn new(repository: R, email_sender: E, secret: impl Into<Vec<u8>>) -> Result<Self, AuthError> {
        let secret = secret.into();
        if secret.len() < 32 {
            return Err(AuthError::InvalidSecret);
        }

        Ok(Self {
            repository,
            email_sender,
            secret,
        })
    }

    pub async fn request_registration_code(
        &self,
        email: &str,
    ) -> Result<CodeRequestAccepted, AuthError> {
        self.request_code(email, ChallengePurpose::Registration).await
    }

    pub async fn request_login_code(&self, email: &str) -> Result<CodeRequestAccepted, AuthError> {
        self.request_code(email, ChallengePurpose::Login).await
    }

    pub async fn verify_registration_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<Session, AuthError> {
        self.verify_code(email, code, ChallengePurpose::Registration)
            .await
    }

    pub async fn verify_login_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<Session, AuthError> {
        self.verify_code(email, code, ChallengePurpose::Login).await
    }

    pub async fn current_session(&self, token: &str) -> Result<UserAccess, AuthError> {
        let token_hash = hash_session_token(token);
        self.repository
            .find_session(token_hash, now()?)
            .await?
            .ok_or(AuthError::SessionInvalid)
    }

    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        self.repository
            .revoke_session(hash_session_token(token))
            .await
    }

    async fn request_code(
        &self,
        email: &str,
        purpose: ChallengePurpose,
    ) -> Result<CodeRequestAccepted, AuthError> {
        let email = normalize_email(email)?;

        let exists = self.repository.user_exists(&email).await?;
        let should_send = match purpose {
            ChallengePurpose::Registration => !exists,
            ChallengePurpose::Login => exists,
        };

        if should_send
            && purpose == ChallengePurpose::Login
            && self.repository.is_bootstrap_admin(&email).await?
        {
            return Ok(CodeRequestAccepted);
        }

        // Both existing and non-existing accounts receive the same public result.
        if !should_send {
            return Ok(CodeRequestAccepted);
        }

        let now = now()?;
        let challenge = Challenge {
            id: EntityId::new(),
            expires_at: now + OTP_TTL_SECONDS,
            max_attempts: OTP_MAX_ATTEMPTS,
        };
        let code = generate_otp()?;
        let code_hash = hash_otp(
            &self.secret,
            challenge.id,
            &email,
            purpose,
            &code,
        )?;

        self.repository
            .create_challenge(
                challenge,
                &email,
                purpose,
                code_hash,
                now,
            )
            .await?;

        self.email_sender
            .send_verification_code(&email, purpose, &code)
            .await?;

        Ok(CodeRequestAccepted)
    }

    async fn verify_code(
        &self,
        email: &str,
        code: &str,
        purpose: ChallengePurpose,
    ) -> Result<Session, AuthError> {
        let email = normalize_email(email)?;
        validate_otp(code)?;

        let now = now()?;
        let challenge = self
            .repository
            .latest_challenge(&email, purpose, now)
            .await?
            .ok_or(AuthError::InvalidCode)?;

        let code_hash = hash_otp(
            &self.secret,
            challenge.id,
            &email,
            purpose,
            code,
        )?;
        let token = generate_session_token()?;
        let token_hash = hash_session_token(&token);
        let expires_at = now + SESSION_TTL_SECONDS;

        let user = match purpose {
            ChallengePurpose::Registration => {
                self.repository
                    .consume_registration_code(
                        challenge.id,
                        &email,
                        code_hash,
                        EntityId::new(),
                        now,
                        token_hash,
                        expires_at,
                    )
                    .await?
            }
            ChallengePurpose::Login => {
                self.repository
                    .consume_login_code(
                        challenge.id,
                        &email,
                        code_hash,
                        now,
                        token_hash,
                        expires_at,
                    )
                    .await?
            }
        };

        Ok(Session {
            user,
            token,
            expires_at,
        })
    }
}

fn normalize_email(email: &str) -> Result<String, AuthError> {
    let email = email.trim().to_ascii_lowercase();

    if email.is_empty()
        || email.len() > 320
        || email.chars().any(char::is_whitespace)
        || email.matches('@').count() != 1
    {
        return Err(AuthError::InvalidEmail);
    }

    Ok(email)
}

fn validate_otp(code: &str) -> Result<(), AuthError> {
    if code.len() != 6 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(AuthError::InvalidCode);
    }

    Ok(())
}

fn generate_otp() -> Result<String, AuthError> {
    let limit = u32::MAX - (u32::MAX % OTP_DIGITS);
    loop {
        let mut bytes = [0u8; 4];
        fill(&mut bytes).map_err(|_| AuthError::Randomness)?;
        let value = u32::from_be_bytes(bytes);
        if value < limit {
            return Ok(format!("{:06}", value % OTP_DIGITS));
        }
    }
}

fn generate_session_token() -> Result<String, AuthError> {
    let mut bytes = [0u8; 32];
    fill(&mut bytes).map_err(|_| AuthError::Randomness)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn hash_otp(
    secret: &[u8],
    challenge_id: EntityId,
    email: &str,
    purpose: ChallengePurpose,
    code: &str,
) -> Result<[u8; 32], AuthError> {
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| AuthError::InvalidSecret)?;
    mac.update(challenge_id.as_uuid().as_bytes());
    mac.update(purpose.as_str().as_bytes());
    mac.update(email.as_bytes());
    mac.update(code.as_bytes());

    let bytes = mac.finalize().into_bytes();
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}

fn hash_session_token(token: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let bytes = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    result
}

fn now() -> Result<i64, AuthError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|_| AuthError::Randomness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_is_six_digits() {
        let code = generate_otp().unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.bytes().all(|byte| byte.is_ascii_digit()));
    }

    #[test]
    fn session_token_is_url_safe_and_opaque() {
        let token = generate_session_token().unwrap();
        assert!(token.len() >= 40);
        assert!(!token.contains('='));
    }

    #[test]
    fn email_normalization_is_case_insensitive() {
        assert_eq!(
            normalize_email("  User@Example.COM ").unwrap(),
            "user@example.com"
        );
    }
}
