pub use crate::models::response::{ApiPage, ApiResponse, ApiResponseWithMeta, PaginationMeta};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub is_admin: bool,
    pub is_premium: bool,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_locked: bool,
    pub locale: String,
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
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

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct MenuListData {
    pub menus: Vec<MenuResponse>,
}

pub type MenuListResponse = MenuListData;

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct UserListData {
    pub users: Vec<UserResponse>,
}

pub type UserListResponse = ApiPage<UserListData>;

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct PremiumResponse {
    pub active: bool,
    pub expires_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct ApiProblem {
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct ProfileRequest {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct AdminEmailRequest {
    pub email: String,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct AdminRoleRequest {
    pub role: String,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct PremiumRequest {
    pub active: bool,
    pub expires_at: Option<i64>,
}
