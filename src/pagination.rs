use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

impl Pagination {
    pub fn new() -> Self {
        Self {
            page: default_page(),
            limit: default_limit(),
        }
    }
}

fn default_page() -> i32 {
    1
}

fn default_limit() -> i32 {
    10
}
