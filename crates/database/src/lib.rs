//! MariaDB access for MiniRust.
//!
//! This crate implements application-layer persistence contracts. Domain and
//! application code do not depend on SQLx or MariaDB types.

use minirust_core::EntityId;
use minirust_services::{
    AdminUserRole, AuthError, AuthRepository, Challenge, ChallengePurpose, ChallengeRef,
    PremiumEntitlement, UserAccess, UserAdminError, UserAdminRepository,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::{MySql, Row, Transaction};
use tracing::error;
use uuid::Uuid;

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
                u.locked_at,
                EXISTS(
                    SELECT 1
                    FROM user_roles ur
                    WHERE ur.user_id = u.id AND ur.role = 'admin'
                ) AS is_admin,
                EXISTS(
                    SELECT 1
                    FROM user_entitlements ue
                    WHERE ue.user_id = u.id
                      AND ue.entitlement = 'premium'
                      AND ue.active = 1
                      AND (ue.expires_at IS NULL OR ue.expires_at > ?)
                ) AS is_premium
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

impl UserAdminRepository for Database {
    async fn create_user(
        &self,
        id: EntityId,
        email: &str,
        now: i64,
    ) -> Result<UserAccess, UserAdminError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        let result = sqlx::query(
            "INSERT INTO users (id, email, bootstrap_admin, created_at)
             VALUES (?, ?, 0, ?)",
        )
        .bind(id.as_uuid().as_bytes().as_slice())
        .bind(email)
        .bind(now)
        .execute(&mut *tx)
        .await;

        if let Err(error) = result {
            if error
                .as_database_error()
                .and_then(|database| database.code().map(|code| code == "1062"))
                .unwrap_or(false)
            {
                return Err(UserAdminError::EmailAlreadyExists);
            }
            error!(%error, "failed to create user");
            return Err(UserAdminError::Persistence);
        }

        let user = self
            .user_by_id(&mut tx, id, now)
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        Ok(user)
    }

    async fn find_user(&self, email: &str) -> Result<Option<UserAccess>, UserAdminError> {
        let row = sqlx::query("SELECT id FROM users WHERE email = ? LIMIT 1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let user_id = row_to_id(&row).map_err(|_| UserAdminError::Persistence)?;
        self.find_user_by_id(user_id).await
    }

    async fn find_user_by_id(
        &self,
        user_id: EntityId,
    ) -> Result<Option<UserAccess>, UserAdminError> {
        let now = current_epoch();
        let row = sqlx::query(Self::user_query())
            .bind(now)
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| {
                error!(%error, "failed to find user by id");
                UserAdminError::Persistence
            })?;

        row.map(|row| row_to_user(&row).map_err(|_| UserAdminError::Persistence))
            .transpose()
    }

    async fn list_users(&self, now: i64) -> Result<Vec<UserAccess>, UserAdminError> {
        let rows = sqlx::query(
            r#"
                SELECT
                    u.id,
                    u.email,
                    u.full_name,
                    u.avatar_url,
                    u.locked_at,
                    EXISTS(
                        SELECT 1 FROM user_roles ur
                        WHERE ur.user_id = u.id AND ur.role = 'admin'
                    ) AS is_admin,
                    EXISTS(
                        SELECT 1 FROM user_entitlements ue
                        WHERE ue.user_id = u.id
                          AND ue.entitlement = 'premium'
                          AND ue.active = 1
                          AND (ue.expires_at IS NULL OR ue.expires_at > ?)
                    ) AS is_premium
                FROM users u
                ORDER BY u.email
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            error!(%error, "failed to list users");
            UserAdminError::Persistence
        })?;

        rows.into_iter()
            .map(|row| row_to_user(&row).map_err(|_| UserAdminError::Persistence))
            .collect()
    }

    async fn update_user_email(
        &self,
        user_id: EntityId,
        new_email: &str,
    ) -> Result<UserAccess, UserAdminError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        let row = sqlx::query(
            "SELECT email, bootstrap_admin
             FROM users
             WHERE id = ?
             FOR UPDATE",
        )
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| UserAdminError::Persistence)?
        .ok_or(UserAdminError::NotFound)?;

        let current_email = row
            .try_get::<String, _>("email")
            .map_err(|_| UserAdminError::Persistence)?;
        let bootstrap_admin = row
            .try_get::<i64, _>("bootstrap_admin")
            .map_err(|_| UserAdminError::Persistence)?
            != 0;

        if bootstrap_admin {
            return Err(UserAdminError::ProtectedUser);
        }

        let result = sqlx::query("UPDATE users SET email = ? WHERE id = ?")
            .bind(new_email)
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .execute(&mut *tx)
            .await;

        if let Err(error) = result {
            if error
                .as_database_error()
                .and_then(|database| database.code().map(|code| code == "1062"))
                .unwrap_or(false)
            {
                return Err(UserAdminError::EmailAlreadyExists);
            }
            return Err(UserAdminError::Persistence);
        }

        sqlx::query(
            "DELETE FROM auth_challenges
             WHERE email = ?",
        )
        .bind(current_email)
        .execute(&mut *tx)
        .await
        .map_err(|_| UserAdminError::Persistence)?;

        let user = self
            .user_by_id(&mut tx, user_id, current_epoch())
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        Ok(user)
    }

    async fn delete_user(&self, user_id: EntityId) -> Result<(), UserAdminError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        let row = sqlx::query(
            "SELECT email, bootstrap_admin
             FROM users
             WHERE id = ?
             FOR UPDATE",
        )
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| UserAdminError::Persistence)?
        .ok_or(UserAdminError::NotFound)?;

        let email = row
            .try_get::<String, _>("email")
            .map_err(|_| UserAdminError::Persistence)?;
        let bootstrap_admin = row
            .try_get::<i64, _>("bootstrap_admin")
            .map_err(|_| UserAdminError::Persistence)?
            != 0;

        if bootstrap_admin {
            return Err(UserAdminError::ProtectedUser);
        }

        sqlx::query("DELETE FROM auth_challenges WHERE email = ?")
            .bind(email)
            .execute(&mut *tx)
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .execute(&mut *tx)
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        tx.commit().await.map_err(|_| UserAdminError::Persistence)
    }

    async fn set_admin_role(
        &self,
        user_id: EntityId,
        role: AdminUserRole,
    ) -> Result<UserAccess, UserAdminError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        let row = sqlx::query(
            "SELECT bootstrap_admin
             FROM users
             WHERE id = ?
             FOR UPDATE",
        )
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| UserAdminError::Persistence)?
        .ok_or(UserAdminError::NotFound)?;

        let bootstrap_admin = row
            .try_get::<i64, _>("bootstrap_admin")
            .map_err(|_| UserAdminError::Persistence)?
            != 0;

        if bootstrap_admin && role == AdminUserRole::None {
            return Err(UserAdminError::ProtectedUser);
        }

        match role {
            AdminUserRole::Admin => {
                sqlx::query(
                    "INSERT INTO user_roles (user_id, role)
                     VALUES (?, 'admin')
                     ON DUPLICATE KEY UPDATE role = VALUES(role)",
                )
                .bind(user_id.as_uuid().as_bytes().as_slice())
                .execute(&mut *tx)
                .await
                .map_err(|_| UserAdminError::Persistence)?;
            }
            AdminUserRole::None => {
                sqlx::query(
                    "DELETE FROM user_roles
                     WHERE user_id = ? AND role = 'admin'",
                )
                .bind(user_id.as_uuid().as_bytes().as_slice())
                .execute(&mut *tx)
                .await
                .map_err(|_| UserAdminError::Persistence)?;
            }
        }

        let user = self
            .user_by_id(&mut tx, user_id, current_epoch())
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        Ok(user)
    }
    async fn get_premium(&self, user_id: EntityId) -> Result<PremiumEntitlement, UserAdminError> {
        let row = sqlx::query(
            "SELECT ue.active, ue.expires_at
             FROM users u
             LEFT JOIN user_entitlements ue
               ON ue.user_id = u.id AND ue.entitlement = 'premium'
             WHERE u.id = ?
             LIMIT 1",
        )
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserAdminError::Persistence)?
        .ok_or(UserAdminError::NotFound)?;

        Ok(PremiumEntitlement {
            active: row
                .try_get::<Option<i64>, _>("active")
                .map_err(|_| UserAdminError::Persistence)?
                .unwrap_or(0)
                != 0,
            expires_at: row
                .try_get("expires_at")
                .map_err(|_| UserAdminError::Persistence)?,
        })
    }

    async fn revoke_premium(&self, user_id: EntityId) -> Result<UserAccess, UserAdminError> {
        self.set_premium(user_id, false, None).await
    }

    async fn set_premium(
        &self,
        user_id: EntityId,
        active: bool,
        expires_at: Option<i64>,
    ) -> Result<UserAccess, UserAdminError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        let exists = sqlx::query("SELECT 1 FROM users WHERE id = ? FOR UPDATE")
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| UserAdminError::Persistence)?
            .is_some();
        if !exists {
            return Err(UserAdminError::NotFound);
        }

        sqlx::query(
            "INSERT INTO user_entitlements (user_id, entitlement, active, expires_at)
             VALUES (?, 'premium', ?, ?)
             ON DUPLICATE KEY UPDATE
                 active = VALUES(active),
                 expires_at = VALUES(expires_at)",
        )
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .bind(if active { 1_i64 } else { 0_i64 })
        .bind(expires_at)
        .execute(&mut *tx)
        .await
        .map_err(|_| UserAdminError::Persistence)?;

        let user = self
            .user_by_id(&mut tx, user_id, current_epoch())
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        Ok(user)
    }
    async fn update_profile(
        &self,
        user_id: EntityId,
        full_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<UserAccess, UserAdminError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| UserAdminError::Persistence)?;
        let exists = sqlx::query("SELECT id FROM users WHERE id = ? FOR UPDATE")
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| UserAdminError::Persistence)?
            .is_some();
        if !exists {
            return Err(UserAdminError::NotFound);
        }

        sqlx::query("UPDATE users SET full_name = ?, avatar_url = ? WHERE id = ?")
            .bind(full_name)
            .bind(avatar_url)
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                error!(%error, "failed to update user profile");
                UserAdminError::Persistence
            })?;

        let user = self
            .user_by_id(&mut tx, user_id, current_epoch())
            .await
            .map_err(|_| UserAdminError::Persistence)?;
        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        Ok(user)
    }

    async fn lock_user(&self, user_id: EntityId) -> Result<(), UserAdminError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| UserAdminError::Persistence)?;
        let row = sqlx::query("SELECT bootstrap_admin FROM users WHERE id = ? FOR UPDATE")
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| UserAdminError::Persistence)?
            .ok_or(UserAdminError::NotFound)?;

        if row
            .try_get::<i64, _>("bootstrap_admin")
            .map_err(|_| UserAdminError::Persistence)?
            != 0
        {
            return Err(UserAdminError::ProtectedUser);
        }

        let now = current_epoch();
        sqlx::query("UPDATE users SET locked_at = ? WHERE id = ?")
            .bind(now)
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                error!(%error, "failed to lock user");
                UserAdminError::Persistence
            })?;

        sqlx::query(
            "UPDATE auth_sessions SET revoked_at = ? WHERE user_id = ? AND revoked_at IS NULL",
        )
        .bind(now)
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .execute(&mut *tx)
        .await
        .map_err(|_| UserAdminError::Persistence)?;

        tx.commit().await.map_err(|_| UserAdminError::Persistence)
    }

    async fn unlock_user(&self, user_id: EntityId) -> Result<UserAccess, UserAdminError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| UserAdminError::Persistence)?;
        let exists = sqlx::query("SELECT 1 FROM users WHERE id = ? FOR UPDATE")
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| UserAdminError::Persistence)?
            .is_some();
        if !exists {
            return Err(UserAdminError::NotFound);
        }
        sqlx::query("UPDATE users SET locked_at = NULL WHERE id = ?")
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .execute(&mut *tx)
            .await
            .map_err(|_| UserAdminError::Persistence)?;

        let user = self
            .user_by_id(&mut tx, user_id, current_epoch())
            .await
            .map_err(|_| UserAdminError::Persistence)?;
        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        Ok(user)
    }
}

