//! Role-aware menu application contracts and CQRS handlers.

use minirust_core::EntityId;

use crate::auth::UserAccess;
use crate::cqrs::{AsyncCommandHandler, AsyncQueryHandler, Command, Query};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuRole {
    User,
    Admin,
}

impl MenuRole {
    pub fn parse(value: &str) -> Result<Self, MenuError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "user" => Ok(Self::User),
            "admin" => Ok(Self::Admin),
            _ => Err(MenuError::InvalidRole),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Admin => "admin",
        }
    }

    pub const fn is_allowed_for(self, user: &UserAccess) -> bool {
        match self {
            Self::User => true,
            Self::Admin => user.is_admin,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    pub id: EntityId,
    pub parent_id: Option<EntityId>,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub required_role: MenuRole,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateMenu {
    pub parent_id: Option<EntityId>,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub required_role: MenuRole,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMenu {
    pub parent_id: Option<EntityId>,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub required_role: MenuRole,
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
    InvalidRole,
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
            Self::InvalidRole => "menu role is invalid",
            Self::InvalidParent => "menu parent is invalid",
            Self::Persistence => "menu persistence failed",
        };
        f.write_str(message)
    }
}

impl std::error::Error for MenuError {}

#[allow(async_fn_in_trait)]
pub trait MenuRepository: Clone + Send + Sync + 'static {
    async fn create_menu(&self, menu: &Menu) -> Result<Menu, MenuError>;
    async fn find_menu(&self, menu_id: EntityId) -> Result<Option<Menu>, MenuError>;
    async fn list_menus(&self) -> Result<Vec<Menu>, MenuError>;
    async fn list_active_menus(&self) -> Result<Vec<Menu>, MenuError>;
    async fn parent_exists(&self, parent_id: EntityId) -> Result<bool, MenuError>;
    async fn update_menu(&self, menu: &Menu) -> Result<Menu, MenuError>;
    async fn delete_menu(&self, menu_id: EntityId) -> Result<(), MenuError>;
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

    pub async fn create(
        &self,
        actor: &UserAccess,
        input: CreateMenu,
    ) -> Result<Menu, MenuError> {
        require_admin(actor)?;
        let menu = self.validate_new(input).await?;
        self.repository.create_menu(&menu).await
    }

    pub async fn get(
        &self,
        actor: &UserAccess,
        menu_id: EntityId,
    ) -> Result<Menu, MenuError> {
        require_admin(actor)?;
        self.repository
            .find_menu(menu_id)
            .await?
            .ok_or(MenuError::NotFound)
    }

    pub async fn list(&self, actor: &UserAccess) -> Result<Vec<Menu>, MenuError> {
        require_admin(actor)?;
        self.repository.list_menus().await
    }

    pub async fn list_for_user(&self, actor: &UserAccess) -> Result<Vec<Menu>, MenuError> {
        let menus = self.repository.list_active_menus().await?;
        Ok(menus
            .into_iter()
            .filter(|menu| menu.required_role.is_allowed_for(actor))
            .collect())
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

        let validated = self.validate_fields(
            menu_id,
            input.parent_id,
            &input.name,
            &input.path,
            input.icon.as_deref(),
            input.required_role,
        )
        .await?;

        menu.parent_id = validated.parent_id;
        menu.name = validated.name;
        menu.path = validated.path;
        menu.icon = validated.icon;
        menu.required_role = validated.required_role;
        menu.sort_order = input.sort_order;
        menu.is_active = input.is_active;

        self.repository.update_menu(&menu).await
    }

    pub async fn delete(
        &self,
        actor: &UserAccess,
        menu_id: EntityId,
    ) -> Result<(), MenuError> {
        require_admin(actor)?;
        self.repository
            .find_menu(menu_id)
            .await?
            .ok_or(MenuError::NotFound)?;
        self.repository.delete_menu(menu_id).await
    }

    async fn validate_new(&self, input: CreateMenu) -> Result<Menu, MenuError> {
        let id = EntityId::new();
        let validated = self
            .validate_fields(
                id,
                input.parent_id,
                &input.name,
                &input.path,
                input.icon.as_deref(),
                input.required_role,
            )
            .await?;

        Ok(Menu {
            id,
            parent_id: validated.parent_id,
            name: validated.name,
            path: validated.path,
            icon: validated.icon,
            required_role: validated.required_role,
            sort_order: input.sort_order,
            is_active: input.is_active,
        })
    }

    async fn validate_fields(
        &self,
        menu_id: EntityId,
        parent_id: Option<EntityId>,
        name: &str,
        path: &str,
        icon: Option<&str>,
        required_role: MenuRole,
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
            required_role,
        })
    }
}

struct ValidatedMenuFields {
    parent_id: Option<EntityId>,
    name: String,
    path: String,
    icon: Option<String>,
    required_role: MenuRole,
}

fn require_admin(actor: &UserAccess) -> Result<(), MenuError> {
    actor.is_admin.then_some(()).ok_or(MenuError::Forbidden)
}

pub enum MenuCommand {
    Create { actor: UserAccess, input: CreateMenu },
    Update {
        actor: UserAccess,
        menu_id: EntityId,
        input: UpdateMenu,
    },
    Delete { actor: UserAccess, menu_id: EntityId },
}

pub enum MenuCommandResult {
    Menu(Menu),
    Deleted,
}

impl Command for MenuCommand {
    type Output = MenuCommandResult;
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
    async fn handle(&self, command: MenuCommand) -> Result<MenuCommandResult, MenuError> {
        match command {
            MenuCommand::Create { actor, input } => self
                .service
                .create(&actor, input)
                .await
                .map(MenuCommandResult::Menu),
            MenuCommand::Update {
                actor,
                menu_id,
                input,
            } => self
                .service
                .update(&actor, menu_id, input)
                .await
                .map(MenuCommandResult::Menu),
            MenuCommand::Delete { actor, menu_id } => self
                .service
                .delete(&actor, menu_id)
                .await
                .map(|_| MenuCommandResult::Deleted),
        }
    }
}

pub enum MenuQuery {
    Get { actor: UserAccess, menu_id: EntityId },
    List { actor: UserAccess },
    ListForUser { actor: UserAccess },
}

pub enum MenuQueryResult {
    Menu(Menu),
    Menus(Vec<Menu>),
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
            MenuQuery::Get { actor, menu_id } => self.service.get(&actor, menu_id).await.map(MenuQueryResult::Menu),
            MenuQuery::List { actor } => self.service.list(&actor).await.map(MenuQueryResult::Menus),
            MenuQuery::ListForUser { actor } => self.service.list_for_user(&actor).await.map(MenuQueryResult::Menus),
        }
    }
}
