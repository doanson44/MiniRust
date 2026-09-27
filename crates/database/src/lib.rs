//! MariaDB access for MiniRust.
//!
//! This crate implements application-layer persistence contracts. Domain and
//! application code do not depend on SQLx or MariaDB types.

use minirust_core::EntityId;
use minirust_services::{
    AuthError, AuthRepository, Challenge, ChallengePurpose, ChallengeRef, UserAccess,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::{MySql, Row, Transaction};
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
            .bind(user_id.as_uuid().as_bytes())
            .fetch_one(&mut **tx)
            .await
            .map_err(|_| AuthError::Persistence)?;

        row_to_user(&row)
    }
}

impl AuthRepository for Database {
    async fn user_exists(&self, email: &str) -> Result<bool, AuthError> {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)",
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::Persistence)?;

        Ok(exists != 0)
    }

    async fn create_challenge(
        &self,
        challenge: Challenge,
        email: &str,
        purpose: ChallengePurpose,
        code_hash: [u8; 32],
        created_at: i64,
    ) -> Result<(), AuthError> {
        let mut tx = self.pool.begin().await.map_err(|_| AuthError::Persistence)?;

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
        .bind(challenge.id.as_uuid().as_bytes())
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

        row.map(|row| row_to_id(&row)).transpose()
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
        let mut tx = self.pool.begin().await.map_err(|_| AuthError::Persistence)?;
        let challenge = lock_challenge(&mut tx, challenge_id).await?;

        validate_challenge(&challenge, email, ChallengePurpose::Registration, now)?;

        if challenge.code_hash != code_hash {
            return Err(record_failed_attempt(&mut tx, challenge_id, challenge.attempts, challenge.max_attempts).await?);
        }

        let insert = sqlx::query(
            "INSERT INTO users (id, email, created_at) VALUES (?, ?, ?)",
        )
        .bind(user_id.as_uuid().as_bytes())
        .bind(email)
        .bind(now)
        .execute(&mut *tx)
        .await;

        if insert.is_err() {
            return Err(AuthError::Persistence);
        }

        consume_challenge(&mut tx, challenge_id, now).await?;
        insert_session(&mut tx, user_id, session_token_hash, now, session_expires_at).await?;

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
        let mut tx = self.pool.begin().await.map_err(|_| AuthError::Persistence)?;
        let challenge = lock_challenge(&mut tx, challenge_id).await?;

        validate_challenge(&challenge, email, ChallengePurpose::Login, now)?;

        if challenge.code_hash != code_hash {
            return Err(record_failed_attempt(&mut tx, challenge_id, challenge.attempts, challenge.max_attempts).await?);
        }

        let row = sqlx::query("SELECT id FROM users WHERE email = ? FOR UPDATE")
            .bind(email)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| AuthError::Persistence)?
            .ok_or(AuthError::Persistence)?;

        let user_id = row_to_id(&row)?;
        consume_challenge(&mut tx, challenge_id, now).await?;
        insert_session(&mut tx, user_id, session_token_hash, now, session_expires_at).await?;

        let user = self.user_by_id(&mut tx, user_id, now).await?;
        tx.commit().await.map_err(|_| AuthError::Persistence)?;
        Ok(user)
    }

    async fn find_session(
        &self,
        session_token_hash: [u8; 32],
        now: i64,
    ) -> Result<Option<UserAccess>, AuthError> {
        let mut tx = self.pool.begin().await.map_err(|_| AuthError::Persistence)?;

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
    .bind(challenge_id.as_uuid().as_bytes())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| AuthError::Persistence)?
    .ok_or(AuthError::InvalidCode)?;

    let hash = row.try_get::<Vec<u8>, _>("code_hash").map_err(|_| AuthError::Persistence)?;
    let code_hash: [u8; 32] = hash.try_into().map_err(|_| AuthError::Persistence)?;

    Ok(StoredChallenge {
        email: row.try_get("email").map_err(|_| AuthError::Persistence)?,
        purpose: row.try_get("purpose").map_err(|_| AuthError::Persistence)?,
        code_hash,
        attempts: row.try_get("attempts").map_err(|_| AuthError::Persistence)?,
        max_attempts: row.try_get("max_attempts").map_err(|_| AuthError::Persistence)?,
        expires_at: row.try_get("expires_at").map_err(|_| AuthError::Persistence)?,
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
        .bind(challenge_id.as_uuid().as_bytes())
        .execute(&mut **tx)
        .await
        .map_err(|_| AuthError::Persistence)?;

    tx.commit().await.map_err(|_| AuthError::Persistence)?;

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
        .bind(challenge_id.as_uuid().as_bytes())
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
    .bind(session_id.as_uuid().as_bytes())
    .bind(user_id.as_uuid().as_bytes())
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
        is_admin: row.try_get::<i64, _>("is_admin").map_err(|_| AuthError::Persistence)? != 0,
        is_premium: row.try_get::<i64, _>("is_premium").map_err(|_| AuthError::Persistence)? != 0,
    })
}

fn current_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs() as i64)
        .unwrap_or(0)
}
