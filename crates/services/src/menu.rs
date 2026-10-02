//! Role-aware menu application contracts and CQRS handlers.

use minirust_core::EntityId;

use crate::auth::UserAccess;
use crate::cqrs::{AsyncCommandHandler, AsyncQueryHandler, Command, Query};
use crate::{Page, Pagination};

/// Which account tiers may open a menu.
///
/// The flags are matched exactly against the caller: a normal account needs
/// `user`, a premium account needs `premium`. A system admin always has full
/// access, so admin-only menus simply leave both flags clear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuAccess {
    pub user: bool,
    pub premium: bool,
}

impl MenuAccess {
    /// Visible to normal and premium accounts.
    pub const ALL: Self = Self {
        user: true,
        premium: true,
    };

    /// Only system admins may open the menu.
    pub const ADMIN_ONLY: Self = Self {
        user: false,
        premium: false,
    };

    pub const fn is_allowed_for(self, user: &UserAccess) -> bool {
        if user.is_admin {
            return true;
        }
        if user.is_premium {
            self.premium
        } else {
            self.user
        }
    }
}

impl Default for MenuAccess {
    fn default() -> Self {
        Self::ALL
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    pub id: EntityId,
    pub parent_id: Option<EntityId>,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub access: MenuAccess,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMenu {
    pub parent_id: Option<EntityId>,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub access: MenuAccess,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuError {
    Forbidden,
    NotFound,
    InvalidName,
    InvalidPath,
    InvalidIcon,
    InvalidParent,
    Persistence,
}

impl std::fmt::Display for MenuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Forbidden => "admin role required",
            Self::NotFound => "menu not found",
            Self::InvalidName => "menu name is invalid",
            Self::InvalidPath => "menu path is invalid",
            Self::InvalidIcon => "menu icon is invalid",
            Self::InvalidParent => "menu parent is invalid",
            Self::Persistence => "menu persistence failed",
        };
        f.write_str(message)
    }
}

impl std::error::Error for MenuError {}

#[allow(async_fn_in_trait)]
pub trait MenuRepository: Clone + Send + Sync + 'static {
    async fn find_menu(&self, menu_id: EntityId) -> Result<Option<Menu>, MenuError>;
    async fn list_menus(&self, pagination: Pagination) -> Result<Page<Menu>, MenuError>;
    async fn list_active_menus(&self) -> Result<Vec<Menu>, MenuError>;
    async fn parent_exists(&self, parent_id: EntityId) -> Result<bool, MenuError>;
    async fn update_menu(&self, menu: &Menu) -> Result<Menu, MenuError>;
}

#[derive(Clone)]
pub struct MenuService<R> {
    repository: R,
}

impl<R> MenuService<R>
where
    R: MenuRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        actor: &UserAccess,
        pagination: Pagination,
    ) -> Result<Page<Menu>, MenuError> {
        require_admin(actor)?;
        self.repository.list_menus(pagination).await
    }

    pub async fn list_for_user(&self, actor: &UserAccess) -> Result<Vec<Menu>, MenuError> {
        let menus = self.repository.list_active_menus().await?;
        Ok(menus
            .into_iter()
            .filter(|menu| menu.access.is_allowed_for(actor))
            .collect())
    }

    /// Decides whether `actor` may open `path` based on the menu registry.
    ///
    /// Paths that are not registered as an active menu are not managed here.
    /// When access is denied, the first menu the actor may open is returned so
    /// the caller can redirect instead of looping on the same path.
    pub async fn decide_access(
        &self,
        actor: &UserAccess,
        path: &str,
    ) -> Result<MenuAccessDecision, MenuError> {
        let menus = self.repository.list_active_menus().await?;
        let Some(menu) = menus.iter().find(|menu| menu.path == path) else {
            return Ok(MenuAccessDecision::Unmanaged);
        };

        if menu.access.is_allowed_for(actor) {
            return Ok(MenuAccessDecision::Allowed);
        }

        Ok(MenuAccessDecision::Denied {
            fallback: menus
                .iter()
                .find(|menu| menu.access.is_allowed_for(actor))
                .map(|menu| menu.path.clone()),
        })
    }

    pub async fn update(
        &self,
        actor: &UserAccess,
        menu_id: EntityId,
        input: UpdateMenu,
    ) -> Result<Menu, MenuError> {
        require_admin(actor)?;
        let mut menu = self
            .repository
            .find_menu(menu_id)
            .await?
            .ok_or(MenuError::NotFound)?;

        let validated = self
            .validate_fields(
                menu_id,
                input.parent_id,
                &input.name,
                &input.path,
                input.icon.as_deref(),
            )
            .await?;

        menu.parent_id = validated.parent_id;
        menu.name = validated.name;
        menu.path = validated.path;
        menu.icon = validated.icon;
        menu.access = input.access;
        menu.sort_order = input.sort_order;
        menu.is_active = input.is_active;

        self.repository.update_menu(&menu).await
    }

    async fn validate_fields(
        &self,
        menu_id: EntityId,
        parent_id: Option<EntityId>,
        name: &str,
        path: &str,
        icon: Option<&str>,
    ) -> Result<ValidatedMenuFields, MenuError> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 200 {
            return Err(MenuError::InvalidName);
        }

        let path = path.trim();
        if path.is_empty() || path.len() > 512 || !path.starts_with('/') {
            return Err(MenuError::InvalidPath);
        }

        let icon = icon.map(str::trim);
        if icon.is_some_and(|value| value.is_empty() || value.chars().count() > 100) {
            return Err(MenuError::InvalidIcon);
        }

        if parent_id == Some(menu_id) {
            return Err(MenuError::InvalidParent);
        }

        if let Some(parent_id) = parent_id {
            if !self.repository.parent_exists(parent_id).await? {
                return Err(MenuError::InvalidParent);
            }
        }

        Ok(ValidatedMenuFields {
            parent_id,
            name: name.to_owned(),
            path: path.to_owned(),
            icon: icon.map(ToOwned::to_owned),
        })
    }
}