impl AuthRepository for Database {
    async fn user_exists(&self, email: &str) -> Result<bool, AuthError> {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)")
                .bind(email)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| AuthError::Persistence)?;

        Ok(exists != 0)
    }

    async fn is_bootstrap_admin(&self, email: &str) -> Result<bool, AuthError> {
        let bootstrap_admin =
            sqlx::query_scalar::<_, i64>("SELECT bootstrap_admin FROM users WHERE email = ?")
                .bind(email)
                .fetch_optional(&self.pool)
                .await
                .map_err(|_| AuthError::Persistence)?
                .unwrap_or(0);

        Ok(bootstrap_admin != 0)
    }

    async fn create_challenge(
        &self,
        challenge: Challenge,
        email: &str,
        purpose: ChallengePurpose,
        code_hash: [u8; 32],
        created_at: i64,
    ) -> Result<(), AuthError> {
        if purpose == ChallengePurpose::Login {
            let bootstrap_admin =
                sqlx::query_scalar::<_, i64>("SELECT bootstrap_admin FROM users WHERE email = ?")
                    .bind(email)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|_| AuthError::Persistence)?
                    .unwrap_or(0);

            if bootstrap_admin != 0 {
                return Ok(());
            }
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| AuthError::Persistence)?;

        sqlx::query(
            "DELETE FROM auth_challenges WHERE email = ? AND purpose = ? AND consumed_at IS NULL",
        )
        .bind(email)
        .bind(purpose.as_str())
        .execute(&mut *tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

        sqlx::query(
            "INSERT INTO auth_challenges
                (id, email, purpose, code_hash, attempts, max_attempts, expires_at, created_at)
             VALUES (?, ?, ?, ?, 0, ?, ?, ?)",
        )
        .bind(challenge.id.as_uuid().as_bytes().as_slice())
        .bind(email)
        .bind(purpose.as_str())
        .bind(code_hash.as_slice())
        .bind(challenge.max_attempts)
        .bind(challenge.expires_at)
        .bind(created_at)
        .execute(&mut *tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

        tx.commit().await.map_err(|_| AuthError::Persistence)
    }

    async fn latest_challenge(
        &self,
        email: &str,
        purpose: ChallengePurpose,
        now: i64,
    ) -> Result<Option<ChallengeRef>, AuthError> {
        let row = sqlx::query(
            "SELECT id
             FROM auth_challenges
             WHERE email = ?
               AND purpose = ?
               AND consumed_at IS NULL
               AND expires_at > ?
               AND attempts < max_attempts
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(email)
        .bind(purpose.as_str())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Persistence)?;

        row.map(|row| row_to_id(&row).map(|id| ChallengeRef { id }))
            .transpose()
    }

    async fn consume_registration_code(
        &self,
        challenge_id: EntityId,
        email: &str,
        code_hash: [u8; 32],
        user_id: EntityId,
        now: i64,
        session_token_hash: [u8; 32],
        session_expires_at: i64,
    ) -> Result<UserAccess, AuthError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| AuthError::Persistence)?;
        let challenge = lock_challenge(&mut tx, challenge_id).await?;

        validate_challenge(&challenge, email, ChallengePurpose::Registration, now)?;

        if challenge.code_hash != code_hash {
            let error = record_failed_attempt(
                &mut tx,
                challenge_id,
                challenge.attempts,
                challenge.max_attempts,
            )
            .await?;
            tx.commit().await.map_err(|_| AuthError::Persistence)?;
            return Err(error);
        }

        let insert = sqlx::query(
            "INSERT INTO users (id, email, bootstrap_admin, created_at) VALUES (?, ?, 0, ?)",
        )
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .bind(email)
        .bind(now)
        .execute(&mut *tx)
        .await;

        if let Err(error) = insert {
            if error
                .as_database_error()
                .and_then(|database| database.code())
                .map(|code| code == "1062")
                .unwrap_or(false)
            {
                return Err(AuthError::EmailAlreadyExists);
            }
            return Err(AuthError::Persistence);
        }

        consume_challenge(&mut tx, challenge_id, now).await?;
        insert_session(
            &mut tx,
            user_id,
            session_token_hash,
            now,
            session_expires_at,
        )
        .await?;

        let user = self.user_by_id(&mut tx, user_id, now).await?;
        tx.commit().await.map_err(|_| AuthError::Persistence)?;
        Ok(user)
    }

    async fn consume_login_code(
        &self,
        challenge_id: EntityId,
        email: &str,
        code_hash: [u8; 32],
        now: i64,
        session_token_hash: [u8; 32],
        session_expires_at: i64,
    ) -> Result<UserAccess, AuthError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| AuthError::Persistence)?;
        let challenge = lock_challenge(&mut tx, challenge_id).await?;

        validate_challenge(&challenge, email, ChallengePurpose::Login, now)?;

        if challenge.code_hash != code_hash {
            let error = record_failed_attempt(
                &mut tx,
                challenge_id,
                challenge.attempts,
                challenge.max_attempts,
            )
            .await?;
            tx.commit().await.map_err(|_| AuthError::Persistence)?;
            return Err(error);
        }

        let row = sqlx::query(
            "SELECT id, bootstrap_admin, locked_at FROM users WHERE email = ? FOR UPDATE",
        )
        .bind(email)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| AuthError::Persistence)?
        .ok_or(AuthError::Persistence)?;

        let user_id = row_to_id(&row)?;
        let locked = row
            .try_get::<Option<i64>, _>("locked_at")
            .map_err(|_| AuthError::Persistence)?
            .is_some();
        if locked {
            return Err(AuthError::AccountLocked);
        }

        let bootstrap_admin = row
            .try_get::<i64, _>("bootstrap_admin")
            .map_err(|_| AuthError::Persistence)?
            != 0;
        if !bootstrap_admin {
            consume_challenge(&mut tx, challenge_id, now).await?;
        }
        insert_session(
            &mut tx,
            user_id,
            session_token_hash,
            now,
            session_expires_at,
        )
        .await?;

        let user = self.user_by_id(&mut tx, user_id, now).await?;
        tx.commit().await.map_err(|_| AuthError::Persistence)?;
        Ok(user)
    }

    async fn find_session(
        &self,
        session_token_hash: [u8; 32],
        now: i64,
    ) -> Result<Option<UserAccess>, AuthError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| AuthError::Persistence)?;

        let row = sqlx::query(
            "SELECT user_id
             FROM auth_sessions
             WHERE token_hash = ?
               AND revoked_at IS NULL
               AND expires_at > ?
             LIMIT 1",
        )
        .bind(session_token_hash.as_slice())
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

        let Some(row) = row else {
            tx.commit().await.map_err(|_| AuthError::Persistence)?;
            return Ok(None);
        };

        let user_id = row_to_id(&row)?;
        let user = self.user_by_id(&mut tx, user_id, now).await?;
        tx.commit().await.map_err(|_| AuthError::Persistence)?;
        Ok(Some(user))
    }

    async fn revoke_session(&self, session_token_hash: [u8; 32]) -> Result<(), AuthError> {
        sqlx::query(
            "UPDATE auth_sessions
             SET revoked_at = ?
             WHERE token_hash = ? AND revoked_at IS NULL",
        )
        .bind(current_epoch())
        .bind(session_token_hash.as_slice())
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| AuthError::Persistence)
    }

    async fn discard_challenge(&self, challenge_id: EntityId) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM auth_challenges WHERE id = ?")
            .bind(challenge_id.as_uuid().as_bytes().as_slice())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| AuthError::Persistence)
    }
}

