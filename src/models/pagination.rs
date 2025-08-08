//! 分页相关的数据结构

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
        }
    }
}

impl PaginationParams {
    pub fn new(page: Option<u32>, per_page: Option<u32>) -> Self {
        Self {
            page: page.map(|p| p as i64),
            page_size: per_page.map(|p| p as i64),
        }
    }

    pub fn get_offset(&self) -> i64 {
        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(20);
        (page - 1) * page_size
    }

    pub fn get_limit(&self) -> i64 {
        self.page_size.unwrap_or(20)
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: i64, page_size: i64, total: i64) -> Self {
        let total_pages = (total + page_size - 1) / page_size;
        Self {
            data,
            page,
            page_size,
            total,
            total_pages,
        }
    }
}