/// Outcome of evaluating the menu policy for a request path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAccessDecision {
    /// The path is not registered as an active menu.
    Unmanaged,
    /// The actor may open the path.
    Allowed,
    /// The path is a menu the actor may not open; `fallback` is the first
    /// accessible menu path when one exists.
    Denied { fallback: Option<String> },
}

struct ValidatedMenuFields {
    parent_id: Option<EntityId>,
    name: String,
    path: String,
    icon: Option<String>,
}

fn require_admin(actor: &UserAccess) -> Result<(), MenuError> {
    actor.is_admin.then_some(()).ok_or(MenuError::Forbidden)
}

pub enum MenuCommand {
    Update {
        actor: UserAccess,
        menu_id: EntityId,
        input: UpdateMenu,
    },
}

impl Command for MenuCommand {
    type Output = Menu;
    type Error = MenuError;
}

#[derive(Clone)]
pub struct MenuCommandHandler<R>
where
    R: MenuRepository,
{
    service: MenuService<R>,
}

impl<R> MenuCommandHandler<R>
where
    R: MenuRepository,
{
    pub fn new(service: MenuService<R>) -> Self {
        Self { service }
    }
}

impl<R> AsyncCommandHandler<MenuCommand> for MenuCommandHandler<R>
where
    R: MenuRepository,
{
    async fn handle(&self, command: MenuCommand) -> Result<Menu, MenuError> {
        match command {
            MenuCommand::Update {
                actor,
                menu_id,
                input,
            } => self.service.update(&actor, menu_id, input).await,
        }
    }
}

pub enum MenuQuery {
    List {
        actor: UserAccess,
        pagination: Pagination,
    },
    ListForUser {
        actor: UserAccess,
    },
}

pub enum MenuQueryResult {
    List(Page<Menu>),
    ListForUser(Vec<Menu>),
}

impl Query for MenuQuery {
    type Output = Result<MenuQueryResult, MenuError>;
}

#[derive(Clone)]
pub struct MenuQueryHandler<R>
where
    R: MenuRepository,
{
    service: MenuService<R>,
}

impl<R> MenuQueryHandler<R>
where
    R: MenuRepository,
{
    pub fn new(service: MenuService<R>) -> Self {
        Self { service }
    }
}

