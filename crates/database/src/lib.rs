//! MariaDB access for MiniRust.
//!
//! This crate implements application-layer persistence contracts. Domain and
//! application code do not depend on SQLx or MariaDB types.

use minirust_core::EntityId;
use minirust_services::{AuthError, UserAccess};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::{MySql, Row, Transaction};
use tracing::error;
use uuid::Uuid;

mod auth;
mod user_admin;

use auth::seed_otp_hash;

#[derive(Clone)]
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    pub async fn seed_admin(&self, email: &str, otp: &str, secret: &[u8]) -> Result<(), AuthError> {
        let email = email.trim().to_ascii_lowercase();
        if email.is_empty()
            || secret.len() < 32
            || otp.len() != 6
            || !otp.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(AuthError::InvalidSecret);
        }

        let now = current_epoch();
        let user_id = EntityId::from_uuid(Uuid::now_v7()).ok_or(AuthError::Persistence)?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| AuthError::Persistence)?;

        let existing_id = sqlx::query(
            "SELECT id, bootstrap_admin, locked_at FROM users WHERE email = ? FOR UPDATE",
        )
        .bind(&email)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

        let user_id = match existing_id {
            Some(row) => {
                let bootstrap_admin = row
                    .try_get::<i64, _>("bootstrap_admin")
                    .map_err(|_| AuthError::Persistence)?
                    != 0;
                if !bootstrap_admin {
                    return Err(AuthError::BootstrapAdminConflict);
                }
                row_to_id(&row)?
            }
            None => {
                sqlx::query("INSERT INTO users (id, email, bootstrap_admin, created_at) VALUES (?, ?, 1, ?)")
                    .bind(user_id.as_uuid().as_bytes().as_slice())
                    .bind(&email)
                    .bind(now)
                    .execute(&mut *tx)
                    .await
                    .map_err(|_| AuthError::Persistence)?;
                user_id
            }
        };

        sqlx::query("UPDATE users SET bootstrap_admin = 1 WHERE id = ?")
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .execute(&mut *tx)
            .await
            .map_err(|_| AuthError::Persistence)?;

        sqlx::query(
            "INSERT INTO user_roles (user_id, role)
             VALUES (?, 'admin')
             ON DUPLICATE KEY UPDATE role = VALUES(role)",
        )
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .execute(&mut *tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

        let challenge_id = EntityId::new();
        let code_hash = seed_otp_hash(secret, challenge_id, &email, otp)?;

        sqlx::query(
            "UPDATE auth_challenges
             SET consumed_at = ?
             WHERE email = ? AND purpose = 'login' AND consumed_at IS NULL",
        )
        .bind(now)
        .bind(&email)
        .execute(&mut *tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

        sqlx::query(
            "INSERT INTO auth_challenges
                (id, email, purpose, code_hash, attempts, max_attempts, expires_at, created_at)
             VALUES (?, ?, 'login', ?, 0, 5, ?, ?)",
        )
        .bind(challenge_id.as_uuid().as_bytes().as_slice())
        .bind(&email)
        .bind(code_hash.as_slice())
        .bind(now + 365 * 24 * 60 * 60)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

        tx.commit().await.map_err(|_| AuthError::Persistence)
    }

    pub async fn health(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    fn user_query() -> &'static str {
        r#"
            SELECT
                u.id,
                u.email,
                u.full_name,
                u.avatar_url,
                u.locale,
                CAST(u.locked_at IS NOT NULL AS SIGNED) AS is_locked,
                CAST(EXISTS(
                    SELECT 1
                    FROM user_roles ur
                    WHERE ur.user_id = u.id AND ur.role = 'admin'
                ) AS SIGNED) AS is_admin,
                CAST(EXISTS(
                    SELECT 1
                    FROM user_entitlements ue
                    WHERE ue.user_id = u.id
                      AND ue.entitlement = 'premium'
                      AND ue.active = 1
                      AND (ue.expires_at IS NULL OR ue.expires_at > ?)
                ) AS SIGNED) AS is_premium
            FROM users u
            WHERE u.id = ?
        "#
    }

    async fn user_by_id(
        &self,
        tx: &mut Transaction<'_, MySql>,
        user_id: EntityId,
        now: i64,
    ) -> Result<UserAccess, AuthError> {
        let row = sqlx::query(Self::user_query())
            .bind(now)
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .fetch_one(&mut **tx)
            .await
            .map_err(|error| {
                error!(%error, "failed to load user projection by id");
                AuthError::Persistence
            })?;

        row_to_user(&row)
    }
}

pub(crate) fn row_to_id(row: &sqlx::mysql::MySqlRow) -> Result<EntityId, AuthError> {
    let bytes = row
        .try_get::<Vec<u8>, _>("id")
        .map_err(|_| AuthError::Persistence)?;
    let uuid = Uuid::from_slice(&bytes).map_err(|_| AuthError::Persistence)?;
    EntityId::from_uuid(uuid).ok_or(AuthError::Persistence)
}

pub(crate) fn row_to_user(row: &sqlx::mysql::MySqlRow) -> Result<UserAccess, AuthError> {
    let bytes = row.try_get::<Vec<u8>, _>("id").map_err(|error| {
        error!(%error, "failed to decode user projection field");
        AuthError::Persistence
    })?;
    let uuid = Uuid::from_slice(&bytes).map_err(|error| {
        error!(%error, "failed to parse user projection id");
        AuthError::Persistence
    })?;
    let id = EntityId::from_uuid(uuid).ok_or_else(|| {
        error!("failed to convert user projection id to EntityId");
        AuthError::Persistence
    })?;

    let email = row.try_get::<String, _>("email").map_err(|error| {
        error!(%error, "failed to decode user projection field email");
        AuthError::Persistence
    })?;
    let full_name = row
        .try_get::<Option<String>, _>("full_name")
        .map_err(|error| {
            error!(%error, "failed to decode user projection field full_name");
            AuthError::Persistence
        })?;
    let avatar_url = row
        .try_get::<Option<String>, _>("avatar_url")
        .map_err(|error| {
            error!(%error, "failed to decode user projection field avatar_url");
            AuthError::Persistence
        })?;
    let locale = minirust_services::UserLocale::parse(
        &row.try_get::<String, _>("locale").map_err(|error| {
            error!(%error, "failed to decode user projection field locale");
            AuthError::Persistence
        })?,
    )
    .ok_or(AuthError::Persistence)?;
    let is_locked = row.try_get::<i64, _>("is_locked").map_err(|error| {
        error!(%error, "failed to decode user projection field is_locked");
        AuthError::Persistence
    })? != 0;
    let is_admin = row.try_get::<i64, _>("is_admin").map_err(|error| {
        error!(%error, "failed to decode user projection field is_admin");
        AuthError::Persistence
    })? != 0;
    let is_premium = row.try_get::<i64, _>("is_premium").map_err(|error| {
        error!(%error, "failed to decode user projection field is_premium");
        AuthError::Persistence
    })? != 0;

    Ok(UserAccess {
        id,
        email,
        full_name,
        avatar_url,
        is_locked,
        locale,
        is_admin,
        is_premium,
    })
}

pub(crate) fn current_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs() as i64)
        .unwrap_or(0)
}
