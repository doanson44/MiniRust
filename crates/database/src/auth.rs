use minirust_core::EntityId;
use minirust_services::{
    AuthError, AuthRepository, Challenge, ChallengePurpose, ChallengeRef, UserAccess,
};
use sqlx::{MySql, Row, Transaction};

use crate::{current_epoch, row_to_id, Database};

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
                .unwrap_or(0)
                != 0;

        Ok(bootstrap_admin)
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
                    .unwrap_or(0)
                    != 0;

            if bootstrap_admin {
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
            .map_err(|_| AuthError::Persistence)?;
        if bootstrap_admin == 0 {
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
            "SELECT user_id AS id
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

pub(crate) struct StoredChallenge {
    pub email: String,
    pub purpose: ChallengePurpose,
    pub code_hash: [u8; 32],
    pub attempts: u8,
    pub max_attempts: u8,
    pub expires_at: i64,
}

pub(crate) async fn lock_challenge(
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
        purpose: ChallengePurpose::parse(
            &row.try_get::<String, _>("purpose")
                .map_err(|_| AuthError::Persistence)?,
        )?,
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

pub(crate) fn validate_challenge(
    challenge: &StoredChallenge,
    email: &str,
    purpose: ChallengePurpose,
    now: i64,
) -> Result<(), AuthError> {
    if challenge.email != email || challenge.purpose != purpose {
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

pub(crate) async fn record_failed_attempt(
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

pub(crate) async fn consume_challenge(
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

pub(crate) async fn insert_session(
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

pub(crate) fn seed_otp_hash(
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