impl<R> AsyncQueryHandler<MenuQuery> for MenuQueryHandler<R>
where
    R: MenuRepository,
{
    async fn handle(&self, query: MenuQuery) -> Result<MenuQueryResult, MenuError> {
        match query {
            MenuQuery::List { actor, pagination } => self
                .service
                .list(&actor, pagination)
                .await
                .map(MenuQueryResult::List),
            MenuQuery::ListForUser { actor } => self
                .service
                .list_for_user(&actor)
                .await
                .map(MenuQueryResult::ListForUser),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::UserLocale;
    use crate::PaginationMeta;

    fn actor(is_admin: bool, is_premium: bool) -> UserAccess {
        UserAccess {
            id: EntityId::new(),
            email: "user@example.com".to_owned(),
            is_admin,
            is_premium,
            premium_expires_at: None,
            full_name: None,
            avatar_url: None,
            is_locked: false,
            locale: UserLocale::Vi,
        }
    }

    fn menu(path: &str, access: MenuAccess, is_active: bool) -> Menu {
        Menu {
            id: EntityId::new(),
            parent_id: None,
            name: path.trim_start_matches('/').to_owned(),
            path: path.to_owned(),
            icon: None,
            access,
            sort_order: 0,
            is_active,
        }
    }

    #[derive(Clone)]
    struct FakeMenus {
        menus: Vec<Menu>,
    }

    impl MenuRepository for FakeMenus {
        async fn find_menu(&self, menu_id: EntityId) -> Result<Option<Menu>, MenuError> {
            Ok(self.menus.iter().find(|menu| menu.id == menu_id).cloned())
        }

        async fn list_menus(&self, pagination: Pagination) -> Result<Page<Menu>, MenuError> {
            let total = self.menus.len() as u64;
            let items = match pagination {
                Pagination::All => self.menus.clone(),
                Pagination::Paged { page_size, .. } => self
                    .menus
                    .iter()
                    .skip(pagination.offset() as usize)
                    .take(page_size as usize)
                    .cloned()
                    .collect(),
            };
            Ok(Page {
                items,
                meta: PaginationMeta::from_pagination(pagination, total),
            })
        }

        async fn list_active_menus(&self) -> Result<Vec<Menu>, MenuError> {
            Ok(self
                .menus
                .iter()
                .filter(|menu| menu.is_active)
                .cloned()
                .collect())
        }

        async fn parent_exists(&self, parent_id: EntityId) -> Result<bool, MenuError> {
            Ok(self.menus.iter().any(|menu| menu.id == parent_id))
        }

        async fn update_menu(&self, menu: &Menu) -> Result<Menu, MenuError> {
            Ok(menu.clone())
        }
    }

    fn service(menus: Vec<Menu>) -> MenuService<FakeMenus> {
        MenuService::new(FakeMenus { menus })
    }

    #[test]
    fn access_flags_match_the_account_tier_exactly() {
        let normal = actor(false, false);
        let premium = actor(false, true);
        let admin = actor(true, false);
        let premium_admin = actor(true, true);

        for user in [&normal, &premium, &admin, &premium_admin] {
            assert!(MenuAccess::ALL.is_allowed_for(user));
            assert_eq!(MenuAccess::ADMIN_ONLY.is_allowed_for(user), user.is_admin);
        }

        let premium_only = MenuAccess {
            user: false,
            premium: true,
        };
        assert!(!premium_only.is_allowed_for(&normal));
        assert!(premium_only.is_allowed_for(&premium));

        let normal_only = MenuAccess {
            user: true,
            premium: false,
        };
        assert!(normal_only.is_allowed_for(&normal));
        assert!(!normal_only.is_allowed_for(&premium));
    }

    #[tokio::test]
    async fn list_for_user_returns_only_the_menus_matching_the_tier() {
        let service = service(vec![
            menu("/app", MenuAccess::ALL, true),
            menu(
                "/premium",
                MenuAccess {
                    user: false,
                    premium: true,
                },
                true,
            ),
            menu("/admin", MenuAccess::ADMIN_ONLY, true),
            menu("/hidden", MenuAccess::ALL, false),
        ]);

        let paths = |menus: Vec<Menu>| {
            menus
                .into_iter()
                .map(|menu| menu.path)
                .collect::<Vec<String>>()
        };

        assert_eq!(
            paths(service.list_for_user(&actor(false, false)).await.unwrap()),
            vec!["/app".to_owned()]
        );
        assert_eq!(
            paths(service.list_for_user(&actor(false, true)).await.unwrap()),
            vec!["/app".to_owned(), "/premium".to_owned()]
        );
        assert_eq!(
            paths(service.list_for_user(&actor(true, false)).await.unwrap()),
            vec![
                "/app".to_owned(),
                "/premium".to_owned(),
                "/admin".to_owned()
            ]
        );
    }

    #[tokio::test]
    async fn inactive_or_unregistered_paths_are_unmanaged() {
        let service = service(vec![
            menu("/app", MenuAccess::ALL, true),
            menu("/hidden", MenuAccess::ALL, false),
        ]);
        let normal = actor(false, false);

        assert_eq!(
            service.decide_access(&normal, "/other").await.unwrap(),
            MenuAccessDecision::Unmanaged
        );
        assert_eq!(
            service.decide_access(&normal, "/hidden").await.unwrap(),
            MenuAccessDecision::Unmanaged
        );
        assert_eq!(
            service.decide_access(&normal, "/app").await.unwrap(),
            MenuAccessDecision::Allowed
        );
    }

    #[tokio::test]
    async fn denied_paths_redirect_to_the_first_accessible_menu() {
        let service = service(vec![
            menu("/app", MenuAccess::ADMIN_ONLY, true),
            menu("/premium", MenuAccess::ALL, true),
        ]);

        assert_eq!(
            service
                .decide_access(&actor(false, false), "/app")
                .await
                .unwrap(),
            MenuAccessDecision::Denied {
                fallback: Some("/premium".to_owned())
            }
        );
    }

    #[tokio::test]
    async fn denied_paths_without_an_accessible_menu_have_no_fallback() {
        let service = service(vec![menu("/app", MenuAccess::ADMIN_ONLY, true)]);

        assert_eq!(
            service
                .decide_access(&actor(false, false), "/app")
                .await
                .unwrap(),
            MenuAccessDecision::Denied { fallback: None }
        );
    }
}