struct StoredChallenge {
    email: String,
    purpose: String,
    code_hash: [u8; 32],
    attempts: u8,
    max_attempts: u8,
    expires_at: i64,
}

async fn lock_challenge(
    tx: &mut Transaction<'_, MySql>,
    challenge_id: EntityId,
) -> Result<StoredChallenge, AuthError> {
    let row = sqlx::query(
        "SELECT email, purpose, code_hash, attempts, max_attempts, expires_at
         FROM auth_challenges
         WHERE id = ? AND consumed_at IS NULL
         FOR UPDATE",
    )
    .bind(challenge_id.as_uuid().as_bytes().as_slice())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| AuthError::Persistence)?
    .ok_or(AuthError::InvalidCode)?;

    let hash = row
        .try_get::<Vec<u8>, _>("code_hash")
        .map_err(|_| AuthError::Persistence)?;
    let code_hash: [u8; 32] = hash.try_into().map_err(|_| AuthError::Persistence)?;

    Ok(StoredChallenge {
        email: row.try_get("email").map_err(|_| AuthError::Persistence)?,
        purpose: row.try_get("purpose").map_err(|_| AuthError::Persistence)?,
        code_hash,
        attempts: row
            .try_get("attempts")
            .map_err(|_| AuthError::Persistence)?,
        max_attempts: row
            .try_get("max_attempts")
            .map_err(|_| AuthError::Persistence)?,
        expires_at: row
            .try_get("expires_at")
            .map_err(|_| AuthError::Persistence)?,
    })
}

