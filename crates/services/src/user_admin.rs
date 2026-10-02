//! Administrative user management application services and contracts.

use minirust_core::EntityId;

use crate::{Page, Pagination, UserAccess, UserLocale};

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
    CannotChangeOwnRole,
    InvalidFullName,
    InvalidAvatarUrl,
    InvalidLocale,
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
            Self::CannotChangeOwnRole => f.write_str("cannot change own role"),
            Self::InvalidFullName => f.write_str("full name is invalid"),
            Self::InvalidAvatarUrl => f.write_str("avatar URL is invalid"),
            Self::InvalidLocale => f.write_str("locale is invalid"),
            Self::Persistence => f.write_str("user persistence failed"),
        }
    }
}

impl std::error::Error for UserAdminError {}

#[allow(async_fn_in_trait)]
pub trait UserAdminRepository: Clone + Send + Sync + 'static {
    async fn create_user(
        &self,
        id: EntityId,
        email: &str,
        now: i64,
    ) -> Result<UserAccess, UserAdminError>;

    async fn find_user(&self, email: &str) -> Result<Option<UserAccess>, UserAdminError>;

    async fn list_users(
        &self,
        now: i64,
        pagination: Pagination,
    ) -> Result<Page<UserAccess>, UserAdminError>;

    async fn find_user_by_id(
        &self,
        user_id: EntityId,
    ) -> Result<Option<UserAccess>, UserAdminError>;

    async fn update_user(
        &self,
        user_id: EntityId,
        role: AdminUserRole,
        premium_active: bool,
        premium_expires_at: Option<i64>,
        is_locked: bool,
    ) -> Result<UserAccess, UserAdminError>;

    async fn delete_user(&self, user_id: EntityId) -> Result<(), UserAdminError>;

    async fn get_premium(&self, user_id: EntityId) -> Result<PremiumEntitlement, UserAdminError>;
    async fn update_profile(
        &self,
        user_id: EntityId,
        full_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<UserAccess, UserAdminError>;

    async fn update_locale(
        &self,
        user_id: EntityId,
        locale: UserLocale,
    ) -> Result<UserAccess, UserAdminError>;
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

    pub async fn create(
        &self,
        email: &str,
        role: AdminUserRole,
        premium_active: bool,
        premium_expires_at: Option<i64>,
        is_locked: bool,
    ) -> Result<UserAccess, UserAdminError> {
        if role != AdminUserRole::Admin {
            if let Some(expires_at) = premium_expires_at {
                if expires_at <= now() {
                    return Err(UserAdminError::InvalidPremiumExpiry);
                }
            }
        }

        let email = normalize_email(email)?;
        if self.repository.find_user(&email).await?.is_some() {
            return Err(UserAdminError::EmailAlreadyExists);
        }

        let id = EntityId::new();
        let user = self.repository.create_user(id, &email, now()).await?;

        // Apply role and premium after creation within the same logical operation
        let user = if role == AdminUserRole::Admin || premium_active {
            let effective_role = if user.is_admin {
                AdminUserRole::Admin
            } else {
                role
            };
            self.repository
                .update_user(user.id, effective_role, premium_active, premium_expires_at)
                .await?
        } else {
            user
        };

        Ok(user)
    }

    pub async fn get_by_id(&self, user_id: EntityId) -> Result<UserAccess, UserAdminError> {
        self.repository
            .find_user_by_id(user_id)
            .await?
            .ok_or(UserAdminError::NotFound)
    }

    pub async fn get(&self, email: &str) -> Result<UserAccess, UserAdminError> {
        let email = normalize_email(email)?;
        self.repository
            .find_user(&email)
            .await?
            .ok_or(UserAdminError::NotFound)
    }

    pub async fn list(&self, pagination: Pagination) -> Result<Page<UserAccess>, UserAdminError> {
        self.repository.list_users(now(), pagination).await
    }

    pub async fn update_user(
        &self,
        user_id: EntityId,
        role: AdminUserRole,
        premium_active: bool,
        premium_expires_at: Option<i64>,
        is_locked: bool,
    ) -> Result<UserAccess, UserAdminError> {
        if role != AdminUserRole::Admin {
            if let Some(expires_at) = premium_expires_at {
                if expires_at <= now() {
                    return Err(UserAdminError::InvalidPremiumExpiry);
                }
            }
        }

        self.repository
            .update_user(user_id, role, premium_active, premium_expires_at, is_locked)
            .await
    }

    pub async fn delete(&self, user_id: EntityId) -> Result<(), UserAdminError> {
        self.repository.delete_user(user_id).await
    }

    pub async fn get_premium(
        &self,
        user_id: EntityId,
    ) -> Result<PremiumEntitlement, UserAdminError> {
        self.repository.get_premium(user_id).await
    }

    pub async fn update_profile(
        &self,
        user_id: EntityId,
        full_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<UserAccess, UserAdminError> {
        let full_name = normalize_full_name(full_name)?;
        let avatar_url = normalize_avatar_url(avatar_url)?;
        self.repository
            .update_profile(user_id, full_name.as_deref(), avatar_url.as_deref())
            .await
    }

    pub async fn update_locale(
        &self,
        user_id: EntityId,
        locale: UserLocale,
    ) -> Result<UserAccess, UserAdminError> {
        self.repository.update_locale(user_id, locale).await
    }
}

fn normalize_full_name(value: Option<&str>) -> Result<Option<String>, UserAdminError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 {
        return Err(UserAdminError::InvalidFullName);
    }
    Ok(Some(value.to_owned()))
}

