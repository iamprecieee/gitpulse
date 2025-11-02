use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub language: Option<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default = "default_timeframe")]
    pub timeframe: String,
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(default = "default_min_stars")]
    pub min_stars: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pushed_after: Option<String>,
    #[serde(default)]
    pub has_specific_date: bool,
}

fn default_timeframe() -> String {
    "week".to_string()
}

fn default_count() -> usize {
    5
}

fn default_min_stars() -> u32 {
    10
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            language: None,
            topics: vec![],
            timeframe: default_timeframe(),
            count: default_count(),
            min_stars: default_min_stars(),
            date_string: None,
            created_after: None,
            pushed_after: None,
            has_specific_date: false,
        }
    }
}

impl QueryParams {
    pub fn uses_specific_dates(&self) -> bool {
        self.created_after.is_some() || self.pushed_after.is_some()
    }
}
