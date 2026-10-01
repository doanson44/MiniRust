use minirust_core::EntityId;
use minirust_services::{Menu, MenuAccess, MenuError, MenuRepository, Page, Pagination, PaginationMeta};
use sqlx::Row;
use tracing::error;

use crate::{current_epoch, row_to_id, Database};

impl MenuRepository for Database {
    async fn find_menu(&self, menu_id: EntityId) -> Result<Option<Menu>, MenuError> {
        let row = sqlx::query(Self::menu_query())
            .bind(menu_id.as_uuid().as_bytes().as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| {
                error!(%error, "failed to find menu");
                MenuError::Persistence
            })?;

        row.map(|row| row_to_menu(&row).map_err(|_| MenuError::Persistence))
            .transpose()
    }

    async fn list_menus(&self, pagination: Pagination) -> Result<Page<Menu>, MenuError> {
        self.list(false, pagination).await
    }

    async fn list_active_menus(&self) -> Result<Vec<Menu>, MenuError> {
        self.list(true).await
    }

    async fn parent_exists(&self, parent_id: EntityId) -> Result<bool, MenuError> {
        sqlx::query("SELECT 1 FROM menus WHERE id = ? LIMIT 1")
            .bind(parent_id.as_uuid().as_bytes().as_slice())
            .fetch_optional(&self.pool)
            .await
            .map(|row| row.is_some())
            .map_err(|error| {
                error!(%error, "failed to check menu parent");
                MenuError::Persistence
            })
    }

    async fn update_menu(&self, menu: &Menu) -> Result<Menu, MenuError> {
        let now = current_epoch();
        let result = sqlx::query(
            "UPDATE menus
             SET parent_id = ?, name = ?, path = ?, icon = ?, allow_user = ?,
                 allow_premium = ?, sort_order = ?, is_active = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(menu.parent_id.map(|id| id.as_uuid().as_bytes().to_vec()))
        .bind(&menu.name)
        .bind(&menu.path)
        .bind(&menu.icon)
        .bind(menu.access.user)
        .bind(menu.access.premium)
        .bind(menu.sort_order)
        .bind(menu.is_active)
        .bind(now)
        .bind(menu.id.as_uuid().as_bytes().as_slice())
        .execute(&self.pool)
        .await
        .map_err(|error| {
            error!(%error, "failed to update menu");
            MenuError::Persistence
        })?;

        if result.rows_affected() == 0 {
            return Err(MenuError::NotFound);
        }

        self.find_menu(menu.id).await?.ok_or(MenuError::NotFound)
    }
}

impl Database {
    fn menu_query() -> &'static str {
        "SELECT id, parent_id, name, path, icon, allow_user, allow_premium, sort_order, is_active
         FROM menus
         WHERE id = ?"
    }

    async fn list(&self, active_only: bool, pagination: Pagination) -> Result<Page<Menu>, MenuError> {
        let sql = if active_only {
            "SELECT id, parent_id, name, path, icon, allow_user, allow_premium, sort_order, is_active
             FROM menus
             WHERE is_active = 1
             ORDER BY sort_order, name"
        } else {
            "SELECT id, parent_id, name, path, icon, allow_user, allow_premium, sort_order, is_active
             FROM menus
             ORDER BY sort_order, name"
        };

        let rows = match pagination {
            Pagination::All => sqlx::query(sql).fetch_all(&self.pool).await,
            Pagination::Paged { page_size, .. } => sqlx::query(&format!("{sql} LIMIT ? OFFSET ?"))
                .bind(i64::from(page_size))
                .bind(pagination.offset() as i64)
                .fetch_all(&self.pool)
                .await,
        }
        .map_err(|error| {
            error!(%error, "failed to list menus");
            MenuError::Persistence
        })?;

        let total = sqlx::query_scalar::<_, i64>(if active_only {
            "SELECT COUNT(*) FROM menus WHERE is_active = 1"
        } else {
            "SELECT COUNT(*) FROM menus"
        })
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            error!(%error, "failed to count menus");
            MenuError::Persistence
        })? as u64;

        let items = rows
            .iter()
            .map(|row| row_to_menu(row).map_err(|_| MenuError::Persistence))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page {
            items,
            meta: PaginationMeta::from_pagination(pagination, total),
        })
    }
}

fn row_to_menu(row: &sqlx::mysql::MySqlRow) -> Result<Menu, MenuError> {
    let id = row_to_id(row).map_err(|_| MenuError::Persistence)?;
    let parent_id = row
        .try_get::<Option<Vec<u8>>, _>("parent_id")
        .map_err(|_| MenuError::Persistence)?
        .map(|bytes| {
            let uuid = uuid::Uuid::from_slice(&bytes).map_err(|_| MenuError::Persistence)?;
            EntityId::from_uuid(uuid).ok_or(MenuError::Persistence)
        })
        .transpose()?;

    Ok(Menu {
        id,
        parent_id,
        name: row.try_get("name").map_err(|_| MenuError::Persistence)?,
        path: row.try_get("path").map_err(|_| MenuError::Persistence)?,
        icon: row.try_get("icon").map_err(|_| MenuError::Persistence)?,
        access: MenuAccess {
            user: row
                .try_get("allow_user")
                .map_err(|_| MenuError::Persistence)?,
            premium: row
                .try_get("allow_premium")
                .map_err(|_| MenuError::Persistence)?,
        },
        sort_order: row
            .try_get("sort_order")
            .map_err(|_| MenuError::Persistence)?,
        is_active: row
            .try_get::<bool, _>("is_active")
            .map_err(|_| MenuError::Persistence)?,
    })
}
