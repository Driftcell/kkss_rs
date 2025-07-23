use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationInfo,
}

impl PaginationParams {
    pub fn new(page: Option<u32>, per_page: Option<u32>) -> Self {
        Self { page, per_page }
    }

    pub fn get_page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> u32 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }

    pub fn get_offset(&self) -> u32 {
        (self.get_page() - 1) * self.get_per_page()
    }

    pub fn get_limit(&self) -> u32 {
        self.get_per_page()
    }
}

impl PaginationInfo {
    pub fn new(current_page: u32, per_page: u32, total: i64) -> Self {
        let total_pages = if total == 0 {
            1
        } else {
            ((total as f64) / (per_page as f64)).ceil() as u32
        };

        Self {
            current_page,
            per_page,
            total,
            total_pages,
        }
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, params: &PaginationParams, total: i64) -> Self {
        let pagination = PaginationInfo::new(
            params.get_page(),
            params.get_per_page(),
            total,
        );

        Self { items, pagination }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new(Some(2), Some(10));
        assert_eq!(params.get_page(), 2);
        assert_eq!(params.get_per_page(), 10);
        assert_eq!(params.get_offset(), 10);
        assert_eq!(params.get_limit(), 10);
    }

    #[test]
    fn test_pagination_params_defaults() {
        let params = PaginationParams::new(None, None);
        assert_eq!(params.get_page(), 1);
        assert_eq!(params.get_per_page(), 20);
        assert_eq!(params.get_offset(), 0);
        assert_eq!(params.get_limit(), 20);
    }

    #[test]
    fn test_pagination_info() {
        let info = PaginationInfo::new(2, 10, 25);
        assert_eq!(info.current_page, 2);
        assert_eq!(info.per_page, 10);
        assert_eq!(info.total, 25);
        assert_eq!(info.total_pages, 3);
    }
}
