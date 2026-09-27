//! Administrative user management application services and contracts.

use minirust_core::EntityId;

use crate::UserAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminUserRole {
    Admin,
    None,
}

impl AdminUserRole {
    pub fn parse(value: &str) -> Result<Self, UserAdminError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "none" => Ok(Self::None),
            _ => Err(UserAdminError::InvalidRole),
        }
    }

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
    InvalidFullName,
    InvalidAvatarUrl,
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
            Self::InvalidFullName => f.write_str("full name is invalid"),
            Self::InvalidAvatarUrl => f.write_str("avatar URL is invalid"),
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
    async fn update_profile(&self, user_id: EntityId, full_name: Option<&str>, avatar_url: Option<&str>) -> Result<UserAccess, UserAdminError>;
    async fn lock_user(&self, user_id: EntityId) -> Result<(), UserAdminError>;
    async fn unlock_user(&self, email: &str) -> Result<UserAccess, UserAdminError>;
    async fn delete_user_by_id(&self, user_id: EntityId) -> Result<(), UserAdminError>;
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
    pub async fn update_profile(&self, user_id: EntityId, full_name: Option<&str>, avatar_url: Option<&str>) -> Result<UserAccess, UserAdminError> {
        let full_name = normalize_full_name(full_name)?;
        let avatar_url = normalize_avatar_url(avatar_url)?;
        self.repository.update_profile(user_id, full_name.as_deref(), avatar_url.as_deref()).await
    }

    pub async fn lock(&self, user_id: EntityId) -> Result<(), UserAdminError> {
        self.repository.lock_user(user_id).await
    }

    pub async fn delete_by_id(&self, user_id: EntityId) -> Result<(), UserAdminError> {
        self.repository.delete_user_by_id(user_id).await
    }

    pub async fn unlock(&self, email: &str) -> Result<UserAccess, UserAdminError> {
        let email = normalize_email(email)?;
        self.repository.unlock_user(&email).await
    }

}

fn normalize_full_name(value: Option<&str>) -> Result<Option<String>, UserAdminError> {
    let Some(value) = value else { return Ok(None); };
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 { return Err(UserAdminError::InvalidFullName); }
    Ok(Some(value.to_owned()))
}

fn normalize_avatar_url(value: Option<&str>) -> Result<Option<String>, UserAdminError> {
    let Some(value) = value else { return Ok(None); };
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 2048 || !(value.starts_with("https://") || value.starts_with("http://")) { return Err(UserAdminError::InvalidAvatarUrl); }
    Ok(Some(value.to_owned()))
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

use crate::cqrs::{AsyncCommandHandler, AsyncQueryHandler, Command, Query};

pub enum UserAdminCommand {
    CreateUser { email: String },
    UpdateUserEmail { current_email: String, new_email: String },
    DeleteUser { email: String },
    AssignRole { email: String, role: AdminUserRole },
    SetPremium { email: String, active: bool, expires_at: Option<i64> },
    RevokePremium { email: String },
    UpdateProfile { user_id: EntityId, full_name: Option<String>, avatar_url: Option<String> },
    LockUser { user_id: EntityId },
    DeleteUserById { user_id: EntityId },
    UnlockUser { email: String },
}

pub enum UserAdminCommandResult {
    User(UserAccess),
    Deleted,
    Locked,
}

impl Command for UserAdminCommand {
    type Output = UserAdminCommandResult;
    type Error = UserAdminError;
}

#[derive(Clone)]
pub struct UserAdminCommandHandler<R>
where
    R: UserAdminRepository,
{
    service: UserAdminService<R>,
}

impl<R> UserAdminCommandHandler<R>
where
    R: UserAdminRepository,
{
    pub fn new(service: UserAdminService<R>) -> Self {
        Self { service }
    }
}

impl<R> AsyncCommandHandler<UserAdminCommand> for UserAdminCommandHandler<R>
where
    R: UserAdminRepository,
{
    async fn handle(&self, command: UserAdminCommand) -> Result<UserAdminCommandResult, UserAdminError> {
        match command {
            UserAdminCommand::CreateUser { email } => self.service.create(&email).await.map(UserAdminCommandResult::User),
            UserAdminCommand::UpdateUserEmail { current_email, new_email } => self.service.update_email(&current_email, &new_email).await.map(UserAdminCommandResult::User),
            UserAdminCommand::DeleteUser { email } => self.service.delete(&email).await.map(|_| UserAdminCommandResult::Deleted),
            UserAdminCommand::AssignRole { email, role } => self.service.assign_role(&email, role).await.map(UserAdminCommandResult::User),
            UserAdminCommand::SetPremium { email, active, expires_at } => self.service.set_premium(&email, active, expires_at).await.map(UserAdminCommandResult::User),
            UserAdminCommand::RevokePremium { email } => self.service.revoke_premium(&email).await.map(UserAdminCommandResult::User),
            UserAdminCommand::UpdateProfile { user_id, full_name, avatar_url } => self.service.update_profile(user_id, full_name.as_deref(), avatar_url.as_deref()).await.map(UserAdminCommandResult::User),
            UserAdminCommand::LockUser { user_id } => self.service.lock(user_id).await.map(|_| UserAdminCommandResult::Locked),
            UserAdminCommand::DeleteUserById { user_id } => self.service.delete_by_id(user_id).await.map(|_| UserAdminCommandResult::Deleted),
            UserAdminCommand::UnlockUser { email } => self.service.unlock(&email).await.map(UserAdminCommandResult::User),
        }
    }
}

pub enum UserAdminQuery {
    GetUser { email: String },
    ListUsers,
    GetPremium { email: String },
}

pub enum UserAdminQueryResult {
    User(UserAccess),
    Users(Vec<UserAccess>),
    Premium(PremiumEntitlement),
}

impl Query for UserAdminQuery {
    type Output = Result<UserAdminQueryResult, UserAdminError>;
}

#[derive(Clone)]
pub struct UserAdminQueryHandler<R>
where
    R: UserAdminRepository,
{
    service: UserAdminService<R>,
}

impl<R> UserAdminQueryHandler<R>
where
    R: UserAdminRepository,
{
    pub fn new(service: UserAdminService<R>) -> Self {
        Self { service }
    }
}

impl<R> AsyncQueryHandler<UserAdminQuery> for UserAdminQueryHandler<R>
where
    R: UserAdminRepository,
{
    async fn handle(&self, query: UserAdminQuery) -> Result<UserAdminQueryResult, UserAdminError> {
        match query {
            UserAdminQuery::GetUser { email } => self.service.get(&email).await.map(UserAdminQueryResult::User),
            UserAdminQuery::ListUsers => self.service.list().await.map(UserAdminQueryResult::Users),
            UserAdminQuery::GetPremium { email } => self.service.get_premium(&email).await.map(UserAdminQueryResult::Premium),
        }
    }
}
