use std::fmt::Debug;

pub const DEFAULT_PAGE_SIZE: u32 = 20;
pub const MAX_PAGE_SIZE: u32 = 100;
pub const ALL_PAGE_SIZE: i32 = -1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pagination {
    pub page: u32,
    pub page_size: i32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: DEFAULT_PAGE_SIZE as i32,
        }
    }
}

impl Pagination {
    pub const fn all() -> Self {
        Self {
            page: 1,
            page_size: ALL_PAGE_SIZE,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchQuery {
    pub search: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Asc
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sort<T> {
    pub field: T,
    pub direction: SortDirection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListQuery<S, F> {
    pub pagination: Pagination,
    pub search: SearchQuery,
    pub sort: Option<Sort<S>>,
    pub filters: F,
}