fn normalize_avatar_url(value: Option<&str>) -> Result<Option<String>, UserAdminError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty()
        || value.chars().count() > 2048
        || !(value.starts_with("https://") || value.starts_with("http://"))
    {
        return Err(UserAdminError::InvalidAvatarUrl);
    }
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
    CreateUser {
        email: String,
        role: AdminUserRole,
        premium_active: bool,
        premium_expires_at: Option<i64>,
    },
    UpdateUser {
        user_id: EntityId,
        role: AdminUserRole,
        premium_active: bool,
        premium_expires_at: Option<i64>,
        is_locked: bool,
    },
    DeleteUser {
        user_id: EntityId,
    },
    UpdateProfile {
        user_id: EntityId,
        full_name: Option<String>,
        avatar_url: Option<String>,
    },
    SetLocale {
        user_id: EntityId,
        locale: UserLocale,
    },
}

pub enum UserAdminCommandResult {
    User(UserAccess),
    Deleted,
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
    async fn handle(
        &self,
        command: UserAdminCommand,
    ) -> Result<UserAdminCommandResult, UserAdminError> {
        match command {
            UserAdminCommand::CreateUser {
                email,
                role,
                premium_active,
                premium_expires_at,
            } => self
                .service
                .create(&email, role, premium_active, premium_expires_at)
                .await
                .map(UserAdminCommandResult::User),
            UserAdminCommand::UpdateUser {
                user_id,
                role,
                premium_active,
                premium_expires_at,
                is_locked,
            } => self
                .service
                .update_user(user_id, role, premium_active, premium_expires_at, is_locked)
                .await
                .map(UserAdminCommandResult::User),
            UserAdminCommand::DeleteUser { user_id } => self
                .service
                .delete(user_id)
                .await
                .map(|_| UserAdminCommandResult::Deleted),
            UserAdminCommand::UpdateProfile {
                user_id,
                full_name,
                avatar_url,
            } => self
                .service
                .update_profile(user_id, full_name.as_deref(), avatar_url.as_deref())
                .await
                .map(UserAdminCommandResult::User),
            UserAdminCommand::SetLocale { user_id, locale } => self
                .service
                .update_locale(user_id, locale)
                .await
                .map(UserAdminCommandResult::User),
        }
    }
}

pub enum UserAdminQuery {
    GetUser { user_id: EntityId },
    ListUsers { pagination: Pagination },
    GetPremium { user_id: EntityId },
}

pub enum UserAdminQueryResult {
    User(UserAccess),
    Users(Page<UserAccess>),
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
            UserAdminQuery::GetUser { user_id } => self
                .service
                .get_by_id(user_id)
                .await
                .map(UserAdminQueryResult::User),
            UserAdminQuery::ListUsers { pagination } => self
                .service
                .list(pagination)
                .await
                .map(UserAdminQueryResult::Users),
            UserAdminQuery::GetPremium { user_id } => self
                .service
                .get_premium(user_id)
                .await
                .map(UserAdminQueryResult::Premium),
        }
    }
}
