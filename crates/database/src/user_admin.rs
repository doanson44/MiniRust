use minirust_core::EntityId;
use minirust_services::{
    AdminUserRole, Page, Pagination, PaginationMeta, PremiumEntitlement, UserAccess,
    UserAdminError, UserAdminRepository, UserLocale,
};
use sqlx::Row;
use tracing::error;

use crate::{current_epoch, row_to_id, row_to_user, Database};

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

        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        self.find_user_by_id(id)
            .await?
            .ok_or(UserAdminError::NotFound)
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

    async fn list_users(
        &self,
        now: i64,
        pagination: Pagination,
    ) -> Result<Page<UserAccess>, UserAdminError> {
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|error| {
                error!(%error, "failed to count users");
                UserAdminError::Persistence
            })? as u64;

        let rows = match pagination {
            Pagination::All => {
                sqlx::query(
                    r#"
                        SELECT
                            u.id,
                            u.email,
                            u.full_name,
                            u.avatar_url,
                            u.locale,
                            CAST(u.locked_at IS NOT NULL AS SIGNED) AS is_locked,
                            CAST(EXISTS(
                                SELECT 1 FROM user_roles ur
                                WHERE ur.user_id = u.id AND ur.role = 'admin'
                            ) AS SIGNED) AS is_admin,
                            CAST(EXISTS(
                                SELECT 1 FROM user_entitlements ue
                                WHERE ue.user_id = u.id
                                  AND ue.entitlement = 'premium'
                                  AND ue.active = 1
                                  AND (ue.expires_at IS NULL OR ue.expires_at > ?)
                            ) AS SIGNED) AS is_premium
                        FROM users u
                        ORDER BY u.email, u.id
                    "#,
                )
                .bind(now)
                .fetch_all(&self.pool)
                .await
            }
            Pagination::Paged { page_size, .. } => {
                sqlx::query(
                    r#"
                        SELECT
                            u.id,
                            u.email,
                            u.full_name,
                            u.avatar_url,
                            u.locale,
                            CAST(u.locked_at IS NOT NULL AS SIGNED) AS is_locked,
                            CAST(EXISTS(
                                SELECT 1 FROM user_roles ur
                                WHERE ur.user_id = u.id AND ur.role = 'admin'
                            ) AS SIGNED) AS is_admin,
                            CAST(EXISTS(
                                SELECT 1 FROM user_entitlements ue
                                WHERE ue.user_id = u.id
                                  AND ue.entitlement = 'premium'
                                  AND ue.active = 1
                                  AND (ue.expires_at IS NULL OR ue.expires_at > ?)
                            ) AS SIGNED) AS is_premium
                        FROM users u
                        ORDER BY u.email, u.id
                        LIMIT ? OFFSET ?
                    "#,
                )
                .bind(now)
                .bind(i64::from(page_size))
                .bind(pagination.offset() as i64)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|error| {
            error!(%error, "failed to list users");
            UserAdminError::Persistence
        })?;

        let items = rows
            .into_iter()
            .map(|row| row_to_user(&row).map_err(|_| UserAdminError::Persistence))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page {
            items,
            meta: PaginationMeta::from_pagination(pagination, total),
        })
    }

    async fn update_user(
        &self,
        user_id: EntityId,
        new_email: &str,
        role: AdminUserRole,
        premium_active: bool,
        premium_expires_at: Option<i64>,
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

        if bootstrap_admin && current_email != new_email {
            return Err(UserAdminError::ProtectedUser);
        }
        if bootstrap_admin && role == AdminUserRole::None {
            return Err(UserAdminError::ProtectedUser);
        }

        if current_email != new_email {
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

            sqlx::query("DELETE FROM auth_challenges WHERE email = ?")
                .bind(current_email)
                .execute(&mut *tx)
                .await
                .map_err(|_| UserAdminError::Persistence)?;
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

        sqlx::query(
            "INSERT INTO user_entitlements (user_id, entitlement, active, expires_at)
             VALUES (?, 'premium', ?, ?)
             ON DUPLICATE KEY UPDATE
                 active = VALUES(active),
                 expires_at = VALUES(expires_at)",
        )
        .bind(user_id.as_uuid().as_bytes().as_slice())
        .bind(if premium_active { 1_i64 } else { 0_i64 })
        .bind(premium_expires_at)
        .execute(&mut *tx)
        .await
        .map_err(|_| UserAdminError::Persistence)?;

        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        self.find_user_by_id(user_id)
            .await?
            .ok_or(UserAdminError::NotFound)
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

        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        self.find_user_by_id(user_id)
            .await?
            .ok_or(UserAdminError::NotFound)
    }

    async fn update_locale(
        &self,
        user_id: EntityId,
        locale: UserLocale,
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

        sqlx::query("UPDATE users SET locale = ? WHERE id = ?")
            .bind(locale.as_str())
            .bind(user_id.as_uuid().as_bytes().as_slice())
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                error!(%error, "failed to update user locale");
                UserAdminError::Persistence
            })?;

        tx.commit().await.map_err(|_| UserAdminError::Persistence)?;
        self.find_user_by_id(user_id)
            .await?
            .ok_or(UserAdminError::NotFound)
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

        let bootstrap_admin = row.try_get::<i64, _>("bootstrap_admin").map_err(|error| {
            error!(%error, "failed to decode bootstrap_admin while locking user");
            UserAdminError::Persistence
        })?;

        if bootstrap_admin != 0 {
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
