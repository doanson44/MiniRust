//! Administrative user management application services and contracts.

use minirust_core::EntityId;

use crate::UserAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminUserRole {
    Admin,
    None,
}

impl AdminUserRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserAdminError {
    InvalidEmail,
    InvalidRole,
    InvalidPremiumExpiry,
    NotFound,
    EmailAlreadyExists,
    ProtectedUser,
    Persistence,
}

impl std::fmt::Display for UserAdminError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEmail => f.write_str("invalid email"),
            Self::InvalidRole => f.write_str("invalid role"),
            Self::InvalidPremiumExpiry => f.write_str("premium expiry must be in the future"),
            Self::NotFound => f.write_str("user not found"),
            Self::EmailAlreadyExists => f.write_str("email already exists"),
            Self::ProtectedUser => f.write_str("user is protected"),
            Self::Persistence => f.write_str("user persistence failed"),
        }
    }
}

impl std::error::Error for UserAdminError {}

pub trait UserAdminRepository: Clone + Send + Sync + 'static {
    async fn create_user(&self, id: EntityId, email: &str, now: i64)
        -> Result<UserAccess, UserAdminError>;

    async fn find_user(&self, email: &str) -> Result<Option<UserAccess>, UserAdminError>;

    async fn list_users(&self, now: i64) -> Result<Vec<UserAccess>, UserAdminError>;

    async fn update_user_email(
        &self,
        current_email: &str,
        new_email: &str,
    ) -> Result<UserAccess, UserAdminError>;

    async fn delete_user(&self, email: &str) -> Result<(), UserAdminError>;

    async fn set_admin_role(
        &self,
        email: &str,
        role: AdminUserRole,
    ) -> Result<UserAccess, UserAdminError>;

    async fn set_premium(
        &self,
        email: &str,
        active: bool,
        expires_at: Option<i64>,
    ) -> Result<UserAccess, UserAdminError>;

    async fn get_premium(
        &self,
        email: &str,
    ) -> Result<PremiumEntitlement, UserAdminError>;

    async fn revoke_premium(&self, email: &str) -> Result<UserAccess, UserAdminError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PremiumEntitlement {
    pub active: bool,
    pub expires_at: Option<i64>,
}

#[derive(Clone)]
pub struct UserAdminService<R> {
    repository: R,
}

impl<R> UserAdminService<R>
where
    R: UserAdminRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create(&self, email: &str) -> Result<UserAccess, UserAdminError> {
        let email = normalize_email(email)?;
        if self.repository.find_user(&email).await?.is_some() {
            return Err(UserAdminError::EmailAlreadyExists);
        }

        self.repository
            .create_user(EntityId::new(), &email, now())
            .await
    }

    pub async fn get(&self, email: &str) -> Result<UserAccess, UserAdminError> {
        let email = normalize_email(email)?;
        self.repository
            .find_user(&email)
            .await?
            .ok_or(UserAdminError::NotFound)
    }

    pub async fn list(&self) -> Result<Vec<UserAccess>, UserAdminError> {
        self.repository.list_users(now()).await
    }

    pub async fn update_email(
        &self,
        current_email: &str,
        new_email: &str,
    ) -> Result<UserAccess, UserAdminError> {
        let current_email = normalize_email(current_email)?;
        let new_email = normalize_email(new_email)?;

        if current_email == new_email {
            return self.get(&current_email).await;
        }

        if self.repository.find_user(&current_email).await?.is_none() {
            return Err(UserAdminError::NotFound);
        }

        if self.repository.find_user(&new_email).await?.is_some() {
            return Err(UserAdminError::EmailAlreadyExists);
        }

        self.repository
            .update_user_email(&current_email, &new_email)
            .await
    }

    pub async fn delete(&self, email: &str) -> Result<(), UserAdminError> {
        let email = normalize_email(email)?;
        if self.repository.find_user(&email).await?.is_none() {
            return Err(UserAdminError::NotFound);
        }

        self.repository.delete_user(&email).await
    }

    pub async fn assign_role(
        &self,
        email: &str,
        role: AdminUserRole,
    ) -> Result<UserAccess, UserAdminError> {
        let email = normalize_email(email)?;
        self.repository.set_admin_role(&email, role).await
    }

    pub async fn get_premium(&self, email: &str) -> Result<PremiumEntitlement, UserAdminError> {
        let email = normalize_email(email)?;
        self.repository.get_premium(&email).await
    }

    pub async fn revoke_premium(&self, email: &str) -> Result<UserAccess, UserAdminError> {
        let email = normalize_email(email)?;
        self.repository.revoke_premium(&email).await
    }

    pub async fn set_premium(
        &self,
        email: &str,
        active: bool,
        expires_at: Option<i64>,
    ) -> Result<UserAccess, UserAdminError> {
        let email = normalize_email(email)?;
        if let Some(expires_at) = expires_at {
            if expires_at <= now() {
                return Err(UserAdminError::InvalidPremiumExpiry);
            }
        }
        self.repository.set_premium(&email, active, expires_at).await
    }
}

fn normalize_email(email: &str) -> Result<String, UserAdminError> {
    let email = email.trim().to_ascii_lowercase();

    if email.is_empty()
        || email.len() > 320
        || email.chars().any(char::is_whitespace)
        || email.matches('@').count() != 1
    {
        return Err(UserAdminError::InvalidEmail);
    }

    Ok(email)
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64)
}
