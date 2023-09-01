use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: u64,
    pub per_page: u64,
}

impl Pagination {
    pub fn check_range(&self) -> bool {
        self.per_page > 0 && self.per_page < 101
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination {
            page: 0,
            per_page: 10,
        }
    }
}
