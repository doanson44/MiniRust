pub use crate::models::response::{ApiPage, ApiResponse, ApiResponseWithMeta, PaginationMeta};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub is_admin: bool,
    pub is_premium: bool,
    pub premium_expires_at: Option<i64>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_locked: bool,
    pub locale: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MenuResponse {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub allow_user: bool,
    pub allow_premium: bool,
    pub sort_order: i32,
    pub is_active: bool,
}

#[cfg(feature = "hydrate")]
pub type MenuListResponse = MenuListData;

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize)]
pub struct MenuListData {
    pub menus: Vec<MenuResponse>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize)]
pub struct UserListData {
    pub users: Vec<UserResponse>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize)]
pub struct AvatarResponse {
    pub avatar_url: String,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize)]
pub struct PremiumResponse {
    pub active: bool,
    pub expires_at: Option<i64>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize)]
pub struct ApiProblem {
    pub detail: String,
}