fn validate_challenge(
    challenge: &StoredChallenge,
    email: &str,
    purpose: ChallengePurpose,
    now: i64,
) -> Result<(), AuthError> {
    if challenge.email != email || challenge.purpose != purpose.as_str() {
        return Err(AuthError::InvalidCode);
    }
    if challenge.expires_at <= now {
        return Err(AuthError::CodeExpired);
    }
    if challenge.attempts >= challenge.max_attempts {
        return Err(AuthError::CodeAttemptsExceeded);
    }
    Ok(())
}

async fn record_failed_attempt(
    tx: &mut Transaction<'_, MySql>,
    challenge_id: EntityId,
    attempts: u8,
    max_attempts: u8,
) -> Result<AuthError, AuthError> {
    let next = attempts.saturating_add(1);
    sqlx::query("UPDATE auth_challenges SET attempts = ? WHERE id = ?")
        .bind(next)
        .bind(challenge_id.as_uuid().as_bytes().as_slice())
        .execute(&mut **tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

    if next >= max_attempts {
        Ok(AuthError::CodeAttemptsExceeded)
    } else {
        Ok(AuthError::InvalidCode)
    }
}

async fn consume_challenge(
    tx: &mut Transaction<'_, MySql>,
    challenge_id: EntityId,
    now: i64,
) -> Result<(), AuthError> {
    sqlx::query("UPDATE auth_challenges SET consumed_at = ? WHERE id = ?")
        .bind(now)
        .bind(challenge_id.as_uuid().as_bytes().as_slice())
        .execute(&mut **tx)
        .await
        .map(|_| ())
        .map_err(|_| AuthError::Persistence)
}

async fn insert_session(
    tx: &mut Transaction<'_, MySql>,
    user_id: EntityId,
    token_hash: [u8; 32],
    created_at: i64,
    expires_at: i64,
) -> Result<(), AuthError> {
    let session_id = EntityId::new();
    sqlx::query(
        "INSERT INTO auth_sessions
            (id, user_id, token_hash, created_at, expires_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(session_id.as_uuid().as_bytes().as_slice())
    .bind(user_id.as_uuid().as_bytes().as_slice())
    .bind(token_hash.as_slice())
    .bind(created_at)
    .bind(expires_at)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(|_| AuthError::Persistence)
}

fn row_to_id(row: &sqlx::mysql::MySqlRow) -> Result<EntityId, AuthError> {
    let bytes = row
        .try_get::<Vec<u8>, _>("id")
        .map_err(|_| AuthError::Persistence)?;
    let uuid = Uuid::from_slice(&bytes).map_err(|_| AuthError::Persistence)?;
    EntityId::from_uuid(uuid).ok_or(AuthError::Persistence)
}

fn row_to_user(row: &sqlx::mysql::MySqlRow) -> Result<UserAccess, AuthError> {
    let bytes = row
        .try_get::<Vec<u8>, _>("id")
        .map_err(|_| AuthError::Persistence)?;
    let uuid = Uuid::from_slice(&bytes).map_err(|_| AuthError::Persistence)?;
    let id = EntityId::from_uuid(uuid).ok_or(AuthError::Persistence)?;

    Ok(UserAccess {
        id,
        email: row.try_get("email").map_err(|_| AuthError::Persistence)?,
        full_name: row
            .try_get("full_name")
            .map_err(|_| AuthError::Persistence)?,
        avatar_url: row
            .try_get("avatar_url")
            .map_err(|_| AuthError::Persistence)?,
        is_locked: row
            .try_get::<Option<i64>, _>("locked_at")
            .map_err(|_| AuthError::Persistence)?
            .is_some(),
        is_admin: row
            .try_get::<i64, _>("is_admin")
            .map_err(|_| AuthError::Persistence)?
            != 0,
        is_premium: row
            .try_get::<i64, _>("is_premium")
            .map_err(|_| AuthError::Persistence)?
            != 0,
    })
}

fn current_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs() as i64)
        .unwrap_or(0)
}

fn seed_otp_hash(
    secret: &[u8],
    challenge_id: EntityId,
    email: &str,
    otp: &str,
) -> Result<[u8; 32], AuthError> {
    use hmac::{Hmac, KeyInit, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| AuthError::InvalidSecret)?;
    mac.update(challenge_id.as_uuid().as_bytes().as_slice());
    mac.update(b"login");
    mac.update(email.as_bytes());
    mac.update(otp.as_bytes());

    Ok(mac.finalize().into_bytes().into())
}
