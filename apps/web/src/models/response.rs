use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiResponseWithMeta<T, M> {
    pub data: T,
    pub meta: M,
}

pub type ApiPage<T> = ApiResponseWithMeta<T, PaginationMeta>;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: i32,
    pub total: u64,
    pub total_pages: u32,
}
