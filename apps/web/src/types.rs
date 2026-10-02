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

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize)]
pub struct UserListData {
    pub users: Vec<UserResponse>,
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
