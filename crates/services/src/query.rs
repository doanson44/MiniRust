//! Shared application-layer query primitives.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationRequest {
    pub page: u32,
    /// -1 means that pagination is disabled and all matching rows are returned.
    pub page_size: i32,
}

impl Default for PaginationRequest {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl PaginationRequest {
    pub const MAX_PAGE_SIZE: u32 = 100;

    pub fn normalize(self) -> Result<Pagination, PaginationError> {
        if self.page == 0 {
            return Err(PaginationError::InvalidPage);
        }

        match self.page_size {
            -1 => Ok(Pagination::All),
            1..=(Self::MAX_PAGE_SIZE as i32) => Ok(Pagination::Paged {
                page: self.page,
                page_size: self.page_size as u32,
            }),
            _ => Err(PaginationError::InvalidPageSize),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pagination {
    Paged { page: u32, page_size: u32 },
    All,
}

impl Pagination {
    pub fn offset(self) -> u64 {
        match self {
            Self::Paged { page, page_size } => (page as u64 - 1) * page_size as u64,
            Self::All => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationError {
    InvalidPage,
    InvalidPageSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: i32,
    pub total: u64,
    pub total_pages: u32,
}

impl PaginationMeta {
    pub fn from_pagination(pagination: Pagination, total: u64) -> Self {
        match pagination {
            Pagination::All => Self {
                page: 1,
                page_size: -1,
                total,
                total_pages: if total == 0 { 0 } else { 1 },
            },
            Pagination::Paged { page, page_size } => Self {
                page,
                page_size: page_size as i32,
                total,
                total_pages: if total == 0 {
                    0
                } else {
                    total.div_ceil(page_size as u64) as u32
                },
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub meta: PaginationMeta,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_one_means_all() {
        assert_eq!(
            PaginationRequest {
                page: 7,
                page_size: -1,
            }
            .normalize(),
            Ok(Pagination::All)
        );
    }

    #[test]
    fn page_size_is_bounded() {
        assert_eq!(
            PaginationRequest {
                page: 1,
                page_size: 101,
            }
            .normalize(),
            Err(PaginationError::InvalidPageSize)
        );
    }

    #[test]
    fn paged_meta_calculates_total_pages() {
        let meta = PaginationMeta::from_pagination(
            Pagination::Paged {
                page: 2,
                page_size: 20,
            },
            41,
        );

        assert_eq!(meta.page, 2);
        assert_eq!(meta.page_size, 20);
        assert_eq!(meta.total, 41);
        assert_eq!(meta.total_pages, 3);
    }
}
